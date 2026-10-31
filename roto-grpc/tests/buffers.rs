//! `BufferSet` configuration: Huffman header blocks that decode past the default 512-byte HPACK
//! scratch succeed with `GrowableBuffers` / `SliceBuffers` and fail cleanly with `FixedBuffers`.

use core::mem::size_of;

use roto_grpc::h2::{H2Connection, H2Event, Role};
use roto_grpc::hpack::{encode_literal_ext, GrpcTrailers, HpackDecoder};
use roto_grpc::status::percent_decode;
use roto_grpc::{
    BufferSet, FixedBuffers, FixedMetadata, GrpcClient, GrpcCode, GrpcServer, SliceBuffers,
    DEFAULT_HPACK_SCRATCH,
};

/// Metadata large enough to hold the 600-byte header the blocks below decode to.
type Meta = FixedMetadata<8, 2048>;

const OVERFLOW_MSG: &str = "header block exceeds HPACK scratch capacity (use with_buffers with GrowableBuffers or a larger SliceBuffers)";
const LPM_EMPTY: &[u8] = &[0, 0, 0, 0, 0];

/// Encodes literal `fields` plus a 600-byte Huffman-coded `big` header: ~375 wire bytes that
/// decode to more than the default 512-byte scratch.
fn huffman_block(fields: &[(&[u8], &[u8])], big: &[u8]) -> Vec<u8> {
    let mut block = vec![0u8; 2048];
    let mut len = 0;
    for (n, v) in fields {
        len += encode_literal_ext(n, v, false, false, false, &mut block[len..]).unwrap();
    }
    len += encode_literal_ext(big, &[b'a'; 600], false, false, true, &mut block[len..]).unwrap();
    block.truncate(len);
    block
}

fn literal_block(fields: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut block = vec![0u8; 256];
    let mut len = 0;
    for (n, v) in fields {
        len += encode_literal_ext(n, v, false, false, false, &mut block[len..]).unwrap();
    }
    block.truncate(len);
    block
}

fn decode_trailers(block: &[u8]) -> (GrpcCode, String) {
    let mut scratch = [0u8; 512];
    let t = GrpcTrailers::decode(&mut HpackDecoder::<0>::stateless(), block, &mut scratch).unwrap();
    let mut out = [0u8; 256];
    let message = percent_decode(t.grpc_message.unwrap_or_default(), &mut out).unwrap();
    (t.grpc_status.unwrap(), message.to_owned())
}

/// Sends the request `block` to a route-less server and returns its final `(status, message)`.
/// An accepted request reaches routing (`Unimplemented`); a rejected one never gets there.
fn server_status<B: BufferSet>(bufs: B, block: &[u8]) -> (GrpcCode, String) {
    let (mut msg, mut stx) = ([0u8; 256], [0u8; 4096]);
    let mut server: GrpcServer<4, 512, Meta, B> =
        GrpcServer::with_metadata_store(&[], &mut msg, &mut stx).with_buffers(bufs);
    let mut raw: H2Connection = H2Connection::new(Role::Client);
    let mut wire = vec![0u8; 8192];
    let mut pos = raw.poll_transmit(&mut wire);
    let id = raw.open_stream().unwrap();
    pos += raw
        .send_headers_frame(id, block, false, &mut wire[pos..])
        .unwrap();
    pos += raw
        .send_data_frame(id, LPM_EMPTY, true, &mut wire[pos..])
        .unwrap();
    server.recv(&wire[..pos]).unwrap();

    let n = server.poll_transmit(&mut wire);
    let mut input = &wire[..n];
    let mut last = None;
    while !input.is_empty() {
        let (used, event) = raw.recv(input).unwrap();
        input = &input[used..];
        if let Some(H2Event::Headers {
            header_block,
            end_stream: true,
            ..
        }) = event
        {
            last = Some(decode_trailers(header_block));
        }
    }
    last.expect("no trailers")
}

