//! End-to-end Rust keyword escaping and identifier collision test suite for `protoc-gen-roto`.
//!
//! This test suite serves as the canonical specification and verification of how `protoc-gen-roto`
//! translates Protobuf identifiers (packages, messages, nested modules, fields, enums, enum variants,
//! and `oneof` declarations) that collide with Rust keywords or internal `View`/`Builder` identifiers:
//!
//! 1. **Raw-Escapable Rust Keywords (`r#<kw>`)**:
//!    - All strict, reserved (2018/2021/2024), and weak keywords except `self`, `super`, `crate`, and `Self`
//!      are escaped with `r#` when used as module names (`escape_mod_ident`), field accessors (`escape_ident`),
//!      or variant/type identifiers:
//!      `as`, `break`, `const`, `continue`, `else`, `enum`, `extern`, `false`, `fn`, `for`, `if`, `impl`,
//!      `in`, `let`, `loop`, `match`, `mod`, `move`, `mut`, `pub`, `ref`, `return`, `static`, `struct`,
//!      `trait`, `true`, `type`, `unsafe`, `use`, `where`, `while`, `async`, `await`, `dyn`, `abstract`,
//!      `become`, `box`, `do`, `final`, `macro`, `override`, `priv`, `typeof`, `unsized`, `virtual`,
//!      `yield`, `try`, `gen`.
//!    - In `--roto_opt=mode=mod`, raw-escaped module declarations (`pub mod r#type;`) map to unprefixed
//!      filesystem paths (`type.rs`, `type/...`) as required by `rustc`.
//!
//! 2. **Non-Raw-Escapable Path/Self Keywords (`<kw>_`)**:
//!    - Rust forbids `r#self`, `r#super`, `r#crate`, and `r#Self`. These four identifiers are escaped by
//!      appending a trailing underscore: `self_`, `super_`, `crate_`, and `Self_`.
//!    - In `--roto_opt=mode=mod`, a package segment named `self`, `super`, or `crate` emits
//!      `pub mod self_;` / `pub mod super_;` / `pub mod crate_;` and writes `self_.rs` / `super_.rs` / `crate_.rs`.
//!    - A message or enum named `Self` becomes `Self_View<'a>` / `Self_Builder<'b>` / `pub enum Self_`,
//!      and its enclosing nested module becomes `pub mod self_`.
//!
//! 3. **Message, Enum, and Oneof Type/Variant Casing**:
//!    - Message names always receive `View` / `Builder` suffixes (`Struct` -> `StructView<'a>`, `StructBuilder<'b>`),
//!      while `Self` is sanitized to `Self_` before suffixing (`Self_View<'a>`, `Self_Builder<'b>`).
//!    - Enclosing message module names convert `PascalCase` to `snake_case` and then escape keywords
//!      (`Struct` -> `pub mod r#struct`, `Self` -> `pub mod self_`, `Super` -> `pub mod super_`, `Crate` -> `pub mod crate_`).
//!    - Enum and `oneof` type names and variants convert to `PascalCase` (`to_pascal_case`), so lowercase
//!      keywords like `type` or `self` become `Type` or `Self_` (`Self` -> `Self_`).
//!
//! 4. **Internal `View` and `Builder` Identifier Collision Avoidance**:
//!    - **View methods**: Fields named `from_bytes`, `as_bytes`, or `unknown_fields` have their value accessor
//!      suffixed with a trailing underscore (`from_bytes_()`, `as_bytes_()`, `unknown_fields_()`), while their
//!      presence accessors remain `has_from_bytes()`, `has_as_bytes()`, `has_unknown_fields()`.
//!    - **View struct markers & local variables**: Fields whose `snake_case` name collides with `buf`, `item`,
//!      `field`, or starts with `oneof_` or `f_` (or duplicates an earlier marker) use `f_<tag>` for their
//!      internal `FieldMarker` struct field while preserving their public accessor name (`buf()`, `item()`,
//!      `field()`, `oneof_0()`, `f_1()`).
//!    - **Oneof accessor collisions**: If a `oneof` name collides with a field accessor on the same message,
//!      the `oneof` accessor falls back to `<oneof_snake>_oneof()`.
//!    - **Builder methods**: All field writer methods are prefixed with `write_` (`write_<field_snake>`,
//!      `write_packed_<field_snake>`), preventing any collision with `new`, `from_writer`, `writer`,
//!      `writer_mut`, `check`, `finish`, `copy_unknown_fields`, `copy_unset_fields`, or `set_fields`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn create_temp_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "roto_rust_keywords_{}_{}",
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

/// File 1: Package path containing raw-escapable keywords (`type`, `mod`, `match`, `struct`,
/// `enum`, `fn`, `impl`, `trait`, `async`, `await`, `dyn`, `try`, `gen`).
/// Also defines keyword-named messages (`Struct`, `Type`, `Mod`, `Match`, `Enum`, `Fn`, `Impl`,
/// `Trait`, `Async`, `Dyn`, `Try`, `Gen`, `Self`, `Super`, `Crate`) and nested messages/enums inside them.
const KEYWORDS_PKG_RAW_PROTO: &str = r#"syntax = "proto2";

