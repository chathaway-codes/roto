# Padding-Free Protobuf (`roto`)

A minimal, zero-allocation, zero-copy `proto2`-compatible Protocol Buffers reader, writer, and code generator for Rust `#[no_std]` environments.

Standard Protocol Buffers implementations deserialize wire bytes by copying fields into heap-allocated, compiler-padded Rust structs (`String`, `Vec<T>`, `Box<T>`) and require either two passes (compute byte length, then serialize) or intermediate buffers when writing nested messages.

**Padding-Free Protobuf** takes a fundamentally different approach:
1. **`#![no_std]` & Zero Heap Allocations**: Runs in bare-metal, embedded, and kernel environments without `alloc` or `std`.
2. **Zero-Copy View Structs**: Reading a message never copies variable-length or nested data into heap-allocated containers. Instead, view structs decode scalar values eagerly into compact stack fields, track presence in a bitmask (`has_fields: [u32; W]`), and record `FieldMarker` slice spans pointing into `&'a [u8]` for zero-copy string/bytes/sub-message access.
3. **On-Demand Shallow Parsing**: Parsing a message performs a single $O(n)$ scan of its top-level wire tags using a runtime field iterator to decode top-level scalars and record byte-span markers. Nested sub-messages and repeated fields are not parsed eagerly; calling an accessor for a nested message constructs its view struct on demand over its recorded sub-slice.
4. **Single-Pass Backward Builder**: Serialization writes into a fixed-size `&mut [u8]` buffer **from back to front**. Because child fields and nested messages are written before their parent prefixes, exact byte lengths are always known at the moment a length-delimited prefix varint is emitted—enabling true single-pass encoding without backtracking, length pre-computation, or temporary buffers.
5. **Preserved Unknown Fields**: Because view structs retain the underlying wire slice `&'a [u8]` and know which field tags belong to the schema, unrecognized tags are naturally preserved in-place without extra memory overhead, and builders provide `copy_unknown_fields` to losslessly forward them during re-serialization.

---

## Example: Schema & Generated Rust Code

### Input Schema (`sensor.proto`)

```protobuf
syntax = "proto2";

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
```

### Generated Rust Code (Conceptual)

