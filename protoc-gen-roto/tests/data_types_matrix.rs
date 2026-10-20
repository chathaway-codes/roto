//! Exhaustive end-to-end data-type, map, group, boundary-value, and `protoc` wire-interoperability
//! test suite for `protoc-gen-roto` and `roto-runtime`.
//!
//! # Test Sections & Coverage Matrix
//!
//! 1. **[`test_all_18_field_types_boundaries_and_partial_updates`]**:
//!    - Compiles a `syntax = "proto2"` schema covering all 18 `FieldDescriptorProto.Type` variants
//!      (`double`, `float`, `int64`, `uint64`, `int32`, `fixed64`, `fixed32`, `bool`, `string`,
//!      `group`, `message`, `bytes`, `uint32`, `enum`, `sfixed32`, `sfixed64`, `sint32`, `sint64`)
//!      across singular `optional` (with custom `[default = ...]`), `required`, `repeated`
//!      (unpacked, packed, and interleaved packed + unpacked on the same tag), and `oneof` fields.
//!    - Exercises `allow_alias = true` enums with negative (`-1`), zero (`0`), and `i32::MAX`
//!      (`2147483647`) discriminants, plus unknown wire enum discriminant fallback to the first
//!      declared variant.
//!    - Exercises extreme numeric and payload boundary values:
//!      - `i32::MIN`, `-1` (10-byte sign-extended varint), `0`, `i32::MAX`
//!      - `i64::MIN`, `-1`, `0`, `i64::MAX`
//!      - `u32::MAX`, `u64::MAX`
//!      - `f32::MIN`, `f32::MAX`, `f32::INFINITY`, `f32::NEG_INFINITY`, `f32::NAN` (`is_nan()`),
//!        `-0.0f32` (`to_bits()`), and subnormal `f32::from_bits(1)`
//!      - `f64::MIN`, `f64::MAX`, `f64::INFINITY`, `f64::NEG_INFINITY`, `f64::NAN` (`is_nan()`),
//!        `-0.0f64` (`to_bits()`), and subnormal `f64::from_bits(1)`
//!      - Empty `""` and multi-byte UTF-8 `"🦀 roto \u{1F680} café"` strings; empty `&b""[..]`
//!        and full `0x00..=0xFF` 256-byte slices
//!      - Nested `group` fields (both singular and repeated `RepeatedGroupIter`) and
//!        self-referential recursive messages (`TreeNode` with `left` and repeated `children`)
//!      - `oneof` last-wins overwrite semantics (verifying `has_<prev_variant>()` is cleared and
//!        overwritten point accessors return their default / `None`)
//!      - `copy_unset_fields` partial update round-trips (including `oneof` sibling masking and
//!        unknown field preservation)
//!
//! 2. **[`test_all_12_map_key_and_value_types`]**:
//!    - Compiles a schema declaring `map<K, V>` fields for all 12 valid Protobuf map key types:
//!      `int32`, `int64`, `uint32`, `uint64`, `sint32`, `sint64`, `fixed32`, `fixed64`,
//!      `sfixed32`, `sfixed64`, `bool`, and `string`, paired with scalar, `bytes`, `enum`, and
//!      nested `message` values.
//!    - Populates multiple entries per map via generated `<MapField>EntryBuilder` and reads them
//!      back via `<MapField>EntryView` iterators, including omitted key/value fallback to defaults.
//!
//! 3. **[`test_boundary_field_numbers_and_protoc_wire_interoperability`]**:
//!    - Exercises boundary field tag numbers `1`, `15` (max 1-byte tag), `16` (min 2-byte tag),
//!      `2047` (max 2-byte tag), `2048` (min 3-byte tag), and `536870911` (`(1 << 29) - 1`, max
//!      valid Protobuf field number).
//!    - **Roto -> `protoc --decode`**: Serializes a populated message covering all 18 field types,
//!      groups, maps, packed repeateds, negative `int32`/enum, and tag `536870911` using `roto`'s
//!      generated `*Builder`, pipes the wire bytes into `protoc --decode=<Pkg.Msg>`, and verifies
//!      `protoc` decodes every field cleanly.
//!    - **`protoc --encode` -> Roto**: Pipes a canonical textproto payload into
//!      `protoc --encode=<Pkg.Msg>` and verifies `roto`'s generated `*View::from_bytes` decodes
//!      every field and matches all expected values.
//!
//! # Maintenance Contract
//!
//! If the code generator (`protoc-gen-roto`) or runtime (`roto-runtime`) changes any of the
//! following contracts, update the corresponding runner assertions in this file:
//! - `<Msg>Builder<'b>::new(buf: &'b mut [u8]) -> Self` and `from_writer(BackwardWriter<'b>) -> Self`
//! - Builder write methods (`write_<field>`, `write_<singular>`, `write_packed_<field>`,
//!   `copy_unknown_fields`, `copy_unset_fields`) returning `&mut Self`
//! - Nested message and group writer closures taking `FnOnce(&mut SubBuilder<'_>)`
//! - `builder.check() -> Result<(), EncodeError>` and `builder.finish() -> Result<&'b [u8], EncodeError>`
//! - `FieldType::TypeGroup` framing (`WireType::StartGroup` / `WireType::EndGroup`) and
//!   `RepeatedGroupIter<'a, V>`
//! - `oneof` overwrite tracking (`has_<variant>()` and point accessor default fallback)

use std::path::{Path, PathBuf};
use std::process::Command;

fn create_temp_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "roto_data_types_matrix_{}_{}",
        test_name,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

/// Compiles `roto_runtime` and `roto` into `out_dir` as `.rlib`s so generated crates
/// and test binaries can be compiled directly with `rustc` in milliseconds.
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

