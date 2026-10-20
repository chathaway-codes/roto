use std::path::{Path, PathBuf};
use std::process::Command;

fn create_temp_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "roto_e2e_codegen_{}_{}",
        test_name,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

/// Compiles `roto_runtime` and `roto` into `out_dir` as `.rlib`s so generated crates
/// and test binaries can be compiled directly with `rustc` in milliseconds without
/// Cargo lock contention.
fn build_roto_rlibs(out_dir: &Path) -> PathBuf {
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

    roto_rlib
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
fn e2e_sensor_proto_from_readme() {
    let plugin_bin = env!("CARGO_BIN_EXE_protoc-gen-roto");
    let tmp_dir = create_temp_dir("sensor_readme");
    let proto_dir = tmp_dir.join("proto");
    let gen_dir = tmp_dir.join("gen");
    let build_dir = tmp_dir.join("build");
    std::fs::create_dir_all(&proto_dir).unwrap();
    std::fs::create_dir_all(&gen_dir).unwrap();
    std::fs::create_dir_all(&build_dir).unwrap();

    std::fs::write(proto_dir.join("sensor.proto"), SENSOR_PROTO).unwrap();

    let protoc_out = Command::new("protoc")
        .arg(format!("--plugin=protoc-gen-roto={plugin_bin}"))
        .arg(format!("--proto_path={}", proto_dir.display()))
        .arg(format!("--roto_out={}", gen_dir.display()))
        .arg("sensor.proto")
        .output()
        .expect("failed to run protoc on sensor.proto");
    assert!(
        protoc_out.status.success(),
        "protoc failed: {}",
        String::from_utf8_lossy(&protoc_out.stderr)
    );

    let generated_lib = gen_dir.join("lib.rs");
    let lib_content =
        std::fs::read_to_string(&generated_lib).expect("expected generated gen/lib.rs");
    assert!(lib_content.starts_with("#![no_std]\n#![forbid(unsafe_code)]"));
    assert!(lib_content.contains("pub mod telemetry {"));
    assert!(lib_content.contains("pub struct LocationView<'a>"));
    assert!(lib_content.contains("pub struct LocationBuilder<'a, 'b>"));
    assert!(lib_content.contains("pub struct SensorReadingView<'a>"));
    assert!(lib_content.contains("pub struct SensorReadingBuilder<'a, 'b>"));

    let roto_rlib = build_roto_rlibs(&build_dir);
    let sensor_rlib = build_dir.join("libsensor_roto.rlib");

    let compile_lib = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=sensor_roto")
        .arg("-D")
        .arg("warnings")
        .arg(&generated_lib)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&sensor_rlib)
        .output()
        .expect("failed to compile sensor_roto");
    assert!(
        compile_lib.status.success(),
        "compiling sensor_roto failed:\n{}",
        String::from_utf8_lossy(&compile_lib.stderr)
    );

    let runner_src = tmp_dir.join("runner.rs");
    std::fs::write(
        &runner_src,
        r#"
use roto::runtime::{BackwardWriter, DecodeError, WireType};
use sensor_roto::telemetry::{
    LocationBuilder, LocationView, SensorReadingBuilder, SensorReadingView,
};

fn main() -> Result<(), Box<dyn core::error::Error>> {
    // 1. Exact README.md usage example: Single-Pass Write & Shallow Read
    let mut backing = [0u8; 128];
    let mut writer = BackwardWriter::new(&mut backing);

    {
        let mut builder = SensorReadingBuilder::new(&mut writer);

        builder.write_sample(42)?;
        builder.write_sample(108)?;

        builder.write_location(|loc| {
            loc.write_lon_e7(-1224194000)?;
            loc.write_lat_e7(377749000)?;
            Ok(())
        })?;

        builder.write_device_id("sensor-node-01")?;
        builder.write_timestamp_ms(1_700_000_000_000)?;
    }

    let wire_bytes: &[u8] = writer.finish();

    let reading = SensorReadingView::from_bytes(wire_bytes)?;
    assert!(reading.has_timestamp_ms());
    assert_eq!(reading.timestamp_ms()?, 1_700_000_000_000);
    assert!(reading.has_device_id());
    assert_eq!(reading.device_id()?, "sensor-node-01");
    assert!(reading.has_location());

    let loc = reading.location()?.expect("expected location");
    assert!(loc.has_lat_e7());
    assert!(loc.has_lon_e7());
    assert_eq!(loc.lat_e7()?, 377749000);
    assert_eq!(loc.lon_e7()?, -1224194000);

    let samples: Vec<u32> = reading.samples().map(|s| s.unwrap()).collect();
    assert_eq!(samples, vec![108, 42]);

    // 2. Default value when `device_id` is omitted (`"unknown"`) and packed samples
    let mut min_backing = [0u8; 128];
    let mut min_writer = BackwardWriter::new(&mut min_backing);
    {
        let mut b = SensorReadingBuilder::new(&mut min_writer);
        b.write_packed_samples(&[10, 20, 30])?;
        b.write_timestamp_ms(99)?;
    }
    let min_reading = SensorReadingView::from_bytes(min_writer.finish())?;
    assert!(!min_reading.has_device_id());
    assert_eq!(min_reading.device_id()?, "unknown");
    assert!(!min_reading.has_location());
    assert!(min_reading.location()?.is_none());
    let packed_samples: Vec<u32> = min_reading.samples().map(|s| s.unwrap()).collect();
    assert_eq!(packed_samples, vec![10, 20, 30]);

    // 3. Missing required field checks on SensorReading and Location
    assert_eq!(
        SensorReadingView::from_bytes(&[]).unwrap_err(),
        DecodeError::MissingRequiredField("SensorReading.timestamp_ms")
    );
    assert_eq!(
        LocationView::from_bytes(&[]).unwrap_err(),
        DecodeError::MissingRequiredField("Location.lat_e7")
    );
    let mut partial_loc_buf = [0u8; 32];
    let mut partial_loc_writer = BackwardWriter::new(&mut partial_loc_buf);
    {
        let mut lb = LocationBuilder::new(&mut partial_loc_writer);
        lb.write_lat_e7(100)?;
    }
    assert_eq!(
        LocationView::from_bytes(partial_loc_writer.finish()).unwrap_err(),
        DecodeError::MissingRequiredField("Location.lon_e7")
    );

    // 4. Unknown field preservation and lossless copy_unknown_fields forwarding
    let mut unk_backing = [0u8; 128];
    let mut unk_writer = BackwardWriter::new(&mut unk_backing);
    unk_writer.write_varint(777)?;
    unk_writer.write_tag(99, WireType::Varint)?;
    {
        let mut b = SensorReadingBuilder::new(&mut unk_writer);
        b.write_timestamp_ms(1234)?;
    }
    let unk_view = SensorReadingView::from_bytes(unk_writer.finish())?;
    assert_eq!(unk_view.unknown_fields().count(), 1);

    let mut fwd_backing = [0u8; 128];
    let mut fwd_writer = BackwardWriter::new(&mut fwd_backing);
    {
        let mut b = SensorReadingBuilder::new(&mut fwd_writer);
        b.copy_unknown_fields(&unk_view)?;
        b.write_timestamp_ms(unk_view.timestamp_ms()?)?;
    }
    let fwd_view = SensorReadingView::from_bytes(fwd_writer.finish())?;
    assert_eq!(fwd_view.timestamp_ms()?, 1234);
    let unk_spans: Vec<&[u8]> = fwd_view.unknown_fields().collect();
    assert_eq!(unk_spans.len(), 1);

    Ok(())
}
"#,
    )
    .unwrap();

    let runner_bin = build_dir.join("runner");
    let compile_runner = Command::new("rustc")
        .arg("--edition=2021")
        .arg(&runner_src)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("--extern")
        .arg(format!("sensor_roto={}", sensor_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&runner_bin)
        .output()
        .expect("failed to compile runner");
    assert!(
        compile_runner.status.success(),
        "compiling runner failed:\n{}",
        String::from_utf8_lossy(&compile_runner.stderr)
    );

    let run_out = Command::new(&runner_bin)
        .output()
        .expect("failed to execute runner");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    assert!(
        run_out.status.success(),
        "runner failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_out.stdout),
        String::from_utf8_lossy(&run_out.stderr)
    );
}

const COMPREHENSIVE_PROTO2: &str = r#"syntax = "proto2";

package pkg.type.mod.match;

enum TopMode {
    option allow_alias = true;
    TOP_IDLE = 0;
    TOP_ACTIVE = 1;
    TOP_RUNNING = 1;
}

message AllTypes {
    message Outer {
        message Middle {
            enum DeepEnum {
                DEEP_ZERO = 0;
                DEEP_ONE = 1;
            }
            message Inner {
                optional int32 val = 1 [default = 77];
                optional DeepEnum deep_enum = 2 [default = DEEP_ONE];
            }
            optional Inner inner = 1;
        }
        optional Middle middle = 1;
    }

    required uint32 req_id = 1;

    optional double f_double = 2 [default = 1.5];
    optional float f_float = 3 [default = -2.25];
    optional int64 f_int64 = 4 [default = -6400];
    optional uint64 f_uint64 = 5 [default = 6400];
    optional int32 f_int32 = 6 [default = -3200];
    optional fixed64 f_fixed64 = 7 [default = 777];
    optional fixed32 f_fixed32 = 8 [default = 888];
    optional bool f_bool = 9 [default = true];
    optional string f_string = 10 [default = "hello\nworld"];
    optional group SubGroup = 11 {
        optional int32 group_val = 1 [default = 11];
    }
    optional Outer.Middle.Inner f_message = 12;
    optional bytes f_bytes = 13 [default = "\x00\x01\xffabc"];
    optional uint32 f_uint32 = 14 [default = 1400];
    optional TopMode f_enum = 15 [default = TOP_ACTIVE];
    optional sfixed32 f_sfixed32 = 16 [default = -160];
    optional sfixed64 f_sfixed64 = 17 [default = -170];
    optional sint32 f_sint32 = 18 [default = -180];
    optional sint64 f_sint64 = 19 [default = -190];

    repeated double r_doubles = 20;
    repeated float r_floats = 21;
    repeated int64 r_int64s = 22;
    repeated uint64 r_uint64s = 23;
    repeated int32 r_int32s = 24;
    repeated fixed64 r_fixed64s = 25;
    repeated fixed32 r_fixed32s = 26;
    repeated bool r_bools = 27;
    repeated string r_strings = 28;
    repeated group RepGroups = 29 {
        optional int32 rg_val = 1;
    }
    repeated Outer.Middle.Inner r_messages = 30;
    repeated bytes r_bytes = 31;
    repeated uint32 r_uint32s = 32;
    repeated TopMode r_enums = 33;
    repeated sfixed32 r_sfixed32s = 34;
    repeated sfixed64 r_sfixed64s = 35;
    repeated sint32 r_sint32s = 36;
    repeated sint64 r_sint64s = 37;

    // Keyword field names
    optional string type = 38 [default = "kw_type"];
    optional int32 mod = 39 [default = 39];
    optional bool match = 40 [default = true];

    oneof choice {
        string choice_text = 41;
        Outer.Middle.Inner choice_inner = 42;
    }

    oneof scalar_choice {
        sint32 choice_code = 43;
        TopMode choice_mode = 44;
    }
}
"#;

const COMPREHENSIVE_PROTO3: &str = r#"syntax = "proto3";

package pkg.v3;

enum StatusV3 {
    STATUS_V3_UNSPECIFIED = 0;
    STATUS_V3_OK = 1;
}

message EnvelopeV3 {
    string id = 1;
    optional int64 opt_seq = 2;
    StatusV3 status = 3;
    repeated sint32 deltas = 4;

    oneof body {
        bytes raw = 5;
        int32 code = 6;
    }
}
"#;

#[test]
fn e2e_comprehensive_proto2_and_proto3_schema() {
    let plugin_bin = env!("CARGO_BIN_EXE_protoc-gen-roto");
    let tmp_dir = create_temp_dir("comprehensive");
    let proto_dir = tmp_dir.join("proto");
    let gen_lib_dir = tmp_dir.join("gen_lib");
    let gen_mod_dir = tmp_dir.join("gen_mod");
    let build_dir = tmp_dir.join("build");
    std::fs::create_dir_all(&proto_dir).unwrap();
    std::fs::create_dir_all(&gen_lib_dir).unwrap();
    std::fs::create_dir_all(&gen_mod_dir).unwrap();
    std::fs::create_dir_all(&build_dir).unwrap();

    std::fs::write(proto_dir.join("comp_proto2.proto"), COMPREHENSIVE_PROTO2).unwrap();
    std::fs::write(proto_dir.join("comp_proto3.proto"), COMPREHENSIVE_PROTO3).unwrap();

    // 1. Generate in default `mode=lib`
    let protoc_lib = Command::new("protoc")
        .arg(format!("--plugin=protoc-gen-roto={plugin_bin}"))
        .arg(format!("--proto_path={}", proto_dir.display()))
        .arg(format!("--roto_out={}", gen_lib_dir.display()))
        .arg("comp_proto2.proto")
        .arg("comp_proto3.proto")
        .output()
        .expect("failed to run protoc (mode=lib)");
    assert!(
        protoc_lib.status.success(),
        "protoc mode=lib failed: {}",
        String::from_utf8_lossy(&protoc_lib.stderr)
    );

    // 2. Also generate in `mode=mod` and verify it compiles cleanly as a library crate
    let protoc_mod = Command::new("protoc")
        .arg(format!("--plugin=protoc-gen-roto={plugin_bin}"))
        .arg(format!("--proto_path={}", proto_dir.display()))
        .arg("--roto_opt=mode=mod")
        .arg(format!("--roto_out={}", gen_mod_dir.display()))
        .arg("comp_proto2.proto")
        .arg("comp_proto3.proto")
        .output()
        .expect("failed to run protoc (mode=mod)");
    assert!(
        protoc_mod.status.success(),
        "protoc mode=mod failed: {}",
        String::from_utf8_lossy(&protoc_mod.stderr)
    );
    assert!(gen_mod_dir.join("mod.rs").exists());
    assert!(gen_mod_dir.join("pkg/type/mod/match.rs").exists());
    assert!(gen_mod_dir.join("pkg/v3.rs").exists());

    let roto_rlib = build_roto_rlibs(&build_dir);

    // Compile mode=mod output via a wrapper root file that includes mod.rs
    let mod_root = gen_mod_dir.join("lib_mod_root.rs");
    let mod_rs_content = std::fs::read_to_string(gen_mod_dir.join("mod.rs")).unwrap();
    std::fs::write(
        &mod_root,
        format!(
            "#![no_std]\n#![forbid(unsafe_code)]\n#![allow(clippy::all, dead_code, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_imports)]\n\n{mod_rs_content}"
        ),
    )
    .unwrap();
    let comp_mod_rlib = build_dir.join("libcomp_mod.rlib");
    let compile_mod = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=comp_mod")
        .arg("-D")
        .arg("warnings")
        .arg(&mod_root)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&comp_mod_rlib)
        .output()
        .expect("failed to compile mode=mod crate");
    assert!(
        compile_mod.status.success(),
        "compiling mode=mod crate failed:\n{}",
        String::from_utf8_lossy(&compile_mod.stderr)
    );

    // Compile mode=lib output
    let comp_lib_rlib = build_dir.join("libcomp_roto.rlib");
    let compile_lib = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=comp_roto")
        .arg("-D")
        .arg("warnings")
        .arg(gen_lib_dir.join("lib.rs"))
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&comp_lib_rlib)
        .output()
        .expect("failed to compile comp_roto");
    assert!(
        compile_lib.status.success(),
        "compiling comp_roto failed:\n{}",
        String::from_utf8_lossy(&compile_lib.stderr)
    );

    // Compile and run comprehensive round-trip test binary
    let runner_src = tmp_dir.join("comp_runner.rs");
    std::fs::write(
        &runner_src,
        r#"
use comp_roto::pkg::r#type::r#mod::r#match::{
    all_types::{
        outer::{
            middle::{DeepEnum, InnerBuilder, InnerView},
            MiddleBuilder, MiddleView,
        },
        Choice, OuterBuilder, OuterView, RepGroupsBuilder, RepGroupsView, ScalarChoice,
        SubGroupBuilder, SubGroupView,
    },
    AllTypesBuilder, AllTypesView, TopMode,
};
use comp_roto::pkg::v3::{envelope_v3::Body, EnvelopeV3Builder, EnvelopeV3View, StatusV3};
use roto::runtime::BackwardWriter;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    // Verify allow_alias constant on TopMode
    assert_eq!(TopMode::TopRunning, TopMode::TopActive);

    // Verify nested message Builders & Views work directly at arbitrary depth
    let mut nested_buf = [0u8; 128];
    let mut nested_writer = BackwardWriter::new(&mut nested_buf);
    {
        let mut ob = OuterBuilder::new(&mut nested_writer);
        ob.write_middle(|mb: &mut MiddleBuilder<'_, '_>| {
            mb.write_inner(|ib: &mut InnerBuilder<'_, '_>| {
                ib.write_deep_enum(DeepEnum::DeepZero)?;
                ib.write_val(999)
            })
        })?;
    }
    let outer_view = OuterView::from_bytes(nested_writer.finish())?;
    let middle_view: MiddleView<'_> = outer_view.middle()?.unwrap();
    let inner_view: InnerView<'_> = middle_view.inner()?.unwrap();
    assert_eq!(inner_view.val()?, 999);
    assert_eq!(inner_view.deep_enum()?, DeepEnum::DeepZero);

    // 1. Minimal AllTypes (only req_id set) -> verify all custom [default = ...] values
    let mut min_buf = [0u8; 64];
    let mut min_writer = BackwardWriter::new(&mut min_buf);
    {
        let mut b = AllTypesBuilder::new(&mut min_writer);
        b.write_req_id(7)?;
    }
    let min_view = AllTypesView::from_bytes(min_writer.finish())?;
    assert_eq!(min_view.req_id()?, 7);
    assert_eq!(min_view.f_double()?, 1.5);
    assert_eq!(min_view.f_float()?, -2.25);
    assert_eq!(min_view.f_int64()?, -6400);
    assert_eq!(min_view.f_uint64()?, 6400);
    assert_eq!(min_view.f_int32()?, -3200);
    assert_eq!(min_view.f_fixed64()?, 777);
    assert_eq!(min_view.f_fixed32()?, 888);
    assert!(min_view.f_bool()?);
    assert_eq!(min_view.f_string()?, "hello\nworld");
    assert!(min_view.subgroup()?.is_none());
    assert!(min_view.f_message()?.is_none());
    assert_eq!(min_view.f_bytes()?, &b"\x00\x01\xffabc"[..]);
    assert_eq!(min_view.f_uint32()?, 1400);
    assert_eq!(min_view.f_enum()?, TopMode::TopActive);
    assert_eq!(min_view.f_sfixed32()?, -160);
    assert_eq!(min_view.f_sfixed64()?, -170);
    assert_eq!(min_view.f_sint32()?, -180);
    assert_eq!(min_view.f_sint64()?, -190);
    assert_eq!(min_view.r#type()?, "kw_type");
    assert_eq!(min_view.r#mod()?, 39);
    assert!(min_view.r#match()?);
    assert!(min_view.choice()?.is_none());
    assert!(min_view.scalar_choice()?.is_none());

    // 2. Full AllTypes round-trip with all 18 singular types, packed & unpacked repeated fields,
    // groups, nested messages, oneofs, and keyword fields
    let mut full_buf = [0u8; 2048];
    let mut full_writer = BackwardWriter::new(&mut full_buf);
    {
        let mut b = AllTypesBuilder::new(&mut full_writer);
        b.write_choice_mode(TopMode::TopIdle)?;
        b.write_choice_inner(|ib| {
            ib.write_deep_enum(DeepEnum::DeepOne)?;
            ib.write_val(555)
        })?;
        b.write_match(false)?;
        b.write_mod(-99)?;
        b.write_type("custom_type_val")?;

        b.write_packed_r_sint64s(&[-1000, 2000])?;
        b.write_packed_r_sint32s(&[-100, 200])?;
        b.write_packed_r_sfixed64s(&[-50, 60])?;
        b.write_packed_r_sfixed32s(&[-5, 6])?;
        b.write_packed_r_enums(&[TopMode::TopIdle, TopMode::TopActive])?;
        b.write_packed_r_uint32s(&[1, 2, 3])?;
        b.write_r_bytes(b"b2")?;
        b.write_r_bytes(b"b1")?;
        b.write_r_message(|ib| ib.write_val(42))?;
        b.write_repgroup(|gb: &mut RepGroupsBuilder<'_, '_>| gb.write_rg_val(77))?;
        b.write_r_string("s2")?;
        b.write_r_string("s1")?;
        b.write_packed_r_bools(&[true, false])?;
        b.write_packed_r_fixed32s(&[11, 22])?;
        b.write_packed_r_fixed64s(&[111, 222])?;
        b.write_packed_r_int32s(&[-1, -2])?;
        b.write_packed_r_uint64s(&[10, 20])?;
        b.write_packed_r_int64s(&[-10, -20])?;
        b.write_packed_r_floats(&[1.25, 2.5])?;
        b.write_packed_r_doubles(&[3.5, 4.5])?;

        b.write_f_sint64(-9999)?;
        b.write_f_sint32(-999)?;
        b.write_f_sfixed64(-8888)?;
        b.write_f_sfixed32(-888)?;
        b.write_f_enum(TopMode::TopIdle)?;
        b.write_f_uint32(12345)?;
        b.write_f_bytes(b"raw_payload")?;
        b.write_f_message(|ib| ib.write_val(321))?;
        b.write_subgroup(|gb: &mut SubGroupBuilder<'_, '_>| gb.write_group_val(654))?;
        b.write_f_string("override_str")?;
        b.write_f_bool(false)?;
        b.write_f_fixed32(32)?;
        b.write_f_fixed64(64)?;
        b.write_f_int32(-32)?;
        b.write_f_uint64(640)?;
        b.write_f_int64(-640)?;
        b.write_f_float(9.5)?;
        b.write_f_double(19.5)?;
        b.write_req_id(100)?;
    }

    let full = AllTypesView::from_bytes(full_writer.finish())?;
    assert_eq!(full.req_id()?, 100);
    assert_eq!(full.f_double()?, 19.5);
    assert_eq!(full.f_float()?, 9.5);
    assert_eq!(full.f_int64()?, -640);
    assert_eq!(full.f_uint64()?, 640);
    assert_eq!(full.f_int32()?, -32);
    assert_eq!(full.f_fixed64()?, 64);
    assert_eq!(full.f_fixed32()?, 32);
    assert!(!full.f_bool()?);
    assert_eq!(full.f_string()?, "override_str");
    let sg: SubGroupView<'_> = full.subgroup()?.unwrap();
    assert_eq!(sg.group_val()?, 654);
    assert_eq!(full.f_message()?.unwrap().val()?, 321);
    assert_eq!(full.f_bytes()?, b"raw_payload");
    assert_eq!(full.f_uint32()?, 12345);
    assert_eq!(full.f_enum()?, TopMode::TopIdle);
    assert_eq!(full.f_sfixed32()?, -888);
    assert_eq!(full.f_sfixed64()?, -8888);
    assert_eq!(full.f_sint32()?, -999);
    assert_eq!(full.f_sint64()?, -9999);

    assert_eq!(full.r_doubles().map(|x| x.unwrap()).collect::<Vec<_>>(), vec![3.5, 4.5]);
    assert_eq!(full.r_floats().map(|x| x.unwrap()).collect::<Vec<_>>(), vec![1.25, 2.5]);
    assert_eq!(full.r_int64s().map(|x| x.unwrap()).collect::<Vec<_>>(), vec![-10, -20]);
    assert_eq!(full.r_uint64s().map(|x| x.unwrap()).collect::<Vec<_>>(), vec![10, 20]);
    assert_eq!(full.r_int32s().map(|x| x.unwrap()).collect::<Vec<_>>(), vec![-1, -2]);
    assert_eq!(full.r_fixed64s().map(|x| x.unwrap()).collect::<Vec<_>>(), vec![111, 222]);
    assert_eq!(full.r_fixed32s().map(|x| x.unwrap()).collect::<Vec<_>>(), vec![11, 22]);
    assert_eq!(full.r_bools().map(|x| x.unwrap()).collect::<Vec<_>>(), vec![true, false]);
    assert_eq!(full.r_strings().map(|x| x.unwrap()).collect::<Vec<_>>(), vec!["s1", "s2"]);
    let rg_vals: Vec<i32> = full
        .repgroups()
        .map(|g: Result<RepGroupsView<'_>, _>| g.unwrap().rg_val().unwrap())
        .collect();
    assert_eq!(rg_vals, vec![77]);
    assert_eq!(
        full.r_messages().map(|m| m.unwrap().val().unwrap()).collect::<Vec<_>>(),
        vec![42]
    );
    assert_eq!(
        full.r_bytes().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec![&b"b1"[..], &b"b2"[..]]
    );
    assert_eq!(full.r_uint32s().map(|x| x.unwrap()).collect::<Vec<_>>(), vec![1, 2, 3]);
    assert_eq!(
        full.r_enums().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec![TopMode::TopIdle, TopMode::TopActive]
    );
    assert_eq!(full.r_sfixed32s().map(|x| x.unwrap()).collect::<Vec<_>>(), vec![-5, 6]);
    assert_eq!(full.r_sfixed64s().map(|x| x.unwrap()).collect::<Vec<_>>(), vec![-50, 60]);
    assert_eq!(full.r_sint32s().map(|x| x.unwrap()).collect::<Vec<_>>(), vec![-100, 200]);
    assert_eq!(full.r_sint64s().map(|x| x.unwrap()).collect::<Vec<_>>(), vec![-1000, 2000]);

    assert_eq!(full.r#type()?, "custom_type_val");
    assert_eq!(full.r#mod()?, -99);
    assert!(!full.r#match()?);

    match full.choice()?.expect("expected choice") {
        Choice::ChoiceInner(iv) => {
            assert_eq!(iv.val()?, 555);
            assert_eq!(iv.deep_enum()?, DeepEnum::DeepOne);
        }
        other => panic!("unexpected choice: {other:?}"),
    }
    assert_eq!(
        full.scalar_choice()?,
        Some(ScalarChoice::ChoiceMode(TopMode::TopIdle))
    );

    // 3. Proto3 EnvelopeV3 round-trip
    let mut v3_buf = [0u8; 256];
    let mut v3_writer = BackwardWriter::new(&mut v3_buf);
    {
        let mut b = EnvelopeV3Builder::new(&mut v3_writer);
        b.write_code(404)?;
        b.write_packed_deltas(&[-5, 10, -15])?;
        b.write_status(StatusV3::StatusV3Ok)?;
        b.write_opt_seq(99)?;
        b.write_id("env-01")?;
    }
    let v3_view = EnvelopeV3View::from_bytes(v3_writer.finish())?;
    assert_eq!(v3_view.id()?, "env-01");
    assert!(v3_view.has_opt_seq());
    assert_eq!(v3_view.opt_seq()?, 99);
    assert_eq!(v3_view.status()?, StatusV3::StatusV3Ok);
    assert_eq!(
        v3_view.deltas().map(|d| d.unwrap()).collect::<Vec<_>>(),
        vec![-5, 10, -15]
    );
    assert_eq!(v3_view.body()?, Some(Body::Code(404)));

    Ok(())
}
"#,
    )
    .unwrap();

    let runner_bin = build_dir.join("comp_runner");
    let compile_runner = Command::new("rustc")
        .arg("--edition=2021")
        .arg(&runner_src)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("--extern")
        .arg(format!("comp_roto={}", comp_lib_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&runner_bin)
        .output()
        .expect("failed to compile comp_runner");
    assert!(
        compile_runner.status.success(),
        "compiling comp_runner failed:\n{}",
        String::from_utf8_lossy(&compile_runner.stderr)
    );

    let run_out = Command::new(&runner_bin)
        .output()
        .expect("failed to execute comp_runner");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    assert!(
        run_out.status.success(),
        "comp_runner failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_out.stdout),
        String::from_utf8_lossy(&run_out.stderr)
    );
}

const TIMESTAMP_PROTO: &str = r#"syntax = "proto3";

package google.protobuf;

message Timestamp {
    int64 seconds = 1;
    int32 nanos = 2;
}
"#;

const TELEMETRY_EVENT_PROTO: &str = r#"syntax = "proto2";

package telemetry;

import "google/protobuf/timestamp.proto";

message TelemetryEvent {
    required string event_name = 1;
    optional google.protobuf.Timestamp recorded_at = 2;
    repeated google.protobuf.Timestamp history = 3;
}
"#;

#[test]
fn e2e_cross_crate_imports_via_extern_path() {
    let plugin_bin = env!("CARGO_BIN_EXE_protoc-gen-roto");
    let tmp_dir = create_temp_dir("extern_path");
    let up_proto_dir = tmp_dir.join("upstream/proto");
    let up_gen_dir = tmp_dir.join("upstream/src");
    let down_proto_dir = tmp_dir.join("downstream/proto");
    let down_gen_dir = tmp_dir.join("downstream/src");
    let build_dir = tmp_dir.join("build");

    std::fs::create_dir_all(up_proto_dir.join("google/protobuf")).unwrap();
    std::fs::create_dir_all(&up_gen_dir).unwrap();
    std::fs::create_dir_all(down_proto_dir.join("telemetry")).unwrap();
    std::fs::create_dir_all(&down_gen_dir).unwrap();
    std::fs::create_dir_all(&build_dir).unwrap();

    std::fs::write(
        up_proto_dir.join("google/protobuf/timestamp.proto"),
        TIMESTAMP_PROTO,
    )
    .unwrap();
    std::fs::write(
        down_proto_dir.join("telemetry/event.proto"),
        TELEMETRY_EVENT_PROTO,
    )
    .unwrap();

    // 1. Generate upstream `google-protobuf-roto` crate
    let up_protoc = Command::new("protoc")
        .arg(format!("--plugin=protoc-gen-roto={plugin_bin}"))
        .arg(format!("-I{}", up_proto_dir.display()))
        .arg(format!("--roto_out={}", up_gen_dir.display()))
        .arg("google/protobuf/timestamp.proto")
        .output()
        .expect("failed to run protoc on upstream timestamp.proto");
    assert!(
        up_protoc.status.success(),
        "upstream protoc failed: {}",
        String::from_utf8_lossy(&up_protoc.stderr)
    );

    // 2. Generate downstream `telemetry-roto` crate with `--roto_opt=extern_path=google/protobuf=::google_protobuf_roto`
    let down_protoc = Command::new("protoc")
        .arg(format!("--plugin=protoc-gen-roto={plugin_bin}"))
        .arg(format!("-I{}", down_proto_dir.display()))
        .arg(format!("-I{}", up_proto_dir.display()))
        .arg("--roto_opt=extern_path=google/protobuf=::google_protobuf_roto")
        .arg(format!("--roto_out={}", down_gen_dir.display()))
        .arg("telemetry/event.proto")
        .output()
        .expect("failed to run protoc on downstream event.proto");
    assert!(
        down_protoc.status.success(),
        "downstream protoc failed: {}",
        String::from_utf8_lossy(&down_protoc.stderr)
    );

    let down_lib_rs = std::fs::read_to_string(down_gen_dir.join("lib.rs")).unwrap();
    assert!(
        down_lib_rs.contains("::google_protobuf_roto::google::protobuf::TimestampView<'a>"),
        "expected extern TimestampView path in downstream lib.rs:\n{down_lib_rs}"
    );
    assert!(
        down_lib_rs.contains("::google_protobuf_roto::google::protobuf::TimestampBuilder<'_, 'b>"),
        "expected extern TimestampBuilder path in downstream lib.rs:\n{down_lib_rs}"
    );
    assert!(
        !down_lib_rs.contains("pub mod google"),
        "downstream lib.rs must not regenerate google::protobuf module:\n{down_lib_rs}"
    );
    assert!(
        !down_lib_rs.contains("pub struct TimestampView"),
        "downstream lib.rs must not regenerate TimestampView:\n{down_lib_rs}"
    );

    // 3. Compile upstream and downstream crates and run cross-crate read/write test
    let roto_rlib = build_roto_rlibs(&build_dir);
    let up_rlib = build_dir.join("libgoogle_protobuf_roto.rlib");
    let compile_up = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=google_protobuf_roto")
        .arg("-D")
        .arg("warnings")
        .arg(up_gen_dir.join("lib.rs"))
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&up_rlib)
        .output()
        .expect("failed to compile upstream google_protobuf_roto");
    assert!(
        compile_up.status.success(),
        "compiling upstream failed:\n{}",
        String::from_utf8_lossy(&compile_up.stderr)
    );

    let down_rlib = build_dir.join("libtelemetry_roto.rlib");
    let compile_down = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=telemetry_roto")
        .arg("-D")
        .arg("warnings")
        .arg(down_gen_dir.join("lib.rs"))
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("--extern")
        .arg(format!("google_protobuf_roto={}", up_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&down_rlib)
        .output()
        .expect("failed to compile downstream telemetry_roto");
    assert!(
        compile_down.status.success(),
        "compiling downstream failed:\n{}",
        String::from_utf8_lossy(&compile_down.stderr)
    );

    let runner_src = tmp_dir.join("extern_runner.rs");
    std::fs::write(
        &runner_src,
        r#"
use google_protobuf_roto::google::protobuf::TimestampView;
use roto::runtime::BackwardWriter;
use telemetry_roto::telemetry::{TelemetryEventBuilder, TelemetryEventView};

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let mut buf = [0u8; 256];
    let mut writer = BackwardWriter::new(&mut buf);
    {
        let mut eb = TelemetryEventBuilder::new(&mut writer);
        eb.write_history(|ts| {
            ts.write_nanos(200)?;
            ts.write_seconds(1_700_000_002)
        })?;
        eb.write_history(|ts| {
            ts.write_nanos(100)?;
            ts.write_seconds(1_700_000_001)
        })?;
        eb.write_recorded_at(|ts| {
            ts.write_nanos(500_000_000)?;
            ts.write_seconds(1_700_000_000)
        })?;
        eb.write_event_name("boot_complete")?;
    }

    let event = TelemetryEventView::from_bytes(writer.finish())?;
    assert_eq!(event.event_name()?, "boot_complete");

    let recorded: TimestampView<'_> = event.recorded_at()?.expect("expected recorded_at");
    assert_eq!(recorded.seconds()?, 1_700_000_000);
    assert_eq!(recorded.nanos()?, 500_000_000);

    let hist: Vec<(i64, i32)> = event
        .history()
        .map(|res| {
            let ts = res.unwrap();
            (ts.seconds().unwrap(), ts.nanos().unwrap())
        })
        .collect();
    assert_eq!(hist, vec![(1_700_000_001, 100), (1_700_000_002, 200)]);

    Ok(())
}
"#,
    )
    .unwrap();

    let runner_bin = build_dir.join("extern_runner");
    let compile_runner = Command::new("rustc")
        .arg("--edition=2021")
        .arg(&runner_src)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("--extern")
        .arg(format!("google_protobuf_roto={}", up_rlib.display()))
        .arg("--extern")
        .arg(format!("telemetry_roto={}", down_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&runner_bin)
        .output()
        .expect("failed to compile extern_runner");
    assert!(
        compile_runner.status.success(),
        "compiling extern_runner failed:\n{}",
        String::from_utf8_lossy(&compile_runner.stderr)
    );

    let run_out = Command::new(&runner_bin)
        .output()
        .expect("failed to execute extern_runner");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    assert!(
        run_out.status.success(),
        "extern_runner failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_out.stdout),
        String::from_utf8_lossy(&run_out.stderr)
    );
}

#[test]
fn e2e_self_hosting_descriptor_and_plugin_protos() {
    let plugin_bin = env!("CARGO_BIN_EXE_protoc-gen-roto");
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let proto_root = manifest_dir.join("../google-protobuf-roto/proto");

    let tmp_dir = create_temp_dir("self_hosting");
    let gen_dir = tmp_dir.join("gen");
    let build_dir = tmp_dir.join("build");
    std::fs::create_dir_all(&gen_dir).unwrap();
    std::fs::create_dir_all(&build_dir).unwrap();

    let protoc_out = Command::new("protoc")
        .arg(format!("--plugin=protoc-gen-roto={plugin_bin}"))
        .arg(format!("--proto_path={}", proto_root.display()))
        .arg(format!("--roto_out={}", gen_dir.display()))
        .arg("google/protobuf/descriptor.proto")
        .arg("google/protobuf/compiler/plugin.proto")
        .output()
        .expect("failed to run protoc on descriptor.proto and plugin.proto");
    assert!(
        protoc_out.status.success(),
        "protoc failed on descriptor.proto + plugin.proto: {}",
        String::from_utf8_lossy(&protoc_out.stderr)
    );

    let generated_lib = gen_dir.join("lib.rs");
    let lib_content = std::fs::read_to_string(&generated_lib).unwrap();
    assert!(lib_content.starts_with("#![no_std]\n#![forbid(unsafe_code)]"));
    assert!(lib_content.contains("pub mod google {\n    pub mod protobuf {"));
    assert!(lib_content.contains("pub mod compiler {"));

    let roto_rlib = build_roto_rlibs(&build_dir);
    let self_hosted_rlib = build_dir.join("libself_hosted_google_protobuf.rlib");

    let compile_lib = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=self_hosted_google_protobuf")
        .arg("-D")
        .arg("warnings")
        .arg(&generated_lib)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&self_hosted_rlib)
        .output()
        .expect("failed to compile self-hosted descriptor + plugin crate");
    assert!(
        compile_lib.status.success(),
        "compiling self-hosted descriptor + plugin crate failed:\n{}",
        String::from_utf8_lossy(&compile_lib.stderr)
    );

    let runner_src = tmp_dir.join("self_hosted_runner.rs");
    std::fs::write(
        &runner_src,
        r##"
use roto::runtime::BackwardWriter;
use self_hosted_google_protobuf::google::protobuf::{
    compiler::{
        code_generator_response::Feature, CodeGeneratorRequestBuilder, CodeGeneratorRequestView,
        CodeGeneratorResponseBuilder, CodeGeneratorResponseView,
    },
    field_descriptor_proto::{Label, Type},
    file_options::OptimizeMode,
};

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let mut req_buf = [0u8; 512];
    let mut writer = BackwardWriter::new(&mut req_buf);
    {
        let mut req = CodeGeneratorRequestBuilder::new(&mut writer);
        req.write_proto_file(|fb| {
            fb.write_options(|opt| opt.write_optimize_for(OptimizeMode::CodeSize))?;
            fb.write_message_type(|msg| {
                msg.write_field(|f| {
                    f.write_type(Type::TypeUint64)?;
                    f.write_label(Label::LabelRequired)?;
                    f.write_number(1)?;
                    f.write_name("id")
                })?;
                msg.write_name("Sample")
            })?;
            fb.write_package("demo")?;
            fb.write_name("sample.proto")
        })?;
        req.write_file_to_generate("sample.proto")?;
    }

    let req_view = CodeGeneratorRequestView::from_bytes(writer.finish())?;
    let files_to_gen: Vec<&str> = req_view.file_to_generate().map(|f| f.unwrap()).collect();
    assert_eq!(files_to_gen, vec!["sample.proto"]);

    let proto_file = req_view.proto_file().next().unwrap()?;
    assert_eq!(proto_file.name()?, "sample.proto");
    assert_eq!(proto_file.package()?, "demo");
    assert_eq!(
        proto_file.options()?.unwrap().optimize_for()?,
        OptimizeMode::CodeSize
    );

    let mut resp_buf = [0u8; 256];
    let mut resp_writer = BackwardWriter::new(&mut resp_buf);
    {
        let mut resp = CodeGeneratorResponseBuilder::new(&mut resp_writer);
        resp.write_file(|f| {
            f.write_content("#![no_std]\n")?;
            f.write_name("lib.rs")
        })?;
        resp.write_supported_features(Feature::FeatureProto3Optional as u64)?;
    }

    let resp_view = CodeGeneratorResponseView::from_bytes(resp_writer.finish())?;
    assert_eq!(
        resp_view.supported_features()?,
        Feature::FeatureProto3Optional as u64
    );
    let gen_file = resp_view.file().next().unwrap()?;
    assert_eq!(gen_file.name()?, "lib.rs");
    assert_eq!(gen_file.content()?, "#![no_std]\n");

    Ok(())
}
"##,
    )
    .unwrap();

    let runner_bin = build_dir.join("self_hosted_runner");
    let compile_runner = Command::new("rustc")
        .arg("--edition=2021")
        .arg(&runner_src)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("--extern")
        .arg(format!(
            "self_hosted_google_protobuf={}",
            self_hosted_rlib.display()
        ))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&runner_bin)
        .output()
        .expect("failed to compile self_hosted_runner");
    assert!(
        compile_runner.status.success(),
        "compiling self_hosted_runner failed:\n{}",
        String::from_utf8_lossy(&compile_runner.stderr)
    );

    let run_out = Command::new(&runner_bin)
        .output()
        .expect("failed to execute self_hosted_runner");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    assert!(
        run_out.status.success(),
        "self_hosted_runner failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_out.stdout),
        String::from_utf8_lossy(&run_out.stderr)
    );
}
