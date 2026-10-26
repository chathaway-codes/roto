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
        .arg("--cfg")
        .arg("feature=\"std\"")
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
    rpc WatchReadings (ReportRequest) returns (stream ReportResponse);
    rpc RecordReadings (stream ReportRequest) returns (ReportResponse);
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
        let fds =
            google_protobuf_roto_bootstrap::google::protobuf::FileDescriptorSetView::from_bytes(
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
        let fds =
            google_protobuf_roto_bootstrap::google::protobuf::FileDescriptorSetView::from_bytes(
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
    assert!(grpc_rs_content
        .contains("pub const SERVICE_NAME: &str = \"telemetry.v1.TelemetryService\";"));
    assert!(grpc_rs_content.contains(
        "pub const PATH_REPORT_READING: &str = \"/telemetry.v1.TelemetryService/ReportReading\";"
    ));
    assert!(grpc_rs_content.contains(
        "pub const PATH_WATCH_READINGS: &str = \"/telemetry.v1.TelemetryService/WatchReadings\";"
    ));
    assert!(grpc_rs_content.contains(
        "pub const PATH_RECORD_READINGS: &str = \"/telemetry.v1.TelemetryService/RecordReadings\";"
    ));
    assert!(grpc_rs_content.contains(
        "pub const PATH_STREAM_READINGS: &str = \"/telemetry.v1.TelemetryService/StreamReadings\";"
    ));
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
    GrpcClient, GrpcCode, GrpcServer, GrpcStatus, H2LoopbackChannel, LoopbackChannel, RecvStream,
    Route, RpcContext, RpcMetadata, SendStream,
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
        meta: &mut RpcMetadata<'_>,
        req: ReportRequestView<'a>,
        resp: &mut ReportResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        if let Some(token) = meta.request().get_str("x-auth-token") {
            meta.response_headers_mut().insert("x-echo-token", token)?;
            meta.response_trailers_mut().insert("x-stage", "committed")?;
        }
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

    fn watch_readings<'a, 'b>(
        &self,
        _meta: &mut RpcMetadata<'_>,
        req: ReportRequestView<'a>,
        tx: &mut impl SendStream<ReportResponseBuilder<'b>>,
    ) -> Result<(), GrpcStatus<'static>> {
        let count = req.temperature_c().map_err(|_| GrpcStatus::internal("decode count"))?;
        if count < 0 {
            return Err(GrpcStatus::invalid_argument("negative watch count"));
        }
        let dev = req.device_id().map_err(|_| GrpcStatus::internal("decode device_id"))?;
        for i in 0..count {
            tx.send(|resp| {
                resp.write_echoed_meta(|mb| {
                    mb.write_request_id(i as u64 + 1).write_trace_id(dev);
                })
                .write_status_msg(dev)
                .write_accepted(true);
            })?;
        }
        Ok(())
    }

    fn record_readings<'a, 'b>(
        &self,
        _meta: &mut RpcMetadata<'_>,
        rx: &mut impl RecvStream<ReportRequestView<'a>>,
        resp: &mut ReportResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let mut sum: i64 = 0;
        let mut count: u64 = 0;
        while let Some(req) = rx.recv()? {
            let temp = req.temperature_c().map_err(|_| GrpcStatus::internal("decode temp"))?;
            if temp < self.min_temp {
                return Err(GrpcStatus::invalid_argument("temperature below minimum threshold"));
            }
            sum += i64::from(temp);
            count += 1;
        }
        resp.write_echoed_meta(|mb| {
            mb.write_request_id(sum as u64).write_trace_id("summary");
        })
        .write_status_msg("batch-ok")
        .write_accepted(count > 0);
        Ok(())
    }

    fn stream_readings<'a, 'b>(
        &self,
        meta: &mut RpcMetadata<'_>,
        rx: &mut impl RecvStream<ReportRequestView<'a>>,
        tx: &mut impl SendStream<ReportResponseBuilder<'b>>,
    ) -> Result<(), GrpcStatus<'static>> {
        while let Some(req) = rx.recv()? {
            <Self as TelemetryService>::report_reading(
                self,
                meta,
                req,
                &mut ReportResponseBuilder::new(&mut [0u8; 64]),
            )?;
            let temp = req.temperature_c().map_err(|_| GrpcStatus::internal("decode temp"))?;
            let dev = req.device_id().map_err(|_| GrpcStatus::internal("decode dev"))?;
            tx.send(|resp| {
                resp.write_echoed_meta(|mb| {
                    mb.write_request_id(temp as u64).write_trace_id(dev);
                })
                .write_status_msg(dev)
                .write_accepted(true);
            })?;
        }
        Ok(())
    }
}

impl AsyncTelemetryService for MyTelemetryHandler {
    async fn report_reading<'a, 'b>(
        &'a self,
        meta: &'a mut RpcMetadata<'_>,
        req: ReportRequestView<'a>,
        resp: &'a mut ReportResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as TelemetryService>::report_reading(self, meta, req, resp)
    }

    async fn watch_readings<'a, 'b>(
        &'a self,
        meta: &'a mut RpcMetadata<'_>,
        req: ReportRequestView<'a>,
        tx: &'a mut impl SendStream<ReportResponseBuilder<'b>>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as TelemetryService>::watch_readings(self, meta, req, tx)
    }

    async fn record_readings<'a, 'r, 'b>(
        &'a self,
        meta: &'a mut RpcMetadata<'_>,
        rx: &'a mut impl RecvStream<ReportRequestView<'r>>,
        resp: &'a mut ReportResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as TelemetryService>::record_readings(self, meta, rx, resp)
    }

    async fn stream_readings<'a, 'r, 'b>(
        &'a self,
        meta: &'a mut RpcMetadata<'_>,
        rx: &'a mut impl RecvStream<ReportRequestView<'r>>,
        tx: &'a mut impl SendStream<ReportResponseBuilder<'b>>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as TelemetryService>::stream_readings(self, meta, rx, tx)
    }
}

struct MyHealthHandler;

impl HealthService for MyHealthHandler {
    fn check<'a, 'b>(
        &self,
        _meta: &mut RpcMetadata<'_>,
        _req: HealthCheckRequestView<'a>,
        resp: &mut HealthCheckResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        resp.write_healthy(true);
        Ok(())
    }

    fn check_meta<'a, 'b>(
        &self,
        _meta: &mut RpcMetadata<'_>,
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
        _meta: &mut RpcMetadata<'_>,
        req: RootPingView<'a>,
        resp: &mut RootPongBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let v = req.value().map_err(|_| GrpcStatus::internal("decode value"))?;
        resp.write_doubled(v * 2);
        Ok(())
    }

    fn r#match<'a, 'b>(
        &self,
        _meta: &mut RpcMetadata<'_>,
        req: RootPingView<'a>,
        resp: &mut RootPongBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let v = req.value().map_err(|_| GrpcStatus::internal("decode value"))?;
        resp.write_doubled(v + 100);
        Ok(())
    }

    fn self_<'a, 'b>(
        &self,
        _meta: &mut RpcMetadata<'_>,
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
        meta: &'a mut RpcMetadata<'_>,
        req: RootPingView<'a>,
        resp: &'a mut RootPongBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as RootEcho>::echo(self, meta, req, resp)
    }

    async fn r#match<'a, 'b>(
        &'a self,
        meta: &'a mut RpcMetadata<'_>,
        req: RootPingView<'a>,
        resp: &'a mut RootPongBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as RootEcho>::r#match(self, meta, req, resp)
    }

    async fn self_<'a, 'b>(
        &'a self,
        meta: &'a mut RpcMetadata<'_>,
        req: RootPingView<'a>,
        resp: &'a mut RootPongBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as RootEcho>::self_(self, meta, req, resp)
    }
}

struct MyKwMatchHandler;