fn compile_and_run_generated_test(
    test_name: &str,
    proto_filename: &str,
    proto_content: &str,
    crate_name: &str,
    runner_code: &str,
) {
    let plugin_bin = env!("CARGO_BIN_EXE_protoc-gen-roto");
    let tmp_dir = create_temp_dir(test_name);
    let proto_dir = tmp_dir.join("proto");
    let gen_dir = tmp_dir.join("gen");
    let build_dir = tmp_dir.join("build");
    std::fs::create_dir_all(&proto_dir).unwrap();
    std::fs::create_dir_all(&gen_dir).unwrap();
    std::fs::create_dir_all(&build_dir).unwrap();

    let proto_file = proto_dir.join(proto_filename);
    std::fs::write(&proto_file, proto_content).unwrap();

    let protoc_out = Command::new("protoc")
        .arg(format!("--plugin=protoc-gen-roto={plugin_bin}"))
        .arg(format!("--proto_path={}", proto_dir.display()))
        .arg(format!("--roto_out={}", gen_dir.display()))
        .arg(proto_filename)
        .output()
        .expect("failed to run protoc");
    assert!(
        protoc_out.status.success(),
        "protoc failed for {proto_filename}:\n{}",
        String::from_utf8_lossy(&protoc_out.stderr)
    );

    let generated_lib = gen_dir.join("lib.rs");
    let roto_rlib = build_roto_rlibs(&build_dir);
    let proto_rlib = build_dir.join(format!("lib{crate_name}.rlib"));

    let compile_lib = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg(format!("--crate-name={crate_name}"))
        .arg("-D")
        .arg("warnings")
        .arg(&generated_lib)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&proto_rlib)
        .output()
        .expect("failed to compile generated proto crate");
    assert!(
        compile_lib.status.success(),
        "compiling generated crate {crate_name} failed:\n{}",
        String::from_utf8_lossy(&compile_lib.stderr)
    );

    let runner_src = tmp_dir.join("runner.rs");
    std::fs::write(&runner_src, runner_code).unwrap();

    let runner_bin = build_dir.join("runner");
    let compile_runner = Command::new("rustc")
        .arg("--edition=2021")
        .arg("-D")
        .arg("warnings")
        .arg(&runner_src)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("--extern")
        .arg(format!("{crate_name}={}", proto_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&runner_bin)
        .output()
        .expect("failed to compile runner");
    assert!(
        compile_runner.status.success(),
        "compiling runner for {test_name} failed:\n{}",
        String::from_utf8_lossy(&compile_runner.stderr)
    );

    let run_out = Command::new(&runner_bin)
        .env("PROTO_DIR", &proto_dir)
        .env("PROTO_FILE", &proto_file)
        .output()
        .expect("failed to execute runner");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    assert!(
        run_out.status.success(),
        "runner for {test_name} failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run_out.stdout),
        String::from_utf8_lossy(&run_out.stderr)
    );
}

const ALL_18_TYPES_PROTO: &str = r#"syntax = "proto2";

package matrix.types;

enum ExtremeEnum {
    option allow_alias = true;
    EXT_ZERO = 0;
    EXT_NEG_ONE = -1;
    EXT_ALIAS_NEG = -1;
    EXT_MAX = 2147483647;
    EXT_ALIAS_MAX = 2147483647;
}

message SubPayload {
    optional int32 code = 1 [default = -42];
    optional string label = 2 [default = "sub-default"];
}

message TreeNode {
    optional int32 val = 1;
    optional TreeNode left = 2;
    repeated TreeNode children = 3;
}

message All18Matrix {
    // Required fields covering multiple wire types
    required int32 req_int32 = 1;
    required string req_string = 2;
    required fixed64 req_fixed64 = 3;

    // All 18 FieldDescriptorProto.Type variants as singular optional with custom defaults
    optional double o_double = 4 [default = -123.456];
    optional float o_float = 5 [default = 3.5];
    optional int64 o_int64 = 6 [default = -9000000000];
    optional uint64 o_uint64 = 7 [default = 18000000000];
    optional int32 o_int32 = 8 [default = -123456];
    optional fixed64 o_fixed64 = 9 [default = 9876543210];
    optional fixed32 o_fixed32 = 10 [default = 12345678];
    optional bool o_bool = 11 [default = true];
    optional string o_string = 12 [default = "default_str"];
    optional group OptGroup = 13 {
        optional int32 g_id = 1 [default = 1300];
        optional string g_tag = 2 [default = "g_default"];
        optional group NestedGroup = 3 {
            optional sint64 deep_val = 1 [default = -777];
        }
    }
    optional SubPayload o_message = 14;
    optional bytes o_bytes = 15 [default = "\x00\xffroto"];
    optional uint32 o_uint32 = 16 [default = 4000000000];
    optional ExtremeEnum o_enum = 17 [default = EXT_NEG_ONE];
    optional sfixed32 o_sfixed32 = 18 [default = -2000000000];
    optional sfixed64 o_sfixed64 = 19 [default = -8000000000000];
    optional sint32 o_sint32 = 20 [default = -1000000];
    optional sint64 o_sint64 = 21 [default = -500000000000];

    // All 18 FieldDescriptorProto.Type variants as repeated fields
    repeated double r_doubles = 22;
    repeated float r_floats = 23;
    repeated int64 r_int64s = 24;
    repeated uint64 r_uint64s = 25;
    repeated int32 r_int32s = 26;
    repeated fixed64 r_fixed64s = 27;
    repeated fixed32 r_fixed32s = 28;
    repeated bool r_bools = 29;
    repeated string r_strings = 30;
    repeated group RepGroups = 31 {
        optional int32 rg_id = 1;
        optional string rg_name = 2;
    }
    repeated SubPayload r_messages = 32;
    repeated bytes r_bytes = 33;
    repeated uint32 r_uint32s = 34;
    repeated ExtremeEnum r_enums = 35;
    repeated sfixed32 r_sfixed32s = 36;
    repeated sfixed64 r_sfixed64s = 37;
    repeated sint32 r_sint32s = 38;
    repeated sint64 r_sint64s = 39;

    // Self-referential recursive message field
    optional TreeNode root_tree = 40;

    // Oneofs covering all 18 types (including group and message variants)
    oneof primary_oneof {
        double oo_double = 41;
        float oo_float = 42;
        int64 oo_int64 = 43;
        uint64 oo_uint64 = 44;
        int32 oo_int32 = 45;
        fixed64 oo_fixed64 = 46;
        fixed32 oo_fixed32 = 47;
        bool oo_bool = 48;
        string oo_string = 49;
        group OoGroup = 50 {
            optional int32 oog_val = 1 [default = 50];
        }
        SubPayload oo_message = 51;
        bytes oo_bytes = 52;
        uint32 oo_uint32 = 53;
        ExtremeEnum oo_enum = 54;
        sfixed32 oo_sfixed32 = 55;
        sfixed64 oo_sfixed64 = 56;
        sint32 oo_sint32 = 57;
        sint64 oo_sint64 = 58;
    }
}
"#;

