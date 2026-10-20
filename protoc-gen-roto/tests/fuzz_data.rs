//! End-to-end generated-code data fuzzer for `protoc-gen-roto` (`unparse -> parse -> unparse`
//! and `parse -> unparse -> parse`).
//!
//! This integration test invokes `protoc` with `protoc-gen-roto` on a comprehensive schema
//! covering all 18 Protocol Buffers field types, `required` and `optional` fields with custom
//! defaults, `repeated` fields (both packed and unpacked), nested messages, `proto2` `group`s,
//! `oneof`s, and unrecognized wire tags. It compiles the generated `#![no_std]` code alongside
//! a deterministic fuzz runner binary and executes two complementary fuzzing modes:
//!
//! 1. **Fuzz Mode 1 (`unparse -> parse -> unparse` on random structured data)**:
//!    - Generates random structured `FuzzMsg` instances with random field presence, boundary
//!      integers, IEEE-754 special floats (`NaN` payloads, `+/-Inf`, `-0.0`, subnormals),
//!      random UTF-8 strings and binary blobs, nested messages, groups, mixed packed/unpacked
//!      repeated sequences, `oneof` variants, and injected unknown wire fields.
//!    - **Unparse**: Encodes each generated instance into a stack/fixed slice using
//!      `FuzzMsgBuilder::new(&mut buf)` and chained `write_*` / `write_packed_*` calls.
//!    - **Parse**: Decodes the resulting wire slice with `FuzzMsgView::from_bytes` and verifies
//!      every `has_*` predicate, scalar/slice accessor, nested message/group view, repeated
//!      iterator, `oneof` accessor, and `unknown_fields()` span against the source struct
//!      (using `.to_bits()` for `f32`/`f64`).
//!    - **Re-Unparse**: Re-serializes the decoded `FuzzMsgView` both via manual field-by-field
//!      builder reconstruction (`+ copy_unknown_fields`) and via `copy_unset_fields`, re-parses
//!      each resulting buffer, and asserts bit-for-bit value equivalence.
//!
//! 2. **Fuzz Mode 2 (`parse -> unparse -> parse` on arbitrary & mutated wire bytes)**:
//!    - Starts from valid encoded `FuzzMsg` wire buffers and applies random mutations (bit flips,
//!      byte insertions, deletions, truncations, splicing two messages together, and appending
//!      random wire tags/types) as well as purely random byte buffers.
//!    - **Parse**: Invokes `FuzzMsgView::from_bytes` and exercises every field accessor and
//!      repeated iterator on the mutated buffer to verify panic-freedom.
//!    - **Unparse -> Parse**: Whenever `from_bytes` and all accessors/iterators succeed (`Ok`),
//!      re-encodes the extracted view values into a fresh `FuzzMsgBuilder`, re-parses with
//!      `FuzzMsgView::from_bytes`, and asserts that the second view yields the exact same
//!      values as the first view.
//!
//! ## Configuration via Environment Variables
//! - `ROTO_FUZZ_SEED`: 64-bit PRNG seed in decimal or `0x` hex (default: `0x524F_544F_4441_5441`).
//! - `ROTO_FUZZ_ITERS`: Number of fuzz iterations per mode (default: `500`).

use std::path::{Path, PathBuf};
use std::process::Command;

fn create_temp_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "roto_fuzz_data_{}_{}",
        test_name,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

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

const FUZZ_PROTO: &str = r#"syntax = "proto2";

package fuzz.pkg;

enum FuzzEnum {
    FUZZ_ZERO = 0;
    FUZZ_ONE = 1;
    FUZZ_TWO = 2;
    FUZZ_NEG = -5;
}

message FuzzSubMsg {
    required int32 sub_req = 1;
    optional string sub_label = 2 [default = "sub_default"];
    repeated sint32 sub_vals = 3;
}

message FuzzMsg {
    required uint32 req_id = 1;

    optional double f_double = 2 [default = 42.5];
    optional float f_float = 3 [default = -1.25];
    optional int64 f_int64 = 4 [default = -6400];
    optional uint64 f_uint64 = 5 [default = 6400];
    optional int32 f_int32 = 6 [default = -3200];
    optional fixed64 f_fixed64 = 7 [default = 777];
    optional fixed32 f_fixed32 = 8 [default = 888];
    optional bool f_bool = 9 [default = true];
    optional string f_string = 10 [default = "default_str"];
    optional group FuzzGroup = 11 {
        optional int32 group_num = 1 [default = 11];
        optional bytes group_payload = 2 [default = "gp"];
    }
    optional FuzzSubMsg f_sub = 12;
    optional bytes f_bytes = 13 [default = "\x00\xff\x7f"];
    optional uint32 f_uint32 = 14 [default = 1400];
    optional FuzzEnum f_enum = 15 [default = FUZZ_ONE];
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
        optional int32 rg_id = 1 [default = -1];
        optional string rg_tag = 2 [default = "rg"];
    }
    repeated FuzzSubMsg r_subs = 30;
    repeated bytes r_bytes = 31;
    repeated uint32 r_uint32s = 32;
    repeated FuzzEnum r_enums = 33;
    repeated sfixed32 r_sfixed32s = 34;
    repeated sfixed64 r_sfixed64s = 35;
    repeated sint32 r_sint32s = 36;
    repeated sint64 r_sint64s = 37;

    oneof pick {
        string pick_text = 38;
        sint64 pick_num = 39;
        FuzzSubMsg pick_sub = 40;
        FuzzEnum pick_enum = 41;
    }
}
"#;

const FUZZ_RUNNER_RS: &str = r#"
use fuzz_roto::fuzz::pkg::{
    fuzz_msg::{FuzzGroupBuilder, FuzzGroupView, Pick, RepGroupsBuilder, RepGroupsView},
    FuzzEnum, FuzzMsgBuilder, FuzzMsgView, FuzzSubMsgBuilder, FuzzSubMsgView,
};
use roto::runtime::{DecodeError, FieldIter, WireType};

const DEFAULT_SEED: u64 = 0x524F_544F_4441_5441; // "ROTODATA"
const DEFAULT_ITERS: usize = 500;