impl Match for MyKwMatchHandler {
    fn r#fn<'a, 'b>(
        &self,
        _meta: &mut RpcMetadata<'_>,
        req: KwReqView<'a>,
        resp: &mut KwRespBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let t = req.r#type().map_err(|_| GrpcStatus::internal("decode type"))?;
        resp.write_match(t);
        Ok(())
    }

    fn r#async<'a, 'b>(
        &self,
        _meta: &mut RpcMetadata<'_>,
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
    let mut rpc_ctx: RpcContext = RpcContext::new();
    let err_unimpl = srv.call_unary(&mut rpc_ctx.as_metadata(), "UnknownMethod", &[], &mut writer).unwrap_err();
    assert_eq!(err_unimpl.code, GrpcCode::Unimplemented);

    let err_bad_path = srv.dispatch_path(&mut rpc_ctx.as_metadata(), "/wrong.Service/ReportReading", &[], &mut writer).unwrap_err();
    assert_eq!(err_bad_path.code, GrpcCode::Unimplemented);

    let err_async_unimpl = block_on(srv.call_unary_async(&mut rpc_ctx.as_metadata(), "UnknownMethod", &[], &mut writer)).unwrap_err();
    assert_eq!(err_async_unimpl.code, GrpcCode::Unimplemented);

    // 8. Corrupted request payload (missing required fields / invalid wire format) -> GrpcCode::Internal
    let err_corrupt = srv.call_unary(&mut rpc_ctx.as_metadata(), "ReportReading", &[0xFF, 0xFF, 0xFF], &mut writer).unwrap_err();
    assert_eq!(err_corrupt.code, GrpcCode::Internal);

    let err_missing_req = srv.call_unary(&mut rpc_ctx.as_metadata(), "ReportReading", &[], &mut writer).unwrap_err();
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
    h2_client.insert_metadata("x-auth-token", "secret-token-42").expect("insert request metadata");
    let reply = h2_client
        .report_reading(&h2_req)
        .expect("report_reading over HTTP/2");
    assert!(reply.accepted().unwrap());
    assert_eq!(reply.echoed_meta().unwrap().unwrap().request_id().unwrap(), 8);
    assert_eq!(h2_client.response_headers().get_str("x-echo-token"), Some("secret-token-42"));
    assert_eq!(h2_client.response_trailers().get_str("x-stage"), Some("committed"));

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

    // Server-streaming, client-streaming, and bidirectional-streaming over HTTP/2 (`H2LoopbackChannel`)
    {
        // Server-streaming (3 messages)
        let mut watch_rx = h2_client
            .watch_readings_with(|r| {
                r.write_temperature_c(3).write_device_id("watch-dev");
            })
            .expect("watch_readings over HTTP/2");
        for expected_id in 1..=3 {
            let item = watch_rx.recv().unwrap().expect("expected watch item");
            assert!(item.accepted().unwrap());
            assert_eq!(item.status_msg().unwrap(), "watch-dev");
            assert_eq!(item.echoed_meta().unwrap().unwrap().request_id().unwrap(), expected_id);
        }
        assert!(watch_rx.recv().unwrap().is_none());
    }
    {
        // Server-streaming (0 messages)
        let mut empty_watch = h2_client
            .watch_readings_with(|r| {
                r.write_temperature_c(0).write_device_id("watch-zero");
            })
            .expect("watch_readings 0 messages");
        assert!(empty_watch.recv().unwrap().is_none());
    }
    {
        // Server-streaming error status propagation
        let err_watch = h2_client
            .watch_readings_with(|r| {
                r.write_temperature_c(-1).write_device_id("watch-neg");
            })
            .err()
            .expect("negative watch count should fail");
        assert_eq!(err_watch.code, GrpcCode::InvalidArgument);
    }
    {
        // Client-streaming (3 messages -> 1 summary response)
        let summary = h2_client
            .record_readings(|tx| {
                for t in [10, 20, 30] {
                    tx.send(|req| {
                        req.write_temperature_c(t).write_device_id("batch-dev");
                    })?;
                }
                Ok(())
            })
            .expect("record_readings over HTTP/2");
        assert!(summary.accepted().unwrap());
        assert_eq!(summary.status_msg().unwrap(), "batch-ok");
        assert_eq!(summary.echoed_meta().unwrap().unwrap().request_id().unwrap(), 60);
    }
    {
        // Client-streaming (0 messages -> accepted == false)
        let empty_summary = h2_client
            .record_readings(|_tx| Ok(()))
            .expect("record_readings 0 messages");
        assert!(!empty_summary.accepted().unwrap());
        assert_eq!(empty_summary.echoed_meta().unwrap().unwrap().request_id().unwrap(), 0);
    }
    {
        // Bidirectional-streaming (3 request messages -> 3 response messages in forward order)
        let mut bidi_rx = h2_client
            .stream_readings(|tx| {
                for (dev, t) in [("d1", 11), ("d2", 22), ("d3", 33)] {
                    tx.send(|req| {
                        req.write_temperature_c(t).write_device_id(dev);
                    })?;
                }
                Ok(())
            })
            .expect("stream_readings over HTTP/2");
        for (expected_dev, expected_temp) in [("d1", 11u64), ("d2", 22), ("d3", 33)] {
            let resp = bidi_rx.recv().unwrap().expect("expected bidi item");
            assert!(resp.accepted().unwrap());
            assert_eq!(resp.status_msg().unwrap(), expected_dev);
            assert_eq!(resp.echoed_meta().unwrap().unwrap().request_id().unwrap(), expected_temp);
        }
        assert!(bidi_rx.recv().unwrap().is_none());
    }

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

                // Async server-streaming
                let mut watch_rx = stub
                    .watch_readings_with_async(|r| {
                        r.write_temperature_c(2).write_device_id("async-watch");
                    })
                    .await
                    .expect("async watch_readings");
                let w1 = watch_rx.recv().unwrap().unwrap();
                assert_eq!(w1.echoed_meta().unwrap().unwrap().request_id().unwrap(), 1);
                let w2 = watch_rx.recv().unwrap().unwrap();
                assert_eq!(w2.echoed_meta().unwrap().unwrap().request_id().unwrap(), 2);
                assert!(watch_rx.recv().unwrap().is_none());

                // Async client-streaming
                let rec = stub
                    .record_readings_async(|tx| {
                        tx.send(|r| {
                            r.write_temperature_c(7).write_device_id("c1");
                        })?;
                        tx.send(|r| {
                            r.write_temperature_c(8).write_device_id("c2");
                        })?;
                        Ok(())
                    })
                    .await
                    .expect("async record_readings");
                assert_eq!(rec.echoed_meta().unwrap().unwrap().request_id().unwrap(), 15);

                // Async bidi-streaming
                let mut bidi_rx = stub
                    .stream_readings_async(|tx| {
                        tx.send(|r| {
                            r.write_temperature_c(3).write_device_id("b1");
                        })?;
                        tx.send(|r| {
                            r.write_temperature_c(4).write_device_id("b2");
                        })?;
                        Ok(())
                    })
                    .await
                    .expect("async stream_readings");
                let b1 = bidi_rx.recv().unwrap().unwrap();
                assert_eq!(b1.status_msg().unwrap(), "b1");
                assert_eq!(b1.echoed_meta().unwrap().unwrap().request_id().unwrap(), 3);
                let b2 = bidi_rx.recv().unwrap().unwrap();
                assert_eq!(b2.status_msg().unwrap(), "b2");
                assert_eq!(b2.echoed_meta().unwrap().unwrap().request_id().unwrap(), 4);
                assert!(bidi_rx.recv().unwrap().is_none());
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
    let raw_files: Vec<FileDescriptorProtoView<'_>> = fds.file().map(|r| r.unwrap()).collect();
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
use roto_grpc::{{GrpcService, GrpcStatus, RecvStream, Route, SendStream}};

struct Dummy;
impl TelemetryService for Dummy {{
    fn report_reading<'a, 'b>(
        &self,
        _meta: &mut roto_grpc::RpcMetadata<'_>,
        _req: ReportRequestView<'a>,
        _resp: &mut ReportResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {{
        Ok(())
    }}

    fn watch_readings<'a, 'b>(
        &self,
        _meta: &mut roto_grpc::RpcMetadata<'_>,
        _req: ReportRequestView<'a>,
        _tx: &mut impl SendStream<ReportResponseBuilder<'b>>,
    ) -> Result<(), GrpcStatus<'static>> {{
        Ok(())
    }}

