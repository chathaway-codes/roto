//! End-to-end validation of well-known Google ecosystem Protobuf packages (`google/rpc`,
//! `google/type`, and `google/api`) importing the `google-protobuf-roto` standard library crate
//! across crate boundaries via `--roto_opt=extern_path=google/protobuf=::google_protobuf_roto`.
//!
//! # Validated Ecosystem Schemas
//! - **`google.rpc`**:
//!   - `google/rpc/code.proto`: `Code` canonical status enum (`OK`, `CANCELLED`,
//!     `INVALID_ARGUMENT`, `NOT_FOUND`, `PERMISSION_DENIED`, `INTERNAL`, `UNAVAILABLE`).
//!   - `google/rpc/status.proto`: `Status` (`code`, `message`, `repeated google.protobuf.Any details`).
//!   - `google/rpc/error_details.proto`: `ErrorInfo` (with `map<string, string> metadata`
//!     synthesized as `error_info::MetadataEntry`), `RetryInfo` (referencing external
//!     `google.protobuf.Duration`), `DebugInfo`, `QuotaFailure` (`Violation`), `BadRequest`
//!     (`FieldViolation`), `Help` (`Link`), and `LocalizedMessage`.
//! - **`google.type`** (emitted as escaped Rust module `google::r#type`):
//!   - `google/type/date.proto`: `Date` (`year`, `month`, `day`).
//!   - `google/type/timeofday.proto`: `TimeOfDay` (`hours`, `minutes`, `seconds`, `nanos`).
//!   - `google/type/latlng.proto`: `LatLng` (`latitude`, `longitude`).
//!   - `google/type/money.proto`: `Money` (`currency_code`, `units`, `nanos`).
//!   - `google/type/color.proto`: `Color` (`red`, `green`, `blue`, and external wrapper
//!     `google.protobuf.FloatValue alpha`).
//!   - `google/type/expr.proto`: `Expr` (`expression`, `title`, `description`, `location`).
//! - **`google.api`**:
//!   - `google/api/http.proto`: `Http`, `HttpRule` (with `oneof pattern` variants `get`, `put`,
//!     `post`, `delete`, `patch`, `CustomHttpPattern custom`, and recursive
//!     `repeated HttpRule additional_bindings`), and `CustomHttpPattern`.
//!
//! # Validated Code Generator & Runtime Contracts
//! 1. **Cross-Crate `extern_path` Resolution**:
//!    `--roto_opt=extern_path=google/protobuf=::google_protobuf_roto` maps `.google.protobuf.*`
//!    types to `::google_protobuf_roto::google::protobuf::*` (`AnyView`, `AnyBuilder`,
//!    `DurationView`, `DurationBuilder`, `FloatValueView`, `FloatValueBuilder`) without
//!    regenerating `pub mod protobuf` inside the downstream `googleapis` crate.
//! 2. **Dual Output Modes (`mode=lib` and `mode=mod`)**:
//!    Both single-file `lib.rs` (`mode=lib`) and multi-file module tree (`mode=mod` producing
//!    `mod.rs`, `google/rpc.rs`, `google/type.rs`, `google/api.rs`) compile cleanly under
//!    `#![no_std]`, `#![forbid(unsafe_code)]`, and `rustc -D warnings`.
//! 3. **Single-Pass Backward Builders & Zero-Copy Views**:
//!    Exercises `<Msg>Builder::new(&mut buf)`, chained `&mut Self` `write_*` methods, closure-based
//!    nested message builders, `copy_unset_fields` partial updates (including `oneof` sibling
//!    masking), `check()`, `finish()`, and `<Msg>View::from_bytes` zero-copy decoding.

use std::path::{Path, PathBuf};
use std::process::Command;

fn create_temp_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "roto_ecosystem_packages_{}_{}",
        test_name,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