```rust
#![no_std]

use roto::runtime::{
    BackwardWriter, DecodeError, EncodeError, FieldIter, FieldMarker,
    RepeatedVarintIter, UnknownFieldsIter, WireType,
};

// ============================================================================
// Location (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct LocationView<'a> {
    buf: &'a [u8],
    lat_e7: i32,
    lon_e7: i32,
    has_fields: [u32; 1],
}

impl<'a> LocationView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

    /// Performs a single O(n) shallow scan over the top-level fields in `buf`,
    /// decoding scalar values eagerly and recording presence in `has_fields`.
    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> { ... }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] { ... }

    #[inline]
    pub const fn has_lat_e7(&self) -> bool { ... }

    /// Returns field 1 (`required int32 lat_e7`) decoded during `from_bytes`.
    #[inline]
    pub const fn lat_e7(&self) -> Result<i32, DecodeError> { ... }

    #[inline]
    pub const fn has_lon_e7(&self) -> bool { ... }

    /// Returns field 2 (`required int32 lon_e7`) decoded during `from_bytes`.
    #[inline]
    pub const fn lon_e7(&self) -> Result<i32, DecodeError> { ... }

    /// Iterates over raw wire spans (tag + payload) of unknown fields in `self.buf`.
    #[inline]
    pub fn unknown_fields(&self) -> UnknownFieldsIter<'a> { ... }
}

pub struct LocationBuilder<'b> {
    pub writer: BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> LocationBuilder<'b> {
    #[inline]
    pub fn new(buf: &'b mut [u8]) -> Self { ... }

    #[inline]
    pub const fn from_writer(writer: BackwardWriter<'b>) -> Self { ... }

    #[inline]
    pub const fn writer(&self) -> &BackwardWriter<'b> { ... }

    #[inline]
    pub fn writer_mut(&mut self) -> &mut BackwardWriter<'b> { ... }

    /// Returns `Err(EncodeError::BufferOverflow)` if any write on the underlying
    /// writer has overflowed the buffer.
    #[inline]
    pub const fn check(&self) -> Result<(), EncodeError> { ... }

    #[inline]
    pub fn finish(self) -> Result<&'b [u8], EncodeError> { ... }

    /// Copies all unknown fields preserved in `view` directly into the writer.
    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &LocationView<'_>) -> &mut Self { ... }

    /// Copies all unset schema fields and unknown fields from `view` into the writer.
    #[inline]
    pub fn copy_unset_fields(&mut self, view: &LocationView<'_>) -> &mut Self { ... }

    #[inline]
    pub fn write_lon_e7(&mut self, val: i32) -> &mut Self { ... }

    #[inline]
    pub fn write_lat_e7(&mut self, val: i32) -> &mut Self { ... }
}

// ============================================================================
// SensorReading (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct SensorReadingView<'a> {
    buf: &'a [u8],
    timestamp_ms: u64,
    device_id: FieldMarker,
    location: FieldMarker,
    has_fields: [u32; 1],
}

impl<'a> SensorReadingView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4];

    /// Performs a single O(n) shallow scan over `buf` using the runtime field iterator.
    /// Scalars (`timestamp_ms`) are decoded eagerly; nested messages (`location`) store only their byte span marker.
    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> { ... }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] { ... }

    #[inline]
    pub const fn has_timestamp_ms(&self) -> bool { ... }

    /// Returns `required uint64 timestamp_ms = 1` decoded during `from_bytes`.
    #[inline]
    pub const fn timestamp_ms(&self) -> Result<u64, DecodeError> { ... }

    #[inline]
    pub const fn has_device_id(&self) -> bool { ... }

    /// Reads `optional string device_id = 2 [default = "unknown"]`.
    /// Zero-copy: slices `self.buf` directly using `self.device_id` span marker.
    #[inline]
    pub fn device_id(&self) -> Result<&'a str, DecodeError> { ... }

    #[inline]
    pub const fn has_location(&self) -> bool { ... }

    /// Reads `optional Location location = 3`.
    /// Shallow/on-demand: `LocationView::from_bytes` is only invoked when `.location()` is called.
    #[inline]
    pub fn location(&self) -> Result<Option<LocationView<'a>>, DecodeError> { ... }

    /// Returns a zero-allocation iterator over `repeated uint32 samples = 4`.
    #[inline]
    pub fn samples(&self) -> RepeatedVarintIter<'a, u32> { ... }

    /// Iterates over raw wire spans (tag + payload) of unknown fields in `self.buf`.
    #[inline]
    pub fn unknown_fields(&self) -> UnknownFieldsIter<'a> { ... }
}

pub struct SensorReadingBuilder<'b> {
    pub writer: BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> SensorReadingBuilder<'b> {
    #[inline]
    pub fn new(buf: &'b mut [u8]) -> Self { ... }

    #[inline]
    pub const fn from_writer(writer: BackwardWriter<'b>) -> Self { ... }

    #[inline]
    pub const fn writer(&self) -> &BackwardWriter<'b> { ... }

    #[inline]
    pub fn writer_mut(&mut self) -> &mut BackwardWriter<'b> { ... }

    /// Returns `Err(EncodeError::BufferOverflow)` if any write on the underlying
    /// writer has overflowed the buffer.
    #[inline]
    pub const fn check(&self) -> Result<(), EncodeError> { ... }

    #[inline]
    pub fn finish(self) -> Result<&'b [u8], EncodeError> { ... }

    /// Copies all unknown fields preserved in `view` directly into the writer.
    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &SensorReadingView<'_>) -> &mut Self { ... }

    /// Copies all unset schema fields and unknown fields from `view` into the writer.
    #[inline]
    pub fn copy_unset_fields(&mut self, view: &SensorReadingView<'_>) -> &mut Self { ... }

    #[inline]
    pub fn write_sample(&mut self, sample: u32) -> &mut Self { ... }

    /// Writes a nested `Location` message in a single backward pass.
    /// The closure populates the child message first; once complete, the exact
    /// byte length is known and the length + tag varints are written immediately before it.
    #[inline]
    pub fn write_location<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut LocationBuilder<'_>),
    { ... }

    #[inline]
    pub fn write_device_id(&mut self, device_id: &str) -> &mut Self { ... }

    #[inline]
    pub fn write_timestamp_ms(&mut self, ts: u64) -> &mut Self { ... }
}
```

### Usage Example: Single-Pass Write & Shallow Read

