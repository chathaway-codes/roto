use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(suffix: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "cargo_roto_cli_driver_e2e_{}_{}_{}",
            std::process::id(),
            nanos,
            suffix
        ));
        fs::create_dir_all(&dir).expect("create temp test dir");
        Self { path: dir }
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

const SENSOR_PROTO: &str = r#"syntax = "proto2";

package telemetry;

message Location {
    required int32 lat_e7 = 1;
    required int32 lon_e7 = 2;
}

message SensorReading {
    required uint64 timestamp_ms = 1;
    optional string device_id = 2 [default = "unknown"];
    optional Location location = 3;
    repeated uint32 samples = 4;
}
"#;

#[test]
fn cli_binaries_help_and_error_diagnostics() {
    let cargo_roto_bin = env!("CARGO_BIN_EXE_cargo-roto");
    let roto_build_bin = env!("CARGO_BIN_EXE_roto-build");

    // 1. `cargo-roto roto --help` prints usage to stdout and exits 0 with empty stderr
    let help_out = Command::new(cargo_roto_bin)
        .args(["roto", "--help"])
        .output()
        .expect("execute cargo-roto --help");
    assert!(help_out.status.success());
    assert!(help_out.stderr.is_empty());
    let stdout_str = String::from_utf8_lossy(&help_out.stdout);
    assert!(stdout_str.contains("Usage: cargo roto [build|generate] [OPTIONS]"));

    // 2. `roto-build -h` prints usage to stdout and exits 0 with empty stderr
    let rb_help = Command::new(roto_build_bin)
        .arg("-h")
        .output()
        .expect("execute roto-build -h");
    assert!(rb_help.status.success());
    assert!(rb_help.stderr.is_empty());
    let rb_stdout = String::from_utf8_lossy(&rb_help.stdout);
    assert!(rb_stdout.contains("Usage: roto-build [build|generate] [OPTIONS]"));

    // 3. Unknown flag exits non-zero with diagnostic on stderr and empty stdout
    let bad_flag = Command::new(cargo_roto_bin)
        .args(["roto", "--not-a-valid-flag"])
        .output()
        .expect("execute cargo-roto with invalid flag");
    assert!(!bad_flag.status.success());
    assert!(bad_flag.stdout.is_empty());
    let bad_stderr = String::from_utf8_lossy(&bad_flag.stderr);
    assert!(bad_stderr.contains("error: unknown flag '--not-a-valid-flag'"));

    // 4. Missing `--plugin` binary exits non-zero with diagnostic on stderr
    let bad_plugin = Command::new(roto_build_bin)
        .args(["--plugin", "/nonexistent/path/to/protoc-gen-roto"])
        .output()
        .expect("execute roto-build with missing plugin");
    assert!(!bad_plugin.status.success());
    assert!(bad_plugin.stdout.is_empty());
    let plugin_stderr = String::from_utf8_lossy(&bad_plugin.stderr);
    assert!(plugin_stderr.contains("error: --plugin path '/nonexistent/path/to/protoc-gen-roto'"));

    // 5. Missing `$PROTOC` binary exits non-zero with diagnostic on stderr
    let bad_protoc = Command::new(cargo_roto_bin)
        .env("PROTOC", "/nonexistent/path/to/protoc")
        .arg("roto")
        .output()
        .expect("execute cargo-roto with invalid PROTOC");
    assert!(!bad_protoc.status.success());
    assert!(bad_protoc.stdout.is_empty());
    let protoc_stderr = String::from_utf8_lossy(&bad_protoc.stderr);
    assert!(protoc_stderr.contains("error: PROTOC environment variable points to"));
}

