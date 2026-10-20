//! Deterministic random descriptor schema & data fuzzer for `protoc-gen-roto`.
//!
//! This integration test implements a self-contained, zero-dependency random Protocol Buffers
//! schema and data generator (`RandomSchemaGenerator`) that exercises the entire `roto`
//! compiler bootstrap, code generator, and runtime pipeline end-to-end without invoking an
//! external `protoc` binary.
//!
//! ## Architecture & Three-Stage Verification Pipeline
//!
//! Each fuzz batch generates `ROTO_FUZZ_SCHEMAS_PER_BATCH` random multi-file, multi-package
//! (`proto2` and `proto3`) schemas and runs them through three verification stages:
//!
//! 1. **Stage A — Descriptor Unparse & Parse Verification**:
//!    - **Unparse**: Encodes the randomly generated `FileDescriptorProto` trees into a raw
//!      `CodeGeneratorRequest` wire buffer using
//!      `google_protobuf_roto_bootstrap::google::protobuf::compiler::CodeGeneratorRequestBuilder`.
//!    - **Parse & Validate**: Decodes the wire buffer back into a zero-copy
//!      `CodeGeneratorRequestView` via `CodeGeneratorRequestView::from_bytes`, walks and
//!      validates all descriptor fields via `protoc_gen_roto::validate_request`, and asserts
//!      that descriptor metadata (file names, packages, message/enum counts, field tags)
//!      matches the source schema.
//!    - **Descriptor Re-Unparse Round-Trip**: Re-serializes the decoded `CodeGeneratorRequestView`
//!      into a fresh buffer via `CodeGeneratorRequestBuilder::copy_unset_fields`, asserts
//!      byte-for-byte identity with the original request wire bytes, and re-validates the
//!      copied view. Also exercises full plugin framing (`CodeGeneratorRequest` ->
//!      `CodeGeneratorResponse`) via `protoc_gen_roto::process_bytes`.
//!
//! 2. **Stage B — Multi-Mode Codegen & Compilation (`mode=lib` and `mode=mod`)**:
//!    - Invokes `protoc_gen_roto::generate_request` on the parsed `CodeGeneratorRequestView`
//!      in both `mode=lib` (single-file `lib.rs` with inline nested package modules) and
//!      `mode=mod` (`mod.rs` plus per-package module files).
//!    - Writes both generated crate layouts to a temporary build directory and compiles both
//!      with `rustc --edition=2021 --crate-type=rlib -D warnings` against `roto` and
//!      `roto-runtime`, verifying zero compiler errors or lint warnings even in the presence
//!      of Rust keyword package segments (`type`, `mod`, `match`, `self`, `super`, `crate`,
//!      `struct`, `enum`, `fn`, `async`, `gen`) and keyword field identifiers.
//!
//! 3. **Stage C — Dynamic Data `unparse -> parse -> unparse` & Mutated Wire `parse` Fuzzing**:
//!    - Emits and compiles a standalone Rust test runner binary linked against the compiled
//!      random schema crate that exercises every generated top-level, nested, doubly-nested,
//!      and `group` message type across three sub-checks:
//!      1. **Empty & Default Value Verification**: Parses `&[]` with `<Msg>View::from_bytes(&[])`
//!         (asserting `Ok` when the message has no `required` fields and
//!         `Err(DecodeError::MissingRequiredField(_))` when `required` fields exist) and
//!         verifies that every unset `optional` field returns its exact expected `[default = ...]`
//!         or canonical zero/empty/first-variant default (using `.is_nan()` for `NaN` float
//!         defaults and `.to_bits()` for finite/`+/-Inf` floats).
//!      2. **Random Populated Data `unparse -> parse -> unparse`**: Generates 3 random populated
//!         instances per message covering all 18 `FieldType`s, `required`/`optional`/`repeated`
//!         (both packed and unpacked), `proto2` `group`s, nested messages, `proto3_optional`,
//!         real `oneof` variants, and injected unknown wire fields. Encodes each instance with
//!         `<Msg>Builder::new(&mut buf)` (`unparse`), decodes with `<Msg>View::from_bytes`
//!         (`parse`), asserts all `has_*`, scalar/slice/message/group accessors, repeated
//!         iterators, `oneof` enum accessors, and `unknown_fields()`, then re-encodes via both
//!         `copy_unset_fields` and `copy_unknown_fields` (`unparse`), re-parses (`parse`), and
//!         asserts bit-for-bit accessor equivalence.
//!      3. **Mutated Wire `parse` Safety**: Mutates valid encoded message buffers (all prefix
//!         truncations, single-byte bit-flips, appended tags, and random slices) and drives
//!         `<Msg>View::from_bytes` plus every generated accessor and bounded repeated iterator
//!         to guarantee panic-free decoding on arbitrary wire input.
//!
//! ## Configuration via Environment Variables
//! - `ROTO_FUZZ_SEED`: 64-bit PRNG seed in decimal or `0x` hex (default: `0x524F_544F_4445_5343`).
//! - `ROTO_FUZZ_BATCHES`: Number of compiled schema batches to execute (default: `2`).
//! - `ROTO_FUZZ_SCHEMAS_PER_BATCH`: Number of multi-file schemas per batch (default: `4`).
//!
//! ## Maintenance Guide
//! If the generated code or runtime contracts change, update:
//! - ` serialize_schema_request` if `google_protobuf_roto_bootstrap` builder signatures change.
//! - `RandomSchemaGenerator` if new descriptor options, field types, or identifier escaping
//!   conventions are added to `protoc_gen_roto::generator::ident` or `defaults`.
//! - `emit_message_test_fns` if the generated `*View<'a>` or `*Builder<'b>` method signatures
//!   (such as `from_bytes`, `has_*`, `write_*`, `write_packed_*`, `copy_unset_fields`,
//!   `copy_unknown_fields`, or `finish`) change.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use google_protobuf_roto_bootstrap::google::protobuf::{
    compiler::{CodeGeneratorRequestBuilder, CodeGeneratorRequestView, CodeGeneratorResponseView},
    field_descriptor_proto::{Label as FieldLabel, Type as FieldType},
    DescriptorProtoBuilder, EnumDescriptorProtoBuilder, FieldDescriptorProtoBuilder,
};
use protoc_gen_roto::generator::{
    defaults::{format_bytes_literal, format_string_literal, unescape_c_bytes},
    ident::{escape_ident, escape_mod_ident, sanitize_type_ident, to_pascal_case, to_snake_case},
};

const DEFAULT_SEED: u64 = 0x524F_544F_4445_5343; // "ROTODESC"
const DEFAULT_BATCHES: usize = 2;
const DEFAULT_SCHEMAS_PER_BATCH: usize = 4;

const ALL_FIELD_TYPES: [FieldType; 18] = [
    FieldType::TypeDouble,
    FieldType::TypeFloat,
    FieldType::TypeInt64,
    FieldType::TypeUint64,
    FieldType::TypeInt32,
    FieldType::TypeFixed64,
    FieldType::TypeFixed32,
    FieldType::TypeBool,
    FieldType::TypeString,
    FieldType::TypeGroup,
    FieldType::TypeMessage,
    FieldType::TypeBytes,
    FieldType::TypeUint32,
    FieldType::TypeEnum,
    FieldType::TypeSfixed32,
    FieldType::TypeSfixed64,
    FieldType::TypeSint32,
    FieldType::TypeSint64,
];

const KEYWORD_SEGMENTS: &[&str] = &[
    "type", "mod", "match", "self", "super", "crate", "struct", "enum", "fn", "async", "gen",
];

/// Fast, deterministic `SplitMix64` PRNG.
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
        let f: f32 = match self.gen_range(11) {
            0 => 0.0,
            1 => -0.0,
            2 => 1.5,
            3 => -2.25,
            4 => f32::MIN_POSITIVE,
            5 => f32::MAX,
            6 => f32::MIN,
            7 => f32::INFINITY,
            8 => f32::NEG_INFINITY,
            9 => f32::NAN,
            _ => f32::from_bits(0x7FC0_0000 | (self.next_u32() & 0x000F_FFFF).max(1)),
        };
        f.to_bits()
    }

    fn gen_f64_bits(&mut self) -> u64 {
        let f: f64 = match self.gen_range(11) {
            0 => 0.0,
            1 => -0.0,
            2 => 3.5,
            3 => -7.125,
            4 => f64::MIN_POSITIVE,
            5 => f64::MAX,
            6 => f64::MIN,
            7 => f64::INFINITY,
            8 => f64::NEG_INFINITY,
            9 => f64::NAN,
            _ => f64::from_bits(0x7FF8_0000_0000_0000 | (self.next_u64() & 0x0000_FFFF_FFFF).max(1)),
        };
        f.to_bits()
    }

    fn gen_string(&mut self, max_len: usize) -> String {
        let len = self.gen_range(max_len + 1);
        let mut s = String::with_capacity(len);
        const CHARS: &[char] = &[
            'a', 'b', 'z', '0', '9', '_', '-', '\n', '\r', '\t', '"', '\\', 'é', 'λ', '🦀',
        ];
        while s.len() < len {
            let ch = CHARS[self.gen_range(CHARS.len())];
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
        let mut out = vec![0u8; len];
        for b in &mut out {
            *b = (self.next_u64() & 0xFF) as u8;
        }
        out
    }

    fn gen_field_number(&mut self, used: &mut BTreeSet<i32>) -> i32 {
        loop {
            let num: i32 = match self.gen_range(6) {
                0 | 1 => 1 + (self.gen_range(15) as i32), // 1..=15 (1-byte tag)
                2 | 3 => 16 + (self.gen_range(2032) as i32), // 16..=2047 (2-byte tag)
                4 => 2048 + (self.gen_range(60_000) as i32), // medium tag
                _ => 1 + (self.gen_range(536_870_911) as i32), // up to max protobuf tag 2^29 - 1
            };
            if (19000..=19999).contains(&num) {
                continue;
            }
            if used.insert(num) {
                return num;
            }
        }
    }
}

fn read_fuzz_config() -> (u64, usize, usize) {
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

    let batches = std::env::var("ROTO_FUZZ_BATCHES")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(DEFAULT_BATCHES);

    let schemas_per_batch = std::env::var("ROTO_FUZZ_SCHEMAS_PER_BATCH")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(DEFAULT_SCHEMAS_PER_BATCH);

    (seed, batches, schemas_per_batch)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProtoSyntax {
    Proto2,
    Proto3,
}

impl ProtoSyntax {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Proto2 => "proto2",
            Self::Proto3 => "proto3",
        }
    }
}