```rust
fn main() -> Result<(), Box<dyn core::error::Error>> {
    // 1. Serialize into a fixed-size stack buffer (no heap allocation, single pass)
    let mut backing = [0u8; 128];
    let mut builder = SensorReadingBuilder::new(&mut backing);

    // Because the buffer fills backwards, repeated items written in reverse
    // order appear in forward order on the wire. Use `builder.check()?` to
    // exit early if a loop overflows the fixed-size buffer:
    for sample in [42, 108].into_iter().rev() {
        builder.write_sample(sample);
        builder.check()?;
    }

    // Builder methods return `&mut Self` for ergonomic chaining, and nested
    // message closures require no trailing `Ok(())` boilerplate:
    builder
        .write_location(|loc| {
            loc.write_lon_e7(-1224194000)
                .write_lat_e7(377749000);
        })
        .write_device_id("sensor-node-01")
        .write_timestamp_ms(1_700_000_000_000);

    // `builder.finish()?` surfaces any sticky `BufferOverflow` error and returns
    // the written tail slice `&backing[cursor..128]`
    let wire_bytes: &[u8] = builder.finish()?;

    // 2. O(n) shallow scan builds markers for top-level fields in `SensorReadingView`
    let reading = SensorReadingView::from_bytes(wire_bytes)?;
    assert_eq!(reading.timestamp_ms()?, 1_700_000_000_000);
    assert_eq!(reading.device_id()?, "sensor-node-01");

    // `LocationView` is only scanned/constructed on demand when `.location()` is called
    if let Some(loc) = reading.location()? {
        assert_eq!(loc.lat_e7()?, 377749000);
        assert_eq!(loc.lon_e7()?, -1224194000);
    }

    Ok(())
}
```

---

## Packaging & Cross-Crate Proto Imports

Generated `roto` crates are designed to be committed and published directly to GitHub (or a crate registry) with both their generated Rust code (`src/`) and raw `.proto` files (`proto/`). Downstream Rust applications can depend on pre-generated proto crates via standard Cargo git dependencies without running `protoc` at compile time, while downstream proto crates can import upstream `.proto` schemas seamlessly.

### 1. Standard Crate Layout
Every `roto` proto crate follows a standard layout where raw `.proto` files live under a top-level `proto/` directory matching their import path:

```text
google-protobuf-roto/
├── Cargo.toml
├── proto/
│   └── google/
│       └── protobuf/
│           └── timestamp.proto
└── src/
    └── lib.rs          # Pre-generated Rust code
```

### 2. Crate & Module Naming Conventions
- **Cargo Package vs. Rust Crate Identifier**: Cargo package names conventionally use `kebab-case` (e.g., `google-protobuf-roto`), which Cargo normalizes to `snake_case` (`::google_protobuf_roto`) for Rust crate identifiers.
- **1:1 Proto Package to Module Hierarchy**: A `.proto` file's `package` declaration (e.g., `package google.protobuf;`) maps 1:1 to nested Rust modules exported from the crate root (`pub mod google { pub mod protobuf { ... } }`). Rust keywords in path segments (such as `google.type`) are escaped as raw identifiers (`r#type`).

### 3. Declaring the Source Map in `Cargo.toml`
Each proto crate declares the `.proto` directory prefixes (namespaces) it provides using Cargo's [`[package.metadata]`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-metadata-table) table (`[package.metadata.roto]`). Cargo ignores this table during normal builds and exposes it via `cargo metadata`:

```toml
# google-protobuf-roto/Cargo.toml
[package]
name = "google-protobuf-roto"
version = "0.1.0"
edition = "2021"

[dependencies]
roto-runtime = { git = "https://github.com/you/roto" }

[package.metadata.roto]
provides = ["google/protobuf"]
```

A crate that bundles multiple namespaces lists each prefix it exports:
```toml
# googleapis-roto/Cargo.toml
[package.metadata.roto]
provides = ["google/api", "google/rpc", "google/type"]
```

#### Supported `[package.metadata.roto]` Keys

`cargo-roto` / `roto-build` validates `[package.metadata.roto]` and rejects unknown keys with an error diagnostic. Explicit CLI flags (`--proto-dir`, `--out-dir`, `--mode`, `--grpc`) take precedence over manifest metadata:

| Key | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `provides` | `Vec<String>` | Exported `.proto` directory/package prefixes for discovery and downstream `extern_path` mapping | All `.proto` files under `proto_dir` |
| `proto_dir` / `proto_paths` | `String` / `Vec<String>` | Source `.proto` root directory (or list of `.proto` include directories) relative to the crate root | `"proto"` |
| `inputs` | `Vec<String>` | Explicit `.proto` input files to compile (relative to the crate root or `proto_dir`) | Discovered via `provides` / `proto_dir` |
| `out_dir` | `String` | Output directory for generated Rust files relative to the crate root | `"src"` |
| `mode` | `"lib" \| "mod"` | Emit single-file `lib.rs` (`"lib"`) or `mod.rs` + per-package files (`"mod"`) | `"lib"` |
| `grpc` | `bool` | Enable opt-in `protoc-gen-roto-grpc` codegen (`grpc.rs` + `pub mod grpc;`) | `false` |

### 4. Codegen Workflow (`protoc` Plugin & Metadata Helper)
Code generation uses `protoc` with two (or optionally three) components:
1. **`protoc-gen-roto` (`protoc` plugin)**: Reads a serialized `CodeGeneratorRequest` from `stdin` and writes a `CodeGeneratorResponse` to `stdout`.
2. **`protoc-gen-roto-grpc` (opt-in `protoc` plugin)**: Generates `#![no_std]` zero-copy gRPC service traits, dispatchers, and client stubs into `grpc.rs` when `--grpc` or `grpc = true` is enabled.
3. **`roto-build` / `cargo-roto` (Metadata Helper)**: Prepares and invokes `protoc` for a crate by inspecting `Cargo.toml` dependencies.

When generating a downstream proto crate (e.g., `telemetry-roto` importing `google/protobuf/timestamp.proto`):

```toml
# telemetry-roto/Cargo.toml
[package]
name = "telemetry-roto"
version = "0.1.0"
edition = "2021"

[dependencies]
roto-runtime = { git = "https://github.com/you/roto" }
google-protobuf-roto = { git = "https://github.com/you/google-protobuf-roto" }

[package.metadata.roto]
provides = ["telemetry"]
```

Running the `cargo-roto` (`cargo roto`) or `roto-build` helper performs the following steps:
1. **Fetch & Resolve Dependencies**: Invokes `cargo metadata --format-version 1`, which automatically downloads any git or registry dependencies into Cargo's local cache (`~/.cargo/git/checkouts/...`) and outputs the resolved dependency graph.
2. **Collect `-I` Include Paths & Source Maps**: For each dependency containing `[package.metadata.roto]`:
   - Locates the dependency's checked-out root directory via `manifest_path.parent()` and adds `-I <dep_root>/proto` to the `protoc` arguments.
   - Maps each entry in `provides` (e.g., `"google/protobuf"`) to the dependency's resolved Rust `extern crate` name from `resolve.nodes[].deps[].name` (e.g., `::google_protobuf_roto`).
3. **Invoke `protoc`**: Runs `protoc` with `--plugin=protoc-gen-roto` (and `--plugin=protoc-gen-roto-grpc` when `--grpc` or `grpc = true` is set), passing the collected `-I` include flags and `extern_path` mappings so references to imported types emit `::google_protobuf_roto::google::protobuf::TimestampView<'a>` instead of regenerating imported schemas.

---

## CLI Usage (`cargo-roto`, `roto-build`, `protoc-gen-roto`, and `protoc-gen-roto-grpc`)

### 1. Generating a Crate with `cargo roto` or `roto-build`

Install the binaries locally or run them directly from the workspace:

```bash
# Install `cargo-roto` (`cargo roto`), `roto-build`, `protoc-gen-roto`, and `protoc-gen-roto-grpc`
cargo install --path cargo-roto --path protoc-gen-roto --path protoc-gen-roto-grpc
```

Run inside any proto crate directory (or pass `--manifest-path`):

```bash
# Invoke as a Cargo subcommand (defaults to ./Cargo.toml, ./proto, and ./src)
cargo roto
cargo roto build

# Include opt-in gRPC service trait, server router, and client stub generation
cargo roto --grpc

# Or invoke the standalone helper binary
roto-build --manifest-path path/to/Cargo.toml

# Regenerate the self-hosted google-protobuf-roto crate in this workspace
cargo run -p cargo-roto -- --manifest-path google-protobuf-roto/Cargo.toml
```

#### Supported CLI Flags (`cargo-roto` / `roto-build`)