#[derive(Clone, Debug)]
struct Rng {
    state: u64,
}

impl Rng {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1) != 0
    }

    fn gen_range(&mut self, upper_exclusive: usize) -> usize {
        if upper_exclusive <= 1 {
            return 0;
        }
        (self.next_u64() as usize) % upper_exclusive
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) {
        let mut i = 0;
        while i + 8 <= buf.len() {
            let w = self.next_u64().to_le_bytes();
            buf[i..i + 8].copy_from_slice(&w);
            i += 8;
        }
        if i < buf.len() {
            let w = self.next_u64().to_le_bytes();
            let rem = buf.len() - i;
            buf[i..].copy_from_slice(&w[..rem]);
        }
    }

    fn gen_i32(&mut self) -> i32 {
        match self.gen_range(10) {
            0 => 0,
            1 => 1,
            2 => -1,
            3 => 127,
            4 => -128,
            5 => i32::MAX,
            6 => i32::MIN,
            _ => self.next_u32() as i32,
        }
    }

    fn gen_u32(&mut self) -> u32 {
        match self.gen_range(8) {
            0 => 0,
            1 => 1,
            2 => 127,
            3 => 128,
            4 => 16_384,
            5 => u32::MAX,
            _ => self.next_u32(),
        }
    }

    fn gen_i64(&mut self) -> i64 {
        match self.gen_range(10) {
            0 => 0,
            1 => 1,
            2 => -1,
            3 => i32::MAX as i64,
            4 => i32::MIN as i64,
            5 => i64::MAX,
            6 => i64::MIN,
            _ => self.next_u64() as i64,
        }
    }

    fn gen_u64(&mut self) -> u64 {
        match self.gen_range(8) {
            0 => 0,
            1 => 1,
            2 => 127,
            3 => 128,
            4 => u32::MAX as u64,
            5 => u64::MAX,
            _ => self.next_u64(),
        }
    }

    fn gen_f32_bits(&mut self) -> u32 {
        let f: f32 = match self.gen_range(10) {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            4 => f32::MIN_POSITIVE,
            5 => f32::MAX,
            6 => f32::MIN,
            7 => f32::INFINITY,
            8 => f32::NEG_INFINITY,
            9 => f32::from_bits(0x7FC0_1234), // NaN with custom payload
            _ => f32::from_bits(self.next_u32()),
        };
        f.to_bits()
    }

    fn gen_f64_bits(&mut self) -> u64 {
        let f: f64 = match self.gen_range(10) {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            4 => f64::MIN_POSITIVE,
            5 => f64::MAX,
            6 => f64::MIN,
            7 => f64::INFINITY,
            8 => f64::NEG_INFINITY,
            9 => f64::from_bits(0x7FF8_0000_0000_5678), // NaN with custom payload
            _ => f64::from_bits(self.next_u64()),
        };
        f.to_bits()
    }

    fn gen_enum(&mut self) -> FuzzEnum {
        match self.gen_range(4) {
            0 => FuzzEnum::FuzzZero,
            1 => FuzzEnum::FuzzOne,
            2 => FuzzEnum::FuzzTwo,
            _ => FuzzEnum::FuzzNeg,
        }
    }

    fn gen_string(&mut self, max_len: usize) -> String {
        let len = self.gen_range(max_len + 1);
        let mut s = String::with_capacity(len);
        const ALPHABET: &[char] = &[
            'a', 'b', 'z', '0', '9', '_', '-', '\n', '\t', 'é', 'λ', 'Ω', '🦀',
        ];
        while s.len() < len {
            let ch = ALPHABET[self.gen_range(ALPHABET.len())];
            if s.len() + ch.len_utf8() <= len {
                s.push(ch);
            } else {
                s.push('x');
            }
        }
        s
    }

    fn gen_bytes(&mut self, max_len: usize) -> Vec<u8> {
        let len = self.gen_range(max_len + 1);
        let mut b = vec![0u8; len];
        self.fill_bytes(&mut b);
        b
    }
}

fn read_fuzz_config() -> (u64, usize) {
    let seed = std::env::var("ROTO_FUZZ_SEED")
        .ok()
        .and_then(|s| {
            let s = s.trim();
            if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                u64::from_str_radix(hex, 16).ok()
            } else {
                s.parse::<u64>().ok()
            }
        })
        .unwrap_or(DEFAULT_SEED);

    let iters = std::env::var("ROTO_FUZZ_ITERS")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(DEFAULT_ITERS);

    (seed, iters)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FuzzSubData {
    sub_req: i32,
    sub_label: Option<String>,
    sub_vals: Vec<i32>,
    pack_sub_vals: bool,
}

impl FuzzSubData {
    fn random(rng: &mut Rng) -> Self {
        let val_count = rng.gen_range(6);
        let mut sub_vals = Vec::with_capacity(val_count);
        for _ in 0..val_count {
            sub_vals.push(rng.gen_i32());
        }
        Self {
            sub_req: rng.gen_i32(),
            sub_label: if rng.next_bool() {
                Some(rng.gen_string(16))
            } else {
                None
            },
            sub_vals,
            pack_sub_vals: rng.next_bool(),
        }
    }

    fn write_into(&self, b: &mut FuzzSubMsgBuilder<'_>) {
        if !self.sub_vals.is_empty() {
            if self.pack_sub_vals {
                b.write_packed_sub_vals(&self.sub_vals);
            } else {
                for &v in self.sub_vals.iter().rev() {
                    b.write_sub_val(v);
                }
            }
        }
        if let Some(ref s) = self.sub_label {
            b.write_sub_label(s);
        }
        b.write_sub_req(self.sub_req);
    }

    fn from_view(v: &FuzzSubMsgView<'_>) -> Result<Self, DecodeError> {
        let sub_req = v.sub_req()?;
        let sub_label = if v.has_sub_label() {
            Some(v.sub_label()?.to_string())
        } else {
            // Also verify default accessor works when absent
            assert_eq!(v.sub_label()?, "sub_default");
            None
        };
        let mut sub_vals = Vec::new();
        for item in v.sub_vals() {
            sub_vals.push(item?);
        }
        Ok(Self {
            sub_req,
            sub_label,
            sub_vals,
            pack_sub_vals: true,
        })
    }