#[derive(Clone, Debug)]
struct EnumVariantDef {
    proto_name: String,
    rust_ident: String,
    number: i32,
    is_primary: bool,
    primary_rust_ident: String,
}

#[derive(Clone, Debug)]
struct EnumDef {
    name: String,
    fq_proto_name: String,
    rust_path: String,
    allow_alias: bool,
    variants: Vec<EnumVariantDef>,
}

#[derive(Clone, Debug)]
enum DefaultVal {
    I32 { proto_str: String, val: i32 },
    I64 { proto_str: String, val: i64 },
    U32 { proto_str: String, val: u32 },
    U64 { proto_str: String, val: u64 },
    F32 { proto_str: String, val: f32 },
    F64 { proto_str: String, val: f64 },
    Bool { proto_str: String, val: bool },
    Str { proto_str: String, val: String },
    Bytes { proto_str: String, val: Vec<u8> },
    Enum { proto_str: String, variant_rust_ident: String },
    MessageOrGroup,
}

impl DefaultVal {
    fn proto_default_str(&self) -> &str {
        match self {
            Self::I32 { proto_str, .. }
            | Self::I64 { proto_str, .. }
            | Self::U32 { proto_str, .. }
            | Self::U64 { proto_str, .. }
            | Self::F32 { proto_str, .. }
            | Self::F64 { proto_str, .. }
            | Self::Bool { proto_str, .. }
            | Self::Str { proto_str, .. }
            | Self::Bytes { proto_str, .. }
            | Self::Enum { proto_str, .. } => proto_str,
            Self::MessageOrGroup => "",
        }
    }
}

#[derive(Clone, Debug)]
struct FieldDef {
    name: String,
    accessor_ident: String,
    has_ident: String,
    write_ident: String,
    write_packed_ident: Option<String>,
    number: i32,
    label: FieldLabel,
    field_type: FieldType,
    type_name_for_descriptor: String,
    target_fq_name: String,
    explicit_default: Option<DefaultVal>,
    effective_default: DefaultVal,
    oneof_index: Option<i32>,
    proto3_optional: bool,
    packed_option: Option<bool>,
}

#[derive(Clone, Debug)]
struct OneofDef {
    name: String,
    accessor_ident: String,
    enum_rust_path: String,
    is_synthetic_proto3_optional: bool,
    field_numbers: Vec<i32>,
}

#[derive(Clone, Debug)]
struct MessageDef {
    name: String,
    fn_slug: String,
    fq_proto_name: String,
    view_rust_path: String,
    builder_rust_path: String,
    nested_enums: Vec<EnumDef>,
    nested_messages: Vec<MessageDef>,
    oneofs: Vec<OneofDef>,
    fields: Vec<FieldDef>,
}

#[derive(Clone, Debug)]
struct FileDef {
    name: String,
    package: String,
    syntax: ProtoSyntax,
    dependencies: Vec<String>,
    enums: Vec<EnumDef>,
    messages: Vec<MessageDef>,
}

#[derive(Clone, Debug)]
struct RandomSchema {
    files: Vec<FileDef>,
    all_enums: Vec<EnumDef>,
    all_messages: Vec<MessageDef>,
}

/// Deterministic generator of valid multi-file, multi-package `proto2` and `proto3` schemas.
struct RandomSchemaGenerator<'a> {
    rng: &'a mut Rng,
    all_enums: Vec<EnumDef>,
    all_messages: Vec<MessageDef>,
}

impl<'a> RandomSchemaGenerator<'a> {
    fn new(rng: &'a mut Rng) -> Self {
        Self {
            rng,
            all_enums: Vec::new(),
            all_messages: Vec::new(),
        }
    }

    fn generate_schema(&mut self, schema_idx: usize) -> RandomSchema {
        self.all_enums.clear();
        self.all_messages.clear();

        let pkg0 = self.gen_package_path(schema_idx, 0);
        let pkg1 = self.gen_package_path(schema_idx, 1);
        let pkg2 = self.gen_package_path(schema_idx, 2);

        let file0_name = format!("s{schema_idx}/base_proto2.proto");
        let file1_name = format!("s{schema_idx}/ext_proto2.proto");
        let file2_name = format!("s{schema_idx}/app_proto3.proto");

        let file0 = self.gen_file(
            schema_idx,
            0,
            &file0_name,
            &pkg0,
            ProtoSyntax::Proto2,
            Vec::new(),
        );
        let file1 = self.gen_file(
            schema_idx,
            1,
            &file1_name,
            &pkg1,
            ProtoSyntax::Proto2,
            vec![file0_name.clone()],
        );
        let file2 = self.gen_file(
            schema_idx,
            2,
            &file2_name,
            &pkg2,
            ProtoSyntax::Proto3,
            vec![file0_name, file1_name],
        );

        RandomSchema {
            files: vec![file0, file1, file2],
            all_enums: self.all_enums.clone(),
            all_messages: self.all_messages.clone(),
        }
    }

    fn gen_package_path(&mut self, schema_idx: usize, file_idx: usize) -> String {
        let seg_count = 1 + self.rng.gen_range(3); // 1..=3 segments
        let mut segs = Vec::with_capacity(seg_count);
        // Ensure every file in the batch has a distinct package path while heavily
        // exercising Rust keyword segments (`type`, `mod`, `match`, `self`, `super`, `crate`, etc.).
        let first_seg = format!("s{schema_idx}_f{file_idx}");
        segs.push(first_seg);

        for i in 1..seg_count {
            if self.rng.next_bool() {
                let kw = KEYWORD_SEGMENTS[self.rng.gen_range(KEYWORD_SEGMENTS.len())];
                segs.push(kw.to_string());
            } else {
                segs.push(format!("p{i}_{}", self.rng.gen_range(8)));
            }
        }
        segs.join(".")
    }

    fn package_rust_path(package: &str) -> String {
        let mut parts = vec!["fuzz_schema".to_string()];
        for seg in package.split('.').filter(|s| !s.is_empty()) {
            parts.push(escape_mod_ident(seg));
        }
        parts.join("::")
    }

    fn gen_file(
        &mut self,
        schema_idx: usize,
        file_idx: usize,
        file_name: &str,
        package: &str,
        syntax: ProtoSyntax,
        dependencies: Vec<String>,
    ) -> FileDef {
        let fq_prefix = format!(".{package}");
        let mod_path = Self::package_rust_path(package);

        let enum_count = 1 + self.rng.gen_range(2);
        let mut enums = Vec::with_capacity(enum_count);
        for e_idx in 0..enum_count {
            let name = format!("EnumS{schema_idx}F{file_idx}E{e_idx}");
            let allow_alias = e_idx % 2 == 1 || self.rng.next_bool();
            let en = self.gen_enum(&name, &fq_prefix, &mod_path, syntax, allow_alias);
            self.all_enums.push(en.clone());
            enums.push(en);
        }

        let msg_count = 2;
        let mut messages = Vec::with_capacity(msg_count);
        for m_idx in 0..msg_count {
            let name = format!("MsgS{schema_idx}F{file_idx}M{m_idx}");
            let fn_slug = format!("s{schema_idx}_f{file_idx}_m{m_idx}");
            let include_all_18_types = file_idx == 1 && m_idx == 1;
            let msg = self.gen_message(
                &name,
                &fn_slug,
                &fq_prefix,
                &mod_path,
                syntax,
                0,
                include_all_18_types,
            );
            self.all_messages.push(msg.clone());
            messages.push(msg);
        }

        FileDef {
            name: file_name.to_string(),
            package: package.to_string(),
            syntax,
            dependencies,
            enums,
            messages,
        }
    }