package type.mod.match.struct.enum.fn.impl.trait.async.await.dyn.try.gen;

message Struct {
    message Inner {
        optional int32 type = 1 [default = 10];
    }
    optional Inner match = 1;
}

message Type {
    message Nested {
        optional int32 fn = 1;
    }
    optional Nested mod = 1;
}

message Mod {
    message Nested {
        optional int32 impl = 1;
    }
    optional Nested trait = 1;
}

message Match {
    message Nested {
        optional int32 async = 1;
    }
    optional Nested await = 1;
}

message Enum {
    message Nested {
        optional int32 dyn = 1;
    }
    optional Nested try = 1;
}

message Fn {
    message Nested {
        optional int32 gen = 1;
    }
    optional Nested self = 1;
}

message Impl {
    message Nested {
        optional int32 super = 1;
    }
    optional Nested crate = 1;
}

message Trait {
    message Nested {
        optional int32 Self = 1;
    }
    optional Nested as = 1;
}

message Async {
    message Nested {
        optional int32 break = 1;
    }
    optional Nested const = 1;
}

message Dyn {
    message Nested {
        optional int32 continue = 1;
    }
    optional Nested else = 1;
}

message Try {
    message Nested {
        optional int32 extern = 1;
    }
    optional Nested false = 1;
}

message Gen {
    message Nested {
        optional int32 for = 1;
    }
    optional Nested if = 1;
}

message Self {
    message Self {
        optional int32 self = 1 [default = 42];
    }
    optional Self super = 1;
}

message Super {
    message Nested {
        optional int32 in = 1;
    }
    optional Nested let = 1;
}

message Crate {
    message Nested {
        optional int32 loop = 1;
    }
    optional Nested move = 1;
}

message KeywordEnums {
    message ScopedVariants {
        enum VariantKeywords {
            Self = 0;
            Super = 1;
            Crate = 2;
            Type = 3;
            Match = 4;
            Struct = 5;
            Async = 6;
            Gen = 7;
        }
    }

    message ScopedSelf {
        enum Self {
            SELF_ZERO = 0;
            SELF_ONE = 1;
        }
    }

    enum Super {
        SUPER_ZERO = 0;
        SUPER_ONE = 1;
    }

    enum Crate {
        CRATE_ZERO = 0;
        CRATE_ONE = 1;
    }

    enum Type {
        TYPE_ZERO = 0;
        TYPE_ONE = 1;
    }

    enum Match {
        MATCH_ZERO = 0;
        MATCH_ONE = 1;
    }

    enum Struct {
        STRUCT_ZERO = 0;
        STRUCT_ONE = 1;
    }

    enum Async {
        ASYNC_ZERO = 0;
        ASYNC_ONE = 1;
    }

    enum Gen {
        GEN_ZERO = 0;
        GEN_ONE = 1;
    }

    optional ScopedVariants.VariantKeywords kw_var = 1 [default = Type];
    optional ScopedSelf.Self kw_self = 2 [default = SELF_ONE];
    optional KeywordEnums.Super kw_super = 3 [default = SUPER_ONE];
    optional KeywordEnums.Crate kw_crate = 4 [default = CRATE_ONE];
    optional KeywordEnums.Type kw_type = 5 [default = TYPE_ONE];
    optional KeywordEnums.Match kw_match = 6 [default = MATCH_ONE];
    optional KeywordEnums.Struct kw_struct = 7 [default = STRUCT_ONE];
    optional KeywordEnums.Async kw_async = 8 [default = ASYNC_ONE];
    optional KeywordEnums.Gen kw_gen = 9 [default = GEN_ONE];
}
"#;

/// File 2: Package path with non-raw-escapable path keywords (`self`, `super`, `crate`),
/// testing `self_/super_/crate_.rs` generation in `mode=mod` and cross-package references,
/// plus a comprehensive message with EVERY Rust keyword (strict, reserved 2018/2021/2024, weak,
/// path) and internal View/Builder collision candidate as field names, plus keyword-named oneofs.
const KEYWORDS_PKG_PATH_PROTO: &str = r#"syntax = "proto2";

package self.super.crate;

import "kw_raw.proto";

