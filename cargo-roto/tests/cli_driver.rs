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
    assert!(generated_lib.contains("pub struct SensorReadingBuilder<'a, 'b>"));

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
        .args([
            "check",
            "--manifest-path",
        ])
        .arg(&manifest_path)
        .args([
            "--target",
            "thumbv7em-none-eabihf",
            "--target-dir",
        ])
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
    assert!(lib_rs.contains("pub struct CodeGeneratorResponseBuilder<'a, 'b>"));
}