#[test]
fn single_crate_end_to_end_generation_and_no_std_compilation() {
    let cargo_roto_bin = env!("CARGO_BIN_EXE_cargo-roto");
    let roto_build_bin = env!("CARGO_BIN_EXE_roto-build");
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cargo-roto is in workspace root");
    let roto_crate_path = workspace_root.join("roto");

    let tmp = TempDirGuard::new("sensor_crate");
    let proto_dir = tmp.path.join("proto/telemetry");
    fs::create_dir_all(&proto_dir).unwrap();
    fs::write(proto_dir.join("sensor.proto"), SENSOR_PROTO).unwrap();

    // Create minimal Cargo.toml (note: cargo metadata requires at least a placeholder src/lib.rs or target)
    let src_dir = tmp.path.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "// placeholder\n").unwrap();

    let cargo_toml = format!(
        r#"[package]
name = "sensor-roto-test"
version = "0.1.0"
edition = "2021"

[dependencies]
roto = {{ path = "{}" }}

[package.metadata.roto]
provides = ["telemetry"]
"#,
        roto_crate_path.display()
    );
    let manifest_path = tmp.path.join("Cargo.toml");
    fs::write(&manifest_path, cargo_toml).unwrap();

    // Generate `Cargo.lock` once so `--offline` works deterministically
    let lock_out = Command::new("cargo")
        .args(["generate-lockfile", "--manifest-path"])
        .arg(&manifest_path)
        .output()
        .expect("cargo generate-lockfile");
    assert!(
        lock_out.status.success(),
        "generate-lockfile failed: {}",
        String::from_utf8_lossy(&lock_out.stderr)
    );

    // 1. Run `cargo-roto roto build --manifest-path ... --offline`
    let out1 = Command::new(cargo_roto_bin)
        .args(["roto", "build", "--manifest-path"])
        .arg(&manifest_path)
        .arg("--offline")
        .output()
        .expect("execute cargo-roto");
    assert!(
        out1.status.success(),
        "cargo-roto failed with stderr: {}",
        String::from_utf8_lossy(&out1.stderr)
    );
    // Verify Unix Rule of Silence on success
    assert!(out1.stdout.is_empty(), "expected empty stdout");
    assert!(out1.stderr.is_empty(), "expected empty stderr");

    let generated_lib = fs::read_to_string(src_dir.join("lib.rs")).expect("read generated lib.rs");
    assert!(generated_lib.starts_with("#![no_std]"));
    assert!(generated_lib.contains("pub mod telemetry"));
    assert!(generated_lib.contains("pub struct SensorReadingView<'a>"));
    assert!(generated_lib.contains("pub struct SensorReadingBuilder<'b>"));

    // 2. Run `roto-build` directly in `tmp.path` without `--manifest-path` (tests default `./Cargo.toml`)
    fs::write(src_dir.join("lib.rs"), "// overwritten\n").unwrap();
    let out2 = Command::new(roto_build_bin)
        .current_dir(&tmp.path)
        .arg("--offline")
        .output()
        .expect("execute roto-build");
    assert!(
        out2.status.success(),
        "roto-build failed with stderr: {}",
        String::from_utf8_lossy(&out2.stderr)
    );
    assert!(out2.stdout.is_empty());
    assert!(out2.stderr.is_empty());

    let regenerated_lib =
        fs::read_to_string(src_dir.join("lib.rs")).expect("read regenerated lib.rs");
    assert_eq!(generated_lib, regenerated_lib);

    // 3. Verify the generated crate compiles cleanly on host and bare-metal `thumbv7em-none-eabihf`
    let target_dir = tmp.path.join("target");
    let check_host = Command::new("cargo")
        .args(["check", "--manifest-path"])
        .arg(&manifest_path)
        .arg("--target-dir")
        .arg(&target_dir)
        .arg("--offline")
        .output()
        .expect("cargo check host");
    assert!(
        check_host.status.success(),
        "cargo check failed: {}",
        String::from_utf8_lossy(&check_host.stderr)
    );

    let check_no_std = Command::new("cargo")
        .args(["check", "--manifest-path"])
        .arg(&manifest_path)
        .args(["--target", "thumbv7em-none-eabihf", "--target-dir"])
        .arg(&target_dir)
        .arg("--offline")
        .output()
        .expect("cargo check thumbv7em-none-eabihf");
    assert!(
        check_no_std.status.success(),
        "cargo check thumbv7em-none-eabihf failed: {}",
        String::from_utf8_lossy(&check_no_std.stderr)
    );
}