    fn record_readings<'a, 'b>(
        &self,
        _meta: &mut roto_grpc::RpcMetadata<'_>,
        _rx: &mut impl RecvStream<ReportRequestView<'a>>,
        _resp: &mut ReportResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {{
        Ok(())
    }}

    fn stream_readings<'a, 'b>(
        &self,
        _meta: &mut roto_grpc::RpcMetadata<'_>,
        _rx: &mut impl RecvStream<ReportRequestView<'a>>,
        _tx: &mut impl SendStream<ReportResponseBuilder<'b>>,
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

#[test]
fn e2e_opt_in_http_route_and_protobuf_binding_sync_and_async() {
    use google_protobuf_roto_bootstrap::google::protobuf::{
        FileDescriptorProtoView, FileDescriptorSetView,
    };

    let tmp_dir = create_temp_dir("http_route_codegen");
    let proto_dir = tmp_dir.join("proto");
    std::fs::create_dir_all(&proto_dir).unwrap();

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let bundled_proto_dir = workspace_root.join("google-protobuf-roto/proto");

    let shelf_proto = r#"syntax = "proto3";
package acme.shelf.v1;

import "google/api/annotations.proto";

enum ShelfTheme {
    SHELF_THEME_UNSPECIFIED = 0;
    SHELF_THEME_SCIFI = 1;
    SHELF_THEME_HISTORY = 2;
}

message Shelf {
    string id = 1;
    string title = 2;
    ShelfTheme theme = 3;
    int32 capacity = 4;
}

message Book {
    string id = 1;
    string author = 2;
    uint32 pages = 3;
}

message GetShelfRequest {
    Shelf shelf = 1;
    repeated string tags = 2;
    bool include_archived = 3;
}

message CreateBookRequest {
    string shelf_id = 1;
    Book book = 2;
}

message UpdateBookRequest {
    string shelf_id = 1;
    Book book = 2;
    string extra_note = 3;
}

message UpdateBookResponse {
    Book book = 1;
    string status = 2;
}

message DeleteShelfRequest {
    string shelf_id = 1;
    int64 purge_after_sec = 2;
}

message DeleteShelfResponse {
    bool deleted = 1;
}

service ShelfService {
    rpc GetShelf(GetShelfRequest) returns (Shelf) {
        option (google.api.http) = {
            get: "/v1/shelves/{shelf.id}/themes/{shelf.theme}"
            additional_bindings {
                get: "/v1/{shelf.id=orgs/*/shelves/**}:inspect"
            }
        };
    }

    rpc CreateBook(CreateBookRequest) returns (Book) {
        option (google.api.http) = {
            post: "/v1/books"
            body: "*"
            additional_bindings {
                post: "/v1/shelves/{shelf_id}/books"
                body: "book"
            }
        };
    }

    rpc UpdateBook(UpdateBookRequest) returns (UpdateBookResponse) {
        option (google.api.http) = {
            patch: "/v1/shelves/{shelf_id}/books/{book.id}"
            body: "book"
            response_body: "book"
            additional_bindings {
                put: "/v1/shelves/{shelf_id}/books/{book.id}"
                body: "*"
            }
            additional_bindings {
                custom: {
                    kind: "MERGE"
                    path: "/v1/shelves/{shelf_id}:merge"
                }
                body: "extra_note"
                response_body: "status"
            }
        };
    }

    rpc DeleteShelf(DeleteShelfRequest) returns (DeleteShelfResponse) {
        option (google.api.http) = {
            delete: "/v1/shelves/{shelf_id}"
        };
    }
}
"#;
    std::fs::write(proto_dir.join("shelf.proto"), shelf_proto).unwrap();

    let fds_out = Command::new("protoc")
        .arg(format!("--proto_path={}", proto_dir.display()))
        .arg(format!("--proto_path={}", bundled_proto_dir.display()))
        .arg("--include_imports")
        .arg("--descriptor_set_out=/dev/stdout")
        .arg("shelf.proto")
        .output()
        .expect("protoc shelf.proto");
    assert!(
        fds_out.status.success(),
        "protoc failed: {}",
        String::from_utf8_lossy(&fds_out.stderr)
    );

    let fds = FileDescriptorSetView::from_bytes(&fds_out.stdout).unwrap();
    let raw_files: Vec<FileDescriptorProtoView<'_>> = fds.file().map(|r| r.unwrap()).collect();

    let mut roto_req_buf = vec![0u8; 131072];
    let roto_req_bytes = {
        let mut b = CodeGeneratorRequestBuilder::new(&mut roto_req_buf);
        b.write_file_to_generate("shelf.proto");
        for fd in raw_files.iter().rev() {
            b.write_proto_file(|fb| {
                fb.copy_unset_fields(fd);
            });
        }
        b.write_parameter("mode=lib");
        b.finish().unwrap().to_vec()
    };

    let mut default_grpc_req_buf = vec![0u8; 131072];
    let default_grpc_req_bytes = {
        let mut b = CodeGeneratorRequestBuilder::new(&mut default_grpc_req_buf);
        b.write_file_to_generate("shelf.proto");
        for fd in raw_files.iter().rev() {
            b.write_proto_file(|fb| {
                fb.copy_unset_fields(fd);
            });
        }
        b.write_parameter("mode=mod,file=grpc.rs");
        b.finish().unwrap().to_vec()
    };

    let mut http_grpc_req_buf = vec![0u8; 131072];
    let http_grpc_req_bytes = {
        let mut b = CodeGeneratorRequestBuilder::new(&mut http_grpc_req_buf);
        b.write_file_to_generate("shelf.proto");
        for fd in raw_files.iter().rev() {
            b.write_proto_file(|fb| {
                fb.copy_unset_fields(fd);
            });
        }
        b.write_parameter("mode=mod,file=grpc.rs,http=true");
        b.finish().unwrap().to_vec()
    };

    let roto_req_view = CodeGeneratorRequestView::from_bytes(&roto_req_bytes).unwrap();
    let default_grpc_view = CodeGeneratorRequestView::from_bytes(&default_grpc_req_bytes).unwrap();
    let http_grpc_view = CodeGeneratorRequestView::from_bytes(&http_grpc_req_bytes).unwrap();

    let roto_files = protoc_gen_roto::generate_request(&roto_req_view).unwrap();
    let default_grpc_files = generate_request(&default_grpc_view).unwrap();
    let http_grpc_files = generate_request(&http_grpc_view).unwrap();

    assert!(!default_grpc_files[0].content.contains("HttpService"));
    assert!(!default_grpc_files[0].content.contains("AsyncHttpService"));
    assert!(http_grpc_files[0].content.contains("HttpService"));
    assert!(http_grpc_files[0].content.contains("AsyncHttpService"));
    // Verify CreateBook (`body: "*"` with no path vars) uses zero-copy fast path `from_bytes(req.body)`
    assert!(
        http_grpc_files[0]
            .content
            .contains("CreateBookRequestView::from_bytes(req.body)"),
        "expected zero-copy fast path for CreateBook"
    );

    let build_dir = tmp_dir.join("build");
    std::fs::create_dir_all(&build_dir).unwrap();
    let rlibs = build_roto_and_grpc_rlibs(&build_dir);

    let gen_dir = tmp_dir.join("gen");
    std::fs::create_dir_all(&gen_dir).unwrap();
    let mut lib_content = roto_files[0].content.clone();
    lib_content.push_str("\npub mod grpc;\n");
    std::fs::write(gen_dir.join("lib.rs"), &lib_content).unwrap();
    std::fs::write(gen_dir.join("grpc.rs"), &http_grpc_files[0].content).unwrap();

    let gen_rlib = build_dir.join("libshelf_http.rlib");
    let compile_gen = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=shelf_http")
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
        "rustc shelf_http failed:\n{}",
        String::from_utf8_lossy(&compile_gen.stderr)
    );

    let runner_rs = tmp_dir.join("runner.rs");
    std::fs::write(
        &runner_rs,
        r#"
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

use roto::runtime::BackwardWriter;
use roto_grpc::{
    AsyncClientTransport, AsyncGrpcServer, AsyncGrpcService, AsyncIo, CLIENT_CONNECTION_PREFACE,
    FLAG_END_HEADERS, FLAG_END_STREAM, FRAME_HEADER_LEN, FrameHeader, FrameType, GrpcClient,
    GrpcCode, GrpcServer, GrpcService, GrpcStatus, H2LoopbackChannel, HeaderField, HpackDecoder,
    HpackEncoder, MemoryDuplex, Route, RpcMetadata, accept_and_serve_std, block_on_pair,
    connect_std_tcp, serve_async_connection, serve_connection,
    http::{
        AsyncHttpService, CONTENT_TYPE_PROTOBUF, CONTENT_TYPE_TEXT_PLAIN, HttpMethod, HttpRequest,
        HttpService,
    },
};
use shelf_http::acme::shelf::v1::{
    BookBuilder, BookView, CreateBookRequestBuilder, CreateBookRequestView,
    DeleteShelfRequestBuilder, DeleteShelfRequestView, DeleteShelfResponseBuilder,
    DeleteShelfResponseView, GetShelfRequestBuilder, GetShelfRequestView, ShelfBuilder, ShelfTheme,
    ShelfView, UpdateBookRequestBuilder, UpdateBookRequestView, UpdateBookResponseBuilder,
    UpdateBookResponseView,
};
use shelf_http::grpc::acme::shelf::v1::shelf_service::{
    AsyncShelfService, ShelfService, ShelfServiceClient, ShelfServiceServer,
};

struct TestShelfImpl;

impl ShelfService for TestShelfImpl {
    fn get_shelf<'a, 'b>(
        &self,
        _meta: &mut RpcMetadata<'_>,
        req: GetShelfRequestView<'a>,
        resp: &mut ShelfBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let shelf = req
            .shelf()
            .map_err(|_| GrpcStatus::invalid_argument("bad shelf"))?
            .ok_or_else(|| GrpcStatus::invalid_argument("missing shelf"))?;
        let id = shelf.id().unwrap_or("");
        match id {
            "missing-shelf" => return Err(GrpcStatus::not_found("shelf not found")),
            "conflict-shelf" => return Err(GrpcStatus::already_exists("shelf conflict")),
            "broken-shelf" => return Err(GrpcStatus::internal("db error")),
            "bad-shelf" => return Err(GrpcStatus::invalid_argument("invalid shelf id")),
            _ => {}
        }
        let mut tags_joined = String::from(shelf.title().unwrap_or(""));
        for t in req.tags() {
            let t = t.map_err(|_| GrpcStatus::invalid_argument("bad tag"))?;
            if !tags_joined.is_empty() {
                tags_joined.push(',');
            }
            tags_joined.push_str(t);
        }
        let base_cap = shelf.capacity().unwrap_or(0);
        let cap = if req.include_archived().unwrap_or(false) {
            base_cap + 100
        } else {
            base_cap
        };
        resp.write_capacity(cap)
            .write_theme(shelf.theme().unwrap_or(ShelfTheme::ShelfThemeUnspecified))
            .write_title(&tags_joined)
            .write_id(id);
        Ok(())
    }

    fn create_book<'a, 'b>(
        &self,
        _meta: &mut RpcMetadata<'_>,
        req: CreateBookRequestView<'a>,
        resp: &mut BookBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let book = req
            .book()
            .map_err(|_| GrpcStatus::invalid_argument("bad book"))?
            .ok_or_else(|| GrpcStatus::invalid_argument("missing book"))?;
        let full_id = format!("{}/{}", req.shelf_id().unwrap_or(""), book.id().unwrap_or(""));
        resp.write_pages(book.pages().unwrap_or(0))
            .write_author(book.author().unwrap_or(""))
            .write_id(&full_id);
        Ok(())
    }

    fn update_book<'a, 'b>(
        &self,
        _meta: &mut RpcMetadata<'_>,
        req: UpdateBookRequestView<'a>,
        resp: &mut UpdateBookResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let status_str = format!(
            "updated:{}:{}",
            req.shelf_id().unwrap_or(""),
            req.extra_note().unwrap_or("")
        );
        resp.write_status(&status_str);
        if let Ok(Some(book)) = req.book() {
            resp.write_book(|b| {
                b.write_pages(book.pages().unwrap_or(0))
                    .write_author(book.author().unwrap_or(""))
                    .write_id(book.id().unwrap_or(""));
            });
        }
        Ok(())
    }

    fn delete_shelf<'a, 'b>(
        &self,
        _meta: &mut RpcMetadata<'_>,
        req: DeleteShelfRequestView<'a>,
        resp: &mut DeleteShelfResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let ok = req.shelf_id() == Ok("s-del") && req.purge_after_sec() == Ok(60);
        resp.write_deleted(ok);
        Ok(())
    }
}

impl AsyncShelfService for TestShelfImpl {
    async fn get_shelf<'a, 'm, 'b>(
        &'a self,
        meta: &'a mut RpcMetadata<'m>,
        req: GetShelfRequestView<'a>,
        resp: &'a mut ShelfBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as ShelfService>::get_shelf(self, meta, req, resp)
    }

    async fn create_book<'a, 'm, 'b>(
        &'a self,
        meta: &'a mut RpcMetadata<'m>,
        req: CreateBookRequestView<'a>,
        resp: &'a mut BookBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as ShelfService>::create_book(self, meta, req, resp)
    }

    async fn update_book<'a, 'm, 'b>(
        &'a self,
        meta: &'a mut RpcMetadata<'m>,
        req: UpdateBookRequestView<'a>,
        resp: &'a mut UpdateBookResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as ShelfService>::update_book(self, meta, req, resp)
    }

    async fn delete_shelf<'a, 'm, 'b>(
        &'a self,
        meta: &'a mut RpcMetadata<'m>,
        req: DeleteShelfRequestView<'a>,
        resp: &'a mut DeleteShelfResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as ShelfService>::delete_shelf(self, meta, req, resp)
    }
}