| Flag | Description | Default |
| :--- | :--- | :--- |
| `--manifest-path <PATH>` | Path to target crate's `Cargo.toml` | `./Cargo.toml` |
| `--proto-dir <DIR>` | Override source `.proto` directory | `<crate_root>/proto` |
| `--out-dir <DIR>` | Override output directory passed to `--roto_out` | `<crate_root>/src` |
| `--plugin <PATH>` | Explicit path to the `protoc-gen-roto` binary | Auto-detected |
| `--grpc` | Enable opt-in gRPC service and client code generation (`grpc.rs`) | `false` |
| `--grpc-plugin <PATH>` | Explicit path to the `protoc-gen-roto-grpc` binary | Auto-detected |
| `--mode <lib\|mod>` | Emit single-file `lib.rs` (`lib`) or multi-file `mod.rs` + per-package files (`mod`) | `lib` |
| `--offline` | Forward `--offline` to `cargo metadata` | `false` |
| `--locked` | Forward `--locked` to `cargo metadata` | `false` |
| `--frozen` | Forward `--frozen` to `cargo metadata` | `false` |
| `-h`, `--help` | Print CLI usage summary | — |

#### Environment Variables & Binary Discovery
- **`PROTOC`**: Explicit path to `protoc` (falls back to `protoc` on `$PATH`).
- **`PROTOC_GEN_ROTO`**: Explicit path to `protoc-gen-roto` (resolved in order: `--plugin <PATH>` $\to$ `$PROTOC_GEN_ROTO` $\to$ sibling binary next to `cargo-roto`/`roto-build` $\to$ `$PATH`).
- **`PROTOC_GEN_ROTO_GRPC`**: Explicit path to `protoc-gen-roto-grpc` (resolved in order: `--grpc-plugin <PATH>` $\to$ `$PROTOC_GEN_ROTO_GRPC` $\to$ sibling binary next to `cargo-roto`/`roto-build` $\to$ `$PATH`).

---

### 2. Direct `protoc` Invocation with `protoc-gen-roto`

You can also invoke `protoc-gen-roto` directly via `protoc` without `cargo-roto`:

```bash
protoc \
  --plugin=protoc-gen-roto=./target/debug/protoc-gen-roto \
  --roto_opt=mode=lib,extern_path=google/protobuf=::google_protobuf_roto \
  --roto_out=src \
  -I proto \
  -I ../google-protobuf-roto/proto \
  telemetry/sensor.proto
```

#### Supported `protoc-gen-roto` Parameters (`--roto_opt`)
- `mode=lib` *(default)*: Emits a single `#![no_std]` `lib.rs` containing the nested `pub mod <pkg>` hierarchy.
- `mode=mod`: Emits `mod.rs` alongside per-package module files (e.g., `google/protobuf.rs`).
- `extern_path=<proto_prefix>=<rust_crate_path>`: Maps a `.proto` package or directory prefix (e.g., `google/protobuf` or `.google.protobuf`) to an external Rust crate path (e.g., `::google_protobuf_roto`).

---

## Workspace Crates

| Crate | Description |
| :--- | :--- |
| [`roto-runtime`](roto-runtime) | `#![no_std]`, zero-allocation wire format runtime (`FieldIter`, `FieldMarker`, `BackwardWriter`, repeated/unknown iterators) |
| [`roto`](roto) | `#![no_std]` user-facing facade crate re-exporting `roto::runtime` |
| [`roto-grpc`](roto-grpc) | `#![no_std]`, zero-allocation gRPC over Sans-I/O HTTP/2 (`GrpcServer`, `GrpcClient`, HPACK, 5-byte LPM framing, `GrpcStatus`, and optional `std`/`embedded-io-async`/`tokio` transport drivers) |
| [`google-protobuf-roto-bootstrap`](google-protobuf-roto-bootstrap) | Thin self-hosted `#![no_std]` compiler bootstrap views and builders for `google/protobuf/descriptor.proto` and `google/protobuf/compiler/plugin.proto` |
| [`google-protobuf-roto`](google-protobuf-roto) | Full `#![no_std]` `google/protobuf` Well-Known Types (`Timestamp`, `Duration`, `Any`, `Empty`, `FieldMask`, `SourceContext`, `Struct`/`Value`/`ListValue`, `Type`, `Api`, wrappers, descriptors, plugin, and edition feature sets) |
| [`protoc-gen-roto`](protoc-gen-roto) | Zero-dependency `protoc` code generator plugin emitting `#![no_std]` `*View<'a>` and `*Builder<'b>` types |
| [`protoc-gen-roto-grpc`](protoc-gen-roto-grpc) | Opt-in zero-dependency `protoc` code generator plugin emitting `#![no_std]` gRPC service traits, routers, and client stubs (see [`docs/grpc.md`](docs/grpc.md)) |
| [`cargo-roto`](cargo-roto) | Zero-dependency Cargo subcommand (`cargo roto`) and `roto-build` CLI driver for `[package.metadata.roto]` resolution |
| [`tests/no_std_check`](tests/no_std_check) | Bare-metal `#![no_std]` + `no_alloc` verification crate cross-compiled for `thumbv7em-none-eabihf` and `thumbv6m-none-eabi` |

