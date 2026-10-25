//! `helloworld.Greeter/SayHello` echo server for LesnyRumcajs/grpc_bench.

use roto_grpc::transport::serve_tokio_io;
use roto_grpc::{GrpcServer, GrpcStatus, Route};
use rust_roto_bench::grpc::helloworld::greeter::{Greeter, GreeterServer};
use rust_roto_bench::helloworld::{HelloReplyBuilder, HelloRequestView};
use tokio::net::TcpListener;

/// Concurrent streams per connection. grpc_bench defaults to 1000 concurrent calls over
/// 50 connections (20 per connection); the server advertises this via
/// `SETTINGS_MAX_CONCURRENT_STREAMS`, so ghz queues rather than fails beyond it.
const MAX_STREAMS: usize = 32;
/// Per-stream reassembly capacity; must hold the largest scenario request (`string_10kB`)
/// when it straddles HTTP/2 DATA frames.
const REASM_CAP: usize = 16 * 1024;
/// Response encoding scratch space (one response at a time).
const MSG_BUF_LEN: usize = 16 * 1024;
/// Outbound frame staging for all in-flight responses on a connection.
const TX_BUF_LEN: usize = MAX_STREAMS * MSG_BUF_LEN;

struct Echo;

impl Greeter for Echo {
    fn say_hello<'a, 'b>(
        &self,
        _meta: &mut roto_grpc::RpcMetadata<'_>,
        req: HelloRequestView<'a>,
        resp: &mut HelloReplyBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let hello = req
            .request()
            .map_err(|_| GrpcStatus::invalid_argument("corrupt request"))?;
        if let Some(hello) = hello {
            resp.write_response(|b| {
                b.copy_unset_fields(&hello);
            });
        }
        Ok(())
    }
}

static SVC: GreeterServer<Echo> = GreeterServer::new(Echo);
static ROUTES: [&dyn Route; 1] = [&SVC];

fn main() -> std::io::Result<()> {
    let cpus = std::env::var("GRPC_SERVER_CPUS")
        .map(|v| v.parse().expect("GRPC_SERVER_CPUS must be an integer"))
        .unwrap_or(1);

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(cpus)
        // `GrpcServer<MAX_STREAMS, REASM_CAP>` is ~1.1 MB and is built by value before
        // being boxed; tokio's default 2 MiB worker stack overflows.
        .thread_stack_size(16 * 1024 * 1024)
        .enable_io()
        .build()?
        .block_on(serve())
}

async fn serve() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:50051").await?;
    loop {
        let (stream, _) = listener.accept().await?;
        stream.set_nodelay(true)?;
        tokio::spawn(async move {
            let mut msg_buf = vec![0u8; MSG_BUF_LEN];
            let mut tx_buf = vec![0u8; TX_BUF_LEN];
            let mut server: Box<GrpcServer<'_, MAX_STREAMS, REASM_CAP>> =
                Box::new(GrpcServer::new(&ROUTES, &mut msg_buf, &mut tx_buf));
            if let Err(err) = serve_tokio_io(&mut server, stream).await {
                eprintln!("connection error: {err}");
            }
        });
    }
}