#[test]
fn test_all_18_field_types_boundaries_and_partial_updates() {
    let runner = r#"
use roto::runtime::{BackwardWriter, DecodeError, WireType};
use matrix_types::matrix::types::{
    all18_matrix::{
        opt_group::NestedGroupView, opt_group::NestedGroupBuilder, OptGroupBuilder, OptGroupView,
        OoGroupBuilder, PrimaryOneof, RepGroupsBuilder, RepGroupsView,
    },
    All18MatrixBuilder, All18MatrixView, ExtremeEnum, SubPayloadBuilder, TreeNodeBuilder,
    TreeNodeView,
};

fn main() -> Result<(), Box<dyn core::error::Error>> {
    // 1. Verify enum allow_alias constants and negative / MAX discriminants
    assert_eq!(ExtremeEnum::ExtAliasNeg, ExtremeEnum::ExtNegOne);
    assert_eq!(ExtremeEnum::ExtAliasMax, ExtremeEnum::ExtMax);
    assert_eq!(ExtremeEnum::ExtNegOne.as_i32(), -1);
    assert_eq!(ExtremeEnum::ExtZero.as_i32(), 0);
    assert_eq!(ExtremeEnum::ExtMax.as_i32(), i32::MAX);
    assert_eq!(ExtremeEnum::from_i32(-1), Some(ExtremeEnum::ExtNegOne));
    assert_eq!(ExtremeEnum::from_i32(0), Some(ExtremeEnum::ExtZero));
    assert_eq!(ExtremeEnum::from_i32(i32::MAX), Some(ExtremeEnum::ExtMax));
    assert_eq!(ExtremeEnum::from_i32(999), None);

    // 2. Verify required field enforcement and custom [default = ...] values
    assert_eq!(
        All18MatrixView::from_bytes(&[]).unwrap_err(),
        DecodeError::MissingRequiredField("All18Matrix.req_int32")
    );

    let mut min_buf = [0u8; 256];
    let mut mb = All18MatrixBuilder::new(&mut min_buf);
    mb.write_req_fixed64(0xDEAD_BEEF_CAFE_BABEu64)
        .write_req_string("req-ok")
        .write_req_int32(-1);
    let min_view = All18MatrixView::from_bytes(mb.finish()?)?;
    assert_eq!(min_view.req_int32()?, -1);
    assert_eq!(min_view.req_string()?, "req-ok");
    assert_eq!(min_view.req_fixed64()?, 0xDEAD_BEEF_CAFE_BABEu64);
    assert!(!min_view.has_o_double());
    assert_eq!(min_view.o_double()?, -123.456);
    assert_eq!(min_view.o_float()?, 3.5);
    assert_eq!(min_view.o_int64()?, -9000000000);
    assert_eq!(min_view.o_uint64()?, 18000000000);
    assert_eq!(min_view.o_int32()?, -123456);
    assert_eq!(min_view.o_fixed64()?, 9876543210);
    assert_eq!(min_view.o_fixed32()?, 12345678);
    assert!(min_view.o_bool()?);
    assert_eq!(min_view.o_string()?, "default_str");
    assert!(min_view.optgroup()?.is_none());
    assert!(min_view.o_message()?.is_none());
    assert_eq!(min_view.o_bytes()?, &b"\x00\xffroto"[..]);
    assert_eq!(min_view.o_uint32()?, 4000000000);
    assert_eq!(min_view.o_enum()?, ExtremeEnum::ExtNegOne);
    assert_eq!(min_view.o_sfixed32()?, -2000000000);
    assert_eq!(min_view.o_sfixed64()?, -8000000000000);
    assert_eq!(min_view.o_sint32()?, -1000000);
    assert_eq!(min_view.o_sint64()?, -500000000000);
    assert!(min_view.primary_oneof()?.is_none());

    // 3. Extreme boundary values across all 18 types (unpacked, packed, and interleaved packed+unpacked)
    let all_256_bytes: Vec<u8> = (0u8..=255u8).collect();
    let utf8_str = "🦀 roto \u{1F680} café";
    let f32_subnormal = f32::from_bits(1);
    let f64_subnormal = f64::from_bits(1);

    let mut full_buf = vec![0u8; 16384];
    let mut b = All18MatrixBuilder::new(&mut full_buf);

    // Populate recursive TreeNode: root(10) -> left(5 -> left(2)), children([20 -> children([21, 22]), 30])
    b.write_root_tree(|root: &mut TreeNodeBuilder<'_>| {
        root.write_children(|c2| {
            c2.write_val(30);
        })
        .write_children(|c1| {
            c1.write_children(|gc2| {
                gc2.write_val(22);
            })
            .write_children(|gc1| {
                gc1.write_val(21);
            })
            .write_val(20);
        })
        .write_left(|l1| {
            l1.write_left(|l2| {
                l2.write_val(2);
            })
            .write_val(5);
        })
        .write_val(10);
    });

    // Interleaved unpacked + packed + unpacked on repeated fields (written in reverse for BackwardWriter)
    // r_int32s: unpacked i32::MAX, packed [-1, 0], unpacked i32::MIN -> forward: [i32::MIN, -1, 0, i32::MAX]
    b.write_r_int32(i32::MAX)
        .write_packed_r_int32s(&[-1, 0])
        .write_r_int32(i32::MIN);

    // r_int64s: unpacked i64::MAX, packed [-1, 0], unpacked i64::MIN
    b.write_r_int64(i64::MAX)
        .write_packed_r_int64s(&[-1, 0])
        .write_r_int64(i64::MIN);

    // r_uint32s: unpacked u32::MAX, packed [1, 2], unpacked 0
    b.write_r_uint32(u32::MAX)
        .write_packed_r_uint32s(&[1, 2])
        .write_r_uint32(0);

    // r_uint64s: unpacked u64::MAX, packed [1, 2], unpacked 0
    b.write_r_uint64(u64::MAX)
        .write_packed_r_uint64s(&[1, 2])
        .write_r_uint64(0);

    // r_sint32s: unpacked i32::MAX, packed [-1, 0], unpacked i32::MIN
    b.write_r_sint32(i32::MAX)
        .write_packed_r_sint32s(&[-1, 0])
        .write_r_sint32(i32::MIN);

    // r_sint64s: unpacked i64::MAX, packed [-1, 0], unpacked i64::MIN
    b.write_r_sint64(i64::MAX)
        .write_packed_r_sint64s(&[-1, 0])
        .write_r_sint64(i64::MIN);

    // r_fixed32s & r_sfixed32s
    b.write_r_fixed32(u32::MAX)
        .write_packed_r_fixed32s(&[0, 42]);
    b.write_r_sfixed32(i32::MAX)
        .write_packed_r_sfixed32s(&[i32::MIN, -1, 0]);

    // r_fixed64s & r_sfixed64s
    b.write_r_fixed64(u64::MAX)
        .write_packed_r_fixed64s(&[0, 42]);
    b.write_r_sfixed64(i64::MAX)
        .write_packed_r_sfixed64s(&[i64::MIN, -1, 0]);

    // r_floats: [f32::MIN, f32::MAX, f32::INFINITY, f32::NEG_INFINITY, f32::NAN, -0.0f32, f32_subnormal]
    b.write_r_float(f32_subnormal)
        .write_packed_r_floats(&[
            f32::MIN,
            f32::MAX,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
            -0.0f32,
        ]);

    // r_doubles: [f64::MIN, f64::MAX, f64::INFINITY, f64::NEG_INFINITY, f64::NAN, -0.0f64, f64_subnormal]
    b.write_r_double(f64_subnormal)
        .write_packed_r_doubles(&[
            f64::MIN,
            f64::MAX,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            -0.0f64,
        ]);

    // r_bools: unpacked true, packed [false, true] -> forward [false, true, true]
    b.write_r_bool(true).write_packed_r_bools(&[false, true]);

    // r_enums: unpacked ExtMax, packed [ExtNegOne, ExtZero] -> forward [ExtNegOne, ExtZero, ExtMax]
    b.write_r_enum(ExtremeEnum::ExtMax)
        .write_packed_r_enums(&[ExtremeEnum::ExtNegOne, ExtremeEnum::ExtZero]);

    // r_strings: ["", utf8_str]
    b.write_r_string(utf8_str).write_r_string("");

    // r_bytes: [&b""[..], &all_256_bytes]
    b.write_r_bytes(&all_256_bytes).write_r_bytes(b"");

    // r_messages & RepGroups
    b.write_r_message(|sub: &mut SubPayloadBuilder<'_>| {
        sub.write_label("msg-2").write_code(i32::MAX);
    })
    .write_r_message(|sub: &mut SubPayloadBuilder<'_>| {
        sub.write_label("").write_code(i32::MIN);
    })
    .write_repgroup(|rg: &mut RepGroupsBuilder<'_>| {
        rg.write_rg_name("grp-2").write_rg_id(-2);
    })
    .write_repgroup(|rg: &mut RepGroupsBuilder<'_>| {
        rg.write_rg_name("grp-1").write_rg_id(1);
    });

    // Singular optional fields with boundary values
    b.write_o_sint64(i64::MIN)
        .write_o_sint32(i32::MIN)
        .write_o_sfixed64(i64::MIN)
        .write_o_sfixed32(i32::MIN)
        .write_o_enum(ExtremeEnum::ExtMax)
        .write_o_uint32(u32::MAX)
        .write_o_bytes(&all_256_bytes)
        .write_o_message(|sub| {
            sub.write_label(utf8_str).write_code(-1);
        })
        .write_optgroup(|og: &mut OptGroupBuilder<'_>| {
            og.write_nestedgroup(|ng: &mut NestedGroupBuilder<'_>| {
                ng.write_deep_val(i64::MAX);
            })
            .write_g_tag(utf8_str)
            .write_g_id(i32::MIN);
        })
        .write_o_string(utf8_str)
        .write_o_bool(false)
        .write_o_fixed32(u32::MAX)
        .write_o_fixed64(u64::MAX)
        .write_o_int32(i32::MIN)
        .write_o_uint64(u64::MAX)
        .write_o_int64(i64::MIN)
        .write_o_float(-0.0f32)
        .write_o_double(-0.0f64)
        .write_req_fixed64(u64::MAX)
        .write_req_string("")
        .write_req_int32(-1)
        .check()?;

    let wire = b.finish()?;
    let view = All18MatrixView::from_bytes(wire)?;

    // Verify required + singular optional boundary values
    assert_eq!(view.req_int32()?, -1);
    assert_eq!(view.req_string()?, "");
    assert_eq!(view.req_fixed64()?, u64::MAX);

    assert_eq!(view.o_double()?.to_bits(), (-0.0f64).to_bits());
    assert_eq!(view.o_float()?.to_bits(), (-0.0f32).to_bits());
    assert_eq!(view.o_int64()?, i64::MIN);
    assert_eq!(view.o_uint64()?, u64::MAX);
    assert_eq!(view.o_int32()?, i32::MIN);
    assert_eq!(view.o_fixed64()?, u64::MAX);
    assert_eq!(view.o_fixed32()?, u32::MAX);
    assert!(!view.o_bool()?);
    assert_eq!(view.o_string()?, utf8_str);

    let og: OptGroupView<'_> = view.optgroup()?.expect("expected optgroup");
    assert_eq!(og.g_id()?, i32::MIN);
    assert_eq!(og.g_tag()?, utf8_str);
    let ng: NestedGroupView<'_> = og.nestedgroup()?.expect("expected nestedgroup");
    assert_eq!(ng.deep_val()?, i64::MAX);

    let om = view.o_message()?.expect("expected o_message");
    assert_eq!(om.code()?, -1);
    assert_eq!(om.label()?, utf8_str);

    assert_eq!(view.o_bytes()?, all_256_bytes.as_slice());
    assert_eq!(view.o_uint32()?, u32::MAX);
    assert_eq!(view.o_enum()?, ExtremeEnum::ExtMax);
    assert_eq!(view.o_sfixed32()?, i32::MIN);
    assert_eq!(view.o_sfixed64()?, i64::MIN);
    assert_eq!(view.o_sint32()?, i32::MIN);
    assert_eq!(view.o_sint64()?, i64::MIN);

    // Verify interleaved unpacked + packed repeated sequences
    assert_eq!(
        view.r_int32s().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec![i32::MIN, -1, 0, i32::MAX]
    );
    assert_eq!(
        view.r_int64s().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec![i64::MIN, -1, 0, i64::MAX]
    );
    assert_eq!(
        view.r_uint32s().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec![0, 1, 2, u32::MAX]
    );
    assert_eq!(
        view.r_uint64s().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec![0, 1, 2, u64::MAX]
    );
    assert_eq!(
        view.r_sint32s().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec![i32::MIN, -1, 0, i32::MAX]
    );
    assert_eq!(
        view.r_sint64s().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec![i64::MIN, -1, 0, i64::MAX]
    );
    assert_eq!(
        view.r_fixed32s().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec![0, 42, u32::MAX]
    );
    assert_eq!(
        view.r_sfixed32s().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec![i32::MIN, -1, 0, i32::MAX]
    );
    assert_eq!(
        view.r_fixed64s().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec![0, 42, u64::MAX]
    );
    assert_eq!(
        view.r_sfixed64s().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec![i64::MIN, -1, 0, i64::MAX]
    );

    let r_floats: Vec<f32> = view.r_floats().map(|x| x.unwrap()).collect();
    assert_eq!(r_floats.len(), 7);
    assert_eq!(r_floats[0], f32::MIN);
    assert_eq!(r_floats[1], f32::MAX);
    assert_eq!(r_floats[2], f32::INFINITY);
    assert_eq!(r_floats[3], f32::NEG_INFINITY);
    assert!(r_floats[4].is_nan());
    assert_eq!(r_floats[5].to_bits(), (-0.0f32).to_bits());
    assert_eq!(r_floats[6].to_bits(), f32_subnormal.to_bits());

    let r_doubles: Vec<f64> = view.r_doubles().map(|x| x.unwrap()).collect();
    assert_eq!(r_doubles.len(), 7);
    assert_eq!(r_doubles[0], f64::MIN);
    assert_eq!(r_doubles[1], f64::MAX);
    assert_eq!(r_doubles[2], f64::INFINITY);
    assert_eq!(r_doubles[3], f64::NEG_INFINITY);
    assert!(r_doubles[4].is_nan());
    assert_eq!(r_doubles[5].to_bits(), (-0.0f64).to_bits());
    assert_eq!(r_doubles[6].to_bits(), f64_subnormal.to_bits());

    assert_eq!(
        view.r_bools().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec![false, true, true]
    );
    assert_eq!(
        view.r_enums().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec![ExtremeEnum::ExtNegOne, ExtremeEnum::ExtZero, ExtremeEnum::ExtMax]
    );
    assert_eq!(
        view.r_strings().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec!["", utf8_str]
    );
    assert_eq!(
        view.r_bytes().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec![&b""[..], all_256_bytes.as_slice()]
    );

    let r_groups: Vec<(i32, &str)> = view
        .repgroups()
        .map(|g: Result<RepGroupsView<'_>, _>| {
            let gv = g.unwrap();
            (gv.rg_id().unwrap(), gv.rg_name().unwrap())
        })
        .collect();
    assert_eq!(r_groups, vec![(1, "grp-1"), (-2, "grp-2")]);

    let r_msgs: Vec<(i32, &str)> = view
        .r_messages()
        .map(|m| {
            let mv = m.unwrap();
            (mv.code().unwrap(), mv.label().unwrap())
        })
        .collect();
    assert_eq!(r_msgs, vec![(i32::MIN, ""), (i32::MAX, "msg-2")]);

    // Verify recursive TreeNode
    let tree: TreeNodeView<'_> = view.root_tree()?.expect("expected root_tree");
    assert_eq!(tree.val()?, 10);
    let left1 = tree.left()?.expect("expected left1");
    assert_eq!(left1.val()?, 5);
    let left2 = left1.left()?.expect("expected left2");
    assert_eq!(left2.val()?, 2);
    assert!(left2.left()?.is_none());
    let children: Vec<TreeNodeView<'_>> = tree.children().map(|c| c.unwrap()).collect();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].val()?, 20);
    let grandchildren: Vec<i32> = children[0]
        .children()
        .map(|gc| gc.unwrap().val().unwrap())
        .collect();
    assert_eq!(grandchildren, vec![21, 22]);
    assert_eq!(children[1].val()?, 30);

    // 4. Test oneof overwrite behavior and group variant inside oneof
    // In BackwardWriter, the variant written first is at the end of the wire buffer (last-wins).
    let mut oo_buf = [0u8; 512];
    let mut ob = All18MatrixBuilder::new(&mut oo_buf);
    ob.write_oogroup(|gb: &mut OoGroupBuilder<'_>| {
        gb.write_oog_val(777);
    })
    .write_oo_string("earlier_overwritten")
    .write_oo_int32(-99)
    .write_req_fixed64(1)
    .write_req_string("r")
    .write_req_int32(1);
    let oo_view = All18MatrixView::from_bytes(ob.finish()?)?;
    assert!(oo_view.has_oogroup());
    assert!(!oo_view.has_oo_string());
    assert!(!oo_view.has_oo_int32());
    assert_eq!(oo_view.oo_string()?, "");
    assert_eq!(oo_view.oo_int32()?, 0);
    assert_eq!(oo_view.oogroup()?.unwrap().oog_val()?, 777);
    match oo_view.primary_oneof()?.expect("expected primary_oneof") {
        PrimaryOneof::Oogroup(gv) => assert_eq!(gv.oog_val()?, 777),
        other => panic!("expected Oogroup, got {other:?}"),
    }

    // 5. Test copy_unset_fields partial update round-trip (including unknown field & oneof override)
    let mut unk_buf = [0u8; 512];
    let mut unk_writer = BackwardWriter::new(&mut unk_buf);
    unk_writer.write_varint(12345)?;
    unk_writer.write_tag(999, WireType::Varint)?;
    let mut base_b = All18MatrixBuilder::from_writer(unk_writer);
    base_b
        .write_oo_string("initial_oo")
        .write_o_string("initial_opt")
        .write_o_int32(42)
        .write_req_fixed64(55)
        .write_req_string("base_req")
        .write_req_int32(10);
    let base_view = All18MatrixView::from_bytes(base_b.finish()?)?;
    assert_eq!(base_view.unknown_fields().count(), 1);

    let mut patch_buf = [0u8; 512];
    let mut patch_b = All18MatrixBuilder::new(&mut patch_buf);
    patch_b
        .write_oo_sint64(-999_999)
        .write_o_string("patched_opt")
        .copy_unset_fields(&base_view);
    let patched_view = All18MatrixView::from_bytes(patch_b.finish()?)?;
    assert_eq!(patched_view.req_int32()?, 10);
    assert_eq!(patched_view.req_string()?, "base_req");
    assert_eq!(patched_view.req_fixed64()?, 55);
    assert_eq!(patched_view.o_int32()?, 42);
    assert_eq!(patched_view.o_string()?, "patched_opt");
    assert!(patched_view.has_oo_sint64());
    assert!(!patched_view.has_oo_string());
    assert_eq!(patched_view.oo_sint64()?, -999_999);
    assert_eq!(patched_view.unknown_fields().count(), 1);

    Ok(())
}
"#;

    compile_and_run_generated_test(
        "all_18_types_matrix",
        "all_18_types.proto",
        ALL_18_TYPES_PROTO,
        "matrix_types",
        runner,
    );
}