    fn gen_enum(
        &mut self,
        name: &str,
        parent_fq_scope: &str,
        parent_mod_path: &str,
        syntax: ProtoSyntax,
        allow_alias: bool,
    ) -> EnumDef {
        let fq_proto_name = format!("{parent_fq_scope}.{name}");
        let rust_ident = sanitize_type_ident(name);
        let rust_path = format!("{parent_mod_path}::{rust_ident}");

        let primary_count = 2 + self.rng.gen_range(3); // 2..=4 primary variants
        let mut used_nums = BTreeSet::new();
        let mut variants = Vec::new();

        for v_idx in 0..primary_count {
            let number = if v_idx == 0 && syntax == ProtoSyntax::Proto3 {
                used_nums.insert(0);
                0
            } else if v_idx == 0 && syntax == ProtoSyntax::Proto2 && self.rng.next_bool() {
                // Exercise non-zero and negative first enum values in proto2
                let n = if self.rng.next_bool() {
                    -(1 + (self.rng.gen_range(50) as i32))
                } else {
                    1 + (self.rng.gen_range(50) as i32)
                };
                used_nums.insert(n);
                n
            } else {
                loop {
                    let cand = if self.rng.gen_range(3) == 0 {
                        -(1 + (self.rng.gen_range(200) as i32))
                    } else {
                        self.rng.gen_range(200) as i32
                    };
                    if used_nums.insert(cand) {
                        break cand;
                    }
                }
            };

            let proto_name = format!("{}_{v_idx}", name.to_ascii_uppercase());
            let v_rust = escape_ident(&to_pascal_case(&proto_name));
            variants.push(EnumVariantDef {
                proto_name,
                rust_ident: v_rust.clone(),
                number,
                is_primary: true,
                primary_rust_ident: v_rust,
            });
        }

        if allow_alias {
            let alias_target_idx = self.rng.gen_range(variants.len());
            let target_num = variants[alias_target_idx].number;
            let primary_rust = variants[alias_target_idx].primary_rust_ident.clone();
            let proto_name = format!("{}_ALIAS", name.to_ascii_uppercase());
            let v_rust = escape_ident(&to_pascal_case(&proto_name));
            variants.push(EnumVariantDef {
                proto_name,
                rust_ident: v_rust,
                number: target_num,
                is_primary: false,
                primary_rust_ident: primary_rust,
            });
        }

        EnumDef {
            name: name.to_string(),
            fq_proto_name,
            rust_path,
            allow_alias,
            variants,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn gen_message(
        &mut self,
        name: &str,
        fn_slug: &str,
        parent_fq_scope: &str,
        parent_mod_path: &str,
        syntax: ProtoSyntax,
        depth: usize,
        include_all_18_types: bool,
    ) -> MessageDef {
        let fq_proto_name = format!("{parent_fq_scope}.{name}");
        let rust_ident = sanitize_type_ident(name);
        let view_rust_path = format!("{parent_mod_path}::{rust_ident}View");
        let builder_rust_path = format!("{parent_mod_path}::{rust_ident}Builder");
        let child_mod_path = format!("{parent_mod_path}::{}", escape_mod_ident(name));

        let mut nested_enums = Vec::new();
        let mut nested_messages = Vec::new();

        // Generate nested enums and nested messages up to depth 2 BEFORE generating this
        // message's fields so that this message can acyclically reference its own nested types.
        if depth < 2 {
            let nested_en_name = format!("{name}NestedEnum");
            let nested_allow_alias = self.rng.next_bool();
            let nested_en = self.gen_enum(
                &nested_en_name,
                &fq_proto_name,
                &child_mod_path,
                syntax,
                nested_allow_alias,
            );
            self.all_enums.push(nested_en.clone());
            nested_enums.push(nested_en);

            if depth == 0 || self.rng.next_bool() {
                let nested_msg_name = format!("{name}Sub");
                let nested_slug = format!("{fn_slug}_sub");
                let nested_msg = self.gen_message(
                    &nested_msg_name,
                    &nested_slug,
                    &fq_proto_name,
                    &child_mod_path,
                    syntax,
                    depth + 1,
                    false,
                );
                self.all_messages.push(nested_msg.clone());
                nested_messages.push(nested_msg);
            }
        }

        let mut used_tags = BTreeSet::new();
        let mut used_field_snakes = BTreeSet::new();
        let mut fields = Vec::new();
        let mut oneofs = Vec::new();
        let mut group_counter = 0usize;

        // Choose field types for regular (non-oneof) fields
        let mut planned_types: Vec<(FieldType, FieldLabel)> = Vec::new();
        if include_all_18_types {
            for &ft in &ALL_FIELD_TYPES {
                if ft == FieldType::TypeGroup && syntax != ProtoSyntax::Proto2 {
                    continue;
                }
                planned_types.push((ft, FieldLabel::LabelOptional));
                planned_types.push((ft, FieldLabel::LabelRepeated));
            }
            if syntax == ProtoSyntax::Proto2 {
                planned_types.push((FieldType::TypeUint32, FieldLabel::LabelRequired));
                planned_types.push((FieldType::TypeString, FieldLabel::LabelRequired));
            }
        } else {
            let field_count = if depth == 2 {
                2 + self.rng.gen_range(3)
            } else {
                4 + self.rng.gen_range(5)
            };
            for i in 0..field_count {
                let ft = loop {
                    let cand = ALL_FIELD_TYPES[self.rng.gen_range(ALL_FIELD_TYPES.len())];
                    if cand == FieldType::TypeGroup && (syntax != ProtoSyntax::Proto2 || depth >= 2)
                    {
                        continue;
                    }
                    if cand == FieldType::TypeMessage && self.all_messages.is_empty() {
                        continue;
                    }
                    break cand;
                };
                let label = match syntax {
                    ProtoSyntax::Proto2 => match self.rng.gen_range(5) {
                        0 if i == 0 => FieldLabel::LabelRequired,
                        1 | 2 => FieldLabel::LabelRepeated,
                        _ => FieldLabel::LabelOptional,
                    },
                    ProtoSyntax::Proto3 => {
                        if self.rng.gen_range(3) == 0 {
                            FieldLabel::LabelRepeated
                        } else {
                            FieldLabel::LabelOptional
                        }
                    }
                };
                planned_types.push((ft, label));
            }
        }

        for (idx, (ft, label)) in planned_types.into_iter().enumerate() {
            let raw_name = if idx < KEYWORD_SEGMENTS.len() && self.rng.gen_range(3) == 0 {
                let kw = KEYWORD_SEGMENTS[idx];
                if used_field_snakes.contains(kw) {
                    format!("f_{idx}_{}", self.rng.gen_range(100))
                } else {
                    kw.to_string()
                }
            } else if label == FieldLabel::LabelRepeated {
                format!("rep_{idx}_items")
            } else {
                format!("fld_{idx}")
            };

            let snake = to_snake_case(&raw_name);
            if !used_field_snakes.insert(snake) {
                continue;
            }

            let tag = self.rng.gen_field_number(&mut used_tags);
            let field_def = self.build_field_def(
                &raw_name,
                tag,
                label,
                ft,
                syntax,
                &fq_proto_name,
                &child_mod_path,
                fn_slug,
                &mut group_counter,
                &mut nested_messages,
                None,
                false,
            );
            fields.push(field_def);
        }

        // In proto3, add a `proto3_optional` field backed by a synthetic `_opt_*` oneof
        if syntax == ProtoSyntax::Proto3 && depth < 2 {
            let oneof_idx = oneofs.len() as i32;
            let opt_field_name = "p3_opt_val";
            if used_field_snakes.insert(opt_field_name.to_string()) {
                let tag = self.rng.gen_field_number(&mut used_tags);
                let opt_ft = if self.rng.next_bool() {
                    FieldType::TypeSint64
                } else {
                    FieldType::TypeString
                };
                let f_def = self.build_field_def(
                    opt_field_name,
                    tag,
                    FieldLabel::LabelOptional,
                    opt_ft,
                    syntax,
                    &fq_proto_name,
                    &child_mod_path,
                    fn_slug,
                    &mut group_counter,
                    &mut nested_messages,
                    Some(oneof_idx),
                    true,
                );
                fields.push(f_def);
                oneofs.push(OneofDef {
                    name: format!("_{opt_field_name}"),
                    accessor_ident: String::new(),
                    enum_rust_path: String::new(),
                    is_synthetic_proto3_optional: true,
                    field_numbers: vec![tag],
                });
            }
        }

        // Add a real `oneof` declaration with 2 to 3 member variants on depth 0/1 messages
        if depth < 2 {
            let oneof_idx = oneofs.len() as i32;
            let oneof_name = "choice_oneof";
            let oneof_enum_ident = sanitize_type_ident(oneof_name);
            let oneof_enum_path = format!("{child_mod_path}::{oneof_enum_ident}");
            let variant_count = 2 + self.rng.gen_range(2);
            let mut member_tags = Vec::with_capacity(variant_count);

            for v_i in 0..variant_count {
                let v_name = format!("oo_var_{v_i}");
                if !used_field_snakes.insert(v_name.clone()) {
                    continue;
                }
                let v_ft = match v_i {
                    0 => FieldType::TypeString,
                    1 => {
                        if !self.all_messages.is_empty() && self.rng.next_bool() {
                            FieldType::TypeMessage
                        } else {
                            FieldType::TypeSint32
                        }
                    }
                    _ => FieldType::TypeEnum,
                };
                let tag = self.rng.gen_field_number(&mut used_tags);
                let f_def = self.build_field_def(
                    &v_name,
                    tag,
                    FieldLabel::LabelOptional,
                    v_ft,
                    syntax,
                    &fq_proto_name,
                    &child_mod_path,
                    fn_slug,
                    &mut group_counter,
                    &mut nested_messages,
                    Some(oneof_idx),
                    false,
                );
                fields.push(f_def);
                member_tags.push(tag);
            }

            if !member_tags.is_empty() {
                oneofs.push(OneofDef {
                    name: oneof_name.to_string(),
                    accessor_ident: escape_ident(&to_snake_case(oneof_name)),
                    enum_rust_path: oneof_enum_path,
                    is_synthetic_proto3_optional: false,
                    field_numbers: member_tags,
                });
            }
        }

        MessageDef {
            name: name.to_string(),
            fn_slug: fn_slug.to_string(),
            fq_proto_name,
            view_rust_path,
            builder_rust_path,
            nested_enums,
            nested_messages,
            oneofs,
            fields,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_field_def(
        &mut self,
        raw_name: &str,
        tag: i32,
        label: FieldLabel,
        field_type: FieldType,
        syntax: ProtoSyntax,
        msg_fq_scope: &str,
        child_mod_path: &str,
        msg_fn_slug: &str,
        group_counter: &mut usize,
        nested_messages: &mut Vec<MessageDef>,
        oneof_index: Option<i32>,
        proto3_optional: bool,
    ) -> FieldDef {
        let snake = to_snake_case(raw_name);
        let mut accessor_ident = escape_ident(&snake);
        if matches!(
            accessor_ident.as_str(),
            "from_bytes" | "as_bytes" | "unknown_fields"
        ) {
            accessor_ident.push('_');
        }
        let has_ident = format!("has_{snake}");
        let write_ident = format!("write_{snake}");

        let is_packable = !matches!(
            field_type,
            FieldType::TypeString
                | FieldType::TypeBytes
                | FieldType::TypeMessage
                | FieldType::TypeGroup
        );
        let write_packed_ident = if label == FieldLabel::LabelRepeated && is_packable {
            Some(format!("write_packed_{snake}"))
        } else {
            None
        };

        let packed_option = if label == FieldLabel::LabelRepeated && is_packable {
            if self.rng.next_bool() {
                Some(self.rng.next_bool())
            } else {
                None
            }
        } else {
            None
        };

        let (type_name_for_descriptor, target_fq_name, chosen_enum) = match field_type {
            FieldType::TypeEnum => {
                let target_en = &self.all_enums[self.rng.gen_range(self.all_enums.len())];
                let fq = target_en.fq_proto_name.clone();
                // Mix relative type_name (when in the same scope) and fully-qualified `.pkg.Enum`
                let desc_tn = if let Some(rel) = fq.strip_prefix(&format!("{msg_fq_scope}.")) {
                    if self.rng.next_bool() {
                        rel.to_string()
                    } else {
                        fq.clone()
                    }
                } else {
                    fq.clone()
                };
                (desc_tn, fq, Some(target_en.clone()))
            }
            FieldType::TypeMessage => {
                let target_msg = &self.all_messages[self.rng.gen_range(self.all_messages.len())];
                let fq = target_msg.fq_proto_name.clone();
                let desc_tn = if let Some(rel) = fq.strip_prefix(&format!("{msg_fq_scope}.")) {
                    if self.rng.next_bool() {
                        rel.to_string()
                    } else {
                        fq.clone()
                    }
                } else {
                    fq.clone()
                };
                (desc_tn, fq, None)
            }
            FieldType::TypeGroup => {
                let g_idx = *group_counter;
                *group_counter += 1;
                let grp_name = format!("Grp{g_idx}");
                let grp_slug = format!("{msg_fn_slug}_grp{g_idx}");
                let grp_fq = format!("{msg_fq_scope}.{grp_name}");
                let grp_view_path = format!("{child_mod_path}::{grp_name}View");
                let grp_builder_path = format!("{child_mod_path}::{grp_name}Builder");

                // Build a simple 2-field group message
                let mut grp_tags = BTreeSet::new();
                let g_tag1 = self.rng.gen_field_number(&mut grp_tags);
                let g_tag2 = self.rng.gen_field_number(&mut grp_tags);
                let mut dummy_nested = Vec::new();
                let mut dummy_cnt = 0;
                let gf1 = self.build_field_def(
                    "g_val",
                    g_tag1,
                    FieldLabel::LabelOptional,
                    FieldType::TypeInt32,
                    ProtoSyntax::Proto2,
                    &grp_fq,
                    child_mod_path,
                    &grp_slug,
                    &mut dummy_cnt,
                    &mut dummy_nested,
                    None,
                    false,
                );
                let gf2 = self.build_field_def(
                    "g_label",
                    g_tag2,
                    FieldLabel::LabelOptional,
                    FieldType::TypeString,
                    ProtoSyntax::Proto2,
                    &grp_fq,
                    child_mod_path,
                    &grp_slug,
                    &mut dummy_cnt,
                    &mut dummy_nested,
                    None,
                    false,
                );
                let grp_def = MessageDef {
                    name: grp_name,
                    fn_slug: grp_slug,
                    fq_proto_name: grp_fq.clone(),
                    view_rust_path: grp_view_path,
                    builder_rust_path: grp_builder_path,
                    nested_enums: Vec::new(),
                    nested_messages: Vec::new(),
                    oneofs: Vec::new(),
                    fields: vec![gf1, gf2],
                };
                self.all_messages.push(grp_def.clone());
                nested_messages.push(grp_def);

                (grp_fq.clone(), grp_fq, None)
            }
            _ => (String::new(), String::new(), None),
        };

        let can_have_explicit_default = syntax == ProtoSyntax::Proto2
            && label == FieldLabel::LabelOptional
            && oneof_index.is_none()
            && !matches!(field_type, FieldType::TypeMessage | FieldType::TypeGroup);

        let (explicit_default, effective_default) =
            self.gen_field_defaults(field_type, can_have_explicit_default, chosen_enum.as_ref());

        FieldDef {
            name: raw_name.to_string(),
            accessor_ident,
            has_ident,
            write_ident,
            write_packed_ident,
            number: tag,
            label,
            field_type,
            type_name_for_descriptor,
            target_fq_name,
            explicit_default,
            effective_default,
            oneof_index,
            proto3_optional,
            packed_option,
        }
    }

    fn gen_field_defaults(
        &mut self,
        field_type: FieldType,
        allow_custom: bool,
        target_enum: Option<&EnumDef>,
    ) -> (Option<DefaultVal>, DefaultVal) {
        let use_custom = allow_custom && self.rng.next_bool();
        match field_type {
            FieldType::TypeInt32 | FieldType::TypeSint32 | FieldType::TypeSfixed32 => {
                if use_custom {
                    let val = self.rng.gen_i32();
                    let proto_str = if field_type == FieldType::TypeInt32
                        && val != i32::MIN
                        && self.rng.gen_range(3) == 0
                    {
                        if val < 0 {
                            format!("-0x{:x}", val.unsigned_abs())
                        } else {
                            format!("0x{val:x}")
                        }
                    } else {
                        val.to_string()
                    };
                    let d = DefaultVal::I32 { proto_str, val };
                    (Some(d.clone()), d)
                } else {
                    (
                        None,
                        DefaultVal::I32 {
                            proto_str: String::new(),
                            val: 0,
                        },
                    )
                }
            }
            FieldType::TypeInt64 | FieldType::TypeSint64 | FieldType::TypeSfixed64 => {
                if use_custom {
                    let val = self.rng.gen_i64();
                    let proto_str = val.to_string();
                    let d = DefaultVal::I64 { proto_str, val };
                    (Some(d.clone()), d)
                } else {
                    (
                        None,
                        DefaultVal::I64 {
                            proto_str: String::new(),
                            val: 0,
                        },
                    )
                }
            }
            FieldType::TypeUint32 | FieldType::TypeFixed32 => {
                if use_custom {
                    let val = self.rng.gen_u32();
                    let proto_str = if self.rng.gen_range(3) == 0 {
                        format!("0x{val:X}")
                    } else {
                        val.to_string()
                    };
                    let d = DefaultVal::U32 { proto_str, val };
                    (Some(d.clone()), d)
                } else {
                    (
                        None,
                        DefaultVal::U32 {
                            proto_str: String::new(),
                            val: 0,
                        },
                    )
                }
            }
            FieldType::TypeUint64 | FieldType::TypeFixed64 => {
                if use_custom {
                    let val = self.rng.gen_u64();
                    let proto_str = val.to_string();
                    let d = DefaultVal::U64 { proto_str, val };
                    (Some(d.clone()), d)
                } else {
                    (
                        None,
                        DefaultVal::U64 {
                            proto_str: String::new(),
                            val: 0,
                        },
                    )
                }
            }
            FieldType::TypeFloat => {
                if use_custom {
                    let (proto_str, val): (String, f32) = match self.rng.gen_range(6) {
                        0 => ("inf".to_string(), f32::INFINITY),
                        1 => ("-inf".to_string(), f32::NEG_INFINITY),
                        2 => ("nan".to_string(), f32::NAN),
                        3 => ("0".to_string(), 0.0),
                        4 => ("-3.25".to_string(), -3.25),
                        _ => ("12.5".to_string(), 12.5),
                    };
                    let d = DefaultVal::F32 { proto_str, val };
                    (Some(d.clone()), d)
                } else {
                    (
                        None,
                        DefaultVal::F32 {
                            proto_str: String::new(),
                            val: 0.0,
                        },
                    )
                }
            }
            FieldType::TypeDouble => {
                if use_custom {
                    let (proto_str, val): (String, f64) = match self.rng.gen_range(6) {
                        0 => ("infinity".to_string(), f64::INFINITY),
                        1 => ("-infinity".to_string(), f64::NEG_INFINITY),
                        2 => ("NaN".to_string(), f64::NAN),
                        3 => ("0".to_string(), 0.0),
                        4 => ("-128.75".to_string(), -128.75),
                        _ => ("1024.125".to_string(), 1024.125),
                    };
                    let d = DefaultVal::F64 { proto_str, val };
                    (Some(d.clone()), d)
                } else {
                    (
                        None,
                        DefaultVal::F64 {
                            proto_str: String::new(),
                            val: 0.0,
                        },
                    )
                }
            }
            FieldType::TypeBool => {
                if use_custom {
                    let val = self.rng.next_bool();
                    let proto_str = if val { "true" } else { "false" }.to_string();
                    let d = DefaultVal::Bool { proto_str, val };
                    (Some(d.clone()), d)
                } else {
                    (
                        None,
                        DefaultVal::Bool {
                            proto_str: String::new(),
                            val: false,
                        },
                    )
                }
            }
            FieldType::TypeString => {
                if use_custom {
                    let val = self.rng.gen_string(16);
                    let d = DefaultVal::Str {
                        proto_str: val.clone(),
                        val,
                    };
                    (Some(d.clone()), d)
                } else {
                    (
                        None,
                        DefaultVal::Str {
                            proto_str: String::new(),
                            val: String::new(),
                        },
                    )
                }
            }
            FieldType::TypeBytes => {
                if use_custom {
                    let val = self.rng.gen_bytes(16);
                    let proto_str = escape_c_bytes_for_descriptor(&val);
                    assert_eq!(
                        unescape_c_bytes(&proto_str).expect("valid C-escaped bytes"),
                        val
                    );
                    let d = DefaultVal::Bytes { proto_str, val };
                    (Some(d.clone()), d)
                } else {
                    (
                        None,
                        DefaultVal::Bytes {
                            proto_str: String::new(),
                            val: Vec::new(),
                        },
                    )
                }
            }
            FieldType::TypeEnum => {
                let en = target_enum.expect("enum field must have target EnumDef");
                if use_custom {
                    let chosen = &en.variants[self.rng.gen_range(en.variants.len())];
                    let d = DefaultVal::Enum {
                        proto_str: chosen.proto_name.clone(),
                        variant_rust_ident: chosen.rust_ident.clone(),
                    };
                    (Some(d.clone()), d)
                } else {
                    let first = &en.variants[0];
                    (
                        None,
                        DefaultVal::Enum {
                            proto_str: String::new(),
                            variant_rust_ident: first.rust_ident.clone(),
                        },
                    )
                }
            }
            FieldType::TypeMessage | FieldType::TypeGroup => {
                (None, DefaultVal::MessageOrGroup)
            }
        }
    }
}

/// Encodes raw bytes into a C-escaped `default_value` string matching `protoc` descriptor conventions.
fn escape_c_bytes_for_descriptor(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            0x07 => out.push_str("\\a"),
            0x08 => out.push_str("\\b"),
            0x0C => out.push_str("\\f"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            0x0B => out.push_str("\\v"),
            b'\\' => out.push_str("\\\\"),
            b'\'' => out.push_str("\\'"),
            b'"' => out.push_str("\\\""),
            b'A'..=b'Z' | b'a'..=b'z' | b'_' | b'-' => out.push(b as char),
            _ => {
                if i % 2 == 0 {
                    // 3-digit octal escape is always unambiguous regardless of following char
                    write!(out, "\\{b:03o}").unwrap();
                } else {
                    // 2-digit hex escape when not followed by an ASCII hex digit
                    let next_is_hex = bytes.get(i + 1).is_some_and(|n| n.is_ascii_hexdigit());
                    if next_is_hex {
                        write!(out, "\\{b:03o}").unwrap();
                    } else {
                        write!(out, "\\x{b:02x}").unwrap();
                    }
                }
            }
        }
    }
    out
}

// ============================================================================
// Stage A: Descriptor Serialization (Unparse) & Validation (Parse)
// ============================================================================

fn write_enum_descriptor(eb: &mut EnumDescriptorProtoBuilder<'_>, en: &EnumDef) {
    if en.allow_alias {
        eb.write_options(|opt| {
            opt.write_allow_alias(true);
        });
    }
    for v in en.variants.iter().rev() {
        eb.write_value(|vb| {
            vb.write_number(v.number).write_name(&v.proto_name);
        });
    }
    eb.write_name(&en.name);
}

fn write_field_descriptor(fb: &mut FieldDescriptorProtoBuilder<'_>, f: &FieldDef) {
    if let Some(packed) = f.packed_option {
        fb.write_options(|opt| {
            opt.write_packed(packed);
        });
    }
    if f.proto3_optional {
        fb.write_proto3_optional(true);
    }
    if let Some(oo_idx) = f.oneof_index {
        fb.write_oneof_index(oo_idx);
    }
    if let Some(ref def) = f.explicit_default {
        fb.write_default_value(def.proto_default_str());
    }
    if !f.type_name_for_descriptor.is_empty() {
        fb.write_type_name(&f.type_name_for_descriptor);
    }
    fb.write_type(f.field_type)
        .write_label(f.label)
        .write_number(f.number)
        .write_name(&f.name);
}

fn write_message_descriptor(mb: &mut DescriptorProtoBuilder<'_>, msg: &MessageDef) {
    for oneof in msg.oneofs.iter().rev() {
        mb.write_oneof_decl(|ob| {
            ob.write_name(&oneof.name);
        });
    }
    for en in msg.nested_enums.iter().rev() {
        mb.write_enum_type(|eb| {
            write_enum_descriptor(eb, en);
        });
    }
    for nested in msg.nested_messages.iter().rev() {
        mb.write_nested_type(|nb| {
            write_message_descriptor(nb, nested);
        });
    }
    for f in msg.fields.iter().rev() {
        mb.write_field(|fb| {
            write_field_descriptor(fb, f);
        });
    }
    mb.write_name(&msg.name);
}

fn serialize_schema_request<'b>(
    files: &[FileDef],
    parameter: Option<&str>,
    buf: &'b mut [u8],
) -> &'b [u8] {
    let mut req = CodeGeneratorRequestBuilder::new(buf);
    if let Some(param) = parameter {
        req.write_parameter(param);
    }
    for file in files.iter().rev() {
        req.write_proto_file(|fb| {
            for msg in file.messages.iter().rev() {
                fb.write_message_type(|mb| {
                    write_message_descriptor(mb, msg);
                });
            }
            for en in file.enums.iter().rev() {
                fb.write_enum_type(|eb| {
                    write_enum_descriptor(eb, en);
                });
            }
            for dep in file.dependencies.iter().rev() {
                fb.write_dependency(dep);
            }
            fb.write_syntax(file.syntax.as_str())
                .write_package(&file.package)
                .write_name(&file.name);
        });
    }
    for file in files.iter().rev() {
        req.write_file_to_generate(&file.name);
    }
    req.finish()
        .expect("CodeGeneratorRequestBuilder buffer overflow")
}

fn verify_descriptor_roundtrip_and_plugin_framing(files: &[FileDef]) {
    let mut req_buf = vec![0u8; 256 * 1024];
    let req_wire = serialize_schema_request(files, None, &mut req_buf);

    // 1. Parse with CodeGeneratorRequestView::from_bytes and validate
    let req_view = CodeGeneratorRequestView::from_bytes(req_wire)
        .expect("CodeGeneratorRequestView::from_bytes must succeed on valid descriptor");
    protoc_gen_roto::validate_request(&req_view)
        .expect("validate_request must succeed on valid random schema");

    let parsed_targets: Vec<&str> = req_view
        .file_to_generate()
        .map(|t| t.expect("valid file_to_generate entry"))
        .collect();
    let expected_targets: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(parsed_targets, expected_targets);

    let parsed_files: Vec<_> = req_view
        .proto_file()
        .map(|f| f.expect("valid FileDescriptorProto entry"))
        .collect();
    assert_eq!(parsed_files.len(), files.len());
    for (pf, ef) in parsed_files.iter().zip(files.iter()) {
        assert_eq!(pf.name().unwrap(), ef.name);
        assert_eq!(pf.package().unwrap(), ef.package);
        assert_eq!(pf.syntax().unwrap(), ef.syntax.as_str());
        assert_eq!(pf.enum_type().count(), ef.enums.len());
        assert_eq!(pf.message_type().count(), ef.messages.len());
    }

    // 2. Re-unparse via copy_unset_fields and assert byte-for-byte identity
    let mut copy_buf = vec![0u8; req_wire.len() + 1024];
    let mut copy_builder = CodeGeneratorRequestBuilder::new(&mut copy_buf);
    copy_builder.copy_unset_fields(&req_view);
    let copy_wire = copy_builder
        .finish()
        .expect("CodeGeneratorRequestBuilder::copy_unset_fields must fit");
    assert_eq!(copy_wire, req_wire);

    let copy_view = CodeGeneratorRequestView::from_bytes(copy_wire)
        .expect("re-parsed CodeGeneratorRequestView must succeed");
    protoc_gen_roto::validate_request(&copy_view)
        .expect("validate_request on copied CodeGeneratorRequestView must succeed");

    // 3. Exercise full plugin framing (`process_bytes` -> `CodeGeneratorResponseView`)
    let resp_wire =
        protoc_gen_roto::process_bytes(req_wire).expect("process_bytes must succeed on schema");
    let resp_view = CodeGeneratorResponseView::from_bytes(&resp_wire)
        .expect("CodeGeneratorResponseView::from_bytes must succeed");
    assert!(!resp_view.has_error(), "unexpected generator error");
    assert_eq!(resp_view.file().count(), 1);
}

// ============================================================================
// Stage C: Dynamic Rust Test Runner Code Emitter
// ============================================================================

#[derive(Clone, Debug)]
enum GeneratedValue {
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    F32Bits(u32),
    F64Bits(u64),
    Bool(bool),
    Str(String),
    Bytes(Vec<u8>),
    Enum {
        rust_path: String,
        variant_ident: String,
    },
    Message {
        fn_slug: String,
        inst_idx: usize,
    },
}

#[derive(Clone, Debug)]
struct GeneratedFieldInstance {
    field: FieldDef,
    present_value: Option<GeneratedValue>,
    repeated_values: Vec<GeneratedValue>,
    use_packed: bool,
}

#[derive(Clone, Debug)]
struct GeneratedOneofInstance {
    oneof: OneofDef,
    active_field_number: Option<i32>,
}

#[derive(Clone, Debug)]
struct GeneratedMessageInstance {
    fields: Vec<GeneratedFieldInstance>,
    oneofs: Vec<GeneratedOneofInstance>,
    unknown_varint: (u32, u64),
}

fn generate_message_instances(
    rng: &mut Rng,
    msg: &MessageDef,
    all_enums: &[EnumDef],
    all_messages: &[MessageDef],
) -> [GeneratedMessageInstance; 3] {
    std::array::from_fn(|inst_idx| {
        let mut active_oneof_tags = BTreeSet::new();
        let mut oneof_instances = Vec::new();

        for oo in &msg.oneofs {
            if oo.is_synthetic_proto3_optional || oo.field_numbers.is_empty() {
                continue;
            }
            // Instance 0 & 1: pick an active variant; Instance 2: either pick a variant or leave None
            let active_tag = if inst_idx < 2 || rng.next_bool() {
                let pick_idx = (inst_idx + rng.gen_range(oo.field_numbers.len()))
                    % oo.field_numbers.len();
                let tag = oo.field_numbers[pick_idx];
                active_oneof_tags.insert(tag);
                Some(tag)
            } else {
                None
            };
            oneof_instances.push(GeneratedOneofInstance {
                oneof: oo.clone(),
                active_field_number: active_tag,
            });
        }

        let real_oneof_all_tags: BTreeSet<i32> = msg
            .oneofs
            .iter()
            .filter(|o| !o.is_synthetic_proto3_optional)
            .flat_map(|o| o.field_numbers.iter().copied())
            .collect();

        let mut field_instances = Vec::with_capacity(msg.fields.len());
        for f in &msg.fields {
            if f.label == FieldLabel::LabelRepeated {
                let count = match inst_idx {
                    0 => 2 + rng.gen_range(2),
                    1 => 1 + rng.gen_range(3),
                    _ => rng.gen_range(3),
                };
                let mut vals = Vec::with_capacity(count);
                for _ in 0..count {
                    vals.push(gen_random_field_value(
                        rng,
                        f,
                        inst_idx,
                        all_enums,
                        all_messages,
                    ));
                }
                let use_packed = f.write_packed_ident.is_some() && (inst_idx % 2 == 0 || rng.next_bool());
                field_instances.push(GeneratedFieldInstance {
                    field: f.clone(),
                    present_value: None,
                    repeated_values: vals,
                    use_packed,
                });
            } else if real_oneof_all_tags.contains(&f.number) {
                let present_value = if active_oneof_tags.contains(&f.number) {
                    Some(gen_random_field_value(
                        rng,
                        f,
                        inst_idx,
                        all_enums,
                        all_messages,
                    ))
                } else {
                    None
                };
                field_instances.push(GeneratedFieldInstance {
                    field: f.clone(),
                    present_value,
                    repeated_values: Vec::new(),
                    use_packed: false,
                });
            } else {
                let is_present = f.label == FieldLabel::LabelRequired
                    || inst_idx == 0
                    || rng.next_bool();
                let present_value = if is_present {
                    Some(gen_random_field_value(
                        rng,
                        f,
                        inst_idx,
                        all_enums,
                        all_messages,
                    ))
                } else {
                    None
                };
                field_instances.push(GeneratedFieldInstance {
                    field: f.clone(),
                    present_value,
                    repeated_values: Vec::new(),
                    use_packed: false,
                });
            }
        }

        // Choose an unknown tag guaranteed not to collide with any schema field in `msg`
        let known_tags: BTreeSet<u32> = msg.fields.iter().map(|f| f.number as u32).collect();
        let mut unk_tag = 20_000 + (rng.gen_range(5_000) as u32);
        while known_tags.contains(&unk_tag) {
            unk_tag += 1;
        }
        let unk_val = rng.gen_u64();

        GeneratedMessageInstance {
            fields: field_instances,
            oneofs: oneof_instances,
            unknown_varint: (unk_tag, unk_val),
        }
    })
}

fn gen_random_field_value(
    rng: &mut Rng,
    f: &FieldDef,
    inst_idx: usize,
    all_enums: &[EnumDef],
    all_messages: &[MessageDef],
) -> GeneratedValue {
    match f.field_type {
        FieldType::TypeInt32 | FieldType::TypeSint32 | FieldType::TypeSfixed32 => {
            GeneratedValue::I32(rng.gen_i32())
        }
        FieldType::TypeInt64 | FieldType::TypeSint64 | FieldType::TypeSfixed64 => {
            GeneratedValue::I64(rng.gen_i64())
        }
        FieldType::TypeUint32 | FieldType::TypeFixed32 => GeneratedValue::U32(rng.gen_u32()),
        FieldType::TypeUint64 | FieldType::TypeFixed64 => GeneratedValue::U64(rng.gen_u64()),
        FieldType::TypeFloat => GeneratedValue::F32Bits(rng.gen_f32_bits()),
        FieldType::TypeDouble => GeneratedValue::F64Bits(rng.gen_f64_bits()),
        FieldType::TypeBool => GeneratedValue::Bool(rng.next_bool()),
        FieldType::TypeString => GeneratedValue::Str(rng.gen_string(18)),
        FieldType::TypeBytes => GeneratedValue::Bytes(rng.gen_bytes(18)),
        FieldType::TypeEnum => {
            let en = all_enums
                .iter()
                .find(|e| e.fq_proto_name == f.target_fq_name)
                .expect("target EnumDef must exist");
            let chosen = &en.variants[rng.gen_range(en.variants.len())];
            GeneratedValue::Enum {
                rust_path: en.rust_path.clone(),
                variant_ident: chosen.rust_ident.clone(),
            }
        }
        FieldType::TypeMessage | FieldType::TypeGroup => {
            let target_msg = all_messages
                .iter()
                .find(|m| m.fq_proto_name == f.target_fq_name)
                .expect("target MessageDef must exist");
            GeneratedValue::Message {
                fn_slug: target_msg.fn_slug.clone(),
                inst_idx: inst_idx % 3,
            }
        }
    }
}

fn value_to_rust_expr(v: &GeneratedValue) -> String {
    match v {
        GeneratedValue::I32(n) => format!("{n}i32"),
        GeneratedValue::I64(n) => format!("{n}i64"),
        GeneratedValue::U32(n) => format!("{n}u32"),
        GeneratedValue::U64(n) => format!("{n}u64"),
        GeneratedValue::F32Bits(bits) => format!("f32::from_bits(0x{bits:08x}u32)"),
        GeneratedValue::F64Bits(bits) => format!("f64::from_bits(0x{bits:016x}u64)"),
        GeneratedValue::Bool(b) => format!("{b}"),
        GeneratedValue::Str(s) => format_string_literal(s),
        GeneratedValue::Bytes(b) => format_bytes_literal(b),
        GeneratedValue::Enum {
            rust_path,
            variant_ident,
        } => format!("{rust_path}::{variant_ident}"),
        GeneratedValue::Message { .. } => unreachable!("composite message handled via closure"),
    }
}

fn emit_scalar_or_enum_assertion(
    out: &mut String,
    pad: &str,
    actual_expr: &str,
    expected: &GeneratedValue,
) {
    match expected {
        GeneratedValue::F32Bits(bits) => {
            writeln!(out, "{pad}assert_eq!(({actual_expr}).to_bits(), 0x{bits:08x}u32);").unwrap();
        }
        GeneratedValue::F64Bits(bits) => {
            writeln!(
                out,
                "{pad}assert_eq!(({actual_expr}).to_bits(), 0x{bits:016x}u64);"
            )
            .unwrap();
        }
        other => {
            let expr = value_to_rust_expr(other);
            writeln!(out, "{pad}assert_eq!({actual_expr}, {expr});").unwrap();
        }
    }
}

fn emit_default_assertion(
    out: &mut String,
    pad: &str,
    view_var: &str,
    f: &FieldDef,
    all_enums: &[EnumDef],
) {
    let acc = &f.accessor_ident;
    match &f.effective_default {
        DefaultVal::I32 { val, .. } => {
            writeln!(out, "{pad}assert_eq!({view_var}.{acc}()?, {val}i32);").unwrap();
        }
        DefaultVal::I64 { val, .. } => {
            writeln!(out, "{pad}assert_eq!({view_var}.{acc}()?, {val}i64);").unwrap();
        }
        DefaultVal::U32 { val, .. } => {
            writeln!(out, "{pad}assert_eq!({view_var}.{acc}()?, {val}u32);").unwrap();
        }
        DefaultVal::U64 { val, .. } => {
            writeln!(out, "{pad}assert_eq!({view_var}.{acc}()?, {val}u64);").unwrap();
        }
        DefaultVal::F32 { val, .. } => {
            if val.is_nan() {
                writeln!(out, "{pad}assert!({view_var}.{acc}()?.is_nan());").unwrap();
            } else {
                let bits = val.to_bits();
                writeln!(
                    out,
                    "{pad}assert_eq!({view_var}.{acc}()?.to_bits(), 0x{bits:08x}u32);"
                )
                .unwrap();
            }
        }
        DefaultVal::F64 { val, .. } => {
            if val.is_nan() {
                writeln!(out, "{pad}assert!({view_var}.{acc}()?.is_nan());").unwrap();
            } else {
                let bits = val.to_bits();
                writeln!(
                    out,
                    "{pad}assert_eq!({view_var}.{acc}()?.to_bits(), 0x{bits:016x}u64);"
                )
                .unwrap();
            }
        }
        DefaultVal::Bool { val, .. } => {
            writeln!(out, "{pad}assert_eq!({view_var}.{acc}()?, {val});").unwrap();
        }
        DefaultVal::Str { val, .. } => {
            let lit = format_string_literal(val);
            writeln!(out, "{pad}assert_eq!({view_var}.{acc}()?, {lit});").unwrap();
        }
        DefaultVal::Bytes { val, .. } => {
            let lit = format_bytes_literal(val);
            writeln!(out, "{pad}assert_eq!({view_var}.{acc}()?, {lit});").unwrap();
        }
        DefaultVal::Enum {
            variant_rust_ident, ..
        } => {
            let en = all_enums
                .iter()
                .find(|e| e.fq_proto_name == f.target_fq_name)
                .expect("target EnumDef");
            writeln!(
                out,
                "{pad}assert_eq!({view_var}.{acc}()?, {}::{variant_rust_ident});",
                en.rust_path
            )
            .unwrap();
        }
        DefaultVal::MessageOrGroup => {
            writeln!(out, "{pad}assert!({view_var}.{acc}()?.is_none());").unwrap();
        }
    }
}

fn emit_message_test_fns(
    out: &mut String,
    msg: &MessageDef,
    instances: &[GeneratedMessageInstance; 3],
    all_enums: &[EnumDef],
) {
    let slug = &msg.fn_slug;
    let view_ty = &msg.view_rust_path;
    let builder_ty = &msg.builder_rust_path;

    // 1. Minimal required-only writer (so parent messages or default checks can always build a valid message)
    writeln!(
        out,
        "fn write_{slug}_minimal(b: &mut {builder_ty}<'_>) {{"
    )
    .unwrap();
    let req_fields: Vec<&FieldDef> = msg
        .fields
        .iter()
        .filter(|f| f.label == FieldLabel::LabelRequired)
        .collect();
    if req_fields.is_empty() {
        writeln!(out, "    let _ = b;").unwrap();
    } else {
        for rf in req_fields.iter().rev() {
            let w_ident = &rf.write_ident;
            let inst0_field = instances[0]
                .fields
                .iter()
                .find(|fi| fi.field.number == rf.number)
                .unwrap();
            let val = inst0_field.present_value.as_ref().unwrap();
            if let GeneratedValue::Message {
                fn_slug: sub_slug, ..
            } = val
            {
                writeln!(out, "    b.{w_ident}(|sub| write_{sub_slug}_minimal(sub));").unwrap();
            } else {
                let expr = value_to_rust_expr(val);
                writeln!(out, "    b.{w_ident}({expr});").unwrap();
            }
        }
    }
    writeln!(out, "}}\n").unwrap();

    // 2. Per-instance writer (`write_<slug>_inst_<k>`) and verifier (`verify_<slug>_inst_<k>`)
    for (k, inst) in instances.iter().enumerate() {
        writeln!(
            out,
            "fn write_{slug}_inst_{k}(b: &mut {builder_ty}<'_>, include_unknown: bool) {{"
        )
        .unwrap();
        writeln!(out, "    if include_unknown {{").unwrap();
        writeln!(
            out,
            "        let _ = b.writer_mut().write_varint({}u64);",
            inst.unknown_varint.1
        )
        .unwrap();
        writeln!(
            out,
            "        let _ = b.writer_mut().write_tag({}, ::roto::runtime::WireType::Varint);",
            inst.unknown_varint.0
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();

        for fi in inst.fields.iter().rev() {
            let w_ident = &fi.field.write_ident;
            if fi.field.label == FieldLabel::LabelRepeated {
                if fi.repeated_values.is_empty() {
                    continue;
                }
                if fi.use_packed {
                    let wp_ident = fi.field.write_packed_ident.as_ref().unwrap();
                    let items: Vec<String> =
                        fi.repeated_values.iter().map(value_to_rust_expr).collect();
                    writeln!(out, "    b.{wp_ident}(&[{}]);", items.join(", ")).unwrap();
                } else {
                    for item in fi.repeated_values.iter().rev() {
                        if let GeneratedValue::Message {
                            fn_slug: sub_slug,
                            inst_idx: sub_k,
                        } = item
                        {
                            writeln!(
                                out,
                                "    b.{w_ident}(|sub| write_{sub_slug}_inst_{sub_k}(sub, false));"
                            )
                            .unwrap();
                        } else {
                            let expr = value_to_rust_expr(item);
                            writeln!(out, "    b.{w_ident}({expr});").unwrap();
                        }
                    }
                }
            } else if let Some(ref val) = fi.present_value {
                if let GeneratedValue::Message {
                    fn_slug: sub_slug,
                    inst_idx: sub_k,
                } = val
                {
                    writeln!(
                        out,
                        "    b.{w_ident}(|sub| write_{sub_slug}_inst_{sub_k}(sub, false));"
                    )
                    .unwrap();
                } else {
                    let expr = value_to_rust_expr(val);
                    writeln!(out, "    b.{w_ident}({expr});").unwrap();
                }
            }
        }
        writeln!(out, "}}\n").unwrap();

        writeln!(
            out,
            "fn verify_{slug}_inst_{k}(v: &{view_ty}<'_>, expect_unknown: bool) -> Result<(), ::roto::runtime::DecodeError> {{"
        )
        .unwrap();
        writeln!(out, "    if expect_unknown {{").unwrap();
        writeln!(out, "        assert_eq!(v.unknown_fields().count(), 1);").unwrap();
        writeln!(out, "    }}").unwrap();

        for fi in &inst.fields {
            let f = &fi.field;
            let acc = &f.accessor_ident;
            let has = &f.has_ident;

            if f.label == FieldLabel::LabelRepeated {
                writeln!(out, "    {{").unwrap();
                writeln!(out, "        let mut items = Vec::new();").unwrap();
                writeln!(out, "        for item in v.{acc}() {{").unwrap();
                writeln!(out, "            items.push(item?);").unwrap();
                writeln!(out, "        }}").unwrap();
                writeln!(
                    out,
                    "        assert_eq!(items.len(), {});",
                    fi.repeated_values.len()
                )
                .unwrap();
                for (r_idx, r_val) in fi.repeated_values.iter().enumerate() {
                    if let GeneratedValue::Message {
                        fn_slug: sub_slug,
                        inst_idx: sub_k,
                    } = r_val
                    {
                        writeln!(
                            out,
                            "        verify_{sub_slug}_inst_{sub_k}(&items[{r_idx}], false)?;"
                        )
                        .unwrap();
                    } else {
                        emit_scalar_or_enum_assertion(
                            out,
                            "        ",
                            &format!("items[{r_idx}]"),
                            r_val,
                        );
                    }
                }
                writeln!(out, "    }}").unwrap();
            } else if let Some(ref val) = fi.present_value {
                writeln!(out, "    assert!(v.{has}());").unwrap();
                if f.label == FieldLabel::LabelRequired {
                    if let GeneratedValue::Message {
                        fn_slug: sub_slug,
                        inst_idx: sub_k,
                    } = val
                    {
                        writeln!(out, "    let sub_v = v.{acc}()?;").unwrap();
                        writeln!(out, "    verify_{sub_slug}_inst_{sub_k}(&sub_v, false)?;")
                            .unwrap();
                    } else {
                        let actual = format!("v.{acc}()?");
                        emit_scalar_or_enum_assertion(out, "    ", &actual, val);
                    }
                } else if let GeneratedValue::Message {
                    fn_slug: sub_slug,
                    inst_idx: sub_k,
                } = val
                {
                    writeln!(
                        out,
                        "    let sub_v = v.{acc}()?.expect(\"expected present message/group\");"
                    )
                    .unwrap();
                    writeln!(out, "    verify_{sub_slug}_inst_{sub_k}(&sub_v, false)?;").unwrap();
                } else {
                    let actual = format!("v.{acc}()?");
                    emit_scalar_or_enum_assertion(out, "    ", &actual, val);
                }
            } else {
                writeln!(out, "    assert!(!v.{has}());").unwrap();
                emit_default_assertion(out, "    ", "v", f, all_enums);
            }
        }

        // Verify real oneof enum accessors
        for oo_inst in &inst.oneofs {
            let oo = &oo_inst.oneof;
            let oo_acc = &oo.accessor_ident;
            let oo_enum_path = &oo.enum_rust_path;
            if let Some(active_tag) = oo_inst.active_field_number {
                let fi = inst
                    .fields
                    .iter()
                    .find(|f| f.field.number == active_tag)
                    .unwrap();
                let variant_ident = escape_ident(&to_pascal_case(&fi.field.name));
                let val = fi.present_value.as_ref().unwrap();
                writeln!(out, "    match v.{oo_acc}()? {{").unwrap();
                writeln!(
                    out,
                    "        Some({oo_enum_path}::{variant_ident}(inner)) => {{"
                )
                .unwrap();
                if let GeneratedValue::Message {
                    fn_slug: sub_slug,
                    inst_idx: sub_k,
                } = val
                {
                    writeln!(out, "            verify_{sub_slug}_inst_{sub_k}(&inner, false)?;")
                        .unwrap();
                } else {
                    emit_scalar_or_enum_assertion(out, "            ", "inner", val);
                }
                writeln!(out, "        }}").unwrap();
                writeln!(
                    out,
                    "        other => panic!(\"unexpected oneof value for {oo_acc}: {{other:?}}\"),"
                )
                .unwrap();
                writeln!(out, "    }}").unwrap();
            } else {
                writeln!(out, "    assert!(v.{oo_acc}()?.is_none());").unwrap();
            }
        }

        writeln!(out, "    Ok(())").unwrap();
        writeln!(out, "}}\n").unwrap();
    }

    // 3. Mutated wire safety exerciser (`exercise_<slug>_view`)
    writeln!(out, "fn exercise_{slug}_view(v: &{view_ty}<'_>) {{").unwrap();
    writeln!(out, "    let _ = v.as_bytes();").unwrap();
    writeln!(out, "    let _ = v.unknown_fields().take(32).count();").unwrap();
    for f in &msg.fields {
        let acc = &f.accessor_ident;
        let has = &f.has_ident;
        if f.label == FieldLabel::LabelRepeated {
            if matches!(f.field_type, FieldType::TypeMessage | FieldType::TypeGroup) {
                writeln!(out, "    for item in v.{acc}().take(16) {{").unwrap();
                writeln!(out, "        if let Ok(sub_v) = item {{").unwrap();
                writeln!(out, "            let _ = sub_v.as_bytes();").unwrap();
                writeln!(out, "        }}").unwrap();
                writeln!(out, "    }}").unwrap();
            } else {
                writeln!(out, "    for item in v.{acc}().take(32) {{").unwrap();
                writeln!(out, "        let _ = item;").unwrap();
                writeln!(out, "    }}").unwrap();
            }
        } else {
            writeln!(out, "    let _ = v.{has}();").unwrap();
            writeln!(out, "    let _ = v.{acc}();").unwrap();
        }
    }
    for oo in &msg.oneofs {
        if !oo.is_synthetic_proto3_optional && !oo.accessor_ident.is_empty() {
            let oo_acc = &oo.accessor_ident;
            writeln!(out, "    let _ = v.{oo_acc}();").unwrap();
        }
    }
    writeln!(out, "}}\n").unwrap();

    // 4. Master per-message test function (`test_<slug>`)
    writeln!(
        out,
        "fn test_{slug}(rng: &mut Rng) -> Result<(), Box<dyn std::error::Error>> {{"
    )
    .unwrap();

    // 4a. Empty & default value verification
    if req_fields.is_empty() {
        writeln!(out, "    let empty_v = {view_ty}::from_bytes(&[])?;").unwrap();
        for f in &msg.fields {
            if f.label == FieldLabel::LabelOptional {
                writeln!(out, "    assert!(!empty_v.{}());", f.has_ident).unwrap();
                emit_default_assertion(out, "    ", "empty_v", f, all_enums);
            } else if f.label == FieldLabel::LabelRepeated {
                writeln!(out, "    assert_eq!(empty_v.{}().count(), 0);", f.accessor_ident)
                    .unwrap();
            }
        }
        for oo in &msg.oneofs {
            if !oo.is_synthetic_proto3_optional && !oo.accessor_ident.is_empty() {
                writeln!(out, "    assert!(empty_v.{}()?.is_none());", oo.accessor_ident).unwrap();
            }
        }
    } else {
        writeln!(
            out,
            "    assert!(matches!({view_ty}::from_bytes(&[]), Err(::roto::runtime::DecodeError::MissingRequiredField(_))));"
        )
        .unwrap();
        writeln!(out, "    let mut min_buf = vec![0u8; 4096];").unwrap();
        writeln!(out, "    let mut min_b = {builder_ty}::new(&mut min_buf);").unwrap();
        writeln!(out, "    write_{slug}_minimal(&mut min_b);").unwrap();
        writeln!(out, "    let min_wire = min_b.finish()?;").unwrap();
        writeln!(out, "    let min_v = {view_ty}::from_bytes(min_wire)?;").unwrap();
        for f in &msg.fields {
            if f.label == FieldLabel::LabelRequired {
                writeln!(out, "    assert!(min_v.{}());", f.has_ident).unwrap();
            } else if f.label == FieldLabel::LabelOptional {
                writeln!(out, "    assert!(!min_v.{}());", f.has_ident).unwrap();
                emit_default_assertion(out, "    ", "min_v", f, all_enums);
            } else if f.label == FieldLabel::LabelRepeated {
                writeln!(out, "    assert_eq!(min_v.{}().count(), 0);", f.accessor_ident)
                    .unwrap();
            }
        }
    }

    // 4b. 3 Populated instances: unparse -> parse -> copy_unset_fields / copy_unknown_fields -> parse
    for k in 0..3 {
        writeln!(out, "    {{").unwrap();
        writeln!(out, "        let mut buf1 = vec![0u8; 16384];").unwrap();
        writeln!(out, "        let mut b1 = {builder_ty}::new(&mut buf1);").unwrap();
        writeln!(out, "        write_{slug}_inst_{k}(&mut b1, true);").unwrap();
        writeln!(out, "        b1.check()?;").unwrap();
        writeln!(out, "        let wire1 = b1.finish()?.to_vec();").unwrap();
        writeln!(out, "        let v1 = {view_ty}::from_bytes(&wire1)?;").unwrap();
        writeln!(out, "        verify_{slug}_inst_{k}(&v1, true)?;").unwrap();
        writeln!(out).unwrap();
        writeln!(out, "        let mut buf2 = vec![0u8; 16384];").unwrap();
        writeln!(out, "        let mut b2 = {builder_ty}::new(&mut buf2);").unwrap();
        writeln!(out, "        b2.copy_unset_fields(&v1);").unwrap();
        writeln!(out, "        let wire2 = b2.finish()?;").unwrap();
        writeln!(out, "        let v2 = {view_ty}::from_bytes(wire2)?;").unwrap();
        writeln!(out, "        verify_{slug}_inst_{k}(&v2, true)?;").unwrap();
        writeln!(out).unwrap();
        writeln!(out, "        let mut buf3 = vec![0u8; 16384];").unwrap();
        writeln!(out, "        let mut b3 = {builder_ty}::new(&mut buf3);").unwrap();
        writeln!(out, "        b3.copy_unknown_fields(&v1);").unwrap();
        writeln!(out, "        write_{slug}_inst_{k}(&mut b3, false);").unwrap();
        writeln!(out, "        let wire3 = b3.finish()?;").unwrap();
        writeln!(out, "        let v3 = {view_ty}::from_bytes(wire3)?;").unwrap();
        writeln!(out, "        verify_{slug}_inst_{k}(&v3, true)?;").unwrap();
        writeln!(out).unwrap();
        writeln!(
            out,
            "        fuzz_mutated_wire(rng, &wire1, |slice| {{"
        )
        .unwrap();
        writeln!(
            out,
            "            if let Ok(mv) = {view_ty}::from_bytes(slice) {{"
        )
        .unwrap();
        writeln!(out, "                exercise_{slug}_view(&mv);").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }});").unwrap();
        writeln!(out, "    }}").unwrap();
    }

    writeln!(out, "    Ok(())").unwrap();
    writeln!(out, "}}\n").unwrap();
}