#[test]
fn generates_google_protobuf_roto_to_custom_out_dir_silently() {
    let cargo_roto_bin = env!("CARGO_BIN_EXE_cargo-roto");
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cargo-roto is in workspace root");
    let wkt_manifest = workspace_root.join("google-protobuf-roto/Cargo.toml");

    let tmp = TempDirGuard::new("wkt_out");
    let custom_out = tmp.path.join("generated_src");

    let out = Command::new(cargo_roto_bin)
        .args(["roto", "generate", "--manifest-path"])
        .arg(&wkt_manifest)
        .arg("--out-dir")
        .arg(&custom_out)
        .arg("--offline")
        .output()
        .expect("execute cargo-roto on google-protobuf-roto");

    assert!(
        out.status.success(),
        "cargo-roto failed on google-protobuf-roto: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.stdout.is_empty());
    assert!(out.stderr.is_empty());

    let lib_rs = fs::read_to_string(custom_out.join("lib.rs")).expect("read generated WKT lib.rs");
    assert!(lib_rs.starts_with("#![no_std]"));
    assert!(lib_rs.contains("pub mod google"));
    assert!(lib_rs.contains("pub mod protobuf"));
    assert!(lib_rs.contains("pub mod compiler"));
    assert!(lib_rs.contains("pub struct FileDescriptorSetView<'a>"));
    assert!(lib_rs.contains("pub struct CodeGeneratorRequestView<'a>"));
    assert!(lib_rs.contains("pub struct CodeGeneratorResponseBuilder<'b>"));
    assert!(!custom_out.join("grpc.rs").exists());
}

const SENSOR_SERVICE_PROTO: &str = r#"syntax = "proto2";

package telemetry;

message Location {
    required int32 lat_e7 = 1;
    required int32 lon_e7 = 2;
}

message SensorReading {
    required uint64 timestamp_ms = 1;
    optional string device_id = 2 [default = "unknown"];
    optional Location location = 3;
}

service SensorCollector {
    rpc RecordReading (SensorReading) returns (Location);
}
"#;

#[test]
fn opt_in_grpc_generation_via_cli_and_metadata_compiles_no_std() {
    let cargo_roto_bin = env!("CARGO_BIN_EXE_cargo-roto");
    let roto_build_bin = env!("CARGO_BIN_EXE_roto-build");
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cargo-roto is in workspace root");
    let roto_crate_path = workspace_root.join("roto");
    let roto_grpc_crate_path = workspace_root.join("roto-grpc");

    let tmp = TempDirGuard::new("sensor_grpc_crate");
    let proto_dir = tmp.path.join("proto/telemetry");
    fs::create_dir_all(&proto_dir).unwrap();
    fs::write(proto_dir.join("sensor.proto"), SENSOR_SERVICE_PROTO).unwrap();

    let src_dir = tmp.path.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "// placeholder\n").unwrap();

    let cargo_toml = format!(
        r#"[package]
name = "sensor-grpc-roto-test"
version = "0.1.0"
edition = "2021"

[dependencies]
roto = {{ path = "{}", default-features = false }}
roto-grpc = {{ path = "{}", default-features = false }}

[package.metadata.roto]
provides = ["telemetry"]
grpc = true
"#,
        roto_crate_path.display(),
        roto_grpc_crate_path.display()
    );
    let manifest_path = tmp.path.join("Cargo.toml");
    fs::write(&manifest_path, cargo_toml).unwrap();

    let lock_out = Command::new("cargo")
        .args(["generate-lockfile", "--manifest-path"])
        .arg(&manifest_path)
        .output()
        .expect("cargo generate-lockfile");
    assert!(
        lock_out.status.success(),
        "generate-lockfile failed: {}",
        String::from_utf8_lossy(&lock_out.stderr)
    );

    // 1. Run `cargo-roto` (which picks up `grpc = true` from `[package.metadata.roto]`)
    let out1 = Command::new(cargo_roto_bin)
        .args(["roto", "build", "--manifest-path"])
        .arg(&manifest_path)
        .arg("--offline")
        .output()
        .expect("execute cargo-roto with grpc = true");
    assert!(
        out1.status.success(),
        "cargo-roto failed with stderr: {}",
        String::from_utf8_lossy(&out1.stderr)
    );
    assert!(out1.stdout.is_empty());
    assert!(out1.stderr.is_empty());

    let lib_rs = fs::read_to_string(src_dir.join("lib.rs")).expect("read generated lib.rs");
    let grpc_rs = fs::read_to_string(src_dir.join("grpc.rs")).expect("read generated grpc.rs");
    assert!(lib_rs.contains("pub mod grpc;"));
    assert!(grpc_rs.contains("pub mod sensor_collector {"));
    assert!(grpc_rs.contains("pub trait SensorCollector {"));
    assert!(grpc_rs.contains("pub trait AsyncSensorCollector {"));
    assert!(grpc_rs.contains("pub struct SensorCollectorServer<T>"));
    assert!(grpc_rs.contains("pub struct SensorCollectorClient<C>"));

    // 2. Also verify `--grpc` CLI flag with `roto-build`
    fs::remove_file(src_dir.join("grpc.rs")).unwrap();
    let out2 = Command::new(roto_build_bin)
        .current_dir(&tmp.path)
        .args(["--grpc", "--offline"])
        .output()
        .expect("execute roto-build --grpc");
    assert!(out2.status.success());
    assert!(src_dir.join("grpc.rs").is_file());

    // 3. Verify generated crate + grpc.rs compiles on host and `thumbv7em-none-eabihf`
    let target_dir = tmp.path.join("target");
    let check_host = Command::new("cargo")
        .args(["check", "--manifest-path"])
        .arg(&manifest_path)
        .arg("--target-dir")
        .arg(&target_dir)
        .arg("--offline")
        .output()
        .expect("cargo check host");
    assert!(
        check_host.status.success(),
        "cargo check host failed: {}",
        String::from_utf8_lossy(&check_host.stderr)
    );

    let check_no_std = Command::new("cargo")
        .args(["check", "--manifest-path"])
        .arg(&manifest_path)
        .args(["--target", "thumbv7em-none-eabihf", "--target-dir"])
        .arg(&target_dir)
        .arg("--offline")
        .output()
        .expect("cargo check thumbv7em-none-eabihf");
    assert!(
        check_no_std.status.success(),
        "cargo check thumbv7em-none-eabihf failed: {}",
        String::from_utf8_lossy(&check_no_std.stderr)
    );
}