---

## Development & Workspace Commands

```bash
# Run full verification suite (issue tracker, Rust unit/integration/e2e tests, and thumbv7em-none-eabihf no_std check)
make test

# Run deterministic wire, data, and descriptor fuzzers with extended iterations
make fuzz

# Run only bare-metal thumbv7em-none-eabihf #![no_std] verification
make test-no-std

# List open issues ready for pickup
make issues-ready

# Launch local read-only issue viewer UI
make issues-ui
```

---

## Architecture Overview

### 1. Shallow $O(n)$ Parsing, Eager Scalars, Field Markers & Unknown Fields
When `FooView::from_bytes(buf)` is called:
- It performs a single $O(n)$ linear scan over the top-level fields in `buf` via a runtime `FieldIter` (`for field in FieldIter::new(buf)`).
- Scalar fields (integers, floats, booleans, enums) are decoded directly from `field.val` during this scan, stored in their native Rust types on the view struct, and tracked in a compact `has_fields: [u32; W]` bitmask, while length-delimited and group fields store a compact `FieldMarker` byte span into `buf`. Required fields are validated up front.
- Scalar accessors are `pub const fn` methods that return the stored value in $O(1)$ without re-reading `buf`, while borrowed accessors (`&'a str`, `&'a [u8]`) use their `FieldMarker` for $O(1)$ slice lookup into `buf`.
- Nested messages (`SubMessage`) are only recorded as a byte span during the parent scan; calling `.sub_message()` runs `SubMessageView::from_bytes(&self.buf[span])` on demand.
- **Unknown Fields**: Because `self.buf` holds the raw wire bytes, unknown fields are never discarded on read. Calling `.unknown_fields()` yields an iterator over raw wire spans for tags not in the schema, and `Builder::copy_unknown_fields(&view)` copies those raw tag+payload bytes directly into the backward writer.

### 2. Single-Pass Backward Serialization
In standard Protobuf wire format, a length-delimited field is encoded as:
```text
[tag (varint)] [byte_length (varint)] [payload bytes...]
```
When writing forward, you cannot write `byte_length` until you know the size of `payload bytes`, forcing either a two-pass size calculation or moving memory.

By filling a fixed-size buffer `&mut [u8]` **backwards** (decrementing a cursor from `buf.len()` down to `0`):
1. The payload bytes are written first at the end of the remaining buffer.
2. The exact number of bytes written (`end_cursor - current_cursor`) is immediately known.
3. The `byte_length` varint and `tag` varint are prepended directly in front of the payload.
4. When finished, `builder.finish()?` checks whether any write overflowed the buffer and returns the contiguous wire slice `&buf[cursor..]`.

#### Chained `&mut Self` Builders & Sticky Overflow Error Model
Generated builders construct a `BackwardWriter` internally via `Builder::new(&mut buf)` and use a sticky error state rather than returning `Result<(), EncodeError>` from every `.write_*()` call:
- **Ergonomic Chaining**: Returning `&mut Self` directly allows chaining multiple `.write_*()` and `.copy_unknown_fields()` calls in a row without `?` noise on every field.
- **Zero Closure Boilerplate for Nested Messages**: Because nested message closures take `FnOnce(&mut SubBuilder<'_>)` instead of returning `Result`, callers never have to append trailing `; Ok(())` boilerplate at the end of nested message closures.
- **Loop Guard for Large Repeated / Slice Writes**: Buffer overflow is most likely to occur when appending large sequences into a fixed-size buffer in a loop. Storing a sticky `EncodeError::BufferOverflow` on `BackwardWriter` (which short-circuits subsequent writes once full) and exposing `builder.check()` allows a clean loop guard (e.g., checking `builder.check()?` inside a loop or `while ... && builder.check().is_ok()`) to exit early as soon as the buffer fills, while still guaranteeing that `builder.finish()?` surfaces any unhandled overflow at the end.