fn build_h1_raw_request(method: &str, uri: &str, content_type: Option<&str>, body: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(format!("{method} {uri} HTTP/1.1\r\nHost: localhost\r\n").as_bytes());
    if let Some(ct) = content_type {
        out.extend_from_slice(format!("Content-Type: {ct}\r\n").as_bytes());
    }
    if !body.is_empty() || matches!(method, "POST" | "PUT" | "PATCH" | "MERGE") {
        out.extend_from_slice(format!("Content-Length: {}\r\n", body.len()).as_bytes());
    }
    out.extend_from_slice(b"Connection: close\r\n\r\n");
    out.extend_from_slice(body);
    out
}

fn parse_h1_raw_response(raw: &[u8]) -> (u16, String, Vec<u8>) {
    let hdr_end = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("HTTP/1.1 header terminator");
    let hdr_str = core::str::from_utf8(&raw[..hdr_end]).expect("UTF-8 headers");
    let mut lines = hdr_str.split("\r\n");
    let status_line = lines.next().expect("status line");
    let mut parts = status_line.split_whitespace();
    assert_eq!(parts.next(), Some("HTTP/1.1"));
    let status: u16 = parts.next().expect("status code").parse().expect("u16 status");
    let mut content_type = String::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-type") {
                content_type = v.trim().to_string();
            }
        }
    }
    let body = raw[hdr_end + 4..].to_vec();
    (status, content_type, body)
}

fn send_h1_over_memory_sync(
    server: &mut GrpcServer<'_>,
    method: &str,
    uri: &str,
    content_type: Option<&str>,
    body: &[u8],
) -> (u16, String, Vec<u8>) {
    server.reset();
    let req_bytes = build_h1_raw_request(method, uri, content_type, body);
    let duplex: MemoryDuplex<4096> = MemoryDuplex::new();
    let (mut client_io, mut server_io) = duplex.split();
    let (resp_bytes, srv_res) = block_on_pair(
        async move {
            client_io.write_all(&req_bytes).await.unwrap();
            client_io.close();
            let mut out = Vec::new();
            let mut buf = [0u8; 512];
            loop {
                let n = client_io.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                }
                out.extend_from_slice(&buf[..n]);
            }
            out
        },
        async move {
            let res = serve_connection(server, &mut server_io).await;
            server_io.close();
            res
        },
    );
    srv_res.expect("serve_connection HTTP/1.1");
    parse_h1_raw_response(&resp_bytes)
}