#[test]
fn metadata_proto_paths_inputs_out_dir_mode_cli_overrides_and_unknown_key_rejection() {
    let cargo_roto_bin = env!("CARGO_BIN_EXE_cargo-roto");
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cargo-roto is in workspace root");
    let roto_crate_path = workspace_root.join("roto");
    let roto_grpc_crate_path = workspace_root.join("roto-grpc");

    let tmp = TempDirGuard::new("docs_grpc_metadata_snippet");
    let proto_dir = tmp.path.join("proto");
    fs::create_dir_all(&proto_dir).unwrap();
    fs::write(proto_dir.join("sensor.proto"), SENSOR_SERVICE_PROTO).unwrap();
    // Place a second broken/unreferenced .proto file in `proto/` to verify `inputs = ["proto/sensor.proto"]`
    // only compiles `sensor.proto` rather than all `proto/**/*.proto` files.
    fs::write(
        proto_dir.join("unselected.proto"),
        "this is not valid protobuf syntax;\n",
    )
    .unwrap();

    let src_dir = tmp.path.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "// placeholder\n").unwrap();

    // Exact `[package.metadata.roto]` snippet documented in `docs/grpc.md`:
    let cargo_toml = format!(
        r#"[package]
name = "sensor-metadata-config-test"
version = "0.1.0"
edition = "2021"

[dependencies]
roto = {{ path = "{}", default-features = false }}
roto-grpc = {{ path = "{}", default-features = false }}

[package.metadata.roto]
proto_paths = ["proto"]
inputs = ["proto/sensor.proto"]
out_dir = "src/generated"
grpc = true
"#,
        roto_crate_path.display(),
        roto_grpc_crate_path.display()
    );
    let manifest_path = tmp.path.join("Cargo.toml");
    fs::write(&manifest_path, cargo_toml).unwrap();

    let lock_out = Command::new("cargo")
        .args(["generate-lockfile", "--manifest-path"])
        .arg(&manifest_path)
        .output()
        .expect("cargo generate-lockfile");
    assert!(
        lock_out.status.success(),
        "generate-lockfile failed: {}",
        String::from_utf8_lossy(&lock_out.stderr)
    );

    // 1. Run `cargo-roto` using `[package.metadata.roto]` configuration
    let out1 = Command::new(cargo_roto_bin)
        .args(["roto", "build", "--manifest-path"])
        .arg(&manifest_path)
        .arg("--offline")
        .output()
        .expect("execute cargo-roto with docs/grpc.md metadata snippet");
    assert!(
        out1.status.success(),
        "cargo-roto failed with stderr: {}",
        String::from_utf8_lossy(&out1.stderr)
    );
    assert!(out1.stdout.is_empty());
    assert!(out1.stderr.is_empty());

    let generated_dir = tmp.path.join("src/generated");
    let gen_lib =
        fs::read_to_string(generated_dir.join("lib.rs")).expect("read src/generated/lib.rs");
    let gen_grpc =
        fs::read_to_string(generated_dir.join("grpc.rs")).expect("read src/generated/grpc.rs");
    assert!(gen_lib.contains("pub mod telemetry"));
    assert!(gen_lib.contains("pub mod grpc;"));
    assert!(gen_grpc.contains("pub trait SensorCollector {"));

    // 2. Verify CLI flags (`--out-dir`, `--proto-dir`, `--mode`) override `[package.metadata.roto]`
    let alt_proto_dir = tmp.path.join("alt_proto");
    fs::create_dir_all(&alt_proto_dir).unwrap();
    fs::write(alt_proto_dir.join("sensor.proto"), SENSOR_PROTO).unwrap();
    let alt_out_dir = tmp.path.join("alt_out");

    let out_override = Command::new(cargo_roto_bin)
        .args(["roto", "build", "--manifest-path"])
        .arg(&manifest_path)
        .arg("--proto-dir")
        .arg(&alt_proto_dir)
        .arg("--out-dir")
        .arg(&alt_out_dir)
        .args(["--mode", "mod", "--offline"])
        .output()
        .expect("execute cargo-roto with CLI overrides");
    assert!(
        out_override.status.success(),
        "cargo-roto with CLI overrides failed: {}",
        String::from_utf8_lossy(&out_override.stderr)
    );
    let alt_mod = fs::read_to_string(alt_out_dir.join("mod.rs")).expect("read alt_out/mod.rs");
    assert!(!alt_mod.starts_with("#![no_std]"));
    assert!(alt_mod.contains("pub mod telemetry"));

    // 3. Verify unknown key in `[package.metadata.roto]` is rejected with a clear diagnostic
    let bad_cargo_toml = format!(
        r#"[package]
name = "sensor-metadata-config-test"
version = "0.1.0"
edition = "2021"

[dependencies]
roto = {{ path = "{}", default-features = false }}

[package.metadata.roto]
unknown_setting = true
"#,
        roto_crate_path.display()
    );
    fs::write(&manifest_path, bad_cargo_toml).unwrap();

    let bad_out = Command::new(cargo_roto_bin)
        .args(["roto", "build", "--manifest-path"])
        .arg(&manifest_path)
        .arg("--offline")
        .output()
        .expect("execute cargo-roto with unknown metadata key");
    assert!(!bad_out.status.success());
    assert!(bad_out.stdout.is_empty());
    let bad_stderr = String::from_utf8_lossy(&bad_out.stderr);
    assert!(
        bad_stderr.contains("unknown key 'unknown_setting' in [package.metadata.roto]"),
        "unexpected stderr: {bad_stderr}"
    );
    assert!(
        bad_stderr.contains("provides, proto_dir, proto_paths, inputs, out_dir, mode, grpc"),
        "unexpected stderr: {bad_stderr}"
    );
}

