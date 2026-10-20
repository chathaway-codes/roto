---
name: qa
description: >-
  Runs automated tests and verification checks using the project Makefile.
  Use this skill whenever verifying code changes, checking for regressions,
  or running test suites.
---

# Quality Assurance & Testing

Deterministic verification via `make`. Fast feedback, zero guesswork.

## Core Commands

Always execute targets from the repository root:

| Target | Purpose | Speed |
| :--- | :--- | :--- |
| `make test` | Full test suite (`test-issues`, `test-rust`, `test-no-std`) | Fast (~3s) |
| `make fuzz` | Extended deterministic wire, data, and descriptor fuzzers | Medium (~5-10s) |
| `make test-no-std` | Bare-metal `thumbv7em-none-eabihf` `#![no_std]` verification | Fast (~1s) |
| `make test-issues` | Verify issue tracker tooling and dependency graph | Sub-second (~0.15s) |
| `make clean` | Purge build artifacts, caches, and temporary files | Instant |

## Workspace Test Suite & Refactoring Reference

When modifying runtime contracts or codegen templates, inspect and update the corresponding test files:

- `roto-runtime/src/lib.rs` & `roto-runtime/tests/readme_sensor.rs`: Core wire primitives, `FieldIter`, `Repeated*Iter` (including `RepeatedGroupIter`), `BackwardWriter`, sticky `EncodeError`, and hand-written reference `Location`/`SensorReading` views & builders.
- `roto-runtime/tests/fuzz_wire.rs`: Deterministic PRNG wire fuzzer (`ROTO_FUZZ_SEED`, `ROTO_FUZZ_ITERS`) for `FieldIter`, all `Repeated*Iter` types, `UnknownFieldsIter`, and `BackwardWriter` round-tripping.
- `roto-grpc/tests/framing_and_status.rs`: 5-byte Length-Prefixed Message (LPM) framing, backward frame writer, `GrpcCode`/`GrpcStatus`, and RFC 3986 percent-encoding/decoding tests.
- `google-protobuf-roto-bootstrap/tests/protoc_descriptor_test.rs` & `unit_roundtrip_test.rs`: Thin compiler bootstrap descriptor (`descriptor.proto` + `compiler/plugin.proto`) round-trip and `protoc` descriptor set decoding tests.
- `google-protobuf-roto/tests/wkt_roundtrip_test.rs`, `protoc_descriptor_test.rs`, `unit_roundtrip_test.rs`: Full `google/protobuf` Well-Known Types (`Timestamp`, `Duration`, `Any`, `Empty`, `FieldMask`, `SourceContext`, recursive `Struct`/`Value`/`ListValue`, `Type`, `Api`, all 9 wrapper types, and edition features) round-trip tests.
- `protoc-gen-roto/src/generator/{ident,symbols,enums,defaults,views,builders,modules}.rs`: Unit tests for codegen string templates and exact expected Rust output snippets (update when changing generated `*View<'a>` or `*Builder<'b>` struct/method layouts).
- `protoc-gen-roto/tests/protoc_plugin_integration.rs` & `protoc-gen-roto/tests/e2e_codegen.rs`: `CodeGeneratorRequest`/`CodeGeneratorResponse` framing, `sensor.proto`, comprehensive `proto2`/`proto3` schemas, `extern_path`, and self-hosting tests.
- `protoc-gen-roto/tests/data_types_matrix.rs`: Exhaustive 18-type matrix, boundary scalar values, all 12 `map<K, V>` key types, `group` framing, `oneof` overwrite semantics, boundary tag `536870911`, and bidirectional `protoc --encode`/`--decode` wire interoperability.
- `protoc-gen-roto/tests/rust_keywords_test.rs`: All Rust strict, reserved, weak, and path keywords (`r#<kw>`, `self_`, `super_`, `crate_`, `Self_`) and internal method collision names across packages, messages, nested messages, fields, enums, and oneofs in `mode=lib` and `mode=mod`.
- `protoc-gen-roto/tests/ecosystem_packages_test.rs`: Multi-package Google ecosystem schemas (`google/rpc/*`, `google/type/*`, `google/api/*`) importing `::google_protobuf_roto` via `extern_path`.
- `protoc-gen-roto/tests/fuzz_data.rs`: Generated code data fuzzer (`unparse -> parse -> unparse`, `copy_unset_fields`, `copy_unknown_fields`, and mutated wire `parse -> unparse -> parse`).
- `protoc-gen-roto/tests/fuzz_descriptors.rs`: Random `proto2`/`proto3` multi-file descriptor generator (`RandomSchemaGenerator`), descriptor round-trip, codegen (`mode=lib` and `mode=mod`), and dynamic compiled data `unparse -> parse -> unparse` fuzzer.
- `cargo-roto/tests/cli_driver.rs` & `cargo-roto/tests/e2e_packaging.rs`: CLI flag parsing, `cargo metadata` `[package.metadata.roto]` resolution, cross-crate git dependency packaging, and `google-protobuf-roto-bootstrap` / `google-protobuf-roto` fixed-point self-hosting check.
- `tests/no_std_check/src/lib.rs`: Bare-metal `#![no_std]` and zero-allocation target verification (`thumbv7em-none-eabihf`).

## Hacker QA Discipline

1. **Impatience (Tight Feedback Loops)**:
   - Run tests frequently. Feedback should take seconds, not minutes. Keep tests fast and run them early.
2. **Hubris (Non-Negotiable Quality)**:
   - Before completing any task, run `make test`. All assertions must pass (exit status `0`).
   - If tests fail, do not mock or suppress them. Fix the underlying logic.
3. **No Assumptions (Grounding)**:
   - If a test fails, inspect the exact failure message, line number, and stack trace.
   - Verify file paths, data structures, and types directly with inspection tools before making changes.