---


## Authors opinions

### Why Protobuf?

I often see people argue for JSON instead of Protobuf; presenting the argument itself confuses the matter.
Protobuf has 2 parts:

1. The schema; these are .proto files that describe the structure of your data.
2. The wire format/data; this is the serialized binary representation of your data.

JSON is more similar to the wire format, but attempts to be human-readable by arguing that named keys are easier to
understand than the protobuf wire format. This argument doesn't hold up well at scale; consider projects like https://swagger.io/,
which provide a standardized way to document and visualize API schemas. The fact that these projects not only exist,
but have been very well recieved by the community, clearly demonstrate a need for better structure than free-form key-value pairs.

If we compare the wire formats, Protobuf is more compact and efficient than JSON. I would also argue that it is
easier to parse and understand, but that is just my opinion.

### Why not use existing libraries?

Here are my observations:

1. Often, we only need a handful of fields from messages when repacking proto. For example, we don't need to read the time field in a timeseries to perform multiplication
2. Making a full in-memory copy of the message can get expensive, especially if it has lots of small messages (see [arenas](https://protobuf.dev/reference/cpp/arenas/))
3. All existing protobuf libraries think in terms of data structures, instead of functions
4. Programmers are lazy, often correct, but with protobuf this leads you to spaghetti code faster than cooking pasta

The first point comes from my observations in my domain (capacity planning); it is not always true. I'm 100% sure there are cases where almost
every field is needed in 100% of the cases, but I also confident that there are far more cases where that is not true. And besides, I don't think
there is a substantial penalty to reading each of those fields as-needed than all at once.

The second point comes from observations around the effort spent optimizing allocations. I saw an argument that Google couldn't adopt existing Rust protobuf
libraries because they struggled with some of the internal formats that required gazillions of small allocations, and they needed arenas. A clever person might
point to Rust's ability to have different allocators, but even so, it just leads to more complexity.

The third point comes from what I've seen in Prost and the Go protobuf implementation (although they are trying to change this in Go!).
Prost generates a struct which is fully populated when the proto message is unmarshalled. That struct contains a copy of the data.
While this is an intuitive way to accomplish the goal, it fails to highlight the goal: extract data from a bunch of bytes.
In my thinking through the problem, to extract one field and avoid a O(n) scan, we need to write down the layout of the bytes,
but that's it. Then you consider that some data types (such as ints) are likely smaller than pointers/offsets, so we extract those.
Anyway, here we are.

Finally, the fourth point, I'm tired of reading tests that have hundreds of lines of protobuf fixture configuration. Libraries
that give us structs that are "just usable enough" to let us pass those structs around into our domain code lead to a problem
much faster than you'd think, but not fast enough that any competent engineer (including myself) that actively think to work
around it until we get sad about how terrible life is.

Roto solves these 4 problems:

1. We only read the fields we need, as best as we can with protobuf
2. We don't make a copy of the data. It's a single heap alloc that is free, and cleaned up for free
3. We do have a structure, but only to avoid to O(n) scan for every read
4. Unless you want to read the same field multiple times, you aught to make a struct to copy the data into

---

## Test Suite & Refactoring Reference

When modifying runtime contracts or codegen templates, inspect and update the corresponding test suites across the workspace:

- **[`roto-runtime/src/lib.rs`](roto-runtime/src/lib.rs) & [`roto-runtime/tests/readme_sensor.rs`](roto-runtime/tests/readme_sensor.rs)**: Core wire primitives, `FieldIter`, `Repeated*Iter` (including `RepeatedGroupIter`), `BackwardWriter`, sticky `EncodeError`, and hand-written reference `Location`/`SensorReading` views & builders.
- **[`roto-runtime/tests/fuzz_wire.rs`](roto-runtime/tests/fuzz_wire.rs)**: Deterministic PRNG wire fuzzer (`ROTO_FUZZ_SEED`, `ROTO_FUZZ_ITERS`) for `FieldIter`, all `Repeated*Iter` types, `UnknownFieldsIter`, and `BackwardWriter` round-tripping.
- **[`roto-grpc/tests/framing_and_status.rs`](roto-grpc/tests/framing_and_status.rs)**: 5-byte Length-Prefixed Message (LPM) framing, backward frame writer, `GrpcCode`/`GrpcStatus`, and RFC 3986 percent-encoding/decoding tests.
- **[`google-protobuf-roto-bootstrap/tests/protoc_descriptor_test.rs`](google-protobuf-roto-bootstrap/tests/protoc_descriptor_test.rs) & [`unit_roundtrip_test.rs`](google-protobuf-roto-bootstrap/tests/unit_roundtrip_test.rs)**: Thin compiler bootstrap descriptor (`descriptor.proto` + `compiler/plugin.proto`) round-trip and `protoc` descriptor set decoding tests.
- **[`google-protobuf-roto/tests/wkt_roundtrip_test.rs`](google-protobuf-roto/tests/wkt_roundtrip_test.rs), [`protoc_descriptor_test.rs`](google-protobuf-roto/tests/protoc_descriptor_test.rs), [`unit_roundtrip_test.rs`](google-protobuf-roto/tests/unit_roundtrip_test.rs)**: Full `google/protobuf` Well-Known Types (`Timestamp`, `Duration`, `Any`, `Empty`, `FieldMask`, `SourceContext`, recursive `Struct`/`Value`/`ListValue`, `Type`, `Api`, all 9 wrapper types, and edition features) round-trip tests.
- **`protoc-gen-roto/src/generator/{ident,symbols,enums,defaults,views,builders,modules}.rs`**: Unit tests for codegen string templates and exact expected Rust output snippets (update when changing generated `*View<'a>` or `*Builder<'b>` struct/method layouts).
- **[`protoc-gen-roto/tests/protoc_plugin_integration.rs`](protoc-gen-roto/tests/protoc_plugin_integration.rs) & [`protoc-gen-roto/tests/e2e_codegen.rs`](protoc-gen-roto/tests/e2e_codegen.rs)**: `CodeGeneratorRequest`/`CodeGeneratorResponse` framing, `sensor.proto`, comprehensive `proto2`/`proto3` schemas, `extern_path`, and self-hosting tests.
- **[`protoc-gen-roto/tests/data_types_matrix.rs`](protoc-gen-roto/tests/data_types_matrix.rs)**: Exhaustive 18-type matrix, boundary scalar values, all 12 `map<K, V>` key types, `group` framing, `oneof` overwrite semantics, boundary tag `536870911`, and bidirectional `protoc --encode`/`--decode` wire interoperability.
- **[`protoc-gen-roto/tests/rust_keywords_test.rs`](protoc-gen-roto/tests/rust_keywords_test.rs)**: All Rust strict, reserved, weak, and path keywords (`r#<kw>`, `self_`, `super_`, `crate_`, `Self_`) and internal method collision names across packages, messages, nested messages, fields, enums, and oneofs in `mode=lib` and `mode=mod`.
- **[`protoc-gen-roto/tests/ecosystem_packages_test.rs`](protoc-gen-roto/tests/ecosystem_packages_test.rs)**: Multi-package Google ecosystem schemas (`google/rpc/*`, `google/type/*`, `google/api/*`) importing `::google_protobuf_roto` via `extern_path`.
- **[`protoc-gen-roto/tests/fuzz_data.rs`](protoc-gen-roto/tests/fuzz_data.rs)**: Generated code data fuzzer (`unparse -> parse -> unparse`, `copy_unset_fields`, `copy_unknown_fields`, and mutated wire `parse -> unparse -> parse`).
- **[`protoc-gen-roto/tests/fuzz_descriptors.rs`](protoc-gen-roto/tests/fuzz_descriptors.rs)**: Random `proto2`/`proto3` multi-file descriptor generator (`RandomSchemaGenerator`), descriptor round-trip, codegen (`mode=lib` and `mode=mod`), and dynamic compiled data `unparse -> parse -> unparse` fuzzer.
- **[`cargo-roto/tests/cli_driver.rs`](cargo-roto/tests/cli_driver.rs) & [`cargo-roto/tests/e2e_packaging.rs`](cargo-roto/tests/e2e_packaging.rs)**: CLI flag parsing, `cargo metadata` `[package.metadata.roto]` resolution, cross-crate git dependency packaging, and `google-protobuf-roto-bootstrap` / `google-protobuf-roto` fixed-point self-hosting check.
- **[`tests/no_std_check/src/lib.rs`](tests/no_std_check/src/lib.rs)**: Bare-metal `#![no_std]` and zero-allocation target verification (`thumbv7em-none-eabihf`).