#[test]
fn docs_grpc_sensor_reflection_e2e_auto_descriptors_and_drive_connection() {
    let cargo_roto_bin = env!("CARGO_BIN_EXE_cargo-roto");
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cargo-roto is in workspace root");
    let roto_path = workspace_root.join("roto");
    let roto_runtime_path = workspace_root.join("roto-runtime");
    let roto_grpc_path = workspace_root.join("roto-grpc");
    let grpc_reflection_path = workspace_root.join("grpc-reflection-roto");

    let tmp = TempDirGuard::new("docs_grpc_roto_test_reflection");
    let proto_dir = tmp.path.join("proto");
    fs::create_dir_all(&proto_dir).unwrap();

    // Exact `proto/sensor.proto` from `docs/grpc.md` Step 1:
    let docs_sensor_proto = r#"syntax = "proto3";
package telemetry.v1;

message Location {
  optional int32 lat_e7 = 1;
  optional int32 lon_e7 = 2;
}

message SensorReading {
  optional string device_id = 1;
  optional uint64 timestamp_ms = 2;
  optional Location location = 3;
}

service SensorCollector {
  rpc ReportLocation(SensorReading) returns (Location);
}
"#;
    fs::write(proto_dir.join("sensor.proto"), docs_sensor_proto).unwrap();

    let src_dir = tmp.path.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "// placeholder\n").unwrap();

    // User crate depends on `grpc-reflection-roto` without `[package.metadata.roto]` or `--descriptors`
    let cargo_toml = format!(
        r#"[package]
name = "roto-test"
version = "0.1.0"
edition = "2021"

[dependencies]
roto = {{ path = "{}", default-features = false }}
roto-runtime = {{ path = "{}", default-features = false }}
roto-grpc = {{ path = "{}", default-features = false }}
grpc-reflection-roto = {{ path = "{}", default-features = false }}
"#,
        roto_path.display(),
        roto_runtime_path.display(),
        roto_grpc_path.display(),
        grpc_reflection_path.display(),
    );
    let manifest_path = tmp.path.join("Cargo.toml");
    fs::write(&manifest_path, cargo_toml).unwrap();

    let lock_out = Command::new("cargo")
        .args(["generate-lockfile", "--manifest-path"])
        .arg(&manifest_path)
        .output()
        .expect("cargo generate-lockfile");
    assert!(
        lock_out.status.success(),
        "generate-lockfile failed: {}",
        String::from_utf8_lossy(&lock_out.stderr)
    );

    // Run `cargo roto --grpc` WITHOUT `--descriptors`
    let gen_out = Command::new(cargo_roto_bin)
        .args(["roto", "--grpc", "--manifest-path"])
        .arg(&manifest_path)
        .arg("--offline")
        .output()
        .expect("execute cargo-roto --grpc");
    assert!(
        gen_out.status.success(),
        "cargo-roto --grpc failed: {}",
        String::from_utf8_lossy(&gen_out.stderr)
    );

    let lib_rs = fs::read_to_string(src_dir.join("lib.rs")).expect("read generated lib.rs");
    let grpc_rs = fs::read_to_string(src_dir.join("grpc.rs")).expect("read generated grpc.rs");
    assert!(
        lib_rs.contains("pub const FILE_DESCRIPTOR_BYTES: &[u8] ="),
        "expected FILE_DESCRIPTOR_BYTES to be auto-emitted when depending on grpc-reflection-roto"
    );
    assert!(
        grpc_rs.contains("pub const FILE_DESCRIPTOR_SET: &[&[u8]] = &[FILE_DESCRIPTOR_BYTES];"),
        "expected non-empty FILE_DESCRIPTOR_SET in generated grpc.rs"
    );

    // Write an integration test in `roto-test` using the generated `SensorCollectorServer` +
    // `ReflectionService` + `drive_connection` from `docs/grpc.md`
    let tests_dir = tmp.path.join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    let test_rs = r#"use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use grpc_reflection_roto::{
    grpc::reflection::v1::{self, server_reflection_response::MessageResponse as V1Resp},
    services::grpc::reflection::v1 as v1_grpc,
    ReflectionService,
};
use roto_grpc::{
    h2::{H2Connection, H2Event, Role},
    hpack::GRPC_CONTENT_TYPE,
    GrpcServer, GrpcStatus, Route,
};
use roto_test::grpc::telemetry::v1::sensor_collector::{SensorCollector, SensorCollectorServer};
use roto_test::telemetry::v1::{LocationBuilder, SensorReadingView};

