#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

pub mod client;
mod driver;
pub mod framing;
pub mod h2;
pub mod hpack;
pub mod http;
pub mod server;
pub mod service;
pub mod status;
pub mod streaming;
pub mod transport;

pub use client::{GrpcClient, H2LoopbackChannel, Reply};
pub use framing::{
    decode_grpc_frame, encode_grpc_message, iter_grpc_frames, prepend_grpc_header, take_writer,
    write_grpc_frame, write_grpc_frame_bytes, GrpcFrame, GrpcFrameIter, LpmDecoder,
    DEFAULT_MAX_MESSAGE_SIZE, GRPC_HEADER_LEN,
};
pub use h2::{
    encode_grpc_data_frame, write_grpc_data_frame, write_grpc_data_frame_bytes, FrameHeader,
    FrameType, H2Connection, H2Error, H2ErrorCode, H2Event, H2Settings, Role, StreamState,
    CLIENT_CONNECTION_PREFACE, DEFAULT_INITIAL_WINDOW_SIZE, DEFAULT_MAX_FRAME_SIZE,
    DEFAULT_MAX_STREAMS, DEFAULT_REASM_CAP, FLAG_ACK, FLAG_END_HEADERS, FLAG_END_STREAM,
    FLAG_PADDED, FLAG_PRIORITY, FRAME_HEADER_LEN, MAX_FLOW_CONTROL_WINDOW, MAX_MAX_FRAME_SIZE,
    MAX_STREAM_ID, SETTINGS_ENABLE_PUSH, SETTINGS_HEADER_TABLE_SIZE, SETTINGS_INITIAL_WINDOW_SIZE,
    SETTINGS_MAX_CONCURRENT_STREAMS, SETTINGS_MAX_FRAME_SIZE, SETTINGS_MAX_HEADER_LIST_SIZE,
};
pub use hpack::{
    decode_header_block, decode_prefix_int, decode_string_literal,
    encode_dynamic_table_size_update, encode_grpc_request_headers, encode_grpc_response_headers,
    encode_grpc_trailers, encode_grpc_trailers_backward, encode_grpc_trailers_only, encode_indexed,
    encode_indexed_backward, encode_literal, encode_literal_backward, encode_literal_ext,
    encode_literal_indexed_name, encode_literal_indexed_name_backward,
    encode_literal_indexed_name_ext, encode_prefix_int, encode_prefix_int_backward,
    encode_string_literal, encode_string_literal_backward, huffman_decode, huffman_encode,
    huffman_encoded_len, is_grpc_content_type, static_table_get, DynamicTable, GrpcRequestHeaders,
    GrpcResponseHeaders, GrpcTrailers, HeaderField, HpackDecoder, HpackEncoder, HpackError,
    DEFAULT_DYNAMIC_TABLE_CAP, DYNAMIC_ENTRY_OVERHEAD, GRPC_CONTENT_TYPE, GRPC_CONTENT_TYPE_ROTO,
    HUFFMAN_BIT_LENS, HUFFMAN_CODES, INDEX_ACCEPT_ENCODING, INDEX_AUTHORITY,
    INDEX_CONTENT_ENCODING, INDEX_CONTENT_LENGTH, INDEX_CONTENT_TYPE, INDEX_METHOD_GET,
    INDEX_METHOD_POST, INDEX_PATH_INDEX_HTML, INDEX_PATH_ROOT, INDEX_SCHEME_HTTP,
    INDEX_SCHEME_HTTPS, INDEX_STATUS_200, INDEX_STATUS_204, INDEX_STATUS_206, INDEX_STATUS_304,
    INDEX_STATUS_400, INDEX_STATUS_404, INDEX_STATUS_500, INDEX_USER_AGENT, STATIC_TABLE,
    STATIC_TABLE_LEN,
};
pub use http::{
    dispatch_http_routes, encode_h1_error, encode_h1_http_response, encode_h1_response,
    extract_response_body_field, finish_response, finish_writer_response, http_reason_phrase,
    is_h2_client_preface, parse_bool, parse_enum, parse_f32, parse_f64, parse_h1_request,
    parse_i32, parse_i64, parse_u32, parse_u64, percent_decode_bytes, percent_decode_str,
    query_iter, split_custom_verb, split_uri_path_and_query, wrap_nested_path, write_bool_field,
    write_bytes_field, write_f32_field, write_f64_field, write_fixed32_field, write_fixed64_field,
    write_int32_field, write_int64_field, write_raw_bytes, write_sfixed32_field,
    write_sfixed64_field, write_sint32_field, write_sint64_field, write_str_field,
    write_uint32_field, write_uint64_field, write_varint_field, AsyncHttpFallback, AsyncHttpRouter,
    AsyncHttpService, HttpFallback, HttpMethod, HttpRequest, HttpResponse, HttpRoute, HttpService,
    ParsedH1Request, PathCursor, QueryIter, CONTENT_TYPE_PROTOBUF, CONTENT_TYPE_TEXT_PLAIN,
};
pub use server::{AsyncGrpcServer, AsyncRouter, GrpcServer, Route, MAX_PATH_LEN};
pub use service::{
    extract_method_name, split_grpc_path, AsyncGrpcService, AsyncUnaryChannel, GrpcService,
    LoopbackChannel, UnaryChannel,
};
pub use status::{
    is_unreserved_grpc_message_byte, percent_decode, percent_encode, percent_encode_backward,
    percent_encoded_len, try_percent_decode_in_place, GrpcCode, GrpcStatus,
};
pub use streaming::{LpmRecvStream, RecvStream, SendStream, WriterSendStream};
#[cfg(feature = "std")]
pub use transport::{
    accept_and_serve_std, connect_std_tcp, flush_std_server_outbound, serve_std_io,
    step_std_server, StdClientTransport, StdIo, StdTcpClientTransport,
};
#[cfg(feature = "tokio")]
pub use transport::{
    accept_and_serve_tokio, accept_and_serve_tokio_async, connect_tokio_tcp, serve_tokio_async_io,
    serve_tokio_io, TokioIo, TokioTcpClientTransport,
};
pub use transport::{
    block_on_pair, flush_async_server_outbound, flush_server_outbound, serve_async_connection,
    serve_connection, step_async_server, step_server, AsyncClientTransport, AsyncIo, MemoryDuplex,
    MemoryEndpoint, MemoryIoError, TransportError, DEFAULT_IO_BUF_LEN,
};