fn emit_batch_runner_rs(
    seed: u64,
    all_enums: &[EnumDef],
    all_messages: &[MessageDef],
    rng: &mut Rng,
) -> String {
    let mut out = String::with_capacity(64 * 1024);
    writeln!(
        out,
        "#![allow(dead_code, non_snake_case, non_camel_case_types, non_upper_case_globals, unused_imports, clippy::all)]"
    )
    .unwrap();
    writeln!(out).unwrap();

    // Emit Rng & mutation helper in the runner binary
    writeln!(
        out,
        r#"struct Rng {{
    state: u64,
}}

impl Rng {{
    const fn new(seed: u64) -> Self {{
        Self {{ state: seed }}
    }}

    fn next_u64(&mut self) -> u64 {{
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }}

    fn gen_range(&mut self, upper: usize) -> usize {{
        if upper <= 1 {{
            0
        }} else {{
            (self.next_u64() as usize) % upper
        }}
    }}
}}

fn fuzz_mutated_wire<F: FnMut(&[u8])>(rng: &mut Rng, valid: &[u8], mut f: F) {{
    // 1. All prefix truncations
    for len in 0..=valid.len().min(64) {{
        f(&valid[..len]);
    }}
    // 2. Random bit flips, truncations, and random noise slices
    for _ in 0..24 {{
        let mut buf = valid.to_vec();
        match rng.gen_range(4) {{
            0 if !buf.is_empty() => {{
                let idx = rng.gen_range(buf.len());
                buf[idx] ^= 1u8 << (rng.gen_range(8) as u32);
            }}
            1 if !buf.is_empty() => {{
                let new_len = rng.gen_range(buf.len());
                buf.truncate(new_len);
            }}
            2 => {{
                buf.push((rng.next_u64() & 0xFF) as u8);
                buf.push((rng.next_u64() & 0xFF) as u8);
            }}
            _ => {{
                let len = rng.gen_range(48);
                buf.resize(len, 0);
                for b in &mut buf {{
                    *b = (rng.next_u64() & 0xFF) as u8;
                }}
            }}
        }}
        f(&buf);
    }}
}}
"#
    )
    .unwrap();

    // Pre-generate 3 random instances per message
    let mut msg_instances = Vec::with_capacity(all_messages.len());
    for msg in all_messages {
        let insts = generate_message_instances(rng, msg, all_enums, all_messages);
        msg_instances.push((msg, insts));
    }

    for (msg, insts) in &msg_instances {
        emit_message_test_fns(&mut out, msg, insts, all_enums);
    }

    writeln!(
        out,
        "fn main() -> Result<(), Box<dyn std::error::Error>> {{"
    )
    .unwrap();
    writeln!(out, "    let mut rng = Rng::new(0x{seed:016x}u64);").unwrap();

    // Verify enum discriminants, `from_i32`, `TryFrom<i32>`, and `allow_alias` constants
    for en in all_enums {
        let en_path = &en.rust_path;
        for v in &en.variants {
            let v_ident = &v.rust_ident;
            let num = v.number;
            if v.is_primary {
                writeln!(out, "    assert_eq!({en_path}::{v_ident}.as_i32(), {num});").unwrap();
                writeln!(
                    out,
                    "    assert_eq!({en_path}::from_i32({num}), Some({en_path}::{v_ident}));"
                )
                .unwrap();
            } else {
                let p_ident = &v.primary_rust_ident;
                writeln!(
                    out,
                    "    assert_eq!({en_path}::{v_ident}, {en_path}::{p_ident});"
                )
                .unwrap();
            }
        }
    }

    for (msg, _) in &msg_instances {
        writeln!(out, "    test_{}(&mut rng)?;", msg.fn_slug).unwrap();
    }

    writeln!(out, "    Ok(())").unwrap();
    writeln!(out, "}}").unwrap();
    out
}