pub struct MySensorHandler;

impl SensorCollector for MySensorHandler {
    fn report_location<'a, 'b>(
        &self,
        req: SensorReadingView<'a>,
        resp: &mut LocationBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let Some(loc) = req
            .location()
            .map_err(|_| GrpcStatus::internal("corrupt location submessage"))?
        else {
            return Err(GrpcStatus::invalid_argument("missing location"));
        };
        let lat = loc.lat_e7().map_err(|_| GrpcStatus::internal("corrupt lat"))?;
        let lon = loc.lon_e7().map_err(|_| GrpcStatus::internal("corrupt lon"))?;
        resp.write_lon_e7(lon + 1).write_lat_e7(lat + 1);
        Ok(())
    }
}

fn drive_connection(
    stream: &mut TcpStream,
    server: &mut GrpcServer<'_>,
    io_buf: &mut [u8],
) -> std::io::Result<()> {
    loop {
        loop {
            let tx_n = server.poll_transmit(io_buf);
            if tx_n == 0 {
                break;
            }
            stream.write_all(&io_buf[..tx_n])?;
        }
        stream.flush()?;

        let rx_n = stream.read(io_buf)?;
        if rx_n == 0 {
            return Ok(());
        }

        if server.recv(&io_buf[..rx_n]).is_err() {
            let tx_n = server.poll_transmit(io_buf);
            if tx_n > 0 {
                let _ = stream.write_all(&io_buf[..tx_n]);
            }
            return Ok(());
        }
    }
}