fn send_h1_over_memory_async<R: roto_grpc::AsyncRouter>(
    server: &mut AsyncGrpcServer<'_, R>,
    method: &str,
    uri: &str,
    content_type: Option<&str>,
    body: &[u8],
) -> (u16, String, Vec<u8>) {
    server.reset();
    let req_bytes = build_h1_raw_request(method, uri, content_type, body);
    let duplex: MemoryDuplex<4096> = MemoryDuplex::new();
    let (mut client_io, mut server_io) = duplex.split();
    let (resp_bytes, srv_res) = block_on_pair(
        async move {
            client_io.write_all(&req_bytes).await.unwrap();
            client_io.close();
            let mut out = Vec::new();
            let mut buf = [0u8; 512];
            loop {
                let n = client_io.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                }
                out.extend_from_slice(&buf[..n]);
            }
            out
        },
        async move {
            let res = serve_async_connection(server, &mut server_io).await;
            server_io.close();
            res
        },
    );
    srv_res.expect("serve_async_connection HTTP/1.1");
    parse_h1_raw_response(&resp_bytes)
}

fn build_h2_http_api_frames(
    method: &str,
    path: &str,
    content_type: Option<&str>,
    body: &[u8],
) -> Vec<u8> {
    let mut client_in = vec![0u8; 2048];
    let mut pos = 0;
    client_in[..CLIENT_CONNECTION_PREFACE.len()].copy_from_slice(CLIENT_CONNECTION_PREFACE);
    pos += CLIENT_CONNECTION_PREFACE.len();
    let settings_hdr = FrameHeader::new(0, FrameType::SETTINGS, 0, 0).encode();
    client_in[pos..pos + FRAME_HEADER_LEN].copy_from_slice(&settings_hdr);
    pos += FRAME_HEADER_LEN;

    let enc = HpackEncoder::new();
    let hdr_start = pos + FRAME_HEADER_LEN;
    let mut hpos = hdr_start;
    hpos += enc
        .encode_literal(b":method", method.as_bytes(), false, &mut client_in[hpos..])
        .unwrap();
    hpos += enc
        .encode_literal(b":scheme", b"http", false, &mut client_in[hpos..])
        .unwrap();
    hpos += enc
        .encode_literal(b":path", path.as_bytes(), false, &mut client_in[hpos..])
        .unwrap();
    hpos += enc
        .encode_literal(b":authority", b"localhost", false, &mut client_in[hpos..])
        .unwrap();
    if let Some(ct) = content_type {
        hpos += enc
            .encode_literal(b"content-type", ct.as_bytes(), false, &mut client_in[hpos..])
            .unwrap();
    }
    let block_len = hpos - hdr_start;
    let flags = FLAG_END_HEADERS | if body.is_empty() { FLAG_END_STREAM } else { 0 };
    let hdr = FrameHeader::new(block_len as u32, FrameType::HEADERS, flags, 1);
    client_in[pos..pos + FRAME_HEADER_LEN].copy_from_slice(&hdr.encode());
    pos = hpos;

    if !body.is_empty() {
        let data_hdr = FrameHeader::new(body.len() as u32, FrameType::DATA, FLAG_END_STREAM, 1);
        client_in[pos..pos + FRAME_HEADER_LEN].copy_from_slice(&data_hdr.encode());
        pos += FRAME_HEADER_LEN;
        client_in[pos..pos + body.len()].copy_from_slice(body);
        pos += body.len();
    }
    client_in.truncate(pos);
    client_in
}

fn parse_h2_http_api_response(mut bytes: &[u8]) -> (u16, String, Vec<u8>) {
    let mut status = 0u16;
    let mut content_type = String::new();
    let mut body = Vec::new();
    let mut dec = HpackDecoder::<0>::stateless();
    let mut scratch = [0u8; 512];
    while bytes.len() >= FRAME_HEADER_LEN {
        let hdr_arr: [u8; 9] = bytes[..9].try_into().unwrap();
        let hdr = FrameHeader::decode(hdr_arr);
        let len = hdr.length as usize;
        let payload = &bytes[FRAME_HEADER_LEN..FRAME_HEADER_LEN + len];
        bytes = &bytes[FRAME_HEADER_LEN + len..];
        if hdr.stream_id != 1 {
            continue;
        }
        match hdr.frame_type {
            FrameType::HEADERS => {
                dec.decode(payload, &mut scratch, |hf: HeaderField<'_>| {
                    if hf.name == b":status" {
                        status = core::str::from_utf8(hf.value).unwrap().parse().unwrap();
                    } else if hf.name == b"content-type" {
                        content_type = core::str::from_utf8(hf.value).unwrap().to_string();
                    }
                    Ok(())
                })
                .expect("decode HPACK headers");
            }
            FrameType::DATA => {
                body.extend_from_slice(payload);
            }
            _ => {}
        }
    }
    (status, content_type, body)
}

fn send_h2_http_api_request_sync(
    server: &mut GrpcServer<'_>,
    method: &str,
    path: &str,
    content_type: Option<&str>,
    body: &[u8],
) -> (u16, String, Vec<u8>) {
    server.reset();
    let req_frames = build_h2_http_api_frames(method, path, content_type, body);
    server.recv(&req_frames).expect("server recv h2 http");
    let mut out = Vec::new();
    let mut wire = [0u8; 1024];
    loop {
        let n = server.poll_transmit(&mut wire);
        if n == 0 {
            break;
        }
        out.extend_from_slice(&wire[..n]);
    }
    parse_h2_http_api_response(&out)
}

fn send_h2_http_api_request_async<R: roto_grpc::AsyncRouter>(
    server: &mut AsyncGrpcServer<'_, R>,
    method: &str,
    path: &str,
    content_type: Option<&str>,
    body: &[u8],
) -> (u16, String, Vec<u8>) {
    server.reset();
    let req_frames = build_h2_http_api_frames(method, path, content_type, body);
    let mut out = Vec::new();
    block_on_pair(
        async {
            server.recv(&req_frames).await.expect("async server recv h2 http");
            let mut wire = [0u8; 1024];
            loop {
                let n = server.poll_transmit(&mut wire);
                if n == 0 {
                    break;
                }
                out.extend_from_slice(&wire[..n]);
            }
        },
        async {},
    );
    parse_h2_http_api_response(&out)
}