#[test]
fn server_huffman_block_needs_configured_buffers() {
    let block = huffman_block(
        &[
            (b":method", b"POST"),
            (b":scheme", b"http"),
            (b":path", b"/test.v1.Echo/Echo"),
            (b"content-type", b"application/grpc+roto"),
            (b"te", b"trailers"),
        ],
        b"authorization",
    );

    let mut scratch = [0u8; 2048];
    assert_eq!(
        server_status(SliceBuffers::new(&mut scratch), &block).0,
        GrpcCode::Unimplemented
    );
    #[cfg(feature = "std")]
    assert_eq!(
        server_status(roto_grpc::GrowableBuffers, &block).0,
        GrpcCode::Unimplemented
    );

    assert_eq!(
        server_status(FixedBuffers, &block),
        (GrpcCode::ResourceExhausted, OVERFLOW_MSG.to_owned())
    );
    let mut small = [0u8; 256];
    assert_eq!(
        server_status(SliceBuffers::new(&mut small), &block),
        (GrpcCode::ResourceExhausted, OVERFLOW_MSG.to_owned())
    );
}

/// Replies to one call with `headers`, a message, and `trailers`; returns the client's result.
fn client_status<B: BufferSet>(bufs: B, headers: &[u8], trailers: &[u8]) -> (GrpcCode, String) {
    let (mut ctx, mut buf, mut wire) = ([0u8; 4096], [0u8; 256], [0u8; 4096]);
    let mut client: GrpcClient<4, 512, Meta, B> =
        GrpcClient::with_metadata_store("http", "localhost", &mut ctx).with_buffers(bufs);
    let mut raw: H2Connection = H2Connection::new(Role::Server);

    let sid = client
        .start_call("/test.v1.Echo/Echo", &mut buf, |w| {
            let _ = w.write_bytes(b"req");
        })
        .unwrap();
    let n = client.poll_transmit(&mut wire);
    let mut pos = 0;
    while pos < n {
        pos += raw.recv(&wire[pos..n]).unwrap().0;
    }
    let m = raw.poll_transmit(&mut wire);
    client.recv(&wire[..m]).unwrap();
    let _ = client.poll_transmit(&mut wire);

    let mut len = raw
        .send_headers_frame(sid, headers, false, &mut wire)
        .unwrap();
    len += raw
        .send_data_frame(sid, &[0, 0, 0, 0, 2, b'o', b'k'], false, &mut wire[len..])
        .unwrap();
    len += raw
        .send_headers_frame(sid, trailers, true, &mut wire[len..])
        .unwrap();
    client.recv(&wire[..len]).unwrap();
    let reply = client.take_reply(sid).unwrap();
    match reply.result() {
        Ok(_) => (GrpcCode::Ok, String::new()),
        Err(status) => (status.code, status.message.to_owned()),
    }
}

#[test]
fn client_huffman_headers_and_trailers_need_configured_buffers() {
    let head = [
        (&b":status"[..], &b"200"[..]),
        (b"content-type", b"application/grpc+roto"),
    ];
    let tail = [(&b"grpc-status"[..], &b"0"[..])];
    let cases = [
        (huffman_block(&head, b"x-big"), literal_block(&tail)),
        (literal_block(&head), huffman_block(&tail, b"x-big")),
    ];
    let overflow = (GrpcCode::ResourceExhausted, OVERFLOW_MSG.to_owned());

    for (headers, trailers) in &cases {
        let mut scratch = [0u8; 2048];
        assert_eq!(
            client_status(SliceBuffers::new(&mut scratch), headers, trailers).0,
            GrpcCode::Ok
        );
        #[cfg(feature = "std")]
        assert_eq!(
            client_status(roto_grpc::GrowableBuffers, headers, trailers).0,
            GrpcCode::Ok
        );
        assert_eq!(client_status(FixedBuffers, headers, trailers), overflow);
    }
}

#[test]
fn default_footprint_is_the_inline_hpack_scratch() {
    // `FixedBuffers` stores `DEFAULT_HPACK_SCRATCH` bytes inline; `SliceBuffers` a fat pointer.
    // Everything else in the connection structs is identical.
    type S<B> = GrpcServer<'static, 4, 512, FixedMetadata, B>;
    type C<B> = GrpcClient<'static, 4, 512, FixedMetadata, B>;
    let delta = DEFAULT_HPACK_SCRATCH - size_of::<&mut [u8]>();
    assert_eq!(DEFAULT_HPACK_SCRATCH, 512);
    assert_eq!(
        size_of::<S<FixedBuffers>>(),
        size_of::<S<SliceBuffers<'static>>>() + delta
    );
    assert_eq!(
        size_of::<C<FixedBuffers>>(),
        size_of::<C<SliceBuffers<'static>>>() + delta
    );
    assert_eq!(
        size_of::<GrpcServer<'static, 4, 512>>(),
        size_of::<S<FixedBuffers>>()
    );
}