#[test]
fn grpc_cli_ls_sensor_collector_over_drive_connection() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    let server_thread = std::thread::spawn(move || {
        let svc = SensorCollectorServer::new(MySensorHandler);
        let app_routes: [&dyn Route; 1] = [&svc];
        let reflection = ReflectionService::new(&app_routes);
        let refl_v1 = reflection.v1_server();
        let refl_v1alpha = reflection.v1alpha_server();
        let routes: [&dyn Route; 3] = [&svc, &refl_v1, &refl_v1alpha];

        let mut msg_buf = [0u8; 8192];
        let mut tx_buf = [0u8; 16384];
        let mut io_buf = [0u8; 4096];
        let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg_buf, &mut tx_buf);
        let (mut stream, _) = listener.accept().unwrap();
        let _ = stream.set_nodelay(true);
        server.reset();
        drive_connection(&mut stream, &mut server, &mut io_buf).unwrap();
    });

    let mut cli_stream = TcpStream::connect(addr).unwrap();
    let _ = cli_stream.set_nodelay(true);
    let mut raw_cli: H2Connection<4, 4096> = H2Connection::new(Role::Client);
    let mut wire = [0u8; 8192];
    let mut rx_buf = [0u8; 4096];

    let n0 = raw_cli.poll_transmit(&mut wire);
    cli_stream.write_all(&wire[..n0]).unwrap();
    let (stream_id, hdr_n) = raw_cli
        .send_grpc_request_headers(
            b"http",
            b"127.0.0.1:50051",
            v1_grpc::server_reflection::PATH_SERVER_REFLECTION_INFO.as_bytes(),
            GRPC_CONTENT_TYPE,
            false,
            &mut wire,
        )
        .unwrap();
    cli_stream.write_all(&wire[..hdr_n]).unwrap();

    // Send `FindFileContainingSymbol("telemetry.v1.SensorCollector")` with `end_stream = false`
    let data_n = raw_cli
        .send_grpc_data_frame(stream_id, false, false, &mut wire, |w| {
            let mut b = v1::ServerReflectionRequestBuilder::from_writer(core::mem::replace(
                w,
                roto::runtime::BackwardWriter::new(&mut []),
            ));
            b.write_file_containing_symbol("telemetry.v1.SensorCollector");
            *w = b.writer;
        })
        .unwrap()
        .len();
    let data_start = wire.len() - data_n;
    cli_stream.write_all(&wire[data_start..]).unwrap();
    cli_stream.flush().unwrap();

    let mut resp_payload: Option<Vec<u8>> = None;
    while resp_payload.is_none() {
        let rx_n = cli_stream.read(&mut rx_buf).unwrap();
        assert!(rx_n > 0);
        let mut pos = 0;
        while pos < rx_n {
            let (c, ev) = raw_cli.recv(&rx_buf[pos..rx_n]).unwrap();
            pos += c;
            if let Some(H2Event::GrpcMessage {
                stream_id: 1,
                payload,
                ..
            }) = ev
            {
                resp_payload = Some(payload.to_vec());
            }
        }
    }
    let payload_bytes = resp_payload.unwrap();
    let resp_view = v1::ServerReflectionResponseView::from_bytes(&payload_bytes).unwrap();
    match resp_view.message_response().unwrap() {
        Some(V1Resp::FileDescriptorResponse(fdr)) => {
            let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
            assert_eq!(protos, vec![roto_test::telemetry::v1::FILE_DESCRIPTOR_BYTES]);
        }
        other => panic!("expected FileDescriptorResponse, got {other:?}"),
    }

    cli_stream.shutdown(Shutdown::Both).unwrap();
    drop(cli_stream);
    server_thread.join().unwrap();
}
"#;
    fs::write(tests_dir.join("reflection_sensor_e2e.rs"), test_rs).unwrap();

    let target_dir = tmp.path.join("target");
    let test_out = Command::new("cargo")
        .args(["test", "--manifest-path"])
        .arg(&manifest_path)
        .arg("--target-dir")
        .arg(&target_dir)
        .arg("--offline")
        .output()
        .expect("run cargo test on roto-test");
    assert!(
        test_out.status.success(),
        "cargo test on roto-test failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&test_out.stdout),
        String::from_utf8_lossy(&test_out.stderr)
    );
}