message AllKeywordFields {
    // 1..=51: Every strict, reserved (2018/2021/2024), weak, and path Rust keyword
    optional int32 as = 1;
    optional int32 break = 2;
    optional int32 const = 3;
    optional int32 continue = 4;
    optional int32 else = 5;
    optional int32 enum = 6;
    optional int32 extern = 7;
    optional int32 false = 8;
    optional int32 fn = 9;
    optional int32 for = 10;
    optional int32 if = 11;
    optional int32 impl = 12;
    optional int32 in = 13;
    optional int32 let = 14;
    optional int32 loop = 15;
    optional int32 match = 16;
    optional int32 mod = 17;
    optional int32 move = 18;
    optional int32 mut = 19;
    optional int32 pub = 20;
    optional int32 ref = 21;
    optional int32 return = 22;
    optional int32 static = 23;
    optional int32 struct = 24;
    optional int32 trait = 25;
    optional int32 true = 26;
    optional int32 type = 27;
    optional int32 unsafe = 28;
    optional int32 use = 29;
    optional int32 where = 30;
    optional int32 while = 31;
    optional int32 async = 32;
    optional int32 await = 33;
    optional int32 dyn = 34;
    optional int32 abstract = 35;
    optional int32 become = 36;
    optional int32 box = 37;
    optional int32 do = 38;
    optional int32 final = 39;
    optional int32 macro = 40;
    optional int32 override = 41;
    optional int32 priv = 42;
    optional int32 typeof = 43;
    optional int32 unsized = 44;
    optional int32 virtual = 45;
    optional int32 yield = 46;
    optional int32 try = 47;
    optional int32 gen = 48;
    optional int32 self = 49;
    optional int32 super = 50;
    optional int32 crate = 51;

    // 52..=68: Internal View/Builder method, parameter, and marker collision candidates
    optional int32 from_bytes = 52;
    optional int32 as_bytes = 53;
    optional int32 unknown_fields = 54;
    optional int32 buf = 55;
    optional int32 item = 56;
    optional int32 field = 57;
    optional int32 writer = 58;
    optional int32 view = 59;
    optional int32 new = 60;
    optional int32 from_writer = 61;
    optional int32 check = 62;
    optional int32 finish = 63;
    optional int32 copy_unknown_fields = 64;
    optional int32 copy_unset_fields = 65;
    optional int32 set_fields = 66;
    optional int32 oneof_0 = 67;
    optional int32 f_1 = 68;

    // 69..=71: Cross-package references to keyword-named messages and enums
    optional .type.mod.match.struct.enum.fn.impl.trait.async.await.dyn.try.gen.Struct ref_struct = 69;
    optional .type.mod.match.struct.enum.fn.impl.trait.async.await.dyn.try.gen.Self ref_self_msg = 70;
    optional .type.mod.match.struct.enum.fn.impl.trait.async.await.dyn.try.gen.KeywordEnums.ScopedVariants.VariantKeywords ref_self_enum = 71 [default = Match];
}

message KeywordOneofs {
    // Oneofs named after keywords (`type`, `match`, `self`, `crate`, `super`, `struct`)
    // Note: on the same message, `type` oneof also collides with field `type` on `OneofCollision` below.
    oneof type {
        int32 o_type_int = 1;
        string o_type_str = 2;
    }

    oneof match {
        int32 o_match_int = 3;
        .type.mod.match.struct.enum.fn.impl.trait.async.await.dyn.try.gen.Struct o_match_struct = 4;
    }

    oneof self {
        int32 self_val = 5;
        string self_text = 6;
    }

    oneof crate {
        int32 crate_val = 7;
        bytes crate_bytes = 8;
    }

    oneof super {
        int32 super_val = 9;
        bool super_flag = 10;
    }

    oneof struct {
        int32 struct_val = 11;
        .type.mod.match.struct.enum.fn.impl.trait.async.await.dyn.try.gen.KeywordEnums.ScopedVariants.VariantKeywords struct_enum = 12;
    }
}

message OneofCollision {
    // Oneof whose name (`Type` / `Self` / `Payload`) collides after `to_snake_case` with a
    // field name (`type` / `self` / `payload`) on the same message, triggering the `<name>_oneof()` fallback.
    oneof Type {
        string type = 1;
        int32 match = 2;
    }

    oneof Self {
        int32 self = 3;
        string super = 4;
    }

    oneof Payload {
        string payload = 5;
        int32 code = 6;
    }
}
"#;