// ============================================================================
// Build Helpers & Main Fuzz Test Entrypoint
// ============================================================================

fn create_temp_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "roto_fuzz_descriptors_{}_{}",
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

#[test]
fn fuzz_random_descriptors_codegen_and_data_roundtrip() {
    let (seed, batches, schemas_per_batch) = read_fuzz_config();
    let mut rng = Rng::new(seed);

    let tmp_dir = create_temp_dir("batches");
    let build_dir = tmp_dir.join("build");
    std::fs::create_dir_all(&build_dir).unwrap();
    let roto_rlib = build_roto_rlibs(&build_dir);

    for batch_idx in 0..batches {
        let mut batch_files = Vec::new();
        let mut batch_enums = Vec::new();
        let mut batch_messages = Vec::new();

        for s_idx in 0..schemas_per_batch {
            let global_schema_idx = batch_idx * schemas_per_batch + s_idx;
            let schema = {
                let mut gen = RandomSchemaGenerator::new(&mut rng);
                gen.generate_schema(global_schema_idx)
            };

            // Stage A on individual schema
            verify_descriptor_roundtrip_and_plugin_framing(&schema.files);

            batch_files.extend(schema.files);
            batch_enums.extend(schema.all_enums);
            batch_messages.extend(schema.all_messages);
        }

        // Stage A on combined batch request
        verify_descriptor_roundtrip_and_plugin_framing(&batch_files);

        // Stage B.1: Generate in `mode=lib` and compile with `rustc -D warnings`
        let mut lib_req_buf = vec![0u8; 512 * 1024];
        let lib_req_wire = serialize_schema_request(&batch_files, None, &mut lib_req_buf);
        let lib_req_view = CodeGeneratorRequestView::from_bytes(lib_req_wire).unwrap();
        let lib_generated = protoc_gen_roto::generate_request(&lib_req_view)
            .expect("generate_request (mode=lib) failed on batch");
        assert_eq!(lib_generated.len(), 1);
        assert_eq!(lib_generated[0].name, "lib.rs");

        let batch_lib_dir = tmp_dir.join(format!("batch_{batch_idx}_lib"));
        std::fs::create_dir_all(&batch_lib_dir).unwrap();
        let lib_rs_path = batch_lib_dir.join("lib.rs");
        std::fs::write(&lib_rs_path, &lib_generated[0].content).unwrap();

        let schema_lib_rlib = build_dir.join(format!("libfuzz_schema_{batch_idx}.rlib"));
        let compile_lib = Command::new("rustc")
            .arg("--edition=2021")
            .arg("--crate-type=rlib")
            .arg("--crate-name=fuzz_schema")
            .arg("-D")
            .arg("warnings")
            .arg(&lib_rs_path)
            .arg("--extern")
            .arg(format!("roto={}", roto_rlib.display()))
            .arg("-L")
            .arg(format!("dependency={}", build_dir.display()))
            .arg("-o")
            .arg(&schema_lib_rlib)
            .output()
            .expect("failed to invoke rustc on mode=lib crate");
        assert!(
            compile_lib.status.success(),
            "compiling mode=lib crate failed (seed=0x{seed:016x}, batch={batch_idx}):\n{}",
            String::from_utf8_lossy(&compile_lib.stderr)
        );

        // Stage B.2: Generate in `mode=mod` and compile with `rustc -D warnings`
        let mut mod_req_buf = vec![0u8; 512 * 1024];
        let mut mod_req_b = CodeGeneratorRequestBuilder::new(&mut mod_req_buf);
        mod_req_b
            .write_parameter("mode=mod")
            .copy_unset_fields(&lib_req_view);
        let mod_req_wire = mod_req_b.finish().unwrap();
        let mod_req_view = CodeGeneratorRequestView::from_bytes(mod_req_wire).unwrap();
        let mod_generated = protoc_gen_roto::generate_request(&mod_req_view)
            .expect("generate_request (mode=mod) failed on batch");
        assert!(!mod_generated.is_empty());
        assert_eq!(mod_generated[0].name, "mod.rs");

        let batch_mod_dir = tmp_dir.join(format!("batch_{batch_idx}_mod"));
        std::fs::create_dir_all(&batch_mod_dir).unwrap();
        for gf in &mod_generated {
            let out_path = batch_mod_dir.join(&gf.name);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(&out_path, &gf.content).unwrap();
        }
        let mod_root_rs = batch_mod_dir.join("lib_mod_root.rs");
        std::fs::write(
            &mod_root_rs,
            format!(
                "#![no_std]\n#![forbid(unsafe_code)]\n#![allow(clippy::all, dead_code, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_imports)]\n\n{}",
                mod_generated[0].content
            ),
        )
        .unwrap();

        let schema_mod_rlib = build_dir.join(format!("libfuzz_schema_mod_{batch_idx}.rlib"));
        let compile_mod = Command::new("rustc")
            .arg("--edition=2021")
            .arg("--crate-type=rlib")
            .arg("--crate-name=fuzz_schema_mod")
            .arg("-D")
            .arg("warnings")
            .arg(&mod_root_rs)
            .arg("--extern")
            .arg(format!("roto={}", roto_rlib.display()))
            .arg("-L")
            .arg(format!("dependency={}", build_dir.display()))
            .arg("-o")
            .arg(&schema_mod_rlib)
            .output()
            .expect("failed to invoke rustc on mode=mod crate");
        assert!(
            compile_mod.status.success(),
            "compiling mode=mod crate failed (seed=0x{seed:016x}, batch={batch_idx}):\n{}",
            String::from_utf8_lossy(&compile_mod.stderr)
        );

        // Stage C: Emit, compile, and execute dynamic data round-trip & wire mutation runner
        let runner_code = emit_batch_runner_rs(
            seed.wrapping_add(batch_idx as u64),
            &batch_enums,
            &batch_messages,
            &mut rng,
        );
        let runner_src = tmp_dir.join(format!("runner_{batch_idx}.rs"));
        std::fs::write(&runner_src, &runner_code).unwrap();

        let runner_bin = build_dir.join(format!("runner_{batch_idx}"));
        let compile_runner = Command::new("rustc")
            .arg("--edition=2021")
            .arg("-D")
            .arg("warnings")
            .arg(&runner_src)
            .arg("--extern")
            .arg(format!("roto={}", roto_rlib.display()))
            .arg("--extern")
            .arg(format!("fuzz_schema={}", schema_lib_rlib.display()))
            .arg("-L")
            .arg(format!("dependency={}", build_dir.display()))
            .arg("-o")
            .arg(&runner_bin)
            .output()
            .expect("failed to compile batch runner");
        assert!(
            compile_runner.status.success(),
            "compiling batch runner failed (seed=0x{seed:016x}, batch={batch_idx}):\n{}",
            String::from_utf8_lossy(&compile_runner.stderr)
        );

        let run_out = Command::new(&runner_bin)
            .output()
            .expect("failed to execute batch runner");
        assert!(
            run_out.status.success(),
            "batch runner failed (seed=0x{seed:016x}, batch={batch_idx}):\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run_out.stdout),
            String::from_utf8_lossy(&run_out.stderr)
        );
    }

    let _ = std::fs::remove_dir_all(&tmp_dir);
}