#[test]
fn resolves_bundled_google_api_annotations_without_manual_include_flags() {
    let cargo_roto_bin = env!("CARGO_BIN_EXE_cargo-roto");
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cargo-roto is in workspace root");
    let roto_path = workspace_root.join("roto");
    let roto_grpc_path = workspace_root.join("roto-grpc");

    let tmp = TempDirGuard::new("bundled_google_api_annotations");
    let proto_dir = tmp.path.join("proto");
    fs::create_dir_all(&proto_dir).unwrap();

    let annotated_proto = r#"syntax = "proto3";
package telemetry.v1;

import "google/api/annotations.proto";

message GetReadingRequest {
  string device_id = 1;
}

message Reading {
  string device_id = 1;
  uint64 value = 2;
}

service ReadingService {
  rpc GetReading(GetReadingRequest) returns (Reading) {
    option (google.api.http) = {
      get: "/v1/devices/{device_id}/reading"
    };
  }
}
"#;
    fs::write(proto_dir.join("reading.proto"), annotated_proto).unwrap();

    let src_dir = tmp.path.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "// placeholder\n").unwrap();

    let cargo_toml = format!(
        r#"[package]
name = "annotated-grpc-test"
version = "0.1.0"
edition = "2021"

[dependencies]
roto = {{ path = "{}", default-features = false }}
roto-grpc = {{ path = "{}", default-features = false }}

[package.metadata.roto]
grpc = true
"#,
        roto_path.display(),
        roto_grpc_path.display()
    );
    let manifest_path = tmp.path.join("Cargo.toml");
    fs::write(&manifest_path, cargo_toml).unwrap();

    let lock_out = Command::new("cargo")
        .args(["generate-lockfile", "--manifest-path"])
        .arg(&manifest_path)
        .output()
        .expect("cargo generate-lockfile");
    assert!(lock_out.status.success());

    let out = Command::new(cargo_roto_bin)
        .args(["roto", "build", "--manifest-path"])
        .arg(&manifest_path)
        .arg("--offline")
        .output()
        .expect("execute cargo-roto with google/api/annotations.proto import");
    assert!(
        out.status.success(),
        "cargo-roto failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.stdout.is_empty());
    assert!(out.stderr.is_empty());
    let grpc_default = fs::read_to_string(src_dir.join("grpc.rs")).unwrap();
    assert!(!grpc_default.contains("HttpService"));

    // Run with `--http` CLI flag and verify HttpService / AsyncHttpService are generated
    let out_cli_http = Command::new(cargo_roto_bin)
        .args(["roto", "build", "--http", "--manifest-path"])
        .arg(&manifest_path)
        .arg("--offline")
        .output()
        .expect("execute cargo-roto --http");
    assert!(
        out_cli_http.status.success(),
        "cargo-roto --http failed: {}",
        String::from_utf8_lossy(&out_cli_http.stderr)
    );
    let grpc_cli_http = fs::read_to_string(src_dir.join("grpc.rs")).unwrap();
    assert!(grpc_cli_http.contains("HttpService"));
    assert!(grpc_cli_http.contains("AsyncHttpService"));

    // Configure `http = true` in `[package.metadata.roto]` (which also implies `grpc = true`),
    // regenerate, wire `pub mod grpc;` into `src/lib.rs`, and verify `cargo check` succeeds.
    fs::remove_file(src_dir.join("grpc.rs")).unwrap();
    let cargo_toml_http = format!(
        r#"[package]
name = "annotated-grpc-test"
version = "0.1.0"
edition = "2021"

[dependencies]
roto = {{ path = "{}", default-features = false }}
roto-grpc = {{ path = "{}", default-features = false }}

[package.metadata.roto]
http = true
"#,
        roto_path.display(),
        roto_grpc_path.display()
    );
    fs::write(&manifest_path, cargo_toml_http).unwrap();

    let out_meta_http = Command::new(cargo_roto_bin)
        .args(["roto", "build", "--manifest-path"])
        .arg(&manifest_path)
        .arg("--offline")
        .output()
        .expect("execute cargo-roto with http = true metadata");
    assert!(
        out_meta_http.status.success(),
        "cargo-roto metadata http = true failed: {}",
        String::from_utf8_lossy(&out_meta_http.stderr)
    );
    let grpc_meta_http = fs::read_to_string(src_dir.join("grpc.rs")).unwrap();
    assert!(grpc_meta_http.contains("HttpService"));
    assert!(grpc_meta_http.contains("AsyncHttpService"));

    let check_out = Command::new("cargo")
        .args(["check", "--manifest-path"])
        .arg(&manifest_path)
        .arg("--offline")
        .env("RUSTFLAGS", "-D warnings")
        .output()
        .expect("cargo check generated http crate");
    assert!(
        check_out.status.success(),
        "cargo check failed: {}",
        String::from_utf8_lossy(&check_out.stderr)
    );
}
