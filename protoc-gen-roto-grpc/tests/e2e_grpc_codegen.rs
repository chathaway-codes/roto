use std::path::{Path, PathBuf};
use std::process::Command;

use google_protobuf_roto_bootstrap::google::protobuf::{
    compiler::{CodeGeneratorRequestBuilder, CodeGeneratorRequestView, CodeGeneratorResponseView},
    field_descriptor_proto::{Label as FieldLabel, Type as FieldType},
};
use protoc_gen_roto_grpc::{generate_request, process_bytes};

fn create_temp_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "roto_grpc_e2e_codegen_{}_{}",
        test_name,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

struct RotoRlibs {
    runtime: PathBuf,
    roto: PathBuf,
    grpc: PathBuf,
}

fn build_roto_and_grpc_rlibs(out_dir: &Path) -> RotoRlibs {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().expect("workspace root");

    let runtime_src = workspace_root.join("roto-runtime/src/lib.rs");
    let runtime_rlib = out_dir.join("libroto_runtime.rlib");
    let status = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=roto_runtime")
        .arg(&runtime_src)
        .arg("-o")
        .arg(&runtime_rlib)
        .output()
        .expect("failed to run rustc on roto-runtime");
    assert!(
        status.status.success(),
        "rustc roto-runtime failed: {}",
        String::from_utf8_lossy(&status.stderr)
    );

    let roto_src = workspace_root.join("roto/src/lib.rs");
    let roto_rlib = out_dir.join("libroto.rlib");
    let status = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=roto")
        .arg(&roto_src)
        .arg("--extern")
        .arg(format!("roto_runtime={}", runtime_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", out_dir.display()))
        .arg("-o")
        .arg(&roto_rlib)
        .output()
        .expect("failed to run rustc on roto");
    assert!(
        status.status.success(),
        "rustc roto failed: {}",
        String::from_utf8_lossy(&status.stderr)
    );

    let grpc_src = workspace_root.join("roto-grpc/src/lib.rs");
    let grpc_rlib = out_dir.join("libroto_grpc.rlib");
    let status = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=roto_grpc")
        .arg(&grpc_src)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("--extern")
        .arg(format!("roto_runtime={}", runtime_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", out_dir.display()))
        .arg("-o")
        .arg(&grpc_rlib)
        .output()
        .expect("failed to run rustc on roto-grpc");
    assert!(
        status.status.success(),
        "rustc roto-grpc failed: {}",
        String::from_utf8_lossy(&status.stderr)
    );

    RotoRlibs {
        runtime: runtime_rlib,
        roto: roto_rlib,
        grpc: grpc_rlib,
    }
}

const COMMON_PROTO: &str = r#"syntax = "proto3";

package common.v1;

message Metadata {
    string trace_id = 1;
    uint64 request_id = 2;
}
"#;

const TELEMETRY_PROTO: &str = r#"syntax = "proto2";

package telemetry.v1;

import "common/v1/common.proto";

message ReportRequest {
    required string device_id = 1;
    required int32 temperature_c = 2;
    optional common.v1.Metadata meta = 3;
}

message ReportResponse {
    required bool accepted = 1;
    optional string status_msg = 2;
    optional common.v1.Metadata echoed_meta = 3;
}

message HealthCheckRequest {
    optional string service = 1;
}

message HealthCheckResponse {
    required bool healthy = 1;
}

service TelemetryService {
    rpc ReportReading (ReportRequest) returns (ReportResponse);
    rpc StreamReadings (stream ReportRequest) returns (stream ReportResponse);
}

service HealthService {
    rpc Check (HealthCheckRequest) returns (HealthCheckResponse);
    rpc CheckMeta (common.v1.Metadata) returns (HealthCheckResponse);
}
"#;

const ROOT_AND_KEYWORDS_PROTO: &str = r#"syntax = "proto3";

message RootPing {
    int32 value = 1;
}

message RootPong {
    int32 doubled = 1;
}

service RootEcho {
    rpc Echo (RootPing) returns (RootPong);
    rpc match (RootPing) returns (RootPong);
    rpc self (RootPing) returns (RootPong);
}
"#;

const KEYWORD_PKG_PROTO: &str = r#"syntax = "proto3";

package self.type.match;

message KwReq {
    string type = 1;
}

message KwResp {
    string match = 1;
}

service match {
    rpc fn (KwReq) returns (KwResp);
    rpc async (KwReq) returns (KwResp);
}
"#;

#[test]
fn e2e_unary_service_and_client_sync_and_async_with_extern_path_and_keywords() {
    let grpc_plugin_bin = env!("CARGO_BIN_EXE_protoc-gen-roto-grpc");
    let tmp_dir = create_temp_dir("unary_sync_async");
    let proto_dir = tmp_dir.join("proto");
    let common_gen_dir = tmp_dir.join("common_gen");
    let telemetry_gen_dir = tmp_dir.join("telemetry_gen");
    let build_dir = tmp_dir.join("build");

    std::fs::create_dir_all(proto_dir.join("common/v1")).unwrap();
    std::fs::create_dir_all(proto_dir.join("telemetry/v1")).unwrap();
    std::fs::create_dir_all(&common_gen_dir).unwrap();
    std::fs::create_dir_all(&telemetry_gen_dir).unwrap();
    std::fs::create_dir_all(&build_dir).unwrap();

    std::fs::write(proto_dir.join("common/v1/common.proto"), COMMON_PROTO).unwrap();
    std::fs::write(
        proto_dir.join("telemetry/v1/telemetry.proto"),
        TELEMETRY_PROTO,
    )
    .unwrap();
    std::fs::write(proto_dir.join("root.proto"), ROOT_AND_KEYWORDS_PROTO).unwrap();
    std::fs::write(proto_dir.join("kw.proto"), KEYWORD_PKG_PROTO).unwrap();

    let rlibs = build_roto_and_grpc_rlibs(&build_dir);

    // 1. Generate `common_proto` crate via `protoc_gen_roto`
    let common_fds = Command::new("protoc")
        .arg(format!("--proto_path={}", proto_dir.display()))
        .arg("--descriptor_set_out=/dev/stdout")
        .arg("common/v1/common.proto")
        .output()
        .expect("protoc common.proto");
    assert!(common_fds.status.success());

    let mut req_buf = vec![0u8; 65536];
    let common_req_bytes = {
        let fds = google_protobuf_roto_bootstrap::google::protobuf::FileDescriptorSetView::from_bytes(
            &common_fds.stdout,
        )
        .unwrap();
        let mut b = CodeGeneratorRequestBuilder::new(&mut req_buf);
        for f in fds.file() {
            let f = f.unwrap();
            b.write_proto_file(|fb| {
                fb.copy_unset_fields(&f);
            });
        }
        b.write_file_to_generate("common/v1/common.proto");
        b.finish().unwrap().to_vec()
    };
    let common_req = CodeGeneratorRequestView::from_bytes(&common_req_bytes).unwrap();
    let common_files = protoc_gen_roto::generate_request(&common_req).unwrap();
    let common_lib_rs = common_gen_dir.join("lib.rs");
    std::fs::write(&common_lib_rs, &common_files[0].content).unwrap();

    let common_rlib = build_dir.join("libcommon_proto.rlib");
    let compile_common = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=common_proto")
        .arg("-D")
        .arg("warnings")
        .arg(&common_lib_rs)
        .arg("--extern")
        .arg(format!("roto={}", rlibs.roto.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&common_rlib)
        .output()
        .expect("compile common_proto");
    assert!(
        compile_common.status.success(),
        "common_proto compile failed:\n{}",
        String::from_utf8_lossy(&compile_common.stderr)
    );

    // 2. Invoke `protoc` with `--plugin=protoc-gen-roto-grpc` on telemetry.proto, root.proto, and kw.proto
    let protoc_grpc = Command::new("protoc")
        .arg(format!("--plugin=protoc-gen-roto-grpc={grpc_plugin_bin}"))
        .arg(format!("--proto_path={}", proto_dir.display()))
        .arg("--roto-grpc_opt=mode=mod,file=grpc.rs,extern_path=.common.v1=::common_proto")
        .arg(format!("--roto-grpc_out={}", telemetry_gen_dir.display()))
        .arg("telemetry/v1/telemetry.proto")
        .arg("root.proto")
        .arg("kw.proto")
        .output()
        .expect("run protoc with protoc-gen-roto-grpc");
    assert!(
        protoc_grpc.status.success(),
        "protoc-gen-roto-grpc failed:\n{}",
        String::from_utf8_lossy(&protoc_grpc.stderr)
    );

    // Also generate the message types for telemetry.proto, root.proto, and kw.proto
    let telemetry_fds = Command::new("protoc")
        .arg(format!("--proto_path={}", proto_dir.display()))
        .arg("--include_imports")
        .arg("--descriptor_set_out=/dev/stdout")
        .arg("telemetry/v1/telemetry.proto")
        .arg("root.proto")
        .arg("kw.proto")
        .output()
        .expect("protoc descriptor set");
    assert!(telemetry_fds.status.success());

    let mut msg_req_buf = vec![0u8; 131072];
    let msg_req_bytes = {
        let fds = google_protobuf_roto_bootstrap::google::protobuf::FileDescriptorSetView::from_bytes(
            &telemetry_fds.stdout,
        )
        .unwrap();
        let files: Vec<_> = fds.file().map(|f| f.unwrap()).collect();
        let mut b = CodeGeneratorRequestBuilder::new(&mut msg_req_buf);
        b.write_parameter("mode=lib,extern_path=.common.v1=::common_proto");
        for f in files.iter().rev() {
            b.write_proto_file(|fb| {
                fb.copy_unset_fields(f);
            });
        }
        b.write_file_to_generate("kw.proto")
            .write_file_to_generate("root.proto")
            .write_file_to_generate("telemetry/v1/telemetry.proto");
        b.finish().unwrap().to_vec()
    };
    let msg_req = CodeGeneratorRequestView::from_bytes(&msg_req_bytes).unwrap();
    let msg_files = protoc_gen_roto::generate_request(&msg_req).unwrap();
    let mut lib_rs_content = msg_files[0].content.clone();
    lib_rs_content.push_str("\npub mod grpc;\n");

    let telemetry_lib_rs = telemetry_gen_dir.join("lib.rs");
    std::fs::write(&telemetry_lib_rs, &lib_rs_content).unwrap();

    let grpc_rs_content = std::fs::read_to_string(telemetry_gen_dir.join("grpc.rs")).unwrap();
    assert!(grpc_rs_content.contains("pub mod telemetry {"));
    assert!(grpc_rs_content.contains("pub mod telemetry_service {"));
    assert!(grpc_rs_content.contains("pub const SERVICE_NAME: &str = \"telemetry.v1.TelemetryService\";"));
    assert!(grpc_rs_content.contains("pub const PATH_REPORT_READING: &str = \"/telemetry.v1.TelemetryService/ReportReading\";"));
    assert!(!grpc_rs_content.contains("StreamReadings"));
    assert!(grpc_rs_content.contains("pub mod health_service {"));
    assert!(grpc_rs_content.contains("pub mod root_echo {"));
    assert!(grpc_rs_content.contains("pub mod self_ {"));
    assert!(grpc_rs_content.contains("pub mod r#match {"));
    assert!(grpc_rs_content.contains("fn r#fn<'a, 'b>("));
    assert!(grpc_rs_content.contains("fn r#async<'a, 'b>("));

    // Compile `telemetry_proto` crate with `-D warnings`
    let telemetry_rlib = build_dir.join("libtelemetry_proto.rlib");
    let compile_telemetry = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=telemetry_proto")
        .arg("-D")
        .arg("warnings")
        .arg(&telemetry_lib_rs)
        .arg("--extern")
        .arg(format!("roto={}", rlibs.roto.display()))
        .arg("--extern")
        .arg(format!("roto_grpc={}", rlibs.grpc.display()))
        .arg("--extern")
        .arg(format!("common_proto={}", common_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&telemetry_rlib)
        .output()
        .expect("compile telemetry_proto");
    assert!(
        compile_telemetry.status.success(),
        "telemetry_proto compile failed:\n{}",
        String::from_utf8_lossy(&compile_telemetry.stderr)
    );

    // 3. Compile and execute an end-to-end test runner binary exercising sync and async services and clients
    let runner_src = tmp_dir.join("runner.rs");
    let runner_bin = build_dir.join("runner_bin");
    std::fs::write(
        &runner_src,
        r#"
#![forbid(unsafe_code)]

use core::future::Future;
use core::task::{Context, Poll, Waker};

use common_proto::common::v1::{MetadataBuilder, MetadataView};
use roto::runtime::BackwardWriter;
use roto_grpc::{
    GrpcClient, GrpcCode, GrpcServer, GrpcStatus, H2LoopbackChannel, LoopbackChannel, Route,
};
use telemetry_proto::{
    grpc::{
        root_echo::{AsyncRootEcho, RootEcho, RootEchoClient, RootEchoServer, PATH_ECHO},
        self_::r#type::r#match::r#match::{Match, MatchClient, MatchServer},
        telemetry::v1::{
            health_service::{HealthService, HealthServiceClient, HealthServiceServer},
            telemetry_service::{
                AsyncTelemetryService, TelemetryService, TelemetryServiceClient,
                TelemetryServiceServer, PATH_REPORT_READING, SERVICE_NAME,
            },
        },
    },
    self_::r#type::r#match::{KwReqBuilder, KwReqView, KwRespBuilder},
    telemetry::v1::{
        HealthCheckRequestBuilder, HealthCheckRequestView, HealthCheckResponseBuilder,
        ReportRequestBuilder, ReportRequestView, ReportResponseBuilder,
    },
    RootPingBuilder, RootPingView, RootPongBuilder,
};

fn block_on<F: Future>(fut: F) -> F::Output {
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut pinned = core::pin::pin!(fut);
    match pinned.as_mut().poll(&mut cx) {
        Poll::Ready(val) => val,
        Poll::Pending => panic!("unexpected Poll::Pending in ready future"),
    }
}

struct MyTelemetryHandler {
    min_temp: i32,
}

impl TelemetryService for MyTelemetryHandler {
    fn report_reading<'a, 'b>(
        &self,
        req: ReportRequestView<'a>,
        resp: &mut ReportResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let temp = req.temperature_c().map_err(|_| GrpcStatus::internal("decode temp"))?;
        if temp < self.min_temp {
            return Err(GrpcStatus::invalid_argument("temperature below minimum threshold"));
        }
        let dev = req.device_id().map_err(|_| GrpcStatus::internal("decode device_id"))?;
        if dev == "unauthorized" {
            return Err(GrpcStatus::permission_denied("device is not authorized"));
        }

        if let Some(meta) = req.meta().map_err(|_| GrpcStatus::internal("decode meta"))? {
            let trace = meta.trace_id().map_err(|_| GrpcStatus::internal("decode trace"))?;
            let req_id = meta.request_id().map_err(|_| GrpcStatus::internal("decode req_id"))?;
            resp.write_echoed_meta(|mb| {
                mb.write_request_id(req_id + 1).write_trace_id(trace);
            });
        }
        resp.write_status_msg("recorded").write_accepted(true);
        Ok(())
    }
}

impl AsyncTelemetryService for MyTelemetryHandler {
    async fn report_reading<'a, 'b>(
        &'a self,
        req: ReportRequestView<'a>,
        resp: &'a mut ReportResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as TelemetryService>::report_reading(self, req, resp)
    }
}

struct MyHealthHandler;

impl HealthService for MyHealthHandler {
    fn check<'a, 'b>(
        &self,
        _req: HealthCheckRequestView<'a>,
        resp: &mut HealthCheckResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        resp.write_healthy(true);
        Ok(())
    }

    fn check_meta<'a, 'b>(
        &self,
        req: MetadataView<'a>,
        resp: &mut HealthCheckResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let trace = req.trace_id().map_err(|_| GrpcStatus::internal("decode trace"))?;
        resp.write_healthy(trace == "ok-meta");
        Ok(())
    }
}

struct MyRootEchoHandler;

impl RootEcho for MyRootEchoHandler {
    fn echo<'a, 'b>(
        &self,
        req: RootPingView<'a>,
        resp: &mut RootPongBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let v = req.value().map_err(|_| GrpcStatus::internal("decode value"))?;
        resp.write_doubled(v * 2);
        Ok(())
    }

    fn r#match<'a, 'b>(
        &self,
        req: RootPingView<'a>,
        resp: &mut RootPongBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let v = req.value().map_err(|_| GrpcStatus::internal("decode value"))?;
        resp.write_doubled(v + 100);
        Ok(())
    }

    fn self_<'a, 'b>(
        &self,
        req: RootPingView<'a>,
        resp: &mut RootPongBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let v = req.value().map_err(|_| GrpcStatus::internal("decode value"))?;
        resp.write_doubled(v + 200);
        Ok(())
    }
}

impl AsyncRootEcho for MyRootEchoHandler {
    async fn echo<'a, 'b>(
        &'a self,
        req: RootPingView<'a>,
        resp: &'a mut RootPongBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as RootEcho>::echo(self, req, resp)
    }

    async fn r#match<'a, 'b>(
        &'a self,
        req: RootPingView<'a>,
        resp: &'a mut RootPongBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as RootEcho>::r#match(self, req, resp)
    }

    async fn self_<'a, 'b>(
        &'a self,
        req: RootPingView<'a>,
        resp: &'a mut RootPongBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as RootEcho>::self_(self, req, resp)
    }
}

struct MyKwMatchHandler;

impl Match for MyKwMatchHandler {
    fn r#fn<'a, 'b>(
        &self,
        req: KwReqView<'a>,
        resp: &mut KwRespBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let t = req.r#type().map_err(|_| GrpcStatus::internal("decode type"))?;
        resp.write_match(t);
        Ok(())
    }

    fn r#async<'a, 'b>(
        &self,
        _req: KwReqView<'a>,
        resp: &mut KwRespBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        resp.write_match("async_ok");
        Ok(())
    }
}

fn main() {
    assert_eq!(SERVICE_NAME, "telemetry.v1.TelemetryService");
    assert_eq!(PATH_REPORT_READING, "/telemetry.v1.TelemetryService/ReportReading");
    assert_eq!(PATH_ECHO, "/RootEcho/Echo");

    // 1. Synchronous client -> server round-trip with borrowed request (&Builder, Builder, &View, View, &[u8])
    let mut req_buf = [0u8; 256];
    let mut resp_buf = [0u8; 256];
    let server = TelemetryServiceServer::new(MyTelemetryHandler { min_temp: -40 });
    let channel = LoopbackChannel::new(server, &mut req_buf, &mut resp_buf);
    let mut client = TelemetryServiceClient::new(channel);

    let mut caller_buf = [0u8; 128];
    let mut req = ReportRequestBuilder::new(&mut caller_buf);
    req.write_meta(|m| {
        m.write_request_id(41).write_trace_id("trace-abc");
    })
    .write_temperature_c(25)
    .write_device_id("sensor-01");

    // Pass `&req` immutably (and verify `req` can be reused across multiple calls!)
    let reply = client
        .report_reading(&req)
        .expect("sync report_reading(&req) should succeed");
    assert!(reply.accepted().unwrap());
    assert_eq!(reply.status_msg().unwrap(), "recorded");
    let echoed = reply.echoed_meta().unwrap().expect("expected echoed_meta");
    assert_eq!(echoed.trace_id().unwrap(), "trace-abc");
    assert_eq!(echoed.request_id().unwrap(), 42);

    // Reuse `&req` for a second call, then pass `&View` and `View` and `&[u8]`, then owned `req`
    let req_bytes = req.writer().as_written_slice();
    let req_view = ReportRequestView::from_bytes(req_bytes).unwrap();
    assert!(client.report_reading(&req_view).unwrap().accepted().unwrap());
    assert!(client.report_reading(req_view).unwrap().accepted().unwrap());
    assert!(client.report_reading(req_bytes).unwrap().accepted().unwrap());
    assert!(client.report_reading(req).unwrap().accepted().unwrap());

    // Sticky `EncodeError::BufferOverflow` on `<Req>Builder` is caught by `req.check()` and returns `GrpcCode::ResourceExhausted`
    let mut tiny_caller_buf = [0u8; 3];
    let mut overflow_req = ReportRequestBuilder::new(&mut tiny_caller_buf);
    overflow_req.write_temperature_c(25).write_device_id("too-long-for-3-bytes");
    assert!(overflow_req.check().is_err());
    let err_overflow_ref = client.report_reading(&overflow_req).unwrap_err();
    assert_eq!(err_overflow_ref.code, GrpcCode::ResourceExhausted);
    let err_overflow_owned = client.report_reading(overflow_req).unwrap_err();
    assert_eq!(err_overflow_owned.code, GrpcCode::ResourceExhausted);

    // 2. Application error status codes propagate cleanly (via both borrowed request and `_with` closure)
    let mut bad_buf = [0u8; 64];
    let mut bad_req = ReportRequestBuilder::new(&mut bad_buf);
    bad_req.write_temperature_c(-100).write_device_id("sensor-01");
    let err_arg = client.report_reading(&bad_req).unwrap_err();
    assert_eq!(err_arg.code, GrpcCode::InvalidArgument);
    assert_eq!(err_arg.message, "temperature below minimum threshold");

    let err_perm = client
        .report_reading_with(|req| {
            req.write_temperature_c(20).write_device_id("unauthorized");
        })
        .unwrap_err();
    assert_eq!(err_perm.code, GrpcCode::PermissionDenied);

    // 3. Asynchronous client -> server round-trip (borrowed `&req`, `&View`, `View`, and `_with_async`)
    let mut async_req_buf = [0u8; 128];
    let mut async_req = ReportRequestBuilder::new(&mut async_req_buf);
    async_req
        .write_meta(|m| {
            m.write_request_id(99).write_trace_id("async-trace");
        })
        .write_temperature_c(30)
        .write_device_id("sensor-async");
    let async_reply = block_on(client.report_reading_async(&async_req))
        .expect("async report_reading_async(&async_req) should succeed");
    assert!(async_reply.accepted().unwrap());
    let async_meta = async_reply.echoed_meta().unwrap().unwrap();
    assert_eq!(async_meta.trace_id().unwrap(), "async-trace");
    assert_eq!(async_meta.request_id().unwrap(), 100);

    let async_view = ReportRequestView::from_bytes(async_req.writer().as_written_slice()).unwrap();
    assert!(block_on(client.report_reading_async(&async_view)).unwrap().accepted().unwrap());
    assert!(block_on(client.report_reading_async(async_view)).unwrap().accepted().unwrap());
    assert!(
        block_on(client.report_reading_with_async(|req| {
            req.write_temperature_c(25).write_device_id("sensor-async-with");
        }))
        .unwrap()
        .accepted()
        .unwrap()
    );

    // Async sticky overflow check
    let mut tiny_async_buf = [0u8; 2];
    let mut overflow_async_req = ReportRequestBuilder::new(&mut tiny_async_buf);
    overflow_async_req.write_temperature_c(25).write_device_id("overflow");
    assert_eq!(
        block_on(client.report_reading_async(&overflow_async_req)).unwrap_err().code,
        GrpcCode::ResourceExhausted
    );

    // 4. Second service in the same package (`HealthService`) including `extern_path` input message `common.v1.Metadata`
    let mut h_req = [0u8; 64];
    let mut h_resp = [0u8; 64];
    let mut health_client = HealthServiceClient::new(LoopbackChannel::new(
        HealthServiceServer::new(MyHealthHandler),
        &mut h_req,
        &mut h_resp,
    ));
    let mut hc_buf = [0u8; 32];
    let mut hc_req = HealthCheckRequestBuilder::new(&mut hc_buf);
    hc_req.write_service("telemetry");
    let h_reply = health_client.check(&hc_req).unwrap();
    assert!(h_reply.healthy().unwrap());

    let mut meta_buf = [0u8; 64];
    let mut meta_req = MetadataBuilder::new(&mut meta_buf);
    meta_req.write_request_id(1).write_trace_id("ok-meta");
    assert!(health_client.check_meta(&meta_req).unwrap().healthy().unwrap());
    let meta_view = MetadataView::from_bytes(meta_req.writer().as_written_slice()).unwrap();
    assert!(health_client.check_meta(&meta_view).unwrap().healthy().unwrap());

    // 5. Empty package service (`RootEcho`) + keyword methods (`r#match`, `self_`)
    let mut r_req = [0u8; 64];
    let mut r_resp = [0u8; 64];
    let mut root_client = RootEchoClient::new(LoopbackChannel::new(
        RootEchoServer::new(MyRootEchoHandler),
        &mut r_req,
        &mut r_resp,
    ));
    let mut p_buf = [0u8; 16];
    let mut ping21 = RootPingBuilder::new(&mut p_buf);
    ping21.write_value(21);
    assert_eq!(
        root_client.echo(&ping21).unwrap().doubled().unwrap(),
        42
    );
    let mut p7_buf = [0u8; 16];
    let mut ping7 = RootPingBuilder::new(&mut p7_buf);
    ping7.write_value(7);
    assert_eq!(
        root_client.r#match(&ping7).unwrap().doubled().unwrap(),
        107
    );
    assert_eq!(
        root_client.self_(&ping7).unwrap().doubled().unwrap(),
        207
    );
    let mut p9_buf = [0u8; 16];
    let mut ping9 = RootPingBuilder::new(&mut p9_buf);
    ping9.write_value(9);
    assert_eq!(
        block_on(root_client.match_async(&ping9)).unwrap().doubled().unwrap(),
        109
    );
    assert_eq!(
        block_on(root_client.self_async(&ping9)).unwrap().doubled().unwrap(),
        209
    );

    // 6. Keyword package (`self_.r#type.r#match`) and keyword service (`match`)
    let mut kw_req = [0u8; 64];
    let mut kw_resp = [0u8; 64];
    let mut kw_client = MatchClient::new(LoopbackChannel::new(
        MatchServer::new(MyKwMatchHandler),
        &mut kw_req,
        &mut kw_resp,
    ));
    let mut kw_b_buf = [0u8; 32];
    let mut kw_b = KwReqBuilder::new(&mut kw_b_buf);
    kw_b.write_type("hello-kw");
    assert_eq!(
        kw_client.r#fn(&kw_b).unwrap().r#match().unwrap(),
        "hello-kw"
    );
    let empty_kw = KwReqBuilder::new(&mut []);
    assert_eq!(
        kw_client.r#async(&empty_kw).unwrap().r#match().unwrap(),
        "async_ok"
    );

    // 7. Unknown method & unknown path -> GrpcCode::Unimplemented
    let srv = TelemetryServiceServer::new(MyTelemetryHandler { min_temp: 0 });
    let mut out_buf = [0u8; 128];
    let mut writer = BackwardWriter::new(&mut out_buf);
    let err_unimpl = srv.call_unary("UnknownMethod", &[], &mut writer).unwrap_err();
    assert_eq!(err_unimpl.code, GrpcCode::Unimplemented);

    let err_bad_path = srv.dispatch_path("/wrong.Service/ReportReading", &[], &mut writer).unwrap_err();
    assert_eq!(err_bad_path.code, GrpcCode::Unimplemented);

    let err_async_unimpl = block_on(srv.call_unary_async("UnknownMethod", &[], &mut writer)).unwrap_err();
    assert_eq!(err_async_unimpl.code, GrpcCode::Unimplemented);

    // 8. Corrupted request payload (missing required fields / invalid wire format) -> GrpcCode::Internal
    let err_corrupt = srv.call_unary("ReportReading", &[0xFF, 0xFF, 0xFF], &mut writer).unwrap_err();
    assert_eq!(err_corrupt.code, GrpcCode::Internal);

    let err_missing_req = srv.call_unary("ReportReading", &[], &mut writer).unwrap_err();
    assert_eq!(err_missing_req.code, GrpcCode::Internal);

    // 9. Buffer overflow on response writer -> GrpcCode::ResourceExhausted
    let mut tiny_req_buf = [0u8; 128];
    let mut tiny_resp_buf = [0u8; 4]; // too small for 5-byte LPM header + response
    let mut tiny_client = TelemetryServiceClient::new(LoopbackChannel::new(
        TelemetryServiceServer::new(MyTelemetryHandler { min_temp: 0 }),
        &mut tiny_req_buf,
        &mut tiny_resp_buf,
    ));
    let mut valid_req_buf = [0u8; 32];
    let mut valid_req = ReportRequestBuilder::new(&mut valid_req_buf);
    valid_req.write_temperature_c(10).write_device_id("dev");
    let err_exhausted = tiny_client.report_reading(&valid_req).unwrap_err();
    assert_eq!(err_exhausted.code, GrpcCode::ResourceExhausted);

    // 10. Generated clients over real HTTP/2 framing: one connection, two routed services
    let telemetry = TelemetryServiceServer::new(MyTelemetryHandler { min_temp: -40 });
    let health = HealthServiceServer::new(MyHealthHandler);
    let routes: [&dyn Route; 2] = [&telemetry, &health];
    let (mut msg, mut stx, mut ctx, mut call) = ([0u8; 512], [0u8; 4096], [0u8; 4096], [0u8; 512]);
    let server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
    let client: GrpcClient = GrpcClient::new("http", "localhost", &mut ctx);
    let mut h2_client = TelemetryServiceClient::new(H2LoopbackChannel::new(client, server, &mut call));

    let mut h2_req_buf = [0u8; 128];
    let mut h2_req = ReportRequestBuilder::new(&mut h2_req_buf);
    h2_req
        .write_meta(|m| {
            m.write_request_id(7).write_trace_id("h2-trace");
        })
        .write_temperature_c(21)
        .write_device_id("sensor-h2");
    let reply = h2_client
        .report_reading(&h2_req)
        .expect("report_reading over HTTP/2");
    assert!(reply.accepted().unwrap());
    assert_eq!(reply.echoed_meta().unwrap().unwrap().request_id().unwrap(), 8);

    let err = h2_client.report_reading(&bad_req).unwrap_err();
    assert_eq!(err.code, GrpcCode::InvalidArgument);
    assert_eq!(err.message, "temperature below minimum threshold");

    // Undecodable request payload (missing required fields & corrupted wire bytes) over H2LoopbackChannel -> Internal
    let empty_req = ReportRequestBuilder::new(&mut []);
    let err_missing = h2_client.report_reading(&empty_req).unwrap_err();
    assert_eq!(err_missing.code, GrpcCode::Internal);

    let err_corrupt = h2_client
        .report_reading(&[0xFFu8, 0xFF, 0xFF][..])
        .unwrap_err();
    assert_eq!(err_corrupt.code, GrpcCode::Internal);

    let mut h2_health = HealthServiceClient::new(h2_client.into_inner());
    assert!(h2_health.check(&hc_req).unwrap().healthy().unwrap());

    // A service the server does not register -> Unimplemented over the same connection
    let mut h2_root = RootEchoClient::new(h2_health.into_inner());
    let err = h2_root.echo(&ping21).unwrap_err();
    assert_eq!(err.code, GrpcCode::Unimplemented);

    // 11. Generated async client stubs (`AsyncUnaryChannel`) + `AsyncGrpcServer` over `MemoryDuplex`
    let async_router = (
        TelemetryServiceServer::new(MyTelemetryHandler { min_temp: -40 }),
        RootEchoServer::new(MyRootEchoHandler),
    );
    let duplex: roto_grpc::MemoryDuplex<1024> = roto_grpc::MemoryDuplex::new();
    let (client_io, server_io) = duplex.split();
    let client_io = client_io.with_max_chunk(29).with_pending_yields(true);
    let mut server_io = server_io.with_max_chunk(31).with_pending_yields(true);
    let (mut amsg, mut astx, mut actx, mut acall) = ([0u8; 512], [0u8; 4096], [0u8; 4096], [0u8; 512]);
    let mut async_srv: roto_grpc::AsyncGrpcServer<_> =
        roto_grpc::AsyncGrpcServer::new(&async_router, &mut amsg, &mut astx);
    let async_cli: GrpcClient = GrpcClient::new("http", "localhost", &mut actx);
    let ((), srv_res) = roto_grpc::block_on_pair(
        async move {
            let mut transport = roto_grpc::AsyncClientTransport::new(client_io, async_cli, &mut acall);
            {
                let mut stub = TelemetryServiceClient::new(&mut transport);
                let mut req_buf = [0u8; 128];
                let mut req = ReportRequestBuilder::new(&mut req_buf);
                req.write_meta(|m| {
                    m.write_request_id(50).write_trace_id("transport-async");
                })
                .write_temperature_c(18)
                .write_device_id("sensor-transport");
                let r = stub
                    .report_reading_async(&req)
                    .await
                    .expect("async transport report_reading_async");
                assert!(r.accepted().unwrap());
                assert_eq!(r.echoed_meta().unwrap().unwrap().request_id().unwrap(), 51);
            }
            {
                let mut root_stub = RootEchoClient::new(&mut transport);
                let mut p_buf = [0u8; 16];
                let mut p15 = RootPingBuilder::new(&mut p_buf);
                p15.write_value(15);
                let pong = root_stub.match_async(&p15).await.unwrap();
                assert_eq!(pong.doubled().unwrap(), 115);
            }
        },
        roto_grpc::serve_async_connection(&mut async_srv, &mut server_io),
    );
    assert!(srv_res.is_ok());
}
"#,
    )
    .unwrap();

    let compile_runner = Command::new("rustc")
        .arg("--edition=2021")
        .arg("-D")
        .arg("warnings")
        .arg(&runner_src)
        .arg("--extern")
        .arg(format!("roto={}", rlibs.roto.display()))
        .arg("--extern")
        .arg(format!("roto_runtime={}", rlibs.runtime.display()))
        .arg("--extern")
        .arg(format!("roto_grpc={}", rlibs.grpc.display()))
        .arg("--extern")
        .arg(format!("common_proto={}", common_rlib.display()))
        .arg("--extern")
        .arg(format!("telemetry_proto={}", telemetry_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&runner_bin)
        .output()
        .expect("compile runner");
    assert!(
        compile_runner.status.success(),
        "runner compile failed:\n{}",
        String::from_utf8_lossy(&compile_runner.stderr)
    );

    let run_out = Command::new(&runner_bin).output().expect("execute runner");
    assert!(
        run_out.status.success(),
        "runner failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_out.stdout),
        String::from_utf8_lossy(&run_out.stderr)
    );

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

#[test]
fn rejects_invalid_service_descriptors_and_parameters_cleanly() {
    // 1. Enum used as RPC input_type is rejected
    {
        let mut buf = vec![0u8; 4096];
        let mut req = CodeGeneratorRequestBuilder::new(&mut buf);
        req.write_proto_file(|fb| {
            fb.write_service(|svc| {
                svc.write_method(|m| {
                    m.write_output_type(".pkg.Msg")
                        .write_input_type(".pkg.MyEnum")
                        .write_name("BadRpc");
                })
                .write_name("BadSvc");
            })
            .write_enum_type(|en| {
                en.write_value(|v| {
                    v.write_number(0).write_name("ZERO");
                })
                .write_name("MyEnum");
            })
            .write_message_type(|msg| {
                msg.write_name("Msg");
            })
            .write_package("pkg")
            .write_name("bad.proto");
        })
        .write_file_to_generate("bad.proto");

        let req_view = CodeGeneratorRequestView::from_bytes(req.finish().unwrap()).unwrap();
        let err = generate_request(&req_view).unwrap_err();
        assert!(
            err.contains("must be a message, not an enum"),
            "unexpected error: {err}"
        );
    }

    // 2. Duplicate method name in service is rejected
    {
        let mut buf = vec![0u8; 4096];
        let mut req = CodeGeneratorRequestBuilder::new(&mut buf);
        req.write_proto_file(|fb| {
            fb.write_service(|svc| {
                svc.write_method(|m| {
                    m.write_output_type(".pkg.Msg")
                        .write_input_type(".pkg.Msg")
                        .write_name("Ping");
                })
                .write_method(|m| {
                    m.write_output_type(".pkg.Msg")
                        .write_input_type(".pkg.Msg")
                        .write_name("ping");
                })
                .write_name("DupMethodSvc");
            })
            .write_message_type(|msg| {
                msg.write_name("Msg");
            })
            .write_package("pkg")
            .write_name("dup.proto");
        })
        .write_file_to_generate("dup.proto");

        let req_view = CodeGeneratorRequestView::from_bytes(req.finish().unwrap()).unwrap();
        let err = generate_request(&req_view).unwrap_err();
        assert!(
            err.contains("duplicate method name"),
            "unexpected error: {err}"
        );
    }

    // 3. `error=...` parameter surfaces via `CodeGeneratorResponse.error` in `process_bytes`
    {
        let mut buf = vec![0u8; 4096];
        let mut req = CodeGeneratorRequestBuilder::new(&mut buf);
        req.write_parameter("error=custom_grpc_error")
            .write_proto_file(|fb| {
                fb.write_message_type(|msg| {
                    msg.write_field(|f| {
                        f.write_type(FieldType::TypeInt32)
                            .write_label(FieldLabel::LabelOptional)
                            .write_number(1)
                            .write_name("x");
                    })
                    .write_name("Msg");
                })
                .write_name("a.proto");
            })
            .write_file_to_generate("a.proto");

        let resp_bytes = process_bytes(req.finish().unwrap()).unwrap();
        let resp_view = CodeGeneratorResponseView::from_bytes(&resp_bytes).unwrap();
        assert_eq!(resp_view.error().unwrap(), "custom_grpc_error");
    }
}

#[test]
fn e2e_opt_in_unstripped_file_descriptors_roundtrip_and_route_exposure() {
    use google_protobuf_roto_bootstrap::google::protobuf::{
        FileDescriptorProtoView, FileDescriptorSetView,
    };

    let tmp_dir = create_temp_dir("opt_in_descriptors");
    let proto_dir = tmp_dir.join("proto");
    std::fs::create_dir_all(proto_dir.join("common/v1")).unwrap();
    std::fs::create_dir_all(proto_dir.join("telemetry/v1")).unwrap();

    std::fs::write(proto_dir.join("common/v1/common.proto"), COMMON_PROTO).unwrap();
    std::fs::write(
        proto_dir.join("telemetry/v1/telemetry.proto"),
        "// Leading comment preserved in source_code_info\n".to_string() + TELEMETRY_PROTO,
    )
    .unwrap();

    let fds_out = Command::new("protoc")
        .arg(format!("--proto_path={}", proto_dir.display()))
        .arg("--include_imports")
        .arg("--include_source_info")
        .arg("--descriptor_set_out=/dev/stdout")
        .arg("telemetry/v1/telemetry.proto")
        .output()
        .expect("protoc descriptor_set_out");
    assert!(fds_out.status.success());

    let fds = FileDescriptorSetView::from_bytes(&fds_out.stdout).unwrap();
    let raw_files: Vec<FileDescriptorProtoView<'_>> =
        fds.file().map(|r| r.unwrap()).collect();
    assert_eq!(raw_files.len(), 2);
    let common_fd = raw_files
        .iter()
        .find(|f| f.name() == Ok("common/v1/common.proto"))
        .unwrap();
    let telemetry_fd = raw_files
        .iter()
        .find(|f| f.name() == Ok("telemetry/v1/telemetry.proto"))
        .unwrap();
    assert!(telemetry_fd.has_source_code_info());

    let mut req_buf = vec![0u8; 65536];
    let req_bytes = {
        let mut b = CodeGeneratorRequestBuilder::new(&mut req_buf);
        b.write_file_to_generate("telemetry/v1/telemetry.proto")
            .write_file_to_generate("common/v1/common.proto")
            .write_proto_file(|fb| {
                fb.copy_unset_fields(telemetry_fd);
            })
            .write_proto_file(|fb| {
                fb.copy_unset_fields(common_fd);
            })
            .write_parameter("mode=lib,descriptors=true");
        b.finish().unwrap().to_vec()
    };

    let mut grpc_req_buf = vec![0u8; 65536];
    let grpc_req_bytes = {
        let mut b = CodeGeneratorRequestBuilder::new(&mut grpc_req_buf);
        b.write_file_to_generate("telemetry/v1/telemetry.proto")
            .write_file_to_generate("common/v1/common.proto")
            .write_proto_file(|fb| {
                fb.copy_unset_fields(telemetry_fd);
            })
            .write_proto_file(|fb| {
                fb.copy_unset_fields(common_fd);
            })
            .write_parameter("mode=mod,file=grpc.rs,descriptors=true");
        b.finish().unwrap().to_vec()
    };

    let req_view = CodeGeneratorRequestView::from_bytes(&req_bytes).unwrap();
    let grpc_req_view = CodeGeneratorRequestView::from_bytes(&grpc_req_bytes).unwrap();
    let roto_files = protoc_gen_roto::generate_request(&req_view).unwrap();
    let grpc_files = generate_request(&grpc_req_view).unwrap();

    assert_eq!(roto_files.len(), 1);
    assert_eq!(grpc_files.len(), 1);

    // Default (descriptors not set) must not emit FILE_DESCRIPTOR_BYTES
    let mut default_req_buf = vec![0u8; 65536];
    let default_req_bytes = {
        let mut b = CodeGeneratorRequestBuilder::new(&mut default_req_buf);
        b.write_file_to_generate("telemetry/v1/telemetry.proto")
            .write_file_to_generate("common/v1/common.proto")
            .write_proto_file(|fb| {
                fb.copy_unset_fields(telemetry_fd);
            })
            .write_proto_file(|fb| {
                fb.copy_unset_fields(common_fd);
            })
            .write_parameter("mode=lib");
        b.finish().unwrap().to_vec()
    };
    let default_req_view = CodeGeneratorRequestView::from_bytes(&default_req_bytes).unwrap();
    let default_roto = protoc_gen_roto::generate_request(&default_req_view).unwrap();
    let default_grpc = generate_request(&default_req_view).unwrap();
    assert!(!default_roto[0].content.contains("FILE_DESCRIPTOR_BYTES"));
    assert!(!default_grpc[0].content.contains("FILE_DESCRIPTOR_BYTES"));

    // Compile the descriptors=true generated crate and verify unstripped descriptors at runtime
    let build_dir = tmp_dir.join("build");
    std::fs::create_dir_all(&build_dir).unwrap();
    let rlibs = build_roto_and_grpc_rlibs(&build_dir);

    let bootstrap_rlib = build_dir.join("libgoogle_protobuf_roto_bootstrap.rlib");
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let status = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=google_protobuf_roto_bootstrap")
        .arg(workspace_root.join("google-protobuf-roto-bootstrap/src/lib.rs"))
        .arg("--extern")
        .arg(format!("roto={}", rlibs.roto.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&bootstrap_rlib)
        .output()
        .unwrap();
    assert!(status.status.success());

    let gen_dir = tmp_dir.join("gen");
    std::fs::create_dir_all(&gen_dir).unwrap();
    let mut lib_content = roto_files[0].content.clone();
    lib_content.push_str("\npub mod grpc;\n");
    std::fs::write(gen_dir.join("lib.rs"), &lib_content).unwrap();
    std::fs::write(gen_dir.join("grpc.rs"), &grpc_files[0].content).unwrap();

    let gen_rlib = build_dir.join("libdesc_test.rlib");
    let compile_gen = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=desc_test")
        .arg("-D")
        .arg("warnings")
        .arg(gen_dir.join("lib.rs"))
        .arg("--extern")
        .arg(format!("roto={}", rlibs.roto.display()))
        .arg("--extern")
        .arg(format!("roto_grpc={}", rlibs.grpc.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&gen_rlib)
        .output()
        .unwrap();
    assert!(
        compile_gen.status.success(),
        "rustc desc_test failed: {}",
        String::from_utf8_lossy(&compile_gen.stderr)
    );

    let runner_rs = tmp_dir.join("runner.rs");
    let telemetry_expected_lit =
        protoc_gen_roto::generator::modules::format_bytes_literal(telemetry_fd.as_bytes());
    let common_expected_lit =
        protoc_gen_roto::generator::modules::format_bytes_literal(common_fd.as_bytes());
    std::fs::write(
        &runner_rs,
        format!(
            r#"
use desc_test::grpc::telemetry::v1::telemetry_service::{{
    TelemetryService, TelemetryServiceServer, FILE_DESCRIPTOR_BYTES, FILE_DESCRIPTOR_SET,
}};
use desc_test::telemetry::v1::{{ReportRequestView, ReportResponseBuilder}};
use google_protobuf_roto_bootstrap::google::protobuf::FileDescriptorProtoView;
use roto_grpc::{{GrpcService, GrpcStatus, Route}};

struct Dummy;
impl TelemetryService for Dummy {{
    fn report_reading<'a, 'b>(
        &self,
        _req: ReportRequestView<'a>,
        _resp: &mut ReportResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {{
        Ok(())
    }}
}}

fn main() {{
    let expected_telemetry: &[u8] = {telemetry_expected_lit};
    let expected_common: &[u8] = {common_expected_lit};

    assert_eq!(FILE_DESCRIPTOR_BYTES, expected_telemetry);
    assert_eq!(desc_test::telemetry::v1::FILE_DESCRIPTOR_BYTES, expected_telemetry);
    assert_eq!(desc_test::common::v1::FILE_DESCRIPTOR_BYTES, expected_common);
    assert_eq!(desc_test::FILE_DESCRIPTORS, &[expected_common, expected_telemetry]);
    assert_eq!(FILE_DESCRIPTOR_SET, &[expected_telemetry, expected_common]);

    let svc = TelemetryServiceServer::new(Dummy);
    let route: &dyn Route = &svc;
    assert_eq!(route.file_descriptors(), &[expected_telemetry, expected_common]);
    assert_eq!(
        <TelemetryServiceServer<Dummy> as GrpcService>::FILE_DESCRIPTOR_SET,
        &[expected_telemetry, expected_common]
    );

    // Round-trip decode and verify source_code_info was preserved unstripped
    let fd_view = FileDescriptorProtoView::from_bytes(FILE_DESCRIPTOR_BYTES).unwrap();
    assert_eq!(fd_view.name(), Ok("telemetry/v1/telemetry.proto"));
    assert!(fd_view.has_source_code_info());
}}
"#
        ),
    )
    .unwrap();

    let runner_bin = build_dir.join("runner");
    let compile_runner = Command::new("rustc")
        .arg("--edition=2021")
        .arg(&runner_rs)
        .arg("--extern")
        .arg(format!("desc_test={}", gen_rlib.display()))
        .arg("--extern")
        .arg(format!(
            "google_protobuf_roto_bootstrap={}",
            bootstrap_rlib.display()
        ))
        .arg("--extern")
        .arg(format!("roto={}", rlibs.roto.display()))
        .arg("--extern")
        .arg(format!("roto_grpc={}", rlibs.grpc.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&runner_bin)
        .output()
        .unwrap();
    assert!(
        compile_runner.status.success(),
        "rustc runner failed: {}",
        String::from_utf8_lossy(&compile_runner.stderr)
    );

    let run_out = Command::new(&runner_bin).output().unwrap();
    assert!(
        run_out.status.success(),
        "runner failed: {}",
        String::from_utf8_lossy(&run_out.stderr)
    );

    let _ = std::fs::remove_dir_all(&tmp_dir);
}
