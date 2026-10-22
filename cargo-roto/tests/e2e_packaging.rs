use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use cargo_roto::metadata::{resolve_workspace, MetadataOptions};

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
            "cargo_roto_e2e_packaging_{}_{}_{}",
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

fn init_and_commit_git_repo(repo_dir: &Path) {
    let run_git = |args: &[&str]| {
        let out = Command::new("git")
            .current_dir(repo_dir)
            .args([
                "-c",
                "user.name=Roto Test",
                "-c",
                "user.email=test@roto.local",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "init.defaultBranch=main",
            ])
            .args(args)
            .output()
            .expect("failed to execute git");
        assert!(
            out.status.success(),
            "git {:?} in '{}' failed: {}",
            args,
            repo_dir.display(),
            String::from_utf8_lossy(&out.stderr)
        );
    };

    run_git(&["init"]);
    run_git(&["add", "."]);
    run_git(&["commit", "-m", "Initial commit"]);
}

#[test]
fn self_hosting_google_protobuf_roto_matches_committed_lib_rs_byte_for_byte() {
    let cargo_roto_bin = env!("CARGO_BIN_EXE_cargo-roto");
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cargo-roto is in workspace root");

    for crate_name in [
        "google-protobuf-roto-bootstrap",
        "google-protobuf-roto",
        "grpc-reflection-roto",
    ] {
        let manifest = workspace_root.join(format!("{crate_name}/Cargo.toml"));
        let committed_lib_rs = workspace_root.join(format!("{crate_name}/src/lib.rs"));

        let tmp = TempDirGuard::new(&format!("self_host_fixed_point_{crate_name}"));
        let out_dir = tmp.path.join("out");

        let out = Command::new(cargo_roto_bin)
            .args(["roto", "--manifest-path"])
            .arg(&manifest)
            .arg("--out-dir")
            .arg(&out_dir)
            .arg("--offline")
            .output()
            .expect("execute cargo-roto");

        assert!(
            out.status.success(),
            "cargo-roto failed on {crate_name}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(out.stdout.is_empty());
        assert!(out.stderr.is_empty());

        let committed_gen_mod = workspace_root.join(format!("{crate_name}/src/generated/mod.rs"));
        if committed_gen_mod.is_file() {
            let generated_bytes = fs::read(out_dir.join("mod.rs")).expect("read generated mod.rs");
            let committed_bytes = fs::read(&committed_gen_mod).expect("read committed mod.rs");
            assert_eq!(
                generated_bytes, committed_bytes,
                "regenerated {crate_name}/src/generated/mod.rs diverged from committed file"
            );

            let committed_grpc_rs =
                workspace_root.join(format!("{crate_name}/src/generated/grpc.rs"));
            if committed_grpc_rs.is_file() {
                let generated_grpc =
                    fs::read(out_dir.join("grpc.rs")).expect("read generated grpc.rs");
                let committed_grpc = fs::read(&committed_grpc_rs).expect("read committed grpc.rs");
                assert_eq!(
                    generated_grpc, committed_grpc,
                    "regenerated {crate_name}/src/generated/grpc.rs diverged from committed file"
                );
            }
        } else {
            let generated_bytes = fs::read(out_dir.join("lib.rs")).expect("read generated lib.rs");
            let committed_bytes = fs::read(&committed_lib_rs).expect("read committed lib.rs");
            assert_eq!(
                generated_bytes, committed_bytes,
                "regenerated {crate_name}/src/lib.rs diverged from committed file"
            );
        }
    }
}

#[test]
fn multi_crate_git_dependencies_extern_path_roundtrip_and_bare_metal_no_std() {
    let cargo_roto_bin = env!("CARGO_BIN_EXE_cargo-roto");
    let roto_build_bin = env!("CARGO_BIN_EXE_roto-build");
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cargo-roto is in workspace root");
    let roto_crate_path = workspace_root.join("roto");

    let tmp = TempDirGuard::new("multi_crate_git");
    let cargo_home = tmp.path.join("cargo_home");
    fs::create_dir_all(&cargo_home).unwrap();

    // =========================================================================
    // 1. Upstream Crate 1: `google-protobuf-roto`
    // =========================================================================
    let wkt_dir = tmp.path.join("google-protobuf-roto");
    fs::create_dir_all(wkt_dir.join("proto/google/protobuf")).unwrap();
    fs::create_dir_all(wkt_dir.join("src")).unwrap();
    fs::write(wkt_dir.join("src/lib.rs"), "// placeholder\n").unwrap();

    let timestamp_proto = r#"syntax = "proto2";
package google.protobuf;

message Timestamp {
    required int64 seconds = 1;
    optional int32 nanos = 2 [default = 0];
}
"#;
    fs::write(
        wkt_dir.join("proto/google/protobuf/timestamp.proto"),
        timestamp_proto,
    )
    .unwrap();

    let wkt_cargo_toml = format!(
        r#"[package]
name = "google-protobuf-roto"
version = "0.1.0"
edition = "2021"

[dependencies]
roto = {{ path = "{}", default-features = false }}

[package.metadata.roto]
provides = ["google/protobuf"]
"#,
        roto_crate_path.display()
    );
    let wkt_manifest = wkt_dir.join("Cargo.toml");
    fs::write(&wkt_manifest, wkt_cargo_toml).unwrap();

    let wkt_gen = Command::new(cargo_roto_bin)
        .env("CARGO_HOME", &cargo_home)
        .args(["roto", "--manifest-path"])
        .arg(&wkt_manifest)
        .output()
        .expect("run cargo-roto on upstream google-protobuf-roto");
    assert!(
        wkt_gen.status.success(),
        "cargo-roto failed on google-protobuf-roto: {}",
        String::from_utf8_lossy(&wkt_gen.stderr)
    );
    assert!(wkt_gen.stdout.is_empty());
    assert!(wkt_gen.stderr.is_empty());

    let wkt_lib = fs::read_to_string(wkt_dir.join("src/lib.rs")).unwrap();
    assert!(wkt_lib.starts_with("#![no_std]"));
    assert!(wkt_lib.contains("pub struct TimestampView<'a>"));
    assert!(wkt_lib.contains("pub struct TimestampBuilder<'b>"));

    init_and_commit_git_repo(&wkt_dir);

    // =========================================================================
    // 2. Upstream Crate 2: `googleapis-roto` (multi-namespace + keyword `type`)
    // =========================================================================
    let gapis_dir = tmp.path.join("googleapis-roto");
    fs::create_dir_all(gapis_dir.join("proto/google/api")).unwrap();
    fs::create_dir_all(gapis_dir.join("proto/google/rpc")).unwrap();
    fs::create_dir_all(gapis_dir.join("proto/google/type")).unwrap();
    fs::create_dir_all(gapis_dir.join("src")).unwrap();
    fs::write(gapis_dir.join("src/lib.rs"), "// placeholder\n").unwrap();

    let latlng_proto = r#"syntax = "proto2";
package google.type;

message LatLng {
    required int32 latitude_e7 = 1;
    required int32 longitude_e7 = 2;
}
"#;
    fs::write(
        gapis_dir.join("proto/google/type/latlng.proto"),
        latlng_proto,
    )
    .unwrap();

    let status_proto = r#"syntax = "proto2";
package google.rpc;

message Status {
    required int32 code = 1;
    optional string message = 2;
}
"#;
    fs::write(
        gapis_dir.join("proto/google/rpc/status.proto"),
        status_proto,
    )
    .unwrap();

    let gapis_cargo_toml = format!(
        r#"[package]
name = "googleapis-roto"
version = "0.1.0"
edition = "2021"

[dependencies]
roto = {{ path = "{}", default-features = false }}

[package.metadata.roto]
provides = ["google/api", "google/rpc", "google/type"]
"#,
        roto_crate_path.display()
    );
    let gapis_manifest = gapis_dir.join("Cargo.toml");
    fs::write(&gapis_manifest, gapis_cargo_toml).unwrap();

    let gapis_gen = Command::new(roto_build_bin)
        .env("CARGO_HOME", &cargo_home)
        .arg("--manifest-path")
        .arg(&gapis_manifest)
        .output()
        .expect("run roto-build on upstream googleapis-roto");
    assert!(
        gapis_gen.status.success(),
        "roto-build failed on googleapis-roto: {}",
        String::from_utf8_lossy(&gapis_gen.stderr)
    );
    assert!(gapis_gen.stdout.is_empty());
    assert!(gapis_gen.stderr.is_empty());

    let gapis_lib = fs::read_to_string(gapis_dir.join("src/lib.rs")).unwrap();
    assert!(gapis_lib.starts_with("#![no_std]"));
    assert!(gapis_lib.contains("pub mod google"));
    assert!(gapis_lib.contains("pub mod r#type"));
    assert!(gapis_lib.contains("pub mod rpc"));
    assert!(gapis_lib.contains("pub struct LatLngView<'a>"));
    assert!(gapis_lib.contains("pub struct StatusView<'a>"));

    init_and_commit_git_repo(&gapis_dir);

    // =========================================================================
    // 3. Downstream Crate: `telemetry-roto` importing both git dependencies
    // =========================================================================
    let telemetry_dir = tmp.path.join("telemetry-roto");
    fs::create_dir_all(telemetry_dir.join("proto/telemetry")).unwrap();
    fs::create_dir_all(telemetry_dir.join("src")).unwrap();
    fs::write(telemetry_dir.join("src/lib.rs"), "// placeholder\n").unwrap();

    let sensor_proto = r#"syntax = "proto2";
package telemetry;

import "google/protobuf/timestamp.proto";
import "google/type/latlng.proto";

message SensorReading {
    required .google.protobuf.Timestamp recorded_at = 1;
    optional string device_id = 2 [default = "unknown"];
    optional .google.type.LatLng location = 3;
    repeated uint32 samples = 4;
}
"#;
    fs::write(
        telemetry_dir.join("proto/telemetry/sensor.proto"),
        sensor_proto,
    )
    .unwrap();

    let roto_runtime_crate_path = workspace_root.join("roto-runtime");
    let telemetry_cargo_toml = format!(
        r#"[package]
name = "telemetry-roto"
version = "0.1.0"
edition = "2021"

[dependencies]
roto = {{ path = "{roto}", default-features = false }}
google-protobuf-roto = {{ git = "file://{wkt}" }}
googleapis-roto = {{ git = "file://{gapis}" }}

[package.metadata.roto]
provides = ["telemetry"]

[patch."file://{wkt}"]
roto = {{ path = "{roto}" }}
roto-runtime = {{ path = "{roto_runtime}" }}

[patch."file://{gapis}"]
roto = {{ path = "{roto}" }}
roto-runtime = {{ path = "{roto_runtime}" }}
"#,
        roto = roto_crate_path.display(),
        roto_runtime = roto_runtime_crate_path.display(),
        wkt = wkt_dir.display(),
        gapis = gapis_dir.display()
    );
    let telemetry_manifest = telemetry_dir.join("Cargo.toml");
    fs::write(&telemetry_manifest, telemetry_cargo_toml).unwrap();

    // Run `cargo-roto` on `telemetry-roto` with isolated `CARGO_HOME`
    let telemetry_gen = Command::new(cargo_roto_bin)
        .env("CARGO_HOME", &cargo_home)
        .args(["roto", "--manifest-path"])
        .arg(&telemetry_manifest)
        .output()
        .expect("run cargo-roto on downstream telemetry-roto");
    assert!(
        telemetry_gen.status.success(),
        "cargo-roto failed on telemetry-roto: {}",
        String::from_utf8_lossy(&telemetry_gen.stderr)
    );
    assert!(telemetry_gen.stdout.is_empty());
    assert!(telemetry_gen.stderr.is_empty());

    // Verify `cargo metadata` fetched both git repositories into `CARGO_HOME/git/checkouts`
    // and resolved `-I <dep_root>/proto` and `extern_path` mappings for both crates
    let checkouts_dir = cargo_home.join("git/checkouts");
    assert!(
        checkouts_dir.is_dir(),
        "expected Cargo git checkouts cache at '{}'",
        checkouts_dir.display()
    );

    // Inspect resolved metadata using `--offline` now that git checkouts are cached in `CARGO_HOME`
    let metadata_out = Command::new("cargo")
        .env("CARGO_HOME", &cargo_home)
        .args(["metadata", "--format-version", "1", "--manifest-path"])
        .arg(&telemetry_manifest)
        .arg("--offline")
        .output()
        .expect("cargo metadata --offline on telemetry-roto");
    assert!(
        metadata_out.status.success(),
        "cargo metadata --offline failed: {}",
        String::from_utf8_lossy(&metadata_out.stderr)
    );
    let metadata_json = String::from_utf8(metadata_out.stdout).unwrap();
    let resolved =
        cargo_roto::metadata::resolve_from_metadata_json(&metadata_json, Some(&telemetry_manifest))
            .expect("resolve_from_metadata_json");

    assert_eq!(resolved.target_package_name, "telemetry-roto");
    assert_eq!(resolved.dependencies.len(), 2);
    for dep in &resolved.dependencies {
        assert!(
            dep.crate_root.starts_with(&checkouts_dir),
            "expected dependency '{}' crate_root '{}' to be inside CARGO_HOME git checkouts '{}'",
            dep.package_name,
            dep.crate_root.display(),
            checkouts_dir.display()
        );
        assert!(
            dep.proto_include_dir.is_dir(),
            "expected dependency proto include dir '{}' to exist",
            dep.proto_include_dir.display()
        );
        assert!(resolved.include_dirs.contains(&dep.proto_include_dir));
    }

    let extern_opts = resolved.extern_path_options();
    assert!(extern_opts.contains(&"extern_path=google/protobuf=::google_protobuf_roto".to_string()));
    assert!(extern_opts.contains(&"extern_path=google/api=::googleapis_roto".to_string()));
    assert!(extern_opts.contains(&"extern_path=google/rpc=::googleapis_roto".to_string()));
    assert!(extern_opts.contains(&"extern_path=google/type=::googleapis_roto".to_string()));

    // Also verify `resolve_workspace` API directly
    let ws_direct = resolve_workspace(&MetadataOptions {
        manifest_path: Some(telemetry_manifest.clone()),
        offline: false,
        locked: false,
        frozen: false,
    })
    .expect("resolve_workspace on telemetry-roto");
    assert_eq!(ws_direct.extern_path_options(), extern_opts);

    // Verify generated `telemetry-roto/src/lib.rs` contents
    let telemetry_lib = fs::read_to_string(telemetry_dir.join("src/lib.rs")).unwrap();
    assert!(telemetry_lib.starts_with("#![no_std]"));
    assert!(telemetry_lib.contains("pub mod telemetry"));
    assert!(telemetry_lib.contains("pub struct SensorReadingView<'a>"));
    assert!(telemetry_lib.contains("pub struct SensorReadingBuilder<'b>"));
    assert!(
        telemetry_lib.contains("::google_protobuf_roto::google::protobuf::TimestampView<'a>"),
        "expected external TimestampView path in:\n{telemetry_lib}"
    );
    assert!(
        telemetry_lib.contains("::google_protobuf_roto::google::protobuf::TimestampBuilder<'_>"),
        "expected external TimestampBuilder path in:\n{telemetry_lib}"
    );
    assert!(
        telemetry_lib.contains("::googleapis_roto::google::r#type::LatLngView<'a>"),
        "expected external LatLngView path with escaped r#type in:\n{telemetry_lib}"
    );
    assert!(
        telemetry_lib.contains("::googleapis_roto::google::r#type::LatLngBuilder<'_>"),
        "expected external LatLngBuilder path with escaped r#type in:\n{telemetry_lib}"
    );
    assert!(
        !telemetry_lib.contains("pub struct TimestampView"),
        "telemetry-roto must not duplicate TimestampView definition"
    );
    assert!(
        !telemetry_lib.contains("pub struct LatLngView"),
        "telemetry-roto must not duplicate LatLngView definition"
    );

    // =========================================================================
    // 4. Compilation, Round-Trip Execution, & Bare-Metal `thumbv7em-none-eabihf`
    // =========================================================================
    let tests_dir = telemetry_dir.join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    let roundtrip_test_rs = r#"use roto::runtime::WireType;
use telemetry_roto::telemetry::{SensorReadingBuilder, SensorReadingView};

#[test]
fn cross_crate_sensor_reading_roundtrip_and_unknown_fields() {
    let mut backing = [0u8; 256];
    let mut builder = SensorReadingBuilder::new(&mut backing);
    // Preserve an unrecognized future field (tag 99 = 777)
    builder.writer.write_varint(777).unwrap();
    builder.writer.write_tag(99, WireType::Varint).unwrap();

    builder
        .write_sample(104)
        .write_packed_samples(&[100, 102])
        .write_location(|loc| {
            loc.write_longitude_e7(-1224194155)
                .write_latitude_e7(377749295);
        })
        .write_device_id("imu-north-01")
        .write_recorded_at(|ts| {
            ts.write_nanos(500_000_000).write_seconds(1_700_000_000);
        });
    let bytes = builder.finish().unwrap();

    let view = SensorReadingView::from_bytes(bytes).expect("valid SensorReading wire bytes");
    assert_eq!(view.device_id(), Ok("imu-north-01"));

    let ts = view.recorded_at().expect("valid TimestampView");
    assert_eq!(ts.seconds(), Ok(1_700_000_000));
    assert!(ts.has_nanos());
    assert_eq!(ts.nanos(), Ok(500_000_000));

    let loc = view
        .location()
        .expect("decode location")
        .expect("location present");
    assert_eq!(loc.latitude_e7(), Ok(377749295));
    assert_eq!(loc.longitude_e7(), Ok(-1224194155));

    let samples: Vec<u32> = view.samples().map(|s| s.expect("valid sample")).collect();
    assert_eq!(samples, vec![100, 102, 104]);

    assert_eq!(view.unknown_fields().count(), 1);

    // Re-encode copying unknown fields and relying on default device_id
    let mut backing2 = [0u8; 256];
    let mut builder2 = SensorReadingBuilder::new(&mut backing2);
    builder2
        .copy_unknown_fields(&view)
        .write_recorded_at(|ts2| {
            ts2.write_seconds(1_700_000_001);
        });
    let view2 = SensorReadingView::from_bytes(builder2.finish().unwrap()).unwrap();
    assert!(!view2.has_device_id());
    assert_eq!(view2.device_id(), Ok("unknown"));
    assert_eq!(view2.recorded_at().unwrap().seconds(), Ok(1_700_000_001));
    assert_eq!(view2.recorded_at().unwrap().nanos(), Ok(0));
    assert_eq!(view2.unknown_fields().count(), 1);
}
"#;
    fs::write(tests_dir.join("roundtrip.rs"), roundtrip_test_rs).unwrap();

    let target_dir = tmp.path.join("target");
    let cargo_test = Command::new("cargo")
        .env("CARGO_HOME", &cargo_home)
        .args(["test", "--manifest-path"])
        .arg(&telemetry_manifest)
        .arg("--target-dir")
        .arg(&target_dir)
        .arg("--offline")
        .output()
        .expect("run cargo test on telemetry-roto");
    assert!(
        cargo_test.status.success(),
        "cargo test on telemetry-roto failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&cargo_test.stdout),
        String::from_utf8_lossy(&cargo_test.stderr)
    );

    // Verify bare-metal `#![no_std]` compilation for `thumbv7em-none-eabihf` across all 3 generated crates
    let cargo_check_no_std = Command::new("cargo")
        .env("CARGO_HOME", &cargo_home)
        .args(["check", "--manifest-path"])
        .arg(&telemetry_manifest)
        .args(["--target", "thumbv7em-none-eabihf", "--target-dir"])
        .arg(&target_dir)
        .arg("--offline")
        .output()
        .expect("run cargo check --target thumbv7em-none-eabihf on telemetry-roto");
    assert!(
        cargo_check_no_std.status.success(),
        "cargo check --target thumbv7em-none-eabihf failed:\n{}",
        String::from_utf8_lossy(&cargo_check_no_std.stderr)
    );
}