    fn normalize(&mut self) {
        self.pack_sub_vals = true;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FuzzGroupData {
    group_num: Option<i32>,
    group_payload: Option<Vec<u8>>,
}

impl FuzzGroupData {
    fn random(rng: &mut Rng) -> Self {
        Self {
            group_num: if rng.next_bool() {
                Some(rng.gen_i32())
            } else {
                None
            },
            group_payload: if rng.next_bool() {
                Some(rng.gen_bytes(16))
            } else {
                None
            },
        }
    }

    fn write_into(&self, b: &mut FuzzGroupBuilder<'_>) {
        if let Some(ref p) = self.group_payload {
            b.write_group_payload(p);
        }
        if let Some(n) = self.group_num {
            b.write_group_num(n);
        }
    }

    fn from_view(v: &FuzzGroupView<'_>) -> Result<Self, DecodeError> {
        let group_num = if v.has_group_num() {
            Some(v.group_num()?)
        } else {
            assert_eq!(v.group_num()?, 11);
            None
        };
        let group_payload = if v.has_group_payload() {
            Some(v.group_payload()?.to_vec())
        } else {
            assert_eq!(v.group_payload()?, b"gp");
            None
        };
        Ok(Self {
            group_num,
            group_payload,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RepGroupData {
    rg_id: Option<i32>,
    rg_tag: Option<String>,
}

impl RepGroupData {
    fn random(rng: &mut Rng) -> Self {
        Self {
            rg_id: if rng.next_bool() {
                Some(rng.gen_i32())
            } else {
                None
            },
            rg_tag: if rng.next_bool() {
                Some(rng.gen_string(12))
            } else {
                None
            },
        }
    }

    fn write_into(&self, b: &mut RepGroupsBuilder<'_>) {
        if let Some(ref t) = self.rg_tag {
            b.write_rg_tag(t);
        }
        if let Some(id) = self.rg_id {
            b.write_rg_id(id);
        }
    }

    fn from_view(v: &RepGroupsView<'_>) -> Result<Self, DecodeError> {
        let rg_id = if v.has_rg_id() {
            Some(v.rg_id()?)
        } else {
            assert_eq!(v.rg_id()?, -1);
            None
        };
        let rg_tag = if v.has_rg_tag() {
            Some(v.rg_tag()?.to_string())
        } else {
            assert_eq!(v.rg_tag()?, "rg");
            None
        };
        Ok(Self { rg_id, rg_tag })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PickData {
    Text(String),
    Num(i64),
    Sub(FuzzSubData),
    Enum(FuzzEnum),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FuzzMsgData {
    req_id: u32,
    f_double_bits: Option<u64>,
    f_float_bits: Option<u32>,
    f_int64: Option<i64>,
    f_uint64: Option<u64>,
    f_int32: Option<i32>,
    f_fixed64: Option<u64>,
    f_fixed32: Option<u32>,
    f_bool: Option<bool>,
    f_string: Option<String>,
    fuzzgroup: Option<FuzzGroupData>,
    f_sub: Option<FuzzSubData>,
    f_bytes: Option<Vec<u8>>,
    f_uint32: Option<u32>,
    f_enum: Option<FuzzEnum>,
    f_sfixed32: Option<i32>,
    f_sfixed64: Option<i64>,
    f_sint32: Option<i32>,
    f_sint64: Option<i64>,

    r_doubles_bits: Vec<u64>,
    r_floats_bits: Vec<u32>,
    r_int64s: Vec<i64>,
    r_uint64s: Vec<u64>,
    r_int32s: Vec<i32>,
    r_fixed64s: Vec<u64>,
    r_fixed32s: Vec<u32>,
    r_bools: Vec<bool>,
    r_strings: Vec<String>,
    repgroups: Vec<RepGroupData>,
    r_subs: Vec<FuzzSubData>,
    r_bytes: Vec<Vec<u8>>,
    r_uint32s: Vec<u32>,
    r_enums: Vec<FuzzEnum>,
    r_sfixed32s: Vec<i32>,
    r_sfixed64s: Vec<i64>,
    r_sint32s: Vec<i32>,
    r_sint64s: Vec<i64>,

    pick: Option<PickData>,
    unknown_varints: Vec<(u32, u64)>,
    pack_mask: u32,
}

impl FuzzMsgData {
    fn random(rng: &mut Rng) -> Self {
        let mut r_doubles_bits = Vec::new();
        for _ in 0..rng.gen_range(5) {
            r_doubles_bits.push(rng.gen_f64_bits());
        }
        let mut r_floats_bits = Vec::new();
        for _ in 0..rng.gen_range(5) {
            r_floats_bits.push(rng.gen_f32_bits());
        }
        let mut r_int64s = Vec::new();
        for _ in 0..rng.gen_range(5) {
            r_int64s.push(rng.gen_i64());
        }
        let mut r_uint64s = Vec::new();
        for _ in 0..rng.gen_range(5) {
            r_uint64s.push(rng.gen_u64());
        }
        let mut r_int32s = Vec::new();
        for _ in 0..rng.gen_range(5) {
            r_int32s.push(rng.gen_i32());
        }
        let mut r_fixed64s = Vec::new();
        for _ in 0..rng.gen_range(5) {
            r_fixed64s.push(rng.gen_u64());
        }
        let mut r_fixed32s = Vec::new();
        for _ in 0..rng.gen_range(5) {
            r_fixed32s.push(rng.gen_u32());
        }
        let mut r_bools = Vec::new();
        for _ in 0..rng.gen_range(6) {
            r_bools.push(rng.next_bool());
        }
        let mut r_strings = Vec::new();
        for _ in 0..rng.gen_range(4) {
            r_strings.push(rng.gen_string(20));
        }
        let mut repgroups = Vec::new();
        for _ in 0..rng.gen_range(4) {
            repgroups.push(RepGroupData::random(rng));
        }
        let mut r_subs = Vec::new();
        for _ in 0..rng.gen_range(4) {
            r_subs.push(FuzzSubData::random(rng));
        }
        let mut r_bytes = Vec::new();
        for _ in 0..rng.gen_range(4) {
            r_bytes.push(rng.gen_bytes(20));
        }
        let mut r_uint32s = Vec::new();
        for _ in 0..rng.gen_range(5) {
            r_uint32s.push(rng.gen_u32());
        }
        let mut r_enums = Vec::new();
        for _ in 0..rng.gen_range(5) {
            r_enums.push(rng.gen_enum());
        }
        let mut r_sfixed32s = Vec::new();
        for _ in 0..rng.gen_range(5) {
            r_sfixed32s.push(rng.gen_i32());
        }
        let mut r_sfixed64s = Vec::new();
        for _ in 0..rng.gen_range(5) {
            r_sfixed64s.push(rng.gen_i64());
        }
        let mut r_sint32s = Vec::new();
        for _ in 0..rng.gen_range(5) {
            r_sint32s.push(rng.gen_i32());
        }
        let mut r_sint64s = Vec::new();
        for _ in 0..rng.gen_range(5) {
            r_sint64s.push(rng.gen_i64());
        }

        let pick = match rng.gen_range(5) {
            0 => None,
            1 => Some(PickData::Text(rng.gen_string(16))),
            2 => Some(PickData::Num(rng.gen_i64())),
            3 => Some(PickData::Sub(FuzzSubData::random(rng))),
            _ => Some(PickData::Enum(rng.gen_enum())),
        };

        let mut unknown_varints = Vec::new();
        for _ in 0..rng.gen_range(4) {
            let tag = 100 + (rng.gen_range(900) as u32);
            unknown_varints.push((tag, rng.gen_u64()));
        }

        Self {
            req_id: rng.gen_u32(),
            f_double_bits: if rng.next_bool() {
                Some(rng.gen_f64_bits())
            } else {
                None
            },
            f_float_bits: if rng.next_bool() {
                Some(rng.gen_f32_bits())
            } else {
                None
            },
            f_int64: if rng.next_bool() {
                Some(rng.gen_i64())
            } else {
                None
            },
            f_uint64: if rng.next_bool() {
                Some(rng.gen_u64())
            } else {
                None
            },
            f_int32: if rng.next_bool() {
                Some(rng.gen_i32())
            } else {
                None
            },
            f_fixed64: if rng.next_bool() {
                Some(rng.gen_u64())
            } else {
                None
            },
            f_fixed32: if rng.next_bool() {
                Some(rng.gen_u32())
            } else {
                None
            },
            f_bool: if rng.next_bool() {
                Some(rng.next_bool())
            } else {
                None
            },
            f_string: if rng.next_bool() {
                Some(rng.gen_string(24))
            } else {
                None
            },
            fuzzgroup: if rng.next_bool() {
                Some(FuzzGroupData::random(rng))
            } else {
                None
            },
            f_sub: if rng.next_bool() {
                Some(FuzzSubData::random(rng))
            } else {
                None
            },
            f_bytes: if rng.next_bool() {
                Some(rng.gen_bytes(24))
            } else {
                None
            },
            f_uint32: if rng.next_bool() {
                Some(rng.gen_u32())
            } else {
                None
            },
            f_enum: if rng.next_bool() {
                Some(rng.gen_enum())
            } else {
                None
            },
            f_sfixed32: if rng.next_bool() {
                Some(rng.gen_i32())
            } else {
                None
            },
            f_sfixed64: if rng.next_bool() {
                Some(rng.gen_i64())
            } else {
                None
            },
            f_sint32: if rng.next_bool() {
                Some(rng.gen_i32())
            } else {
                None
            },
            f_sint64: if rng.next_bool() {
                Some(rng.gen_i64())
            } else {
                None
            },
            r_doubles_bits,
            r_floats_bits,
            r_int64s,
            r_uint64s,
            r_int32s,
            r_fixed64s,
            r_fixed32s,
            r_bools,
            r_strings,
            repgroups,
            r_subs,
            r_bytes,
            r_uint32s,
            r_enums,
            r_sfixed32s,
            r_sfixed64s,
            r_sint32s,
            r_sint64s,
            pick,
            unknown_varints,
            pack_mask: rng.next_u32(),
        }
    }

    fn normalize(&mut self) {
        if let Some(ref mut sub) = self.f_sub {
            sub.normalize();
        }
        for sub in &mut self.r_subs {
            sub.normalize();
        }
        if let Some(PickData::Sub(ref mut sub)) = self.pick {
            sub.normalize();
        }
        self.pack_mask = 0xFFFF_FFFF;
    }

    fn encode<'b>(&self, buf: &'b mut [u8]) -> &'b [u8] {
        let mut b = FuzzMsgBuilder::new(buf);

        // Write injected unknown fields at the tail first (so they appear in forward order)
        for &(tag, val) in self.unknown_varints.iter().rev() {
            let _ = b.writer_mut().write_varint(val);
            let _ = b.writer_mut().write_tag(tag, WireType::Varint);
        }

        match self.pick {
            None => {}
            Some(PickData::Text(ref s)) => {
                b.write_pick_text(s);
            }
            Some(PickData::Num(n)) => {
                b.write_pick_num(n);
            }
            Some(PickData::Sub(ref sub)) => {
                b.write_pick_sub(|sb| sub.write_into(sb));
            }
            Some(PickData::Enum(e)) => {
                b.write_pick_enum(e);
            }
        }

        // Repeated fields (randomly choose packed vs unpacked per pack_mask bit)
        if !self.r_sint64s.is_empty() {
            if (self.pack_mask & (1 << 0)) != 0 {
                b.write_packed_r_sint64s(&self.r_sint64s);
            } else {
                for &v in self.r_sint64s.iter().rev() {
                    b.write_r_sint64(v);
                }
            }
        }
        if !self.r_sint32s.is_empty() {
            if (self.pack_mask & (1 << 1)) != 0 {
                b.write_packed_r_sint32s(&self.r_sint32s);
            } else {
                for &v in self.r_sint32s.iter().rev() {
                    b.write_r_sint32(v);
                }
            }
        }
        if !self.r_sfixed64s.is_empty() {
            if (self.pack_mask & (1 << 2)) != 0 {
                b.write_packed_r_sfixed64s(&self.r_sfixed64s);
            } else {
                for &v in self.r_sfixed64s.iter().rev() {
                    b.write_r_sfixed64(v);
                }
            }
        }
        if !self.r_sfixed32s.is_empty() {
            if (self.pack_mask & (1 << 3)) != 0 {
                b.write_packed_r_sfixed32s(&self.r_sfixed32s);
            } else {
                for &v in self.r_sfixed32s.iter().rev() {
                    b.write_r_sfixed32(v);
                }
            }
        }
        if !self.r_enums.is_empty() {
            if (self.pack_mask & (1 << 4)) != 0 {
                b.write_packed_r_enums(&self.r_enums);
            } else {
                for &v in self.r_enums.iter().rev() {
                    b.write_r_enum(v);
                }
            }
        }
        if !self.r_uint32s.is_empty() {
            if (self.pack_mask & (1 << 5)) != 0 {
                b.write_packed_r_uint32s(&self.r_uint32s);
            } else {
                for &v in self.r_uint32s.iter().rev() {
                    b.write_r_uint32(v);
                }
            }
        }
        for v in self.r_bytes.iter().rev() {
            b.write_r_byte(v);
        }
        for sub in self.r_subs.iter().rev() {
            b.write_r_sub(|sb| sub.write_into(sb));
        }
        for rg in self.repgroups.iter().rev() {
            b.write_repgroup(|gb| rg.write_into(gb));
        }
        for s in self.r_strings.iter().rev() {
            b.write_r_string(s);
        }
        if !self.r_bools.is_empty() {
            if (self.pack_mask & (1 << 6)) != 0 {
                b.write_packed_r_bools(&self.r_bools);
            } else {
                for &v in self.r_bools.iter().rev() {
                    b.write_r_bool(v);
                }
            }
        }
        if !self.r_fixed32s.is_empty() {
            if (self.pack_mask & (1 << 7)) != 0 {
                b.write_packed_r_fixed32s(&self.r_fixed32s);
            } else {
                for &v in self.r_fixed32s.iter().rev() {
                    b.write_r_fixed32(v);
                }
            }
        }
        if !self.r_fixed64s.is_empty() {
            if (self.pack_mask & (1 << 8)) != 0 {
                b.write_packed_r_fixed64s(&self.r_fixed64s);
            } else {
                for &v in self.r_fixed64s.iter().rev() {
                    b.write_r_fixed64(v);
                }
            }
        }
        if !self.r_int32s.is_empty() {
            if (self.pack_mask & (1 << 9)) != 0 {
                b.write_packed_r_int32s(&self.r_int32s);
            } else {
                for &v in self.r_int32s.iter().rev() {
                    b.write_r_int32(v);
                }
            }
        }
        if !self.r_uint64s.is_empty() {
            if (self.pack_mask & (1 << 10)) != 0 {
                b.write_packed_r_uint64s(&self.r_uint64s);
            } else {
                for &v in self.r_uint64s.iter().rev() {
                    b.write_r_uint64(v);
                }
            }
        }
        if !self.r_int64s.is_empty() {
            if (self.pack_mask & (1 << 11)) != 0 {
                b.write_packed_r_int64s(&self.r_int64s);
            } else {
                for &v in self.r_int64s.iter().rev() {
                    b.write_r_int64(v);
                }
            }
        }
        if !self.r_floats_bits.is_empty() {
            let floats: Vec<f32> = self
                .r_floats_bits
                .iter()
                .map(|&bits| f32::from_bits(bits))
                .collect();
            if (self.pack_mask & (1 << 12)) != 0 {
                b.write_packed_r_floats(&floats);
            } else {
                for &v in floats.iter().rev() {
                    b.write_r_float(v);
                }
            }
        }
        if !self.r_doubles_bits.is_empty() {
            let doubles: Vec<f64> = self
                .r_doubles_bits
                .iter()
                .map(|&bits| f64::from_bits(bits))
                .collect();
            if (self.pack_mask & (1 << 13)) != 0 {
                b.write_packed_r_doubles(&doubles);
            } else {
                for &v in doubles.iter().rev() {
                    b.write_r_double(v);
                }
            }
        }

        // Singular optional & required fields
        if let Some(v) = self.f_sint64 {
            b.write_f_sint64(v);
        }
        if let Some(v) = self.f_sint32 {
            b.write_f_sint32(v);
        }
        if let Some(v) = self.f_sfixed64 {
            b.write_f_sfixed64(v);
        }
        if let Some(v) = self.f_sfixed32 {
            b.write_f_sfixed32(v);
        }
        if let Some(v) = self.f_enum {
            b.write_f_enum(v);
        }
        if let Some(v) = self.f_uint32 {
            b.write_f_uint32(v);
        }
        if let Some(ref v) = self.f_bytes {
            b.write_f_bytes(v);
        }
        if let Some(ref sub) = self.f_sub {
            b.write_f_sub(|sb| sub.write_into(sb));
        }
        if let Some(ref fg) = self.fuzzgroup {
            b.write_fuzzgroup(|gb| fg.write_into(gb));
        }
        if let Some(ref s) = self.f_string {
            b.write_f_string(s);
        }
        if let Some(v) = self.f_bool {
            b.write_f_bool(v);
        }
        if let Some(v) = self.f_fixed32 {
            b.write_f_fixed32(v);
        }
        if let Some(v) = self.f_fixed64 {
            b.write_f_fixed64(v);
        }
        if let Some(v) = self.f_int32 {
            b.write_f_int32(v);
        }
        if let Some(v) = self.f_uint64 {
            b.write_f_uint64(v);
        }
        if let Some(v) = self.f_int64 {
            b.write_f_int64(v);
        }
        if let Some(bits) = self.f_float_bits {
            b.write_f_float(f32::from_bits(bits));
        }
        if let Some(bits) = self.f_double_bits {
            b.write_f_double(f64::from_bits(bits));
        }
        b.write_req_id(self.req_id);

        b.check().expect("fuzz buffer should not overflow");
        b.finish().expect("finish must succeed")
    }

    fn from_view(v: &FuzzMsgView<'_>) -> Result<Self, DecodeError> {
        let req_id = v.req_id()?;
        assert!(v.has_req_id());

        let f_double_bits = if v.has_f_double() {
            Some(v.f_double()?.to_bits())
        } else {
            assert_eq!(v.f_double()?.to_bits(), 42.5f64.to_bits());
            None
        };
        let f_float_bits = if v.has_f_float() {
            Some(v.f_float()?.to_bits())
        } else {
            assert_eq!(v.f_float()?.to_bits(), (-1.25f32).to_bits());
            None
        };
        let f_int64 = if v.has_f_int64() {
            Some(v.f_int64()?)
        } else {
            assert_eq!(v.f_int64()?, -6400);
            None
        };
        let f_uint64 = if v.has_f_uint64() {
            Some(v.f_uint64()?)
        } else {
            assert_eq!(v.f_uint64()?, 6400);
            None
        };
        let f_int32 = if v.has_f_int32() {
            Some(v.f_int32()?)
        } else {
            assert_eq!(v.f_int32()?, -3200);
            None
        };
        let f_fixed64 = if v.has_f_fixed64() {
            Some(v.f_fixed64()?)
        } else {
            assert_eq!(v.f_fixed64()?, 777);
            None
        };
        let f_fixed32 = if v.has_f_fixed32() {
            Some(v.f_fixed32()?)
        } else {
            assert_eq!(v.f_fixed32()?, 888);
            None
        };
        let f_bool = if v.has_f_bool() {
            Some(v.f_bool()?)
        } else {
            assert!(v.f_bool()?);
            None
        };
        let f_string = if v.has_f_string() {
            Some(v.f_string()?.to_string())
        } else {
            assert_eq!(v.f_string()?, "default_str");
            None
        };
        let fuzzgroup = match v.fuzzgroup()? {
            Some(gv) => {
                assert!(v.has_fuzzgroup());
                Some(FuzzGroupData::from_view(&gv)?)
            }
            None => {
                assert!(!v.has_fuzzgroup());
                None
            }
        };
        let f_sub = match v.f_sub()? {
            Some(sv) => {
                assert!(v.has_f_sub());
                Some(FuzzSubData::from_view(&sv)?)
            }
            None => {
                assert!(!v.has_f_sub());
                None
            }
        };
        let f_bytes = if v.has_f_bytes() {
            Some(v.f_bytes()?.to_vec())
        } else {
            assert_eq!(v.f_bytes()?, &b"\x00\xff\x7f"[..]);
            None
        };
        let f_uint32 = if v.has_f_uint32() {
            Some(v.f_uint32()?)
        } else {
            assert_eq!(v.f_uint32()?, 1400);
            None
        };
        let f_enum = if v.has_f_enum() {
            Some(v.f_enum()?)
        } else {
            assert_eq!(v.f_enum()?, FuzzEnum::FuzzOne);
            None
        };
        let f_sfixed32 = if v.has_f_sfixed32() {
            Some(v.f_sfixed32()?)
        } else {
            assert_eq!(v.f_sfixed32()?, -160);
            None
        };
        let f_sfixed64 = if v.has_f_sfixed64() {
            Some(v.f_sfixed64()?)
        } else {
            assert_eq!(v.f_sfixed64()?, -170);
            None
        };
        let f_sint32 = if v.has_f_sint32() {
            Some(v.f_sint32()?)
        } else {
            assert_eq!(v.f_sint32()?, -180);
            None
        };
        let f_sint64 = if v.has_f_sint64() {
            Some(v.f_sint64()?)
        } else {
            assert_eq!(v.f_sint64()?, -190);
            None
        };

        let mut r_doubles_bits = Vec::new();
        for item in v.r_doubles() {
            r_doubles_bits.push(item?.to_bits());
        }
        let mut r_floats_bits = Vec::new();
        for item in v.r_floats() {
            r_floats_bits.push(item?.to_bits());
        }
        let mut r_int64s = Vec::new();
        for item in v.r_int64s() {
            r_int64s.push(item?);
        }
        let mut r_uint64s = Vec::new();
        for item in v.r_uint64s() {
            r_uint64s.push(item?);
        }
        let mut r_int32s = Vec::new();
        for item in v.r_int32s() {
            r_int32s.push(item?);
        }
        let mut r_fixed64s = Vec::new();
        for item in v.r_fixed64s() {
            r_fixed64s.push(item?);
        }
        let mut r_fixed32s = Vec::new();
        for item in v.r_fixed32s() {
            r_fixed32s.push(item?);
        }
        let mut r_bools = Vec::new();
        for item in v.r_bools() {
            r_bools.push(item?);
        }
        let mut r_strings = Vec::new();
        for item in v.r_strings() {
            r_strings.push(item?.to_string());
        }
        let mut repgroups = Vec::new();
        for item in v.repgroups() {
            let gv = item?;
            repgroups.push(RepGroupData::from_view(&gv)?);
        }
        let mut r_subs = Vec::new();
        for item in v.r_subs() {
            let sv = item?;
            r_subs.push(FuzzSubData::from_view(&sv)?);
        }
        let mut r_bytes = Vec::new();
        for item in v.r_bytes() {
            r_bytes.push(item?.to_vec());
        }
        let mut r_uint32s = Vec::new();
        for item in v.r_uint32s() {
            r_uint32s.push(item?);
        }
        let mut r_enums = Vec::new();
        for item in v.r_enums() {
            r_enums.push(item?);
        }
        let mut r_sfixed32s = Vec::new();
        for item in v.r_sfixed32s() {
            r_sfixed32s.push(item?);
        }
        let mut r_sfixed64s = Vec::new();
        for item in v.r_sfixed64s() {
            r_sfixed64s.push(item?);
        }
        let mut r_sint32s = Vec::new();
        for item in v.r_sint32s() {
            r_sint32s.push(item?);
        }
        let mut r_sint64s = Vec::new();
        for item in v.r_sint64s() {
            r_sint64s.push(item?);
        }

        // Also exercise direct oneof variant accessors
        let _ = v.pick_text()?;
        let _ = v.pick_num()?;
        let _ = v.pick_sub()?;
        let _ = v.pick_enum()?;

        let pick = match v.pick()? {
            None => None,
            Some(Pick::PickText(s)) => {
                assert!(v.has_pick_text());
                assert_eq!(v.pick_text()?, s);
                Some(PickData::Text(s.to_string()))
            }
            Some(Pick::PickNum(n)) => {
                assert!(v.has_pick_num());
                assert_eq!(v.pick_num()?, n);
                Some(PickData::Num(n))
            }
            Some(Pick::PickSub(sv)) => {
                assert!(v.has_pick_sub());
                Some(PickData::Sub(FuzzSubData::from_view(&sv)?))
            }
            Some(Pick::PickEnum(e)) => {
                assert!(v.has_pick_enum());
                assert_eq!(v.pick_enum()?, e);
                Some(PickData::Enum(e))
            }
        };

        let mut unknown_varints = Vec::new();
        for raw_unk in v.unknown_fields() {
            for item in FieldIter::new(raw_unk) {
                let f = item?;
                if f.wire_type == WireType::Varint {
                    let val = roto::runtime::decode_varint_at(raw_unk, f.marker)?;
                    unknown_varints.push((f.tag, val));
                }
            }
        }

        Ok(Self {
            req_id,
            f_double_bits,
            f_float_bits,
            f_int64,
            f_uint64,
            f_int32,
            f_fixed64,
            f_fixed32,
            f_bool,
            f_string,
            fuzzgroup,
            f_sub,
            f_bytes,
            f_uint32,
            f_enum,
            f_sfixed32,
            f_sfixed64,
            f_sint32,
            f_sint64,
            r_doubles_bits,
            r_floats_bits,
            r_int64s,
            r_uint64s,
            r_int32s,
            r_fixed64s,
            r_fixed32s,
            r_bools,
            r_strings,
            repgroups,
            r_subs,
            r_bytes,
            r_uint32s,
            r_enums,
            r_sfixed32s,
            r_sfixed64s,
            r_sint32s,
            r_sint64s,
            pick,
            unknown_varints,
            pack_mask: 0xFFFF_FFFF,
        })
    }
}

fn mutate_wire_bytes(rng: &mut Rng, base: &[u8], other: &[u8]) -> Vec<u8> {
    let mut out = base.to_vec();
    match rng.gen_range(7) {
        0 => {
            // Bit flips (1..=4 bits)
            if !out.is_empty() {
                for _ in 0..(1 + rng.gen_range(4)) {
                    let idx = rng.gen_range(out.len());
                    let bit = 1u8 << rng.gen_range(8);
                    out[idx] ^= bit;
                }
            }
        }
        1 => {
            // Random truncation
            if !out.is_empty() {
                let new_len = rng.gen_range(out.len());
                out.truncate(new_len);
            }
        }
        2 => {
            // Byte deletion
            if !out.is_empty() {
                let idx = rng.gen_range(out.len());
                out.remove(idx);
            }
        }
        3 => {
            // Random byte insertion
            let idx = rng.gen_range(out.len() + 1);
            out.insert(idx, rng.next_u32() as u8);
        }
        4 => {
            // Splice prefix of `base` with suffix of `other`
            let cut1 = rng.gen_range(out.len() + 1);
            out.truncate(cut1);
            if !other.is_empty() {
                let cut2 = rng.gen_range(other.len());
                out.extend_from_slice(&other[cut2..]);
            }
        }
        5 => {
            // Append random valid/invalid wire tag + bytes
            let extra_len = 1 + rng.gen_range(16);
            let start = out.len();
            out.resize(start + extra_len, 0);
            rng.fill_bytes(&mut out[start..]);
        }
        _ => {
            // Purely random byte buffer
            out = rng.gen_bytes(256);
        }
    }
    out
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let (seed, iters) = read_fuzz_config();
    let mut rng = Rng::new(seed);

    let mut buf1 = vec![0u8; 32 * 1024];
    let mut buf2 = vec![0u8; 32 * 1024];
    let mut buf3 = vec![0u8; 32 * 1024];

    let mut corpus: Vec<Vec<u8>> = Vec::with_capacity(64);

    // =========================================================================
    // Fuzz Mode 1: unparse -> parse -> unparse on random structured FuzzMsgData
    // =========================================================================
    for _ in 0..iters {
        let orig = FuzzMsgData::random(&mut rng);

        // 1. Unparse: encode `orig` via FuzzMsgBuilder
        let wire1 = orig.encode(&mut buf1).to_vec();
        if corpus.len() < 64 {
            corpus.push(wire1.clone());
        } else {
            let idx = rng.gen_range(corpus.len());
            corpus[idx] = wire1.clone();
        }

        // 2. Parse: decode with FuzzMsgView::from_bytes
        let view1 = FuzzMsgView::from_bytes(&wire1)
            .expect("valid FuzzMsgBuilder output must always parse cleanly");
        let parsed1 = FuzzMsgData::from_view(&view1)
            .expect("all field accessors and iterators on valid FuzzMsgView must succeed");

        let mut expected = orig.clone();
        expected.normalize();
        assert_eq!(parsed1, expected);

        // 3a. Re-Unparse via manual field-by-field encode (+ unknown fields)
        let wire2 = parsed1.encode(&mut buf2);
        let view2 = FuzzMsgView::from_bytes(wire2)
            .expect("re-encoded FuzzMsgBuilder output must parse cleanly");
        let parsed2 = FuzzMsgData::from_view(&view2)
            .expect("re-encoded FuzzMsgView accessors must succeed");
        assert_eq!(parsed2, expected);

        // 3b. Re-Unparse via copy_unset_fields (both full copy and partial override)
        let mut copy_b = FuzzMsgBuilder::new(&mut buf3);
        copy_b.copy_unset_fields(&view1);
        let wire3 = copy_b.finish().expect("copy_unset_fields must fit");
        let view3 = FuzzMsgView::from_bytes(wire3)
            .expect("copy_unset_fields output must parse cleanly");
        let parsed3 = FuzzMsgData::from_view(&view3)
            .expect("copy_unset_fields view accessors must succeed");
        assert_eq!(parsed3, expected);

        // Also test overriding `req_id` and `f_string` before `copy_unset_fields(&view1)`
        let mut patch_b = FuzzMsgBuilder::new(&mut buf3);
        patch_b
            .write_f_string("patched_by_fuzzer")
            .write_req_id(orig.req_id.wrapping_add(1))
            .copy_unset_fields(&view1);
        let wire_patched = patch_b.finish().expect("patched copy_unset_fields must fit");
        let view_patched = FuzzMsgView::from_bytes(wire_patched)?;
        let parsed_patched = FuzzMsgData::from_view(&view_patched)?;
        let mut expected_patched = expected.clone();
        expected_patched.req_id = orig.req_id.wrapping_add(1);
        expected_patched.f_string = Some("patched_by_fuzzer".to_string());
        assert_eq!(parsed_patched, expected_patched);
    }

    // =========================================================================
    // Fuzz Mode 2: parse -> unparse -> parse on arbitrary & mutated wire bytes
    // =========================================================================
    for _ in 0..iters {
        let base = if corpus.is_empty() {
            &[][..]
        } else {
            &corpus[rng.gen_range(corpus.len())]
        };
        let other = if corpus.is_empty() {
            &[][..]
        } else {
            &corpus[rng.gen_range(corpus.len())]
        };
        let mutated = mutate_wire_bytes(&mut rng, base, other);

        // 1. Parse mutated wire slice
        let Ok(view1) = FuzzMsgView::from_bytes(&mutated) else {
            continue;
        };

        // Exercise every accessor & iterator; if any field or nested message is malformed,
        // it must return `Err(DecodeError)` cleanly without panicking.
        let Ok(decoded1) = FuzzMsgData::from_view(&view1) else {
            continue;
        };

        // 2. Unparse: re-encode the extracted values into a fresh FuzzMsgBuilder
        //    (preserving raw unknown fields from view1 via copy_unknown_fields)
        let mut decoded_no_custom_unk = decoded1.clone();
        decoded_no_custom_unk.unknown_varints.clear();

        let reencoded = {
            let mut b = FuzzMsgBuilder::new(&mut buf2);
            b.copy_unknown_fields(&view1);
            // Encode all extracted schema fields on top of unknown fields
            let schema_only = decoded_no_custom_unk.encode(&mut buf1);
            let _ = b.writer_mut().write_bytes(schema_only);
            b.finish().expect("re-encode buffer must fit")
        };

        // 3. Re-Parse and verify exact semantic equivalence with `decoded1`
        let view2 = FuzzMsgView::from_bytes(reencoded)
            .expect("re-serialized valid FuzzMsgData must parse cleanly");
        let decoded2 = FuzzMsgData::from_view(&view2)
            .expect("re-serialized valid FuzzMsgView accessors must succeed");
        assert_eq!(decoded1, decoded2);
        assert_eq!(
            view1.unknown_fields().collect::<Vec<_>>(),
            view2.unknown_fields().collect::<Vec<_>>()
        );
    }

    Ok(())
}
"#;

#[test]
fn fuzz_generated_data_unparse_parse_and_wire_parse_unparse() {
    let plugin_bin = env!("CARGO_BIN_EXE_protoc-gen-roto");
    let tmp_dir = create_temp_dir("roundtrip");
    let proto_dir = tmp_dir.join("proto");
    let gen_dir = tmp_dir.join("gen");
    let build_dir = tmp_dir.join("build");
    std::fs::create_dir_all(&proto_dir).unwrap();
    std::fs::create_dir_all(&gen_dir).unwrap();
    std::fs::create_dir_all(&build_dir).unwrap();

    std::fs::write(proto_dir.join("fuzz.proto"), FUZZ_PROTO).unwrap();

    let protoc_out = Command::new("protoc")
        .arg(format!("--plugin=protoc-gen-roto={plugin_bin}"))
        .arg(format!("--proto_path={}", proto_dir.display()))
        .arg(format!("--roto_out={}", gen_dir.display()))
        .arg("fuzz.proto")
        .output()
        .expect("failed to run protoc on fuzz.proto");
    assert!(
        protoc_out.status.success(),
        "protoc failed: {}",
        String::from_utf8_lossy(&protoc_out.stderr)
    );

    let generated_lib = gen_dir.join("lib.rs");
    let roto_rlib = build_roto_rlibs(&build_dir);
    let fuzz_rlib = build_dir.join("libfuzz_roto.rlib");

    let compile_lib = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=fuzz_roto")
        .arg("-D")
        .arg("warnings")
        .arg(&generated_lib)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&fuzz_rlib)
        .output()
        .expect("failed to compile fuzz_roto");
    assert!(
        compile_lib.status.success(),
        "compiling fuzz_roto failed:\n{}",
        String::from_utf8_lossy(&compile_lib.stderr)
    );

    let runner_src = tmp_dir.join("fuzz_runner.rs");
    std::fs::write(&runner_src, FUZZ_RUNNER_RS).unwrap();

    let runner_bin = build_dir.join("fuzz_runner");
    let compile_runner = Command::new("rustc")
        .arg("--edition=2021")
        .arg("-D")
        .arg("warnings")
        .arg(&runner_src)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("--extern")
        .arg(format!("fuzz_roto={}", fuzz_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&runner_bin)
        .output()
        .expect("failed to compile fuzz_runner");
    assert!(
        compile_runner.status.success(),
        "compiling fuzz_runner failed:\n{}",
        String::from_utf8_lossy(&compile_runner.stderr)
    );

    let run_out = Command::new(&runner_bin)
        .output()
        .expect("failed to execute fuzz_runner");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    assert!(
        run_out.status.success(),
        "fuzz_runner failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_out.stdout),
        String::from_utf8_lossy(&run_out.stderr)
    );
}