const ALL_12_MAPS_PROTO: &str = r#"syntax = "proto3";

package matrix.maps;

enum MapEnum {
    MAP_ENUM_ZERO = 0;
    MAP_ENUM_ONE = 1;
    MAP_ENUM_NEG = -1;
}

message MapSubMsg {
    int32 id = 1;
    string note = 2;
}

message All12Maps {
    map<int32, string> m_int32_str = 1;
    map<int64, int64> m_int64_int64 = 2;
    map<uint32, uint32> m_uint32_uint32 = 3;
    map<uint64, uint64> m_uint64_uint64 = 4;
    map<sint32, sint32> m_sint32_sint32 = 5;
    map<sint64, sint64> m_sint64_sint64 = 6;
    map<fixed32, fixed32> m_fixed32_fixed32 = 7;
    map<fixed64, fixed64> m_fixed64_fixed64 = 8;
    map<sfixed32, sfixed32> m_sfixed32_sfixed32 = 9;
    map<sfixed64, sfixed64> m_sfixed64_sfixed64 = 10;
    map<bool, MapEnum> m_bool_enum = 11;
    map<string, MapSubMsg> m_string_msg = 12;
}
"#;

#[test]
fn test_all_12_map_key_and_value_types() {
    let runner = r#"
use matrix_maps::matrix::maps::{
    all12_maps::{
        MBoolEnumEntryBuilder, MFixed32Fixed32EntryBuilder, MFixed64Fixed64EntryBuilder,
        MInt32StrEntryBuilder, MInt64Int64EntryBuilder, MSfixed32Sfixed32EntryBuilder,
        MSfixed64Sfixed64EntryBuilder, MSint32Sint32EntryBuilder, MSint64Sint64EntryBuilder,
        MStringMsgEntryBuilder, MUint32Uint32EntryBuilder, MUint64Uint64EntryBuilder,
    },
    All12MapsBuilder, All12MapsView, MapEnum, MapSubMsgBuilder,
};

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let mut buf = [0u8; 4096];
    let mut b = All12MapsBuilder::new(&mut buf);

    // 12. map<string, MapSubMsg> (including an entry with omitted key and omitted value for default testing)
    b.write_m_string_msg(|eb: &mut MStringMsgEntryBuilder<'_>| {
        // Omitted key and value -> defaults to ("", None)
        let _ = eb;
    })
    .write_m_string_msg(|eb: &mut MStringMsgEntryBuilder<'_>| {
        eb.write_value(|sub: &mut MapSubMsgBuilder<'_>| {
            sub.write_note("second").write_id(200);
        })
        .write_key("k2");
    })
    .write_m_string_msg(|eb: &mut MStringMsgEntryBuilder<'_>| {
        eb.write_value(|sub: &mut MapSubMsgBuilder<'_>| {
            sub.write_note("first").write_id(100);
        })
        .write_key("k1");
    });

    // 11. map<bool, MapEnum>
    b.write_m_bool_enum(|eb: &mut MBoolEnumEntryBuilder<'_>| {
        eb.write_value(MapEnum::MapEnumNeg).write_key(false);
    })
    .write_m_bool_enum(|eb: &mut MBoolEnumEntryBuilder<'_>| {
        eb.write_value(MapEnum::MapEnumOne).write_key(true);
    });

    // 10. map<sfixed64, sfixed64>
    b.write_m_sfixed64_sfixed64(|eb: &mut MSfixed64Sfixed64EntryBuilder<'_>| {
        eb.write_value(i64::MIN).write_key(i64::MAX);
    })
    .write_m_sfixed64_sfixed64(|eb: &mut MSfixed64Sfixed64EntryBuilder<'_>| {
        eb.write_value(i64::MAX).write_key(i64::MIN);
    });

    // 9. map<sfixed32, sfixed32>
    b.write_m_sfixed32_sfixed32(|eb: &mut MSfixed32Sfixed32EntryBuilder<'_>| {
        eb.write_value(i32::MIN).write_key(i32::MAX);
    })
    .write_m_sfixed32_sfixed32(|eb: &mut MSfixed32Sfixed32EntryBuilder<'_>| {
        eb.write_value(i32::MAX).write_key(i32::MIN);
    });

    // 8. map<fixed64, fixed64>
    b.write_m_fixed64_fixed64(|eb: &mut MFixed64Fixed64EntryBuilder<'_>| {
        eb.write_value(0).write_key(u64::MAX);
    })
    .write_m_fixed64_fixed64(|eb: &mut MFixed64Fixed64EntryBuilder<'_>| {
        eb.write_value(u64::MAX).write_key(0);
    });

    // 7. map<fixed32, fixed32>
    b.write_m_fixed32_fixed32(|eb: &mut MFixed32Fixed32EntryBuilder<'_>| {
        eb.write_value(0).write_key(u32::MAX);
    })
    .write_m_fixed32_fixed32(|eb: &mut MFixed32Fixed32EntryBuilder<'_>| {
        eb.write_value(u32::MAX).write_key(0);
    });

    // 6. map<sint64, sint64>
    b.write_m_sint64_sint64(|eb: &mut MSint64Sint64EntryBuilder<'_>| {
        eb.write_value(-1).write_key(i64::MAX);
    })
    .write_m_sint64_sint64(|eb: &mut MSint64Sint64EntryBuilder<'_>| {
        eb.write_value(i64::MAX).write_key(i64::MIN);
    });

    // 5. map<sint32, sint32>
    b.write_m_sint32_sint32(|eb: &mut MSint32Sint32EntryBuilder<'_>| {
        eb.write_value(-1).write_key(i32::MAX);
    })
    .write_m_sint32_sint32(|eb: &mut MSint32Sint32EntryBuilder<'_>| {
        eb.write_value(i32::MAX).write_key(i32::MIN);
    });

    // 4. map<uint64, uint64>
    b.write_m_uint64_uint64(|eb: &mut MUint64Uint64EntryBuilder<'_>| {
        eb.write_value(100).write_key(u64::MAX);
    })
    .write_m_uint64_uint64(|eb: &mut MUint64Uint64EntryBuilder<'_>| {
        eb.write_value(u64::MAX).write_key(1);
    });

    // 3. map<uint32, uint32>
    b.write_m_uint32_uint32(|eb: &mut MUint32Uint32EntryBuilder<'_>| {
        eb.write_value(100).write_key(u32::MAX);
    })
    .write_m_uint32_uint32(|eb: &mut MUint32Uint32EntryBuilder<'_>| {
        eb.write_value(u32::MAX).write_key(1);
    });

    // 2. map<int64, int64>
    b.write_m_int64_int64(|eb: &mut MInt64Int64EntryBuilder<'_>| {
        eb.write_value(i64::MIN).write_key(-1);
    })
    .write_m_int64_int64(|eb: &mut MInt64Int64EntryBuilder<'_>| {
        eb.write_value(-1).write_key(i64::MIN);
    });

    // 1. map<int32, string> (including an entry omitting value -> default "")
    b.write_m_int32_str(|eb: &mut MInt32StrEntryBuilder<'_>| {
        eb.write_key(0);
    })
    .write_m_int32_str(|eb: &mut MInt32StrEntryBuilder<'_>| {
        eb.write_value("max").write_key(i32::MAX);
    })
    .write_m_int32_str(|eb: &mut MInt32StrEntryBuilder<'_>| {
        eb.write_value("min").write_key(i32::MIN);
    })
    .check()?;

    let view = All12MapsView::from_bytes(b.finish()?)?;

    assert_eq!(
        view.m_int32_str()
            .map(|e| {
                let ev = e.unwrap();
                (ev.key().unwrap(), ev.value().unwrap())
            })
            .collect::<Vec<_>>(),
        vec![(i32::MIN, "min"), (i32::MAX, "max"), (0, "")]
    );

    assert_eq!(
        view.m_int64_int64()
            .map(|e| {
                let ev = e.unwrap();
                (ev.key().unwrap(), ev.value().unwrap())
            })
            .collect::<Vec<_>>(),
        vec![(i64::MIN, -1), (-1, i64::MIN)]
    );

    assert_eq!(
        view.m_uint32_uint32()
            .map(|e| {
                let ev = e.unwrap();
                (ev.key().unwrap(), ev.value().unwrap())
            })
            .collect::<Vec<_>>(),
        vec![(1, u32::MAX), (u32::MAX, 100)]
    );

    assert_eq!(
        view.m_uint64_uint64()
            .map(|e| {
                let ev = e.unwrap();
                (ev.key().unwrap(), ev.value().unwrap())
            })
            .collect::<Vec<_>>(),
        vec![(1, u64::MAX), (u64::MAX, 100)]
    );

    assert_eq!(
        view.m_sint32_sint32()
            .map(|e| {
                let ev = e.unwrap();
                (ev.key().unwrap(), ev.value().unwrap())
            })
            .collect::<Vec<_>>(),
        vec![(i32::MIN, i32::MAX), (i32::MAX, -1)]
    );

    assert_eq!(
        view.m_sint64_sint64()
            .map(|e| {
                let ev = e.unwrap();
                (ev.key().unwrap(), ev.value().unwrap())
            })
            .collect::<Vec<_>>(),
        vec![(i64::MIN, i64::MAX), (i64::MAX, -1)]
    );

    assert_eq!(
        view.m_fixed32_fixed32()
            .map(|e| {
                let ev = e.unwrap();
                (ev.key().unwrap(), ev.value().unwrap())
            })
            .collect::<Vec<_>>(),
        vec![(0, u32::MAX), (u32::MAX, 0)]
    );

    assert_eq!(
        view.m_fixed64_fixed64()
            .map(|e| {
                let ev = e.unwrap();
                (ev.key().unwrap(), ev.value().unwrap())
            })
            .collect::<Vec<_>>(),
        vec![(0, u64::MAX), (u64::MAX, 0)]
    );

    assert_eq!(
        view.m_sfixed32_sfixed32()
            .map(|e| {
                let ev = e.unwrap();
                (ev.key().unwrap(), ev.value().unwrap())
            })
            .collect::<Vec<_>>(),
        vec![(i32::MIN, i32::MAX), (i32::MAX, i32::MIN)]
    );

    assert_eq!(
        view.m_sfixed64_sfixed64()
            .map(|e| {
                let ev = e.unwrap();
                (ev.key().unwrap(), ev.value().unwrap())
            })
            .collect::<Vec<_>>(),
        vec![(i64::MIN, i64::MAX), (i64::MAX, i64::MIN)]
    );

    assert_eq!(
        view.m_bool_enum()
            .map(|e| {
                let ev = e.unwrap();
                (ev.key().unwrap(), ev.value().unwrap())
            })
            .collect::<Vec<_>>(),
        vec![(true, MapEnum::MapEnumOne), (false, MapEnum::MapEnumNeg)]
    );

    let str_msg_entries: Vec<(&str, Option<(i32, &str)>)> = view
        .m_string_msg()
        .map(|e| {
            let ev = e.unwrap();
            let val = ev
                .value()
                .unwrap()
                .map(|mv| (mv.id().unwrap(), mv.note().unwrap()));
            (ev.key().unwrap(), val)
        })
        .collect();
    assert_eq!(
        str_msg_entries,
        vec![
            ("k1", Some((100, "first"))),
            ("k2", Some((200, "second"))),
            ("", None),
        ]
    );

    Ok(())
}
"#;

    compile_and_run_generated_test(
        "all_12_maps_matrix",
        "all_12_maps.proto",
        ALL_12_MAPS_PROTO,
        "matrix_maps",
        runner,
    );
}