fn main() {
    let server = ShelfServiceServer::new(TestShelfImpl);
    let mut scratch = [0u8; 512];
    let mut out_buf = [0u8; 512];

    // 1. Primary GET with nested path vars `{shelf.id}`, `{shelf.theme}`, percent-encoding,
    //    nested query param `shelf.title`, repeated query params `tags`, and bool query param.
    {
        let mut writer = BackwardWriter::new(&mut out_buf);
        let req = HttpRequest::from_uri(
            HttpMethod::Get,
            "/v1/shelves/shelf%201/themes/SHELF_THEME_SCIFI?shelf.title=Foundation&shelf.capacity=25&tags=classic&tags=asimov&include_archived=true",
            None,
            &[],
        );
        let resp = server.call_http(&req, &mut scratch, &mut writer).unwrap().unwrap();
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.content_type, CONTENT_TYPE_PROTOBUF);
        let view = ShelfView::from_bytes(resp.body).unwrap();
        assert_eq!(view.id(), Ok("shelf 1"));
        assert_eq!(view.theme(), Ok(ShelfTheme::ShelfThemeScifi));
        assert_eq!(view.title(), Ok("Foundation,classic,asimov"));
        assert_eq!(view.capacity(), Ok(125));
    }

    // 2. Additional GET binding `/v1/{shelf.id=orgs/*/shelves/**}:inspect` with numeric enum query param
    {
        let mut writer = BackwardWriter::new(&mut out_buf);
        let req = HttpRequest::from_uri(
            HttpMethod::Get,
            "/v1/orgs/acme/shelves/region-eu/s42:inspect?shelf.theme=2&shelf.capacity=10",
            None,
            &[],
        );
        let resp = <ShelfServiceServer<TestShelfImpl> as GrpcService>::call_http(
            &server,
            &req,
            &mut scratch,
            &mut writer,
        )
        .unwrap()
        .unwrap();
        let view = ShelfView::from_bytes(resp.body).unwrap();
        assert_eq!(view.id(), Ok("orgs/acme/shelves/region-eu/s42"));
        assert_eq!(view.theme(), Ok(ShelfTheme::ShelfThemeHistory));
        assert_eq!(view.capacity(), Ok(10));
    }

    // 3. POST `/v1/books` (`body: "*"` with no path variables -> zero-copy fast path works with 0-byte scratch!)
    {
        let mut req_buf = [0u8; 128];
        let mut cb = CreateBookRequestBuilder::new(&mut req_buf);
        cb.write_book(|b| {
            b.write_pages(320).write_author("Herbert").write_id("dune");
        })
        .write_shelf_id("s9");
        let req_bytes = cb.finish().unwrap();

        let mut empty_scratch: [u8; 0] = [];
        let mut writer = BackwardWriter::new(&mut out_buf);
        let req = HttpRequest::from_uri(
            HttpMethod::Post,
            "/v1/books",
            Some(CONTENT_TYPE_PROTOBUF),
            req_bytes,
        );
        let resp = <ShelfServiceServer<TestShelfImpl> as HttpService>::call_http(
            &server,
            &req,
            &mut empty_scratch,
            &mut writer,
        )
        .unwrap()
        .unwrap();
        let book_view = BookView::from_bytes(resp.body).unwrap();
        assert_eq!(book_view.id(), Ok("s9/dune"));
        assert_eq!(book_view.author(), Ok("Herbert"));
        assert_eq!(book_view.pages(), Ok(320));
    }

    // 4. PATCH `/v1/shelves/{shelf_id}/books/{book.id}` with `body: "book"` and `response_body: "book"`
    //    Verify `{book.id}` in URL path overrides `book.id` in `req.body` while preserving `book.author` & `book.pages`,
    //    and `response_body: "book"` returns the inner `Book` message bytes directly.
    {
        let mut body_buf = [0u8; 128];
        let mut bb = BookBuilder::new(&mut body_buf);
        bb.write_pages(250)
            .write_author("Le Guin")
            .write_id("ignored-body-id");
        let body_bytes = bb.finish().unwrap();

        let mut writer = BackwardWriter::new(&mut out_buf);
        let req = HttpRequest::from_uri(
            HttpMethod::Patch,
            "/v1/shelves/s-main/books/earthsea?extra_note=patched",
            Some(CONTENT_TYPE_PROTOBUF),
            body_bytes,
        );
        let resp = server.call_http(&req, &mut scratch, &mut writer).unwrap().unwrap();
        let projected_book = BookView::from_bytes(resp.body).unwrap();
        assert_eq!(projected_book.id(), Ok("earthsea"));
        assert_eq!(projected_book.author(), Ok("Le Guin"));
        assert_eq!(projected_book.pages(), Ok(250));
    }

    // 5. PUT `/v1/shelves/{shelf_id}/books/{book.id}` with `body: "*"`
    //    Verify `{shelf_id}` and `{book.id}` override `req.body` while preserving `book.author` and `extra_note`.
    {
        let mut body_buf = [0u8; 128];
        let mut ub = UpdateBookRequestBuilder::new(&mut body_buf);
        ub.write_extra_note("from-wildcard-body")
            .write_book(|b| {
                b.write_pages(500)
                    .write_author("Tolkien")
                    .write_id("old-book-id");
            })
            .write_shelf_id("old-shelf-id");
        let body_bytes = ub.finish().unwrap();

        let mut writer = BackwardWriter::new(&mut out_buf);
        let req = HttpRequest::from_uri(
            HttpMethod::Put,
            "/v1/shelves/new-shelf/books/new-book",
            Some(CONTENT_TYPE_PROTOBUF),
            body_bytes,
        );
        let resp = server.call_http(&req, &mut scratch, &mut writer).unwrap().unwrap();
        let full_resp = UpdateBookResponseView::from_bytes(resp.body).unwrap();
        assert_eq!(full_resp.status(), Ok("updated:new-shelf:from-wildcard-body"));
        let book = full_resp.book().unwrap().unwrap();
        assert_eq!(book.id(), Ok("new-book"));
        assert_eq!(book.author(), Ok("Tolkien"));
        assert_eq!(book.pages(), Ok(500));
    }

    // 6. Custom verb `MERGE /v1/shelves/{shelf_id}:merge` with `body: "extra_note"` and `response_body: "status"`
    //     plus async dispatch via `call_http_async` and `AsyncGrpcService::call_http_async`.
    block_on_pair(
        async {
            let mut writer = BackwardWriter::new(&mut out_buf);
            let req = HttpRequest::from_uri(
                HttpMethod::Custom("MERGE"),
                "/v1/shelves/s-async:merge",
                None,
                b"fast-merge",
            );
            let resp = server
                .call_http_async(&req, &mut scratch, &mut writer)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(resp.body, b"updated:s-async:fast-merge");

            let mut writer2 = BackwardWriter::new(&mut out_buf);
            let del_req = HttpRequest::from_uri(
                HttpMethod::Delete,
                "/v1/shelves/s-del?purge_after_sec=60",
                None,
                &[],
            );
            let del_resp = <ShelfServiceServer<TestShelfImpl> as AsyncGrpcService>::call_http_async(
                &server,
                &del_req,
                &mut scratch,
                &mut writer2,
            )
            .await
            .unwrap()
            .unwrap();
            let del_view = DeleteShelfResponseView::from_bytes(del_resp.body).unwrap();
            assert_eq!(del_view.deleted(), Ok(true));

            let mut writer3 = BackwardWriter::new(&mut out_buf);
            let _ = <ShelfServiceServer<TestShelfImpl> as AsyncHttpService>::call_http(
                &server,
                &del_req,
                &mut scratch,
                &mut writer3,
            )
            .await
            .unwrap()
            .unwrap();
        },
        async {},
    );

    // 7. Unmatched route returns None; invalid enum/integer parameter returns InvalidArgument
    {
        let mut writer = BackwardWriter::new(&mut out_buf);
        let unmatched = HttpRequest::from_uri(HttpMethod::Get, "/v1/unknown", None, &[]);
        assert!(server.call_http(&unmatched, &mut scratch, &mut writer).is_none());

        let bad_enum = HttpRequest::from_uri(
            HttpMethod::Get,
            "/v1/shelves/s1/themes/NOT_A_THEME",
            None,
            &[],
        );
        let err = server.call_http(&bad_enum, &mut scratch, &mut writer).unwrap().unwrap_err();
        assert_eq!(err.code, GrpcCode::InvalidArgument);
    }

    // 8. Unified `GrpcServer` and `AsyncGrpcServer` over `MemoryDuplex` and `H2LoopbackChannel`:
    //    Prove the exact same `ShelfServiceServer` serves standard gRPC (HTTP/2), HTTP/1.1, and
    //    non-gRPC HTTP/2 API requests, including error status mapping (400, 404, 409, 500).
    {
        let routes: [&dyn Route; 1] = [&server];
        let (mut msg, mut stx, mut ctx, mut call) =
            ([0u8; 1024], [0u8; 4096], [0u8; 4096], [0u8; 1024]);

        // 8a. Standard gRPC client stub over H2LoopbackChannel
        {
            let grpc_srv: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
            let grpc_cli: GrpcClient = GrpcClient::new("http", "localhost", &mut ctx);
            let mut stub = ShelfServiceClient::new(H2LoopbackChannel::new(
                grpc_cli,
                grpc_srv,
                &mut call,
            ));
            let mut req_buf = [0u8; 128];
            let mut req = GetShelfRequestBuilder::new(&mut req_buf);
            req.write_include_archived(true)
                .write_tags("grpc-tag")
                .write_shelf(|s| {
                    s.write_capacity(50)
                        .write_theme(ShelfTheme::ShelfThemeScifi)
                        .write_title("Hyperion")
                        .write_id("grpc-shelf-1");
                });
            let shelf_reply = stub.get_shelf(&req).expect("gRPC get_shelf over HTTP/2");
            assert_eq!(shelf_reply.id(), Ok("grpc-shelf-1"));
            assert_eq!(shelf_reply.title(), Ok("Hyperion,grpc-tag"));
            assert_eq!(shelf_reply.capacity(), Ok(150));
        }

        // 8b. HTTP/1.1 requests over MemoryDuplex against sync GrpcServer
        {
            let mut sync_srv: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);

            // GET with nested path & query params
            let (st, ct, body) = send_h1_over_memory_sync(
                &mut sync_srv,
                "GET",
                "/v1/shelves/h1%2Dshelf/themes/SHELF_THEME_SCIFI?shelf.title=Solaris&shelf.capacity=30&tags=lem&include_archived=true",
                None,
                &[],
            );
            assert_eq!(st, 200);
            assert_eq!(ct, CONTENT_TYPE_PROTOBUF);
            let sv = ShelfView::from_bytes(&body).unwrap();
            assert_eq!(sv.id(), Ok("h1-shelf"));
            assert_eq!(sv.title(), Ok("Solaris,lem"));
            assert_eq!(sv.capacity(), Ok(130));

            // Additional binding GET with multi-segment wildcard `{shelf.id=orgs/*/shelves/**}:inspect`
            let (st, _, body) = send_h1_over_memory_sync(
                &mut sync_srv,
                "GET",
                "/v1/orgs/deepmind/shelves/us/west/s7:inspect?shelf.theme=2&shelf.capacity=15",
                None,
                &[],
            );
            assert_eq!(st, 200);
            let sv = ShelfView::from_bytes(&body).unwrap();
            assert_eq!(sv.id(), Ok("orgs/deepmind/shelves/us/west/s7"));
            assert_eq!(sv.theme(), Ok(ShelfTheme::ShelfThemeHistory));
            assert_eq!(sv.capacity(), Ok(15));

            // POST `/v1/shelves/{shelf_id}/books` (`body: "book"`)
            let mut bbuf = [0u8; 128];
            let mut bb = BookBuilder::new(&mut bbuf);
            bb.write_pages(412).write_author("Gibson").write_id("neuromancer");
            let book_bytes = bb.finish().unwrap();
            let (st, _, body) = send_h1_over_memory_sync(
                &mut sync_srv,
                "POST",
                "/v1/shelves/cyber/books",
                Some(CONTENT_TYPE_PROTOBUF),
                book_bytes,
            );
            assert_eq!(st, 200);
            let bv = BookView::from_bytes(&body).unwrap();
            assert_eq!(bv.id(), Ok("cyber/neuromancer"));
            assert_eq!(bv.author(), Ok("Gibson"));
            assert_eq!(bv.pages(), Ok(412));

            // PATCH `/v1/shelves/{shelf_id}/books/{book.id}` (`body: "book"`, `response_body: "book"`)
            let (st, _, body) = send_h1_over_memory_sync(
                &mut sync_srv,
                "PATCH",
                "/v1/shelves/cyber/books/count-zero",
                Some(CONTENT_TYPE_PROTOBUF),
                book_bytes,
            );
            assert_eq!(st, 200);
            let bv = BookView::from_bytes(&body).unwrap();
            assert_eq!(bv.id(), Ok("count-zero"));
            assert_eq!(bv.author(), Ok("Gibson"));

            // DELETE `/v1/shelves/{shelf_id}?purge_after_sec=60`
            let (st, _, body) = send_h1_over_memory_sync(
                &mut sync_srv,
                "DELETE",
                "/v1/shelves/s-del?purge_after_sec=60",
                None,
                &[],
            );
            assert_eq!(st, 200);
            let dv = DeleteShelfResponseView::from_bytes(&body).unwrap();
            assert_eq!(dv.deleted(), Ok(true));

            // Custom verb `MERGE /v1/shelves/{shelf_id}:merge` (`body: "extra_note"`, `response_body: "status"`)
            let (st, _, body) = send_h1_over_memory_sync(
                &mut sync_srv,
                "MERGE",
                "/v1/shelves/s-h1:merge",
                Some(CONTENT_TYPE_PROTOBUF),
                b"h1-merge-note",
            );
            assert_eq!(st, 200);
            assert_eq!(body, b"updated:s-h1:h1-merge-note");

            // Error propagation from Err(GrpcStatus) to HTTP status codes (400, 404, 409, 500)
            // and malformed query/path parameter rejection (400)
            for (uri, expected_status, expected_msg) in [
                ("/v1/shelves/bad-shelf/themes/1", 400u16, "invalid shelf id"),
                ("/v1/shelves/missing-shelf/themes/1", 404u16, "shelf not found"),
                ("/v1/shelves/conflict-shelf/themes/1", 409u16, "shelf conflict"),
                ("/v1/shelves/broken-shelf/themes/1", 500u16, "db error"),
                ("/v1/shelves/s1/themes/INVALID_ENUM", 400u16, "invalid enum parameter"),
                ("/v1/shelves/s1/themes/1?shelf.capacity=not_an_int", 400u16, "invalid int32 parameter"),
                ("/v1/unmatched/route", 404u16, "not found"),
            ] {
                let (st, ct, body) = send_h1_over_memory_sync(&mut sync_srv, "GET", uri, None, &[]);
                assert_eq!(st, expected_status, "unexpected status for {uri}");
                assert_eq!(ct, CONTENT_TYPE_TEXT_PLAIN);
                assert_eq!(core::str::from_utf8(&body).unwrap(), expected_msg);
            }
        }

        // 8c. AsyncGrpcServer over MemoryDuplex serving both gRPC (AsyncClientTransport) and HTTP/1.1
        {
            let mut async_srv = AsyncGrpcServer::new(&server, &mut msg, &mut stx);

            // Async gRPC call over MemoryDuplex
            let duplex: MemoryDuplex<2048> = MemoryDuplex::new();
            let (client_io, mut server_io) = duplex.split();
            let async_cli: GrpcClient = GrpcClient::new("http", "localhost", &mut ctx);
            let ((), srv_res) = block_on_pair(
                async move {
                    let mut transport = AsyncClientTransport::new(client_io, async_cli, &mut call);
                    {
                        let mut stub = ShelfServiceClient::new(&mut transport);
                        let mut d_buf = [0u8; 64];
                        let mut d_req = DeleteShelfRequestBuilder::new(&mut d_buf);
                        d_req.write_purge_after_sec(60).write_shelf_id("s-del");
                        let reply = stub.delete_shelf_async(&d_req).await.expect("async gRPC delete_shelf");
                        assert_eq!(reply.deleted(), Ok(true));
                    }
                    transport.into_inner().close();
                },
                serve_async_connection(&mut async_srv, &mut server_io),
            );
            srv_res.expect("serve_async_connection gRPC");

            // Async HTTP/1.1 requests over MemoryDuplex on the same AsyncGrpcServer
            let (st, _, body) = send_h1_over_memory_async(
                &mut async_srv,
                "GET",
                "/v1/shelves/async-h1/themes/1?shelf.title=AsyncBook&shelf.capacity=7",
                None,
                &[],
            );
            assert_eq!(st, 200);
            let sv = ShelfView::from_bytes(&body).unwrap();
            assert_eq!(sv.id(), Ok("async-h1"));
            assert_eq!(sv.title(), Ok("AsyncBook"));
            assert_eq!(sv.capacity(), Ok(7));

            let (st, _, body) = send_h1_over_memory_async(
                &mut async_srv,
                "GET",
                "/v1/shelves/missing-shelf/themes/1",
                None,
                &[],
            );
            assert_eq!(st, 404);
            assert_eq!(body, b"shelf not found");
        }

        // 9. Non-gRPC HTTP/2 API requests (`CallMode::Http`) against GrpcServer and AsyncGrpcServer
        {
            let mut sync_srv: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);

            let (st, ct, body) = send_h2_http_api_request_sync(
                &mut sync_srv,
                "GET",
                "/v1/shelves/h2-http-shelf/themes/SHELF_THEME_SCIFI?shelf.title=H2Rest&shelf.capacity=99",
                None,
                &[],
            );
            assert_eq!(st, 200);
            assert_eq!(ct, CONTENT_TYPE_PROTOBUF);
            let sv = ShelfView::from_bytes(&body).unwrap();
            assert_eq!(sv.id(), Ok("h2-http-shelf"));
            assert_eq!(sv.title(), Ok("H2Rest"));
            assert_eq!(sv.capacity(), Ok(99));

            let mut bbuf = [0u8; 128];
            let mut bb = BookBuilder::new(&mut bbuf);
            bb.write_pages(180).write_author("Clarke").write_id("rama");
            let book_bytes = bb.finish().unwrap();
            let (st, _, body) = send_h2_http_api_request_sync(
                &mut sync_srv,
                "PATCH",
                "/v1/shelves/s-h2/books/rendezvous-with-rama",
                Some(CONTENT_TYPE_PROTOBUF),
                book_bytes,
            );
            assert_eq!(st, 200);
            let bv = BookView::from_bytes(&body).unwrap();
            assert_eq!(bv.id(), Ok("rendezvous-with-rama"));
            assert_eq!(bv.author(), Ok("Clarke"));
            assert_eq!(bv.pages(), Ok(180));

            let (st, ct, body) = send_h2_http_api_request_sync(
                &mut sync_srv,
                "GET",
                "/v1/shelves/conflict-shelf/themes/1",
                None,
                &[],
            );
            assert_eq!(st, 409);
            assert_eq!(ct, CONTENT_TYPE_TEXT_PLAIN);
            assert_eq!(body, b"shelf conflict");

            let mut async_srv = AsyncGrpcServer::new(&server, &mut msg, &mut stx);
            let (st, _, body) = send_h2_http_api_request_async(
                &mut async_srv,
                "MERGE",
                "/v1/shelves/h2-async:merge",
                Some(CONTENT_TYPE_PROTOBUF),
                b"h2-async-note",
            );
            assert_eq!(st, 200);
            assert_eq!(body, b"updated:h2-async:h2-async-note");
        }

        // 10. Real `std::net::TcpListener` / `TcpStream` E2E test:
        //     Serve both gRPC (HTTP/2) and HTTP/1.1 connections on the same TCP socket and handler!
        {
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind TcpListener");
            let addr = listener.local_addr().expect("local_addr");

            std::thread::scope(|scope| {
                scope.spawn(|| {
                    let routes: [&dyn Route; 1] = [&server];
                    let (mut msg, mut stx) = ([0u8; 1024], [0u8; 4096]);
                    let mut tcp_srv: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
                    // Serve 4 sequential connections on the same listener:
                    // 1) gRPC over HTTP/2 (`ShelfServiceClient` over `connect_std_tcp`)
                    // 2) HTTP/1.1 `GET /v1/shelves/...`
                    // 3) HTTP/1.1 `PUT /v1/shelves/.../books/...`
                    // 4) HTTP/1.1 `GET` returning 404 Not Found
                    for _ in 0..4 {
                        tcp_srv.reset();
                        accept_and_serve_std(&listener, &mut tcp_srv)
                            .expect("accept_and_serve_std");
                    }
                });

                // Connection 1: gRPC client stub over TCP
                {
                    let (mut ctx, mut call) = ([0u8; 4096], [0u8; 1024]);
                    let mut transport = connect_std_tcp::<_, 4, 1024>(
                        addr,
                        "http",
                        "localhost",
                        &mut ctx,
                        &mut call,
                    )
                    .expect("connect_std_tcp");
                    {
                        let mut stub = ShelfServiceClient::new(&mut transport);
                        let mut cb_buf = [0u8; 128];
                        let mut cb = CreateBookRequestBuilder::new(&mut cb_buf);
                        cb.write_book(|b| {
                            b.write_pages(222).write_author("Banks").write_id("phlebas");
                        })
                        .write_shelf_id("culture");
                        let book = stub.create_book(&cb).expect("TCP gRPC create_book");
                        assert_eq!(book.id(), Ok("culture/phlebas"));
                        assert_eq!(book.author(), Ok("Banks"));
                        assert_eq!(book.pages(), Ok(222));
                    }
                    transport.io_mut().shutdown(Shutdown::Write).unwrap();
                }

                // Connection 2: HTTP/1.1 GET over TCP
                {
                    let mut stream = TcpStream::connect(addr).expect("connect HTTP/1.1 GET");
                    let req = build_h1_raw_request(
                        "GET",
                        "/v1/shelves/tcp-shelf/themes/SHELF_THEME_SCIFI?shelf.title=Excession&shelf.capacity=88",
                        None,
                        &[],
                    );
                    stream.write_all(&req).unwrap();
                    let mut raw = Vec::new();
                    stream.read_to_end(&mut raw).unwrap();
                    let (st, ct, body) = parse_h1_raw_response(&raw);
                    assert_eq!(st, 200);
                    assert_eq!(ct, CONTENT_TYPE_PROTOBUF);
                    let sv = ShelfView::from_bytes(&body).unwrap();
                    assert_eq!(sv.id(), Ok("tcp-shelf"));
                    assert_eq!(sv.title(), Ok("Excession"));
                    assert_eq!(sv.capacity(), Ok(88));
                }

                // Connection 3: HTTP/1.1 PUT over TCP
                {
                    let mut body_buf = [0u8; 128];
                    let mut ub = UpdateBookRequestBuilder::new(&mut body_buf);
                    ub.write_extra_note("tcp-put")
                        .write_book(|b| {
                            b.write_pages(333).write_author("Watts").write_id("old-id");
                        })
                        .write_shelf_id("old-shelf");
                    let req_body = ub.finish().unwrap();

                    let mut stream = TcpStream::connect(addr).expect("connect HTTP/1.1 PUT");
                    let req = build_h1_raw_request(
                        "PUT",
                        "/v1/shelves/firefall/books/blindsight",
                        Some(CONTENT_TYPE_PROTOBUF),
                        req_body,
                    );
                    stream.write_all(&req).unwrap();
                    let mut raw = Vec::new();
                    stream.read_to_end(&mut raw).unwrap();
                    let (st, _, body) = parse_h1_raw_response(&raw);
                    assert_eq!(st, 200);
                    let uv = UpdateBookResponseView::from_bytes(&body).unwrap();
                    assert_eq!(uv.status(), Ok("updated:firefall:tcp-put"));
                    let bv = uv.book().unwrap().unwrap();
                    assert_eq!(bv.id(), Ok("blindsight"));
                    assert_eq!(bv.author(), Ok("Watts"));
                    assert_eq!(bv.pages(), Ok(333));
                }

                // Connection 4: HTTP/1.1 GET returning 404 Not Found over TCP
                {
                    let mut stream = TcpStream::connect(addr).expect("connect HTTP/1.1 404");
                    let req = build_h1_raw_request(
                        "GET",
                        "/v1/shelves/missing-shelf/themes/1",
                        None,
                        &[],
                    );
                    stream.write_all(&req).unwrap();
                    let mut raw = Vec::new();
                    stream.read_to_end(&mut raw).unwrap();
                    let (st, ct, body) = parse_h1_raw_response(&raw);
                    assert_eq!(st, 404);
                    assert_eq!(ct, CONTENT_TYPE_TEXT_PLAIN);
                    assert_eq!(body, b"shelf not found");
                }
            });
        }
    }
}
"#,
    )
    .unwrap();

    let runner_bin = build_dir.join("http_runner");
    let compile_runner = Command::new("rustc")
        .arg("--edition=2021")
        .arg("-D")
        .arg("warnings")
        .arg(&runner_rs)
        .arg("--extern")
        .arg(format!("shelf_http={}", gen_rlib.display()))
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
        "rustc http_runner failed:\n{}",
        String::from_utf8_lossy(&compile_runner.stderr)
    );

    let run_out = Command::new(&runner_bin).output().unwrap();
    assert!(
        run_out.status.success(),
        "http_runner failed:\n{}",
        String::from_utf8_lossy(&run_out.stderr)
    );

    let _ = std::fs::remove_dir_all(&tmp_dir);
}