/// Compiles `roto_runtime`, `roto`, and `google_protobuf_roto` into `out_dir` as `.rlib`s
/// so generated downstream crates and test binaries can be compiled directly with `rustc`.
fn build_core_and_wkt_rlibs(out_dir: &Path) -> (PathBuf, PathBuf) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().expect("workspace root");

    let runtime_src = workspace_root.join("roto-runtime/src/lib.rs");
    let runtime_rlib = out_dir.join("libroto_runtime.rlib");
    let status = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=roto_runtime")
        .arg("-D")
        .arg("warnings")
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
        .arg("-D")
        .arg("warnings")
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

    let wkt_src = workspace_root.join("google-protobuf-roto/src/lib.rs");
    let wkt_rlib = out_dir.join("libgoogle_protobuf_roto.rlib");
    let status = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=google_protobuf_roto")
        .arg("-D")
        .arg("warnings")
        .arg(&wkt_src)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("--extern")
        .arg(format!("roto_runtime={}", runtime_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", out_dir.display()))
        .arg("-o")
        .arg(&wkt_rlib)
        .output()
        .expect("failed to run rustc on google-protobuf-roto");
    assert!(
        status.status.success(),
        "rustc google-protobuf-roto failed: {}",
        String::from_utf8_lossy(&status.stderr)
    );

    (roto_rlib, wkt_rlib)
}

const GOOGLE_RPC_CODE_PROTO: &str = r#"syntax = "proto3";

package google.rpc;

enum Code {
    OK = 0;
    CANCELLED = 1;
    INVALID_ARGUMENT = 3;
    NOT_FOUND = 5;
    PERMISSION_DENIED = 7;
    INTERNAL = 13;
    UNAVAILABLE = 14;
}
"#;

const GOOGLE_RPC_STATUS_PROTO: &str = r#"syntax = "proto3";

package google.rpc;

import "google/protobuf/any.proto";

message Status {
    int32 code = 1;
    string message = 2;
    repeated google.protobuf.Any details = 3;
}
"#;

const GOOGLE_RPC_ERROR_DETAILS_PROTO: &str = r#"syntax = "proto3";

package google.rpc;

import "google/protobuf/duration.proto";

message ErrorInfo {
    string reason = 1;
    string domain = 2;
    map<string, string> metadata = 3;
}

message RetryInfo {
    google.protobuf.Duration retry_delay = 1;
}

message DebugInfo {
    repeated string stack_entries = 1;
    string detail = 2;
}

message QuotaFailure {
    message Violation {
        string subject = 1;
        string description = 2;
    }
    repeated Violation violations = 1;
}

message BadRequest {
    message FieldViolation {
        string field = 1;
        string description = 2;
    }
    repeated FieldViolation field_violations = 1;
}

message Help {
    message Link {
        string description = 1;
        string url = 2;
    }
    repeated Link links = 1;
}

message LocalizedMessage {
    string locale = 1;
    string message = 2;
}
"#;

const GOOGLE_TYPE_DATE_PROTO: &str = r#"syntax = "proto3";

package google.type;

message Date {
    int32 year = 1;
    int32 month = 2;
    int32 day = 3;
}
"#;

const GOOGLE_TYPE_TIMEOFDAY_PROTO: &str = r#"syntax = "proto3";

package google.type;

message TimeOfDay {
    int32 hours = 1;
    int32 minutes = 2;
    int32 seconds = 3;
    int32 nanos = 4;
}
"#;

const GOOGLE_TYPE_LATLNG_PROTO: &str = r#"syntax = "proto3";

package google.type;

message LatLng {
    double latitude = 1;
    double longitude = 2;
}
"#;

const GOOGLE_TYPE_MONEY_PROTO: &str = r#"syntax = "proto3";

package google.type;

message Money {
    string currency_code = 1;
    int64 units = 2;
    int32 nanos = 3;
}
"#;

const GOOGLE_TYPE_COLOR_PROTO: &str = r#"syntax = "proto3";

package google.type;

import "google/protobuf/wrappers.proto";

message Color {
    float red = 1;
    float green = 2;
    float blue = 3;
    google.protobuf.FloatValue alpha = 4;
}
"#;

const GOOGLE_TYPE_EXPR_PROTO: &str = r#"syntax = "proto3";

package google.type;

message Expr {
    string expression = 1;
    string title = 2;
    string description = 3;
    string location = 4;
}
"#;

const GOOGLE_API_HTTP_PROTO: &str = r#"syntax = "proto3";

package google.api;

message Http {
    repeated HttpRule rules = 1;
    bool fully_decode_reserved_expansion = 2;
}

message HttpRule {
    string selector = 1;
    oneof pattern {
        string get = 2;
        string put = 3;
        string post = 4;
        string delete = 5;
        string patch = 6;
        CustomHttpPattern custom = 8;
    }
    string body = 7;
    string response_body = 12;
    repeated HttpRule additional_bindings = 11;
}

message CustomHttpPattern {
    string kind = 1;
    string path = 2;
}
"#;

#[test]
fn ecosystem_packages_google_rpc_type_and_api_with_wkt_extern_path() {
    let plugin_bin = env!("CARGO_BIN_EXE_protoc-gen-roto");
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().expect("workspace root");
    let wkt_proto_dir = workspace_root.join("google-protobuf-roto/proto");

    let tmp_dir = create_temp_dir("googleapis");
    let proto_dir = tmp_dir.join("proto");
    let gen_lib_dir = tmp_dir.join("gen_lib");
    let gen_mod_dir = tmp_dir.join("gen_mod");
    let build_dir = tmp_dir.join("build");

    std::fs::create_dir_all(proto_dir.join("google/rpc")).unwrap();
    std::fs::create_dir_all(proto_dir.join("google/type")).unwrap();
    std::fs::create_dir_all(proto_dir.join("google/api")).unwrap();
    std::fs::create_dir_all(&gen_lib_dir).unwrap();
    std::fs::create_dir_all(&gen_mod_dir).unwrap();
    std::fs::create_dir_all(&build_dir).unwrap();

    let proto_files = [
        ("google/rpc/code.proto", GOOGLE_RPC_CODE_PROTO),
        ("google/rpc/status.proto", GOOGLE_RPC_STATUS_PROTO),
        (
            "google/rpc/error_details.proto",
            GOOGLE_RPC_ERROR_DETAILS_PROTO,
        ),
        ("google/type/date.proto", GOOGLE_TYPE_DATE_PROTO),
        ("google/type/timeofday.proto", GOOGLE_TYPE_TIMEOFDAY_PROTO),
        ("google/type/latlng.proto", GOOGLE_TYPE_LATLNG_PROTO),
        ("google/type/money.proto", GOOGLE_TYPE_MONEY_PROTO),
        ("google/type/color.proto", GOOGLE_TYPE_COLOR_PROTO),
        ("google/type/expr.proto", GOOGLE_TYPE_EXPR_PROTO),
        ("google/api/http.proto", GOOGLE_API_HTTP_PROTO),
    ];

    for (rel_path, content) in &proto_files {
        std::fs::write(proto_dir.join(rel_path), content).unwrap();
    }

    let target_args: Vec<&str> = proto_files.iter().map(|(p, _)| *p).collect();

    // 1. Generate downstream `googleapis` crate in `mode=lib`
    let protoc_lib = Command::new("protoc")
        .arg(format!("--plugin=protoc-gen-roto={plugin_bin}"))
        .arg(format!("-I{}", proto_dir.display()))
        .arg(format!("-I{}", wkt_proto_dir.display()))
        .arg("--roto_opt=mode=lib,extern_path=google/protobuf=::google_protobuf_roto")
        .arg(format!("--roto_out={}", gen_lib_dir.display()))
        .args(&target_args)
        .output()
        .expect("failed to run protoc (mode=lib) on ecosystem protos");
    assert!(
        protoc_lib.status.success(),
        "protoc (mode=lib) failed: {}",
        String::from_utf8_lossy(&protoc_lib.stderr)
    );

    let lib_rs_path = gen_lib_dir.join("lib.rs");
    let lib_rs = std::fs::read_to_string(&lib_rs_path).expect("expected generated gen_lib/lib.rs");
    assert!(lib_rs.starts_with("#![no_std]\n#![forbid(unsafe_code)]"));
    assert!(
        !lib_rs.contains("pub mod protobuf"),
        "downstream googleapis crate must not redefine google::protobuf:\n{lib_rs}"
    );
    for expected_ref in [
        "::google_protobuf_roto::google::protobuf::AnyView",
        "::google_protobuf_roto::google::protobuf::AnyBuilder",
        "::google_protobuf_roto::google::protobuf::DurationView",
        "::google_protobuf_roto::google::protobuf::DurationBuilder",
        "::google_protobuf_roto::google::protobuf::FloatValueView",
        "::google_protobuf_roto::google::protobuf::FloatValueBuilder",
    ] {
        assert!(
            lib_rs.contains(expected_ref),
            "expected cross-crate reference `{expected_ref}` in generated lib.rs:\n{lib_rs}"
        );
    }

    // 2. Generate downstream `googleapis` crate in `mode=mod`
    let protoc_mod = Command::new("protoc")
        .arg(format!("--plugin=protoc-gen-roto={plugin_bin}"))
        .arg(format!("-I{}", proto_dir.display()))
        .arg(format!("-I{}", wkt_proto_dir.display()))
        .arg("--roto_opt=mode=mod,extern_path=google/protobuf=::google_protobuf_roto")
        .arg(format!("--roto_out={}", gen_mod_dir.display()))
        .args(&target_args)
        .output()
        .expect("failed to run protoc (mode=mod) on ecosystem protos");
    assert!(
        protoc_mod.status.success(),
        "protoc (mode=mod) failed: {}",
        String::from_utf8_lossy(&protoc_mod.stderr)
    );
    assert!(gen_mod_dir.join("mod.rs").exists());
    assert!(gen_mod_dir.join("google/rpc.rs").exists());
    assert!(gen_mod_dir.join("google/type.rs").exists());
    assert!(gen_mod_dir.join("google/api.rs").exists());
    assert!(!gen_mod_dir.join("google/protobuf.rs").exists());

    // 3. Compile `roto_runtime`, `roto`, `google_protobuf_roto`, and both `googleapis` crates
    let (roto_rlib, wkt_rlib) = build_core_and_wkt_rlibs(&build_dir);

    let googleapis_lib_rlib = build_dir.join("libgoogleapis_roto.rlib");
    let compile_lib = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=googleapis_roto")
        .arg("-D")
        .arg("warnings")
        .arg(&lib_rs_path)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("--extern")
        .arg(format!("google_protobuf_roto={}", wkt_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&googleapis_lib_rlib)
        .output()
        .expect("failed to compile googleapis_roto (mode=lib)");
    assert!(
        compile_lib.status.success(),
        "compiling googleapis_roto (mode=lib) failed:\n{}",
        String::from_utf8_lossy(&compile_lib.stderr)
    );

    let mod_root = gen_mod_dir.join("lib_mod_root.rs");
    let mod_rs_content = std::fs::read_to_string(gen_mod_dir.join("mod.rs")).unwrap();
    std::fs::write(
        &mod_root,
        format!(
            "#![no_std]\n#![forbid(unsafe_code)]\n#![allow(clippy::all, dead_code, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_imports)]\n\n{mod_rs_content}"
        ),
    )
    .unwrap();
    let googleapis_mod_rlib = build_dir.join("libgoogleapis_mod_roto.rlib");
    let compile_mod = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=googleapis_mod_roto")
        .arg("-D")
        .arg("warnings")
        .arg(&mod_root)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("--extern")
        .arg(format!("google_protobuf_roto={}", wkt_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&googleapis_mod_rlib)
        .output()
        .expect("failed to compile googleapis_mod_roto (mode=mod)");
    assert!(
        compile_mod.status.success(),
        "compiling googleapis_mod_roto (mode=mod) failed:\n{}",
        String::from_utf8_lossy(&compile_mod.stderr)
    );

    // 4. Compile and run end-to-end test binary exercising all ecosystem messages and WKT imports
    let runner_src = tmp_dir.join("ecosystem_runner.rs");
    std::fs::write(
        &runner_src,
        r#"
use google_protobuf_roto::google::protobuf::{AnyView, DurationView, FloatValueView};
use googleapis_roto::google::{
    api::{
        http_rule::Pattern, CustomHttpPatternView, HttpBuilder, HttpRuleBuilder, HttpRuleView,
        HttpView,
    },
    r#type::{
        ColorBuilder, ColorView, DateBuilder, DateView, ExprBuilder, ExprView, LatLngBuilder,
        LatLngView, MoneyBuilder, MoneyView, TimeOfDayBuilder, TimeOfDayView,
    },
    rpc::{
        bad_request::FieldViolationView, quota_failure::ViolationView, BadRequestBuilder,
        BadRequestView, Code, DebugInfoBuilder, DebugInfoView, ErrorInfoBuilder, ErrorInfoView,
        HelpBuilder, HelpView, LocalizedMessageBuilder, LocalizedMessageView, QuotaFailureBuilder,
        QuotaFailureView, RetryInfoBuilder, RetryInfoView, StatusBuilder, StatusView,
    },
};

fn main() -> Result<(), Box<dyn core::error::Error>> {
    // =========================================================================
    // 1. google.rpc: Status with Any-packed ErrorInfo, RetryInfo (Duration), BadRequest,
    //    plus DebugInfo, QuotaFailure, Help, and LocalizedMessage
    // =========================================================================

    // 1a. Serialize ErrorInfo with map<string, string> metadata entries
    let mut err_info_buf = [0u8; 256];
    let mut err_info_b = ErrorInfoBuilder::new(&mut err_info_buf);
    err_info_b
        .write_metadata(|entry| {
            entry.write_value("us-east-1").write_key("zone");
        })
        .write_metadata(|entry| {
            entry.write_value("pubsub.googleapis.com").write_key("service");
        })
        .write_domain("googleapis.com")
        .write_reason("SERVICE_DISABLED")
        .check()?;
    let err_info_bytes = err_info_b.finish()?;

    // 1b. Serialize RetryInfo referencing external google_protobuf_roto::google::protobuf::Duration
    let mut retry_buf = [0u8; 64];
    let mut retry_b = RetryInfoBuilder::new(&mut retry_buf);
    retry_b
        .write_retry_delay(|dur| {
            dur.write_nanos(500_000_000).write_seconds(5);
        })
        .check()?;
    let retry_bytes = retry_b.finish()?;

    // 1c. Serialize BadRequest with repeated FieldViolation
    let mut bad_req_buf = [0u8; 256];
    let mut bad_req_b = BadRequestBuilder::new(&mut bad_req_buf);
    bad_req_b
        .write_field_violations(|fv| {
            fv.write_description("must be a positive integer")
                .write_field("page_size");
        })
        .write_field_violations(|fv| {
            fv.write_description("must not be empty")
                .write_field("resource_id");
        })
        .check()?;
    let bad_req_bytes = bad_req_b.finish()?;

    // 1d. Wrap all three error detail payloads into google.rpc.Status.details (repeated Any)
    let mut status_buf = [0u8; 1024];
    let mut status_b = StatusBuilder::new(&mut status_buf);
    status_b
        .write_details(|any| {
            any.write_value(bad_req_bytes)
                .write_type_url("type.googleapis.com/google.rpc.BadRequest");
        })
        .write_details(|any| {
            any.write_value(retry_bytes)
                .write_type_url("type.googleapis.com/google.rpc.RetryInfo");
        })
        .write_details(|any| {
            any.write_value(err_info_bytes)
                .write_type_url("type.googleapis.com/google.rpc.ErrorInfo");
        })
        .write_message("Invalid request parameters")
        .write_code(Code::InvalidArgument as i32)
        .check()?;
    let status_bytes = status_b.finish()?;

    let status_view = StatusView::from_bytes(status_bytes)?;
    assert_eq!(status_view.code()?, Code::InvalidArgument as i32);
    assert_eq!(status_view.message()?, "Invalid request parameters");

    let details: Vec<AnyView<'_>> = status_view.details().map(|d| d.unwrap()).collect();
    assert_eq!(details.len(), 3);

    // Unpack detail[0]: ErrorInfo
    assert_eq!(
        details[0].type_url()?,
        "type.googleapis.com/google.rpc.ErrorInfo"
    );
    let unpacked_err = ErrorInfoView::from_bytes(details[0].value()?)?;
    assert_eq!(unpacked_err.reason()?, "SERVICE_DISABLED");
    assert_eq!(unpacked_err.domain()?, "googleapis.com");
    let meta: Vec<(&str, &str)> = unpacked_err
        .metadata()
        .map(|e| {
            let entry = e.unwrap();
            (entry.key().unwrap(), entry.value().unwrap())
        })
        .collect();
    assert_eq!(
        meta,
        vec![("service", "pubsub.googleapis.com"), ("zone", "us-east-1")]
    );

    // Unpack detail[1]: RetryInfo -> external DurationView
    assert_eq!(
        details[1].type_url()?,
        "type.googleapis.com/google.rpc.RetryInfo"
    );
    let unpacked_retry = RetryInfoView::from_bytes(details[1].value()?)?;
    let delay: DurationView<'_> = unpacked_retry
        .retry_delay()?
        .expect("expected retry_delay DurationView");
    assert_eq!(delay.seconds()?, 5);
    assert_eq!(delay.nanos()?, 500_000_000);

    // Unpack detail[2]: BadRequest
    assert_eq!(
        details[2].type_url()?,
        "type.googleapis.com/google.rpc.BadRequest"
    );
    let unpacked_bad_req = BadRequestView::from_bytes(details[2].value()?)?;
    let violations: Vec<(&str, &str)> = unpacked_bad_req
        .field_violations()
        .map(|v| {
            let fv: FieldViolationView<'_> = v.unwrap();
            (fv.field().unwrap(), fv.description().unwrap())
        })
        .collect();
    assert_eq!(
        violations,
        vec![
            ("resource_id", "must not be empty"),
            ("page_size", "must be a positive integer"),
        ]
    );

    // 1e. Verify DebugInfo, QuotaFailure, Help, and LocalizedMessage
    let mut dbg_buf = [0u8; 256];
    let mut dbg_b = DebugInfoBuilder::new(&mut dbg_buf);
    dbg_b
        .write_detail("panic in handler")
        .write_stack_entries("frame_1: dispatch")
        .write_stack_entries("frame_0: main");
    let dbg_view = DebugInfoView::from_bytes(dbg_b.finish()?)?;
    assert_eq!(dbg_view.detail()?, "panic in handler");
    let frames: Vec<&str> = dbg_view.stack_entries().map(|s| s.unwrap()).collect();
    assert_eq!(frames, vec!["frame_0: main", "frame_1: dispatch"]);

    let mut qf_buf = [0u8; 256];
    let mut qf_b = QuotaFailureBuilder::new(&mut qf_buf);
    qf_b.write_violations(|v| {
        v.write_description("Daily limit exceeded")
            .write_subject("project:my-proj");
    });
    let qf_view = QuotaFailureView::from_bytes(qf_b.finish()?)?;
    let qf_items: Vec<(&str, &str)> = qf_view
        .violations()
        .map(|v| {
            let vv: ViolationView<'_> = v.unwrap();
            (vv.subject().unwrap(), vv.description().unwrap())
        })
        .collect();
    assert_eq!(qf_items, vec![("project:my-proj", "Daily limit exceeded")]);

    let mut help_buf = [0u8; 256];
    let mut help_b = HelpBuilder::new(&mut help_buf);
    help_b.write_links(|l| {
        l.write_url("https://cloud.google.com/apis")
            .write_description("API Documentation");
    });
    let help_view = HelpView::from_bytes(help_b.finish()?)?;
    let links: Vec<(&str, &str)> = help_view
        .links()
        .map(|l| {
            let lv = l.unwrap();
            (lv.description().unwrap(), lv.url().unwrap())
        })
        .collect();
    assert_eq!(
        links,
        vec![("API Documentation", "https://cloud.google.com/apis")]
    );

    let mut loc_buf = [0u8; 128];
    let mut loc_b = LocalizedMessageBuilder::new(&mut loc_buf);
    loc_b
        .write_message("Ungueltige Anfrage")
        .write_locale("de-DE");
    let loc_view = LocalizedMessageView::from_bytes(loc_b.finish()?)?;
    assert_eq!(loc_view.locale()?, "de-DE");
    assert_eq!(loc_view.message()?, "Ungueltige Anfrage");

    // =========================================================================
    // 2. google.type (`google::r#type`): Color (with FloatValue wrapper), Money,
    //    LatLng, Date, TimeOfDay, Expr
    // =========================================================================

    let mut color_buf = [0u8; 128];
    let mut color_b = ColorBuilder::new(&mut color_buf);
    color_b
        .write_alpha(|alpha| {
            alpha.write_value(0.75);
        })
        .write_blue(0.5)
        .write_green(0.25)
        .write_red(0.125)
        .check()?;
    let color_view = ColorView::from_bytes(color_b.finish()?)?;
    assert_eq!(color_view.red()?, 0.125);
    assert_eq!(color_view.green()?, 0.25);
    assert_eq!(color_view.blue()?, 0.5);
    let alpha_view: FloatValueView<'_> = color_view.alpha()?.expect("expected FloatValue alpha");
    assert_eq!(alpha_view.value()?, 0.75);

    let mut money_buf = [0u8; 128];
    let mut money_b = MoneyBuilder::new(&mut money_buf);
    money_b
        .write_nanos(500_000_000)
        .write_units(42)
        .write_currency_code("USD");
    let money_view = MoneyView::from_bytes(money_b.finish()?)?;
    assert_eq!(money_view.currency_code()?, "USD");
    assert_eq!(money_view.units()?, 42);
    assert_eq!(money_view.nanos()?, 500_000_000);

    let mut latlng_buf = [0u8; 64];
    let mut latlng_b = LatLngBuilder::new(&mut latlng_buf);
    latlng_b
        .write_longitude(-122.4194)
        .write_latitude(37.7749);
    let latlng_view = LatLngView::from_bytes(latlng_b.finish()?)?;
    assert_eq!(latlng_view.latitude()?, 37.7749);
    assert_eq!(latlng_view.longitude()?, -122.4194);

    let mut date_buf = [0u8; 64];
    let mut date_b = DateBuilder::new(&mut date_buf);
    date_b.write_day(20).write_month(10).write_year(2026);
    let date_view = DateView::from_bytes(date_b.finish()?)?;
    assert_eq!(date_view.year()?, 2026);
    assert_eq!(date_view.month()?, 10);
    assert_eq!(date_view.day()?, 20);

    let mut tod_buf = [0u8; 64];
    let mut tod_b = TimeOfDayBuilder::new(&mut tod_buf);
    tod_b
        .write_nanos(999)
        .write_seconds(45)
        .write_minutes(30)
        .write_hours(14);
    let tod_view = TimeOfDayView::from_bytes(tod_b.finish()?)?;
    assert_eq!(tod_view.hours()?, 14);
    assert_eq!(tod_view.minutes()?, 30);
    assert_eq!(tod_view.seconds()?, 45);
    assert_eq!(tod_view.nanos()?, 999);

    let mut expr_buf = [0u8; 256];
    let mut expr_b = ExprBuilder::new(&mut expr_buf);
    expr_b
        .write_location("policy.cel:12")
        .write_description("Require admin role")
        .write_title("AdminCheck")
        .write_expression("request.auth.claims['role'] == 'admin'");
    let expr_view = ExprView::from_bytes(expr_b.finish()?)?;
    assert_eq!(
        expr_view.expression()?,
        "request.auth.claims['role'] == 'admin'"
    );
    assert_eq!(expr_view.title()?, "AdminCheck");
    assert_eq!(expr_view.description()?, "Require admin role");
    assert_eq!(expr_view.location()?, "policy.cel:12");

    // =========================================================================
    // 3. google.api: Http & HttpRule (oneof pattern + recursive additional_bindings
    //    + copy_unset_fields partial update)
    // =========================================================================

    let mut http_buf = [0u8; 512];
    let mut http_b = HttpBuilder::new(&mut http_buf);
    http_b
        .write_fully_decode_reserved_expansion(true)
        .write_rules(|rule| {
            rule.write_additional_bindings(|alt| {
                alt.write_custom(|cust| {
                    cust.write_path("/v1/{name=messages/*}:archive")
                        .write_kind("ARCHIVE");
                })
                .write_selector("google.example.v1.Messaging.GetMessage");
            })
            .write_additional_bindings(|alt| {
                alt.write_body("*")
                    .write_post("/v1/messages:lookup")
                    .write_selector("google.example.v1.Messaging.GetMessage");
            })
            .write_response_body("message")
            .write_get("/v1/{name=messages/*}")
            .write_selector("google.example.v1.Messaging.GetMessage");
        })
        .check()?;
    let http_view = HttpView::from_bytes(http_b.finish()?)?;
    assert!(http_view.fully_decode_reserved_expansion()?);

    let rules: Vec<HttpRuleView<'_>> = http_view.rules().map(|r| r.unwrap()).collect();
    assert_eq!(rules.len(), 1);
    let primary_rule = rules[0];
    assert_eq!(
        primary_rule.selector()?,
        "google.example.v1.Messaging.GetMessage"
    );
    assert_eq!(primary_rule.response_body()?, "message");
    match primary_rule.pattern()? {
        Some(Pattern::Get(path)) => assert_eq!(path, "/v1/{name=messages/*}"),
        other => panic!("expected Pattern::Get, got: {other:?}"),
    }

    let bindings: Vec<HttpRuleView<'_>> = primary_rule
        .additional_bindings()
        .map(|b| b.unwrap())
        .collect();
    assert_eq!(bindings.len(), 2);
    match bindings[0].pattern()? {
        Some(Pattern::Post(path)) => assert_eq!(path, "/v1/messages:lookup"),
        other => panic!("expected Pattern::Post, got: {other:?}"),
    }
    assert_eq!(bindings[0].body()?, "*");
    match bindings[1].pattern()?.expect("expected custom pattern") {
        Pattern::Custom(cust) => {
            let cust_view: CustomHttpPatternView<'_> = cust;
            assert_eq!(cust_view.kind()?, "ARCHIVE");
            assert_eq!(cust_view.path()?, "/v1/{name=messages/*}:archive");
        }
        other => panic!("expected Pattern::Custom, got: {other:?}"),
    }

    // Partial update on HttpRule via copy_unset_fields: switch oneof pattern from `Get` to `Delete`
    // while preserving selector, response_body, and additional_bindings.
    let mut patched_rule_buf = [0u8; 512];
    let mut patched_rule_b = HttpRuleBuilder::new(&mut patched_rule_buf);
    patched_rule_b
        .write_delete("/v2/{name=messages/*}")
        .copy_unset_fields(&primary_rule)
        .check()?;
    let patched_rule = HttpRuleView::from_bytes(patched_rule_b.finish()?)?;
    assert_eq!(
        patched_rule.selector()?,
        "google.example.v1.Messaging.GetMessage"
    );
    assert_eq!(patched_rule.response_body()?, "message");
    match patched_rule.pattern()? {
        Some(Pattern::Delete(path)) => assert_eq!(path, "/v2/{name=messages/*}"),
        other => panic!("expected Pattern::Delete, got: {other:?}"),
    }
    assert!(!patched_rule.has_get());
    assert_eq!(patched_rule.additional_bindings().count(), 2);

    Ok(())
}
"#,
    )
    .unwrap();

    // Run the test binary linked against `googleapis_roto` (`mode=lib`)
    let runner_bin_lib = build_dir.join("ecosystem_runner_lib");
    let compile_runner_lib = Command::new("rustc")
        .arg("--edition=2021")
        .arg("-D")
        .arg("warnings")
        .arg(&runner_src)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("--extern")
        .arg(format!("google_protobuf_roto={}", wkt_rlib.display()))
        .arg("--extern")
        .arg(format!("googleapis_roto={}", googleapis_lib_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&runner_bin_lib)
        .output()
        .expect("failed to compile ecosystem_runner_lib");
    assert!(
        compile_runner_lib.status.success(),
        "compiling ecosystem_runner_lib failed:\n{}",
        String::from_utf8_lossy(&compile_runner_lib.stderr)
    );

    let run_lib = Command::new(&runner_bin_lib)
        .output()
        .expect("failed to execute ecosystem_runner_lib");
    assert!(
        run_lib.status.success(),
        "ecosystem_runner_lib failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_lib.stdout),
        String::from_utf8_lossy(&run_lib.stderr)
    );

    // Also run the exact same test binary linked against `googleapis_mod_roto` (`mode=mod`)
    let runner_bin_mod = build_dir.join("ecosystem_runner_mod");
    let compile_runner_mod = Command::new("rustc")
        .arg("--edition=2021")
        .arg("-D")
        .arg("warnings")
        .arg(&runner_src)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("--extern")
        .arg(format!("google_protobuf_roto={}", wkt_rlib.display()))
        .arg("--extern")
        .arg(format!("googleapis_roto={}", googleapis_mod_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&runner_bin_mod)
        .output()
        .expect("failed to compile ecosystem_runner_mod");
    assert!(
        compile_runner_mod.status.success(),
        "compiling ecosystem_runner_mod failed:\n{}",
        String::from_utf8_lossy(&compile_runner_mod.stderr)
    );

    let run_mod = Command::new(&runner_bin_mod)
        .output()
        .expect("failed to execute ecosystem_runner_mod");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    assert!(
        run_mod.status.success(),
        "ecosystem_runner_mod failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_mod.stdout),
        String::from_utf8_lossy(&run_mod.stderr)
    );
}
