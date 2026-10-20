use std::io::Write;
use std::process::{Command, Stdio};

use google_protobuf_roto::google::protobuf::{
    compiler::{
        code_generator_response::Feature, CodeGeneratorRequestBuilder, CodeGeneratorResponseView,
    },
    FileDescriptorSetView,
};
use roto::runtime::{BackwardWriter, WireType};

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

const COMPLEX_PROTO3_OPTIONAL_PROTO: &str = r#"syntax = "proto3";

package telemetry.v3;

enum DeviceStatus {
    DEVICE_STATUS_UNSPECIFIED = 0;
    DEVICE_STATUS_ACTIVE = 1;
    DEVICE_STATUS_SLEEP = 2 [deprecated = true];
}

message TelemetryEnvelope {
    optional string sensor_id = 1;
    optional int64 sequence = 2;
    DeviceStatus status = 3;
    repeated int32 readings = 4 [packed = true];

    message Metadata {
        string firmware_version = 1;
    }

    Metadata meta = 5;

    oneof payload {
        bytes raw_frame = 6;
        string text_note = 7;
    }
}

service TelemetryCollector {
    rpc Record (TelemetryEnvelope) returns (TelemetryEnvelope);
}
"#;

#[test]
fn protoc_invokes_plugin_on_sensor_proto_and_proto3_optional_schema() {
    let plugin_bin = env!("CARGO_BIN_EXE_protoc-gen-roto");
    let tmp_dir = std::env::temp_dir().join(format!(
        "roto_plugin_integration_sensor_{}",
        std::process::id()
    ));
    let out_dir = tmp_dir.join("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let sensor_path = tmp_dir.join("sensor.proto");
    let complex_path = tmp_dir.join("complex.proto");
    std::fs::write(&sensor_path, SENSOR_PROTO).unwrap();
    std::fs::write(&complex_path, COMPLEX_PROTO3_OPTIONAL_PROTO).unwrap();

    let output = Command::new("protoc")
        .arg(format!("--plugin=protoc-gen-roto={plugin_bin}"))
        .arg(format!("--proto_path={}", tmp_dir.display()))
        .arg(format!("--roto_out={}", out_dir.display()))
        .arg("sensor.proto")
        .arg("complex.proto")
        .output()
        .expect("failed to execute protoc with protoc-gen-roto plugin");

    let _ = std::fs::remove_dir_all(&tmp_dir);

    assert!(
        output.status.success(),
        "protoc failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn protoc_invokes_plugin_on_descriptor_and_plugin_protos() {
    let plugin_bin = env!("CARGO_BIN_EXE_protoc-gen-roto");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let proto_root = format!("{manifest_dir}/../google-protobuf-roto/proto");

    let tmp_dir = std::env::temp_dir().join(format!(
        "roto_plugin_integration_google_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&tmp_dir).unwrap();

    let output = Command::new("protoc")
        .arg(format!("--plugin=protoc-gen-roto={plugin_bin}"))
        .arg(format!("--proto_path={proto_root}"))
        .arg(format!("--roto_out={}", tmp_dir.display()))
        .arg("google/protobuf/descriptor.proto")
        .arg("google/protobuf/compiler/plugin.proto")
        .output()
        .expect("failed to execute protoc on descriptor.proto and plugin.proto");

    let _ = std::fs::remove_dir_all(&tmp_dir);

    assert!(
        output.status.success(),
        "protoc failed on descriptor.proto / plugin.proto: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn plugin_stdio_roundtrip_verifies_response_wire_format() {
    let plugin_bin = env!("CARGO_BIN_EXE_protoc-gen-roto");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let proto_root = format!("{manifest_dir}/../google-protobuf-roto/proto");

    // Obtain real FileDescriptorSet from protoc for plugin.proto and descriptor.proto
    let fds_out = Command::new("protoc")
        .arg(format!("--proto_path={proto_root}"))
        .arg("--include_imports")
        .arg("--include_source_info")
        .arg("--descriptor_set_out=/dev/stdout")
        .arg("google/protobuf/compiler/plugin.proto")
        .output()
        .expect("failed to run protoc");
    assert!(fds_out.status.success());

    let fds = FileDescriptorSetView::from_bytes(&fds_out.stdout).unwrap();
    let files: Vec<_> = fds.file().map(|f| f.unwrap()).collect();
    assert_eq!(files.len(), 2);

    let mut req_backing = vec![0u8; fds_out.stdout.len() * 2 + 1024];
    let mut req = CodeGeneratorRequestBuilder::new(&mut req_backing);
    req.write_compiler_version(|ver| {
        ver.write_suffix("")
            .write_patch(1)
            .write_minor(28)
            .write_major(4);
    })
    .write_source_file_descriptors(|fdp| {
        let _ = fdp.writer.write_bytes(files[1].as_bytes());
    });
    for fdp_view in files.iter().rev() {
        req.write_proto_file(|fdp| {
            let _ = fdp.writer.write_bytes(fdp_view.as_bytes());
        })
        .check()
        .unwrap();
    }
    req.write_parameter("mode=lib")
        .write_file_to_generate("google/protobuf/compiler/plugin.proto");
    let req_bytes = req.finish().unwrap();

    let mut child = Command::new(plugin_bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn protoc-gen-roto");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(req_bytes)
        .expect("failed to write request to stdin");

    let output = child.wait_with_output().expect("failed to wait on plugin");
    assert!(
        output.status.success(),
        "plugin exited non-zero: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let resp_view = CodeGeneratorResponseView::from_bytes(&output.stdout)
        .expect("stdout must be a valid CodeGeneratorResponse");
    assert!(!resp_view.has_error());
    assert_eq!(
        resp_view.supported_features(),
        Ok(Feature::FeatureProto3Optional as u64)
    );
}

#[test]
fn malformed_wire_bytes_on_stdin_exits_nonzero_with_stderr() {
    let plugin_bin = env!("CARGO_BIN_EXE_protoc-gen-roto");

    // Truncated length-delimited field (tag 1, wire type 2, length 50, only 2 bytes of payload)
    let malformed: &[u8] = &[0x0A, 50, 0x01, 0x02];

    let mut child = Command::new(plugin_bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn protoc-gen-roto");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(malformed)
        .expect("failed to write malformed bytes to stdin");

    let output = child.wait_with_output().expect("failed to wait on plugin");
    assert!(
        !output.status.success(),
        "expected non-zero exit code for malformed wire input"
    );
    assert!(output.stdout.is_empty());
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr_str.contains("failed to decode CodeGeneratorRequest"),
        "unexpected stderr: {stderr_str}"
    );
}

#[test]
fn schema_and_generator_errors_reported_via_write_error_with_exit_zero() {
    let plugin_bin = env!("CARGO_BIN_EXE_protoc-gen-roto");

    // Case 1: Request references a `file_to_generate` not present in `proto_file`
    let mut backing = [0u8; 256];
    let mut req = CodeGeneratorRequestBuilder::new(&mut backing);
    req.write_file_to_generate("missing_file.proto");
    let req_bytes = req.finish().unwrap();

    let mut child = Command::new(plugin_bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn protoc-gen-roto");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(req_bytes)
        .expect("failed to write request");

    let output = child.wait_with_output().expect("failed to wait on plugin");
    assert!(
        output.status.success(),
        "generator errors must exit 0 per plugin.proto spec, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let resp = CodeGeneratorResponseView::from_bytes(&output.stdout).unwrap();
    assert!(resp.has_error());
    assert!(
        resp.error()
            .unwrap()
            .contains("file_to_generate 'missing_file.proto' not found in proto_file descriptors"),
        "unexpected response error: {:?}",
        resp.error()
    );

    // Case 2: Corrupt nested FileDescriptorProto inside `proto_file`
    let mut corrupt_backing = [0u8; 256];
    let mut corrupt_writer = BackwardWriter::new(&mut corrupt_backing);
    {
        // Write tag 15 (proto_file, LengthDelimited) with invalid inner wire payload
        corrupt_writer
            .write_length_delimited(&[0x0A, 99, 0x01])
            .unwrap();
        corrupt_writer
            .write_tag(15, WireType::LengthDelimited)
            .unwrap();
    }
    let corrupt_req_bytes = corrupt_writer.finish().unwrap();

    let mut child2 = Command::new(plugin_bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn protoc-gen-roto");

    child2
        .stdin
        .take()
        .unwrap()
        .write_all(corrupt_req_bytes)
        .expect("failed to write corrupt proto_file request");

    let output2 = child2.wait_with_output().expect("failed to wait on plugin");
    assert!(output2.status.success());
    let resp2 = CodeGeneratorResponseView::from_bytes(&output2.stdout).unwrap();
    assert!(resp2.has_error());
    assert!(resp2
        .error()
        .unwrap()
        .contains("invalid FileDescriptorProto in proto_file"));

    // Case 3: End-to-end via `protoc --roto_opt=error=...` verifying protoc surfaces `write_error`
    let tmp_dir = std::env::temp_dir().join(format!(
        "roto_plugin_integration_error_{}",
        std::process::id()
    ));
    let out_dir = tmp_dir.join("out");
    std::fs::create_dir_all(&out_dir).unwrap();
    let sensor_path = tmp_dir.join("sensor.proto");
    std::fs::write(&sensor_path, SENSOR_PROTO).unwrap();

    let protoc_out = Command::new("protoc")
        .arg(format!("--plugin=protoc-gen-roto={plugin_bin}"))
        .arg(format!("--proto_path={}", tmp_dir.display()))
        .arg("--roto_opt=error=unsupported_schema_construct")
        .arg(format!("--roto_out={}", out_dir.display()))
        .arg("sensor.proto")
        .output()
        .expect("failed to execute protoc");

    let _ = std::fs::remove_dir_all(&tmp_dir);

    assert!(
        !protoc_out.status.success(),
        "protoc should fail when plugin sets CodeGeneratorResponse.error"
    );
    let protoc_stderr = String::from_utf8_lossy(&protoc_out.stderr);
    assert!(
        protoc_stderr.contains("unsupported_schema_construct"),
        "expected protoc stderr to contain plugin error message, got: {protoc_stderr}"
    );
}