#[test]
fn rust_keywords_compile_and_round_trip_in_lib_and_mod_modes() {
    let plugin_bin = env!("CARGO_BIN_EXE_protoc-gen-roto");
    let tmp_dir = create_temp_dir("all_keywords");
    let proto_dir = tmp_dir.join("proto");
    let gen_lib_dir = tmp_dir.join("gen_lib");
    let gen_mod_dir = tmp_dir.join("gen_mod");
    let build_dir = tmp_dir.join("build");
    std::fs::create_dir_all(&proto_dir).unwrap();
    std::fs::create_dir_all(&gen_lib_dir).unwrap();
    std::fs::create_dir_all(&gen_mod_dir).unwrap();
    std::fs::create_dir_all(&build_dir).unwrap();

    std::fs::write(proto_dir.join("kw_raw.proto"), KEYWORDS_PKG_RAW_PROTO).unwrap();
    std::fs::write(proto_dir.join("kw_path.proto"), KEYWORDS_PKG_PATH_PROTO).unwrap();

    // 1. Run protoc in `mode=lib`
    let protoc_lib = Command::new("protoc")
        .arg(format!("--plugin=protoc-gen-roto={plugin_bin}"))
        .arg(format!("--proto_path={}", proto_dir.display()))
        .arg("--roto_opt=mode=lib")
        .arg(format!("--roto_out={}", gen_lib_dir.display()))
        .arg("kw_raw.proto")
        .arg("kw_path.proto")
        .output()
        .expect("failed to run protoc (mode=lib)");
    assert!(
        protoc_lib.status.success(),
        "protoc mode=lib failed: {}",
        String::from_utf8_lossy(&protoc_lib.stderr)
    );

    // 2. Run protoc in `mode=mod`
    let protoc_mod = Command::new("protoc")
        .arg(format!("--plugin=protoc-gen-roto={plugin_bin}"))
        .arg(format!("--proto_path={}", proto_dir.display()))
        .arg("--roto_opt=mode=mod")
        .arg(format!("--roto_out={}", gen_mod_dir.display()))
        .arg("kw_raw.proto")
        .arg("kw_path.proto")
        .output()
        .expect("failed to run protoc (mode=mod)");
    assert!(
        protoc_mod.status.success(),
        "protoc mode=mod failed: {}",
        String::from_utf8_lossy(&protoc_mod.stderr)
    );

    // Verify expected filesystem paths in `mode=mod`:
    // - Raw-escaped module declarations (`pub mod r#type { ... pub mod r#gen; }`) map to unprefixed directory/file names
    // - Non-raw-escapable path keywords (`self`, `super`, `crate`) map to `self_/super_/crate_.rs`
    assert!(gen_mod_dir.join("mod.rs").exists());
    assert!(
        gen_mod_dir
            .join("type/mod/match/struct/enum/fn/impl/trait/async/await/dyn/try/gen.rs")
            .exists(),
        "expected raw-escapable keyword package file in mode=mod"
    );
    assert!(
        gen_mod_dir.join("self_/super_/crate_.rs").exists(),
        "expected self_/super_/crate_.rs in mode=mod"
    );

    let mod_rs_content = std::fs::read_to_string(gen_mod_dir.join("mod.rs")).unwrap();
    assert!(
        mod_rs_content.contains("pub mod self_ {\n    pub mod super_ {\n        pub mod crate_;\n    }\n}"),
        "unexpected mod.rs content:\n{mod_rs_content}"
    );

    let roto_rlib = build_roto_rlibs(&build_dir);

    // 3. Compile `mode=mod` crate with `rustc -D warnings`
    let mod_root = gen_mod_dir.join("lib_mod_root.rs");
    std::fs::write(
        &mod_root,
        format!(
            "#![no_std]\n#![forbid(unsafe_code)]\n#![allow(clippy::all, dead_code, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_imports)]\n\n{mod_rs_content}"
        ),
    )
    .unwrap();
    let kw_mod_rlib = build_dir.join("libkw_mod.rlib");
    let compile_mod = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=kw_mod")
        .arg("-D")
        .arg("warnings")
        .arg(&mod_root)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&kw_mod_rlib)
        .output()
        .expect("failed to compile kw_mod crate");
    assert!(
        compile_mod.status.success(),
        "compiling mode=mod crate failed:\n{}",
        String::from_utf8_lossy(&compile_mod.stderr)
    );

    // 4. Compile `mode=lib` crate with `rustc -D warnings`
    let kw_lib_rlib = build_dir.join("libkw_lib.rlib");
    let compile_lib = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=rlib")
        .arg("--crate-name=kw_lib")
        .arg("-D")
        .arg("warnings")
        .arg(gen_lib_dir.join("lib.rs"))
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&kw_lib_rlib)
        .output()
        .expect("failed to compile kw_lib crate");
    assert!(
        compile_lib.status.success(),
        "compiling mode=lib crate failed:\n{}",
        String::from_utf8_lossy(&compile_lib.stderr)
    );

    // 5. Compile and execute a test binary that exercises round-trip serialization,
    // `copy_unset_fields`, nested keyword modules, enums, and oneofs against BOTH `kw_lib` and `kw_mod`.
    let runner_src = tmp_dir.join("kw_runner.rs");
    std::fs::write(
        &runner_src,
        r#"
macro_rules! verify_keyword_crate {
    ($krate:ident) => {{
        use $krate::r#type::r#mod::r#match::r#struct::r#enum::r#fn::r#impl::r#trait::r#async::r#await::r#dyn::r#try::r#gen as raw_pkg;
        use $krate::self_::super_::crate_ as path_pkg;
        use raw_pkg::keyword_enums::{
            scoped_self::Self_ as ScopedSelfEnum, scoped_variants::VariantKeywords,
            Async as AsyncEnum, Crate as CrateEnum, Gen as GenEnum, Match as MatchEnum,
            Struct as StructEnum, Super as SuperEnum, Type as TypeEnum,
        };

        // 1. Verify keyword-named enums and variants
        assert_eq!(VariantKeywords::Self_.as_i32(), 0);
        assert_eq!(VariantKeywords::Super.as_i32(), 1);
        assert_eq!(VariantKeywords::Crate.as_i32(), 2);
        assert_eq!(VariantKeywords::Type.as_i32(), 3);
        assert_eq!(VariantKeywords::Match.as_i32(), 4);
        assert_eq!(VariantKeywords::Struct.as_i32(), 5);
        assert_eq!(VariantKeywords::Async.as_i32(), 6);
        assert_eq!(VariantKeywords::Gen.as_i32(), 7);
        assert_eq!(ScopedSelfEnum::SelfOne.as_i32(), 1);
        assert_eq!(SuperEnum::SuperOne.as_i32(), 1);
        assert_eq!(CrateEnum::CrateOne.as_i32(), 1);
        assert_eq!(TypeEnum::TypeOne.as_i32(), 1);
        assert_eq!(MatchEnum::MatchOne.as_i32(), 1);
        assert_eq!(StructEnum::StructOne.as_i32(), 1);
        assert_eq!(AsyncEnum::AsyncOne.as_i32(), 1);
        assert_eq!(GenEnum::GenOne.as_i32(), 1);

        let mut enum_buf = [0u8; 128];
        let keb = raw_pkg::KeywordEnumsBuilder::new(&mut enum_buf);
        let kev_defaults = raw_pkg::KeywordEnumsView::from_bytes(keb.finish()?)?;
        assert_eq!(kev_defaults.kw_var()?, VariantKeywords::Type);
        assert_eq!(kev_defaults.kw_self()?, ScopedSelfEnum::SelfOne);
        assert_eq!(kev_defaults.kw_super()?, SuperEnum::SuperOne);
        assert_eq!(kev_defaults.kw_crate()?, CrateEnum::CrateOne);
        assert_eq!(kev_defaults.kw_type()?, TypeEnum::TypeOne);
        assert_eq!(kev_defaults.kw_match()?, MatchEnum::MatchOne);
        assert_eq!(kev_defaults.kw_struct()?, StructEnum::StructOne);
        assert_eq!(kev_defaults.kw_async()?, AsyncEnum::AsyncOne);
        assert_eq!(kev_defaults.kw_gen()?, GenEnum::GenOne);

        let mut keb = raw_pkg::KeywordEnumsBuilder::new(&mut enum_buf);
        keb.write_kw_gen(GenEnum::GenZero)
            .write_kw_async(AsyncEnum::AsyncZero)
            .write_kw_struct(StructEnum::StructZero)
            .write_kw_match(MatchEnum::MatchZero)
            .write_kw_type(TypeEnum::TypeZero)
            .write_kw_crate(CrateEnum::CrateZero)
            .write_kw_super(SuperEnum::SuperZero)
            .write_kw_self(ScopedSelfEnum::SelfZero)
            .write_kw_var(VariantKeywords::Self_);
        let kev = raw_pkg::KeywordEnumsView::from_bytes(keb.finish()?)?;
        assert_eq!(kev.kw_var()?, VariantKeywords::Self_);
        assert_eq!(kev.kw_self()?, ScopedSelfEnum::SelfZero);

        // 2. Verify keyword-named messages and their nested keyword modules (`r#struct`, `r#type`,
        // `r#mod`, `r#match`, `r#enum`, `r#fn`, `r#impl`, `r#trait`, `r#async`, `r#dyn`, `r#try`,
        // `r#gen`, `self_`, `super_`, `crate_`)
        let mut buf = [0u8; 256];
        let mut sb = raw_pkg::StructBuilder::new(&mut buf);
        sb.write_match(|ib: &mut raw_pkg::r#struct::InnerBuilder<'_>| {
            ib.write_type(77);
        });
        let sv = raw_pkg::StructView::from_bytes(sb.finish()?)?;
        let inner = sv.r#match()?.expect("expected match inner");
        assert_eq!(inner.r#type()?, 77);

        let mut tb = raw_pkg::TypeBuilder::new(&mut buf);
        tb.write_mod(|nb: &mut raw_pkg::r#type::NestedBuilder<'_>| {
            nb.write_fn(11);
        });
        let tv = raw_pkg::TypeView::from_bytes(tb.finish()?)?;
        assert_eq!(tv.r#mod()?.unwrap().r#fn()?, 11);

        let mut mb = raw_pkg::ModBuilder::new(&mut buf);
        mb.write_trait(|nb: &mut raw_pkg::r#mod::NestedBuilder<'_>| {
            nb.write_impl(12);
        });
        let mv = raw_pkg::ModView::from_bytes(mb.finish()?)?;
        assert_eq!(mv.r#trait()?.unwrap().r#impl()?, 12);

        let mut mcb = raw_pkg::MatchBuilder::new(&mut buf);
        mcb.write_await(|nb: &mut raw_pkg::r#match::NestedBuilder<'_>| {
            nb.write_async(13);
        });
        let mcv = raw_pkg::MatchView::from_bytes(mcb.finish()?)?;
        assert_eq!(mcv.r#await()?.unwrap().r#async()?, 13);

        let mut eb = raw_pkg::EnumBuilder::new(&mut buf);
        eb.write_try(|nb: &mut raw_pkg::r#enum::NestedBuilder<'_>| {
            nb.write_dyn(14);
        });
        let ev = raw_pkg::EnumView::from_bytes(eb.finish()?)?;
        assert_eq!(ev.r#try()?.unwrap().r#dyn()?, 14);

        let mut fnb = raw_pkg::FnBuilder::new(&mut buf);
        fnb.write_self(|nb: &mut raw_pkg::r#fn::NestedBuilder<'_>| {
            nb.write_gen(15);
        });
        let fnv = raw_pkg::FnView::from_bytes(fnb.finish()?)?;
        assert_eq!(fnv.self_()?.unwrap().r#gen()?, 15);

        let mut ib = raw_pkg::ImplBuilder::new(&mut buf);
        ib.write_crate(|nb: &mut raw_pkg::r#impl::NestedBuilder<'_>| {
            nb.write_super(16);
        });
        let iv = raw_pkg::ImplView::from_bytes(ib.finish()?)?;
        assert_eq!(iv.crate_()?.unwrap().super_()?, 16);

        let mut trb = raw_pkg::TraitBuilder::new(&mut buf);
        trb.write_as(|nb: &mut raw_pkg::r#trait::NestedBuilder<'_>| {
            nb.write_self(17);
        });
        let trv = raw_pkg::TraitView::from_bytes(trb.finish()?)?;
        assert_eq!(trv.r#as()?.unwrap().self_()?, 17);

        let mut asb = raw_pkg::AsyncBuilder::new(&mut buf);
        asb.write_const(|nb: &mut raw_pkg::r#async::NestedBuilder<'_>| {
            nb.write_break(18);
        });
        let asv = raw_pkg::AsyncView::from_bytes(asb.finish()?)?;
        assert_eq!(asv.r#const()?.unwrap().r#break()?, 18);

        let mut dyb = raw_pkg::DynBuilder::new(&mut buf);
        dyb.write_else(|nb: &mut raw_pkg::r#dyn::NestedBuilder<'_>| {
            nb.write_continue(19);
        });
        let dyv = raw_pkg::DynView::from_bytes(dyb.finish()?)?;
        assert_eq!(dyv.r#else()?.unwrap().r#continue()?, 19);

        let mut tryb = raw_pkg::TryBuilder::new(&mut buf);
        tryb.write_false(|nb: &mut raw_pkg::r#try::NestedBuilder<'_>| {
            nb.write_extern(20);
        });
        let tryv = raw_pkg::TryView::from_bytes(tryb.finish()?)?;
        assert_eq!(tryv.r#false()?.unwrap().r#extern()?, 20);

        let mut genb = raw_pkg::GenBuilder::new(&mut buf);
        genb.write_if(|nb: &mut raw_pkg::r#gen::NestedBuilder<'_>| {
            nb.write_for(21);
        });
        let genv = raw_pkg::GenView::from_bytes(genb.finish()?)?;
        assert_eq!(genv.r#if()?.unwrap().r#for()?, 21);

        let mut self_b = raw_pkg::Self_Builder::new(&mut buf);
        self_b.write_super(|nb: &mut raw_pkg::self_::Self_Builder<'_>| {
            nb.write_self(22);
        });
        let self_v = raw_pkg::Self_View::from_bytes(self_b.finish()?)?;
        assert_eq!(self_v.super_()?.unwrap().self_()?, 22);

        let mut sup_b = raw_pkg::SuperBuilder::new(&mut buf);
        sup_b.write_let(|nb: &mut raw_pkg::super_::NestedBuilder<'_>| {
            nb.write_in(23);
        });
        let sup_v = raw_pkg::SuperView::from_bytes(sup_b.finish()?)?;
        assert_eq!(sup_v.r#let()?.unwrap().r#in()?, 23);

        let mut cr_b = raw_pkg::CrateBuilder::new(&mut buf);
        cr_b.write_move(|nb: &mut raw_pkg::crate_::NestedBuilder<'_>| {
            nb.write_loop(24);
        });
        let cr_v = raw_pkg::CrateView::from_bytes(cr_b.finish()?)?;
        assert_eq!(cr_v.r#move()?.unwrap().r#loop()?, 24);

        // 3. Populate and verify AllKeywordFields (all 51 lowercase/path Rust keywords + 17 internal collision names + 3 cross-package refs = 71 fields, testing >64-bit `set_fields: [u64; 2]`!)
        let mut all_buf = [0u8; 2048];
        let mut akb = path_pkg::AllKeywordFieldsBuilder::new(&mut all_buf);
        akb.write_ref_self_enum(VariantKeywords::Crate)
            .write_ref_self_msg(|mb| {
                mb.write_super(|nb| {
                    nb.write_self(999);
                });
            })
            .write_ref_struct(|sb| {
                sb.write_match(|ib| {
                    ib.write_type(888);
                });
            })
            .write_f_1(68)
            .write_oneof_0(67)
            .write_set_fields(66)
            .write_copy_unset_fields(65)
            .write_copy_unknown_fields(64)
            .write_finish(63)
            .write_check(62)
            .write_from_writer(61)
            .write_new(60)
            .write_view(59)
            .write_writer(58)
            .write_field(57)
            .write_item(56)
            .write_buf(55)
            .write_unknown_fields(54)
            .write_as_bytes(53)
            .write_from_bytes(52)
            .write_crate(51)
            .write_super(50)
            .write_self(49)
            .write_gen(48)
            .write_try(47)
            .write_yield(46)
            .write_virtual(45)
            .write_unsized(44)
            .write_typeof(43)
            .write_priv(42)
            .write_override(41)
            .write_macro(40)
            .write_final(39)
            .write_do(38)
            .write_box(37)
            .write_become(36)
            .write_abstract(35)
            .write_dyn(34)
            .write_await(33)
            .write_async(32)
            .write_while(31)
            .write_where(30)
            .write_use(29)
            .write_unsafe(28)
            .write_type(27)
            .write_true(26)
            .write_trait(25)
            .write_struct(24)
            .write_static(23)
            .write_return(22)
            .write_ref(21)
            .write_pub(20)
            .write_mut(19)
            .write_move(18)
            .write_mod(17)
            .write_match(16)
            .write_loop(15)
            .write_let(14)
            .write_in(13)
            .write_impl(12)
            .write_if(11)
            .write_for(10)
            .write_fn(9)
            .write_false(8)
            .write_extern(7)
            .write_enum(6)
            .write_else(5)
            .write_continue(4)
            .write_const(3)
            .write_break(2)
            .write_as(1)
            .check()?;

        let akv = path_pkg::AllKeywordFieldsView::from_bytes(akb.finish()?)?;
        assert_eq!(akv.r#as()?, 1);
        assert_eq!(akv.r#break()?, 2);
        assert_eq!(akv.r#const()?, 3);
        assert_eq!(akv.r#continue()?, 4);
        assert_eq!(akv.r#else()?, 5);
        assert_eq!(akv.r#enum()?, 6);
        assert_eq!(akv.r#extern()?, 7);
        assert_eq!(akv.r#false()?, 8);
        assert_eq!(akv.r#fn()?, 9);
        assert_eq!(akv.r#for()?, 10);
        assert_eq!(akv.r#if()?, 11);
        assert_eq!(akv.r#impl()?, 12);
        assert_eq!(akv.r#in()?, 13);
        assert_eq!(akv.r#let()?, 14);
        assert_eq!(akv.r#loop()?, 15);
        assert_eq!(akv.r#match()?, 16);
        assert_eq!(akv.r#mod()?, 17);
        assert_eq!(akv.r#move()?, 18);
        assert_eq!(akv.r#mut()?, 19);
        assert_eq!(akv.r#pub()?, 20);
        assert_eq!(akv.r#ref()?, 21);
        assert_eq!(akv.r#return()?, 22);
        assert_eq!(akv.r#static()?, 23);
        assert_eq!(akv.r#struct()?, 24);
        assert_eq!(akv.r#trait()?, 25);
        assert_eq!(akv.r#true()?, 26);
        assert_eq!(akv.r#type()?, 27);
        assert_eq!(akv.r#unsafe()?, 28);
        assert_eq!(akv.r#use()?, 29);
        assert_eq!(akv.r#where()?, 30);
        assert_eq!(akv.r#while()?, 31);
        assert_eq!(akv.r#async()?, 32);
        assert_eq!(akv.r#await()?, 33);
        assert_eq!(akv.r#dyn()?, 34);
        assert_eq!(akv.r#abstract()?, 35);
        assert_eq!(akv.r#become()?, 36);
        assert_eq!(akv.r#box()?, 37);
        assert_eq!(akv.r#do()?, 38);
        assert_eq!(akv.r#final()?, 39);
        assert_eq!(akv.r#macro()?, 40);
        assert_eq!(akv.r#override()?, 41);
        assert_eq!(akv.r#priv()?, 42);
        assert_eq!(akv.r#typeof()?, 43);
        assert_eq!(akv.r#unsized()?, 44);
        assert_eq!(akv.r#virtual()?, 45);
        assert_eq!(akv.r#yield()?, 46);
        assert_eq!(akv.r#try()?, 47);
        assert_eq!(akv.r#gen()?, 48);
        assert_eq!(akv.self_()?, 49);
        assert_eq!(akv.super_()?, 50);
        assert_eq!(akv.crate_()?, 51);

        // Collision accessors (`from_bytes_`, `as_bytes_`, `unknown_fields_`, `buf`, `item`, `field`, etc.)
        assert!(akv.has_from_bytes());
        assert_eq!(akv.from_bytes_()?, 52);
        assert!(akv.has_as_bytes());
        assert_eq!(akv.as_bytes_()?, 53);
        assert!(akv.has_unknown_fields());
        assert_eq!(akv.unknown_fields_()?, 54);
        assert!(!akv.as_bytes().is_empty());
        assert_eq!(akv.unknown_fields().count(), 0);
        assert_eq!(akv.buf()?, 55);
        assert_eq!(akv.item()?, 56);
        assert_eq!(akv.field()?, 57);
        assert_eq!(akv.writer()?, 58);
        assert_eq!(akv.view()?, 59);
        assert_eq!(akv.new()?, 60);
        assert_eq!(akv.from_writer()?, 61);
        assert_eq!(akv.check()?, 62);
        assert_eq!(akv.finish()?, 63);
        assert_eq!(akv.copy_unknown_fields()?, 64);
        assert_eq!(akv.copy_unset_fields()?, 65);
        assert_eq!(akv.set_fields()?, 66);
        assert_eq!(akv.oneof_0()?, 67);
        assert_eq!(akv.f_1()?, 68);

        let rs = akv.ref_struct()?.expect("expected ref_struct");
        let rs_inner = rs.r#match()?.expect("expected match");
        assert_eq!(rs_inner.r#type()?, 888);

        let rsm = akv.ref_self_msg()?.expect("expected ref_self_msg");
        assert_eq!(rsm.super_()?.unwrap().self_()?, 999);
        assert_eq!(akv.ref_self_enum()?, VariantKeywords::Crate);

        // Verify `copy_unset_fields` on `AllKeywordFieldsBuilder` (exercises multi-word `set_fields: [u64; 2]`)
        let mut patch_buf = [0u8; 2048];
        let mut patch_b = path_pkg::AllKeywordFieldsBuilder::new(&mut patch_buf);
        patch_b
            .write_ref_self_enum(VariantKeywords::Gen)
            .write_from_bytes(5200)
            .write_self(4900)
            .write_type(2700)
            .copy_unset_fields(&akv);
        let patched = path_pkg::AllKeywordFieldsView::from_bytes(patch_b.finish()?)?;
        assert_eq!(patched.r#as()?, 1);
        assert_eq!(patched.r#type()?, 2700);
        assert_eq!(patched.self_()?, 4900);
        assert_eq!(patched.from_bytes_()?, 5200);
        assert_eq!(patched.f_1()?, 68);
        assert_eq!(patched.ref_self_enum()?, VariantKeywords::Gen);

        // 4. Verify `KeywordOneofs` (`type`, `match`, `self`, `crate`, `super`, `struct`)
        let mut ko_buf = [0u8; 512];
        let mut kob = path_pkg::KeywordOneofsBuilder::new(&mut ko_buf);
        kob.write_struct_enum(VariantKeywords::Async)
            .write_super_flag(true)
            .write_crate_bytes(b"crate_payload")
            .write_self_text("self_payload")
            .write_o_match_struct(|sb| {
                sb.write_match(|ib| {
                    ib.write_type(321);
                });
            })
            .write_o_type_str("type_payload");
        let kov = path_pkg::KeywordOneofsView::from_bytes(kob.finish()?)?;
        assert_eq!(
            kov.r#type()?,
            Some(path_pkg::keyword_oneofs::Type::OTypeStr("type_payload"))
        );
        match kov.r#match()?.expect("expected match oneof") {
            path_pkg::keyword_oneofs::Match::OMatchStruct(sv) => {
                assert_eq!(sv.r#match()?.unwrap().r#type()?, 321);
            }
            other => panic!("unexpected match oneof: {other:?}"),
        }
        assert_eq!(
            kov.self_()?,
            Some(path_pkg::keyword_oneofs::Self_::SelfText("self_payload"))
        );
        assert_eq!(
            kov.crate_()?,
            Some(path_pkg::keyword_oneofs::Crate::CrateBytes(b"crate_payload"))
        );
        assert_eq!(
            kov.super_()?,
            Some(path_pkg::keyword_oneofs::Super::SuperFlag(true))
        );
        assert_eq!(
            kov.r#struct()?,
            Some(path_pkg::keyword_oneofs::Struct::StructEnum(VariantKeywords::Async))
        );

        // 5. Verify `OneofCollision` where oneof `Type` collides with field `type` (`type_oneof()`),
        // oneof `Self` collides with field `self` (`self_oneof()`), and `Payload` collides with `payload` (`payload_oneof()`)
        let mut oc_buf = [0u8; 256];
        let mut ocb = path_pkg::OneofCollisionBuilder::new(&mut oc_buf);
        ocb.write_payload("payload_str")
            .write_super("super_str")
            .write_type("type_str");
        let ocv = path_pkg::OneofCollisionView::from_bytes(ocb.finish()?)?;
        assert_eq!(ocv.r#type()?, "type_str");
        assert_eq!(
            ocv.type_oneof()?,
            Some(path_pkg::oneof_collision::Type::Type("type_str"))
        );
        assert_eq!(ocv.super_()?, "super_str");
        assert_eq!(
            ocv.self_oneof()?,
            Some(path_pkg::oneof_collision::Self_::Super("super_str"))
        );
        assert_eq!(ocv.payload()?, "payload_str");
        assert_eq!(
            ocv.payload_oneof()?,
            Some(path_pkg::oneof_collision::Payload::Payload("payload_str"))
        );
    }};
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    verify_keyword_crate!(kw_lib);
    verify_keyword_crate!(kw_mod);
    Ok(())
}
"#,
    )
    .unwrap();

    let runner_bin = build_dir.join("kw_runner");
    let compile_runner = Command::new("rustc")
        .arg("--edition=2021")
        .arg("-D")
        .arg("warnings")
        .arg(&runner_src)
        .arg("--extern")
        .arg(format!("roto={}", roto_rlib.display()))
        .arg("--extern")
        .arg(format!("kw_lib={}", kw_lib_rlib.display()))
        .arg("--extern")
        .arg(format!("kw_mod={}", kw_mod_rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", build_dir.display()))
        .arg("-o")
        .arg(&runner_bin)
        .output()
        .expect("failed to compile kw_runner");
    assert!(
        compile_runner.status.success(),
        "compiling kw_runner failed:\n{}",
        String::from_utf8_lossy(&compile_runner.stderr)
    );

    let run_out = Command::new(&runner_bin)
        .output()
        .expect("failed to execute kw_runner");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    assert!(
        run_out.status.success(),
        "kw_runner failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_out.stdout),
        String::from_utf8_lossy(&run_out.stderr)
    );
}