const INTEROP_PROTO: &str = r#"syntax = "proto2";

package matrix.interop;

enum InteropEnum {
    IE_ZERO = 0;
    IE_NEG = -1;
    IE_MAX = 2147483647;
}

message InteropSub {
    optional int32 sub_id = 1;
    optional string sub_name = 2;
}

message WireInteropMsg {
    // Boundary tag numbers: 1, 15, 16, 2047, 2048, 536870911 ((1 << 29) - 1)
    optional int32 tag_1 = 1;
    optional string tag_15 = 15;
    optional uint64 tag_16 = 16;
    optional sint32 tag_2047 = 2047;
    optional fixed64 tag_2048 = 2048;
    optional string tag_max_536870911 = 536870911;

    // Remaining types for full 18-type + map + group bidirectional protoc interop
    optional double f_double = 2;
    optional float f_float = 3;
    optional int64 f_int64 = 4;
    optional fixed32 f_fixed32 = 5;
    optional bool f_bool = 6;
    optional group InteropGroup = 7 {
        optional int32 g_val = 1;
        optional string g_str = 2;
    }
    optional InteropSub f_sub = 8;
    optional bytes f_bytes = 9;
    optional uint32 f_uint32 = 10;
    optional InteropEnum f_enum = 11;
    optional sfixed32 f_sfixed32 = 12;
    optional sfixed64 f_sfixed64 = 13;
    optional sint64 f_sint64 = 14;

    repeated int32 packed_ints = 17 [packed = true];
    map<string, int32> kv_map = 18;
}
"#;

#[test]
fn test_boundary_field_numbers_and_protoc_wire_interoperability() {
    let runner = r##"
use std::io::Write;
use std::process::{Command, Stdio};

use matrix_interop::matrix::interop::{
    wire_interop_msg::{InteropGroupBuilder, KvMapEntryBuilder},
    InteropEnum, InteropSubBuilder, WireInteropMsgBuilder, WireInteropMsgView,
};

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let proto_dir = std::env::var("PROTO_DIR").expect("PROTO_DIR env");
    let proto_file = std::env::var("PROTO_FILE").expect("PROTO_FILE env");

    // =========================================================================
    // Part A: Roto Builder -> Wire Bytes -> `protoc --decode=matrix.interop.WireInteropMsg`
    // =========================================================================
    let mut buf = [0u8; 4096];
    let mut b = WireInteropMsgBuilder::new(&mut buf);
    b.write_tag_max_536870911("max-tag-payload")
        .write_tag_2048(0xCAFE_BABE_1234_5678u64)
        .write_tag_2047(-123456)
        .write_kv_map(|eb: &mut KvMapEntryBuilder<'_>| {
            eb.write_value(-99).write_key("beta");
        })
        .write_kv_map(|eb: &mut KvMapEntryBuilder<'_>| {
            eb.write_value(42).write_key("alpha");
        })
        .write_packed_packed_ints(&[-100, 0, 100, i32::MAX])
        .write_tag_16(16000)
        .write_tag_15("fifteen")
        .write_f_sint64(-9876543210)
        .write_f_sfixed64(-64646464)
        .write_f_sfixed32(-323232)
        .write_f_enum(InteropEnum::IeNeg)
        .write_f_uint32(32000)
        .write_f_bytes(b"\x00\x01\x02\xff")
        .write_f_sub(|sb: &mut InteropSubBuilder<'_>| {
            sb.write_sub_name("nested-sub").write_sub_id(77);
        })
        .write_interopgroup(|gb: &mut InteropGroupBuilder<'_>| {
            gb.write_g_str("group-hello").write_g_val(88);
        })
        .write_f_bool(true)
        .write_f_fixed32(0x1234_5678)
        .write_f_int64(-1)
        .write_f_float(2.5)
        .write_f_double(6.25)
        .write_tag_1(-1)
        .check()?;

    let roto_wire = b.finish()?;

    let mut decode_child = Command::new("protoc")
        .arg(format!("--proto_path={proto_dir}"))
        .arg("--decode=matrix.interop.WireInteropMsg")
        .arg(&proto_file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn protoc --decode");

    decode_child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(roto_wire)
        .unwrap();
    let decode_out = decode_child.wait_with_output().unwrap();
    assert!(
        decode_out.status.success(),
        "protoc --decode failed:\n{}",
        String::from_utf8_lossy(&decode_out.stderr)
    );

    let decoded_text = String::from_utf8_lossy(&decode_out.stdout);
    assert!(decoded_text.contains("tag_1: -1"), "missing tag_1 in:\n{decoded_text}");
    assert!(decoded_text.contains("f_double: 6.25"), "missing f_double in:\n{decoded_text}");
    assert!(decoded_text.contains("f_float: 2.5"), "missing f_float in:\n{decoded_text}");
    assert!(decoded_text.contains("f_int64: -1"), "missing f_int64 in:\n{decoded_text}");
    assert!(decoded_text.contains("f_fixed32: 305419896"), "missing f_fixed32 in:\n{decoded_text}");
    assert!(decoded_text.contains("f_bool: true"), "missing f_bool in:\n{decoded_text}");
    assert!(decoded_text.contains("InteropGroup {"), "missing InteropGroup in:\n{decoded_text}");
    assert!(decoded_text.contains("g_val: 88"), "missing g_val in:\n{decoded_text}");
    assert!(decoded_text.contains("g_str: \"group-hello\""), "missing g_str in:\n{decoded_text}");
    assert!(decoded_text.contains("sub_id: 77"), "missing sub_id in:\n{decoded_text}");
    assert!(decoded_text.contains("sub_name: \"nested-sub\""), "missing sub_name in:\n{decoded_text}");
    assert!(decoded_text.contains("f_enum: IE_NEG"), "missing f_enum in:\n{decoded_text}");
    assert!(decoded_text.contains("tag_15: \"fifteen\""), "missing tag_15 in:\n{decoded_text}");
    assert!(decoded_text.contains("tag_16: 16000"), "missing tag_16 in:\n{decoded_text}");
    assert!(decoded_text.contains("tag_2047: -123456"), "missing tag_2047 in:\n{decoded_text}");
    assert!(
        decoded_text.contains("tag_2048: 14627333965257922168"),
        "missing tag_2048 in:\n{decoded_text}"
    );
    assert!(
        decoded_text.contains("tag_max_536870911: \"max-tag-payload\""),
        "missing tag_max_536870911 in:\n{decoded_text}"
    );

    // =========================================================================
    // Part B: `protoc --encode=matrix.interop.WireInteropMsg` -> Wire Bytes -> Roto View
    // =========================================================================
    let textproto_input = r#"
tag_1: -2147483648
f_double: -3.125
f_float: 0.5
f_int64: -9223372036854775808
f_fixed32: 4294967295
f_bool: true
InteropGroup {
  g_val: -777
  g_str: "from-protoc-group"
}
f_sub {
  sub_id: 999
  sub_name: "from-protoc-sub"
}
f_bytes: "\000\001\377protoc"
f_uint32: 4294967295
f_enum: IE_MAX
f_sfixed32: -2147483648
f_sfixed64: -9223372036854775808
f_sint64: -1234567890123
tag_15: "boundary-15"
tag_16: 18446744073709551615
packed_ints: -1
packed_ints: 0
packed_ints: 2147483647
kv_map {
  key: "k_one"
  value: 111
}
kv_map {
  key: "k_two"
  value: -222
}
tag_2047: -2147483648
tag_2048: 18446744073709551615
tag_max_536870911: "from-protoc-max-tag"
"#;

    let mut encode_child = Command::new("protoc")
        .arg(format!("--proto_path={proto_dir}"))
        .arg("--encode=matrix.interop.WireInteropMsg")
        .arg(&proto_file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn protoc --encode");

    encode_child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(textproto_input.as_bytes())
        .unwrap();
    let encode_out = encode_child.wait_with_output().unwrap();
    assert!(
        encode_out.status.success(),
        "protoc --encode failed:\n{}",
        String::from_utf8_lossy(&encode_out.stderr)
    );

    let protoc_wire = &encode_out.stdout;
    let view = WireInteropMsgView::from_bytes(protoc_wire)?;

    assert_eq!(view.tag_1()?, i32::MIN);
    assert_eq!(view.f_double()?, -3.125);
    assert_eq!(view.f_float()?, 0.5);
    assert_eq!(view.f_int64()?, i64::MIN);
    assert_eq!(view.f_fixed32()?, u32::MAX);
    assert!(view.f_bool()?);

    let grp = view.interopgroup()?.expect("expected interopgroup");
    assert_eq!(grp.g_val()?, -777);
    assert_eq!(grp.g_str()?, "from-protoc-group");

    let sub = view.f_sub()?.expect("expected f_sub");
    assert_eq!(sub.sub_id()?, 999);
    assert_eq!(sub.sub_name()?, "from-protoc-sub");

    assert_eq!(view.f_bytes()?, &b"\x00\x01\xffprotoc"[..]);
    assert_eq!(view.f_uint32()?, u32::MAX);
    assert_eq!(view.f_enum()?, InteropEnum::IeMax);
    assert_eq!(view.f_sfixed32()?, i32::MIN);
    assert_eq!(view.f_sfixed64()?, i64::MIN);
    assert_eq!(view.f_sint64()?, -1234567890123);
    assert_eq!(view.tag_15()?, "boundary-15");
    assert_eq!(view.tag_16()?, u64::MAX);
    assert_eq!(
        view.packed_ints().map(|x| x.unwrap()).collect::<Vec<_>>(),
        vec![-1, 0, i32::MAX]
    );
    assert_eq!(
        view.kv_map()
            .map(|e| {
                let ev = e.unwrap();
                (ev.key().unwrap(), ev.value().unwrap())
            })
            .collect::<Vec<_>>(),
        vec![("k_one", 111), ("k_two", -222)]
    );
    assert_eq!(view.tag_2047()?, i32::MIN);
    assert_eq!(view.tag_2048()?, u64::MAX);
    assert_eq!(view.tag_max_536870911()?, "from-protoc-max-tag");
    assert_eq!(view.unknown_fields().count(), 0);

    Ok(())
}
"##;

    compile_and_run_generated_test(
        "wire_interop_matrix",
        "interop.proto",
        INTEROP_PROTO,
        "matrix_interop",
        runner,
    );
}
