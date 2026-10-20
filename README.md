# Padding-Free Protobuf (`roto`)

A minimal, zero-allocation, zero-copy `proto2`-compatible Protocol Buffers reader, writer, and code generator for Rust `#[no_std]` environments.

Standard Protocol Buffers implementations deserialize wire bytes by copying fields into heap-allocated, compiler-padded Rust structs (`String`, `Vec<T>`, `Box<T>`) and require either two passes (compute byte length, then serialize) or intermediate buffers when writing nested messages.

**Padding-Free Protobuf** takes a fundamentally different approach:
1. **`#![no_std]` & Zero Heap Allocations**: Runs in bare-metal, embedded, and kernel environments without `alloc` or `std`.
2. **Zero-Copy View Structs**: Reading a message never copies field data into an intermediate struct with memory padding. Instead, it constructs a lightweight table of slice offsets/spans pointing into `&'a [u8]` and exposes typed accessor methods that decode directly from the underlying buffer.
3. **On-Demand Shallow Parsing**: Parsing a message performs a single $O(n)$ scan of its top-level wire tags using a runtime field iterator to record field markers. Nested sub-messages and repeated fields are not parsed eagerly; calling an accessor for a nested message constructs its view struct on demand over its recorded sub-slice.
4. **Single-Pass Backward Builder**: Serialization writes into a fixed-size `&mut [u8]` buffer **from back to front**. Because child fields and nested messages are written before their parent prefixes, exact byte lengths are always known at the moment a length-delimited prefix varint is emitted—enabling true single-pass encoding without backtracking, length pre-computation, or temporary buffers.
5. **Preserved Unknown Fields**: Because view structs retain the underlying wire slice `&'a [u8]` and know which field tags belong to the schema, unrecognized tags are naturally preserved in-place without extra memory overhead, and builders provide `copy_unknown_fields` to losslessly forward them during re-serialization.

---

## Architecture Overview

### 1. Shallow $O(n)$ Parsing, Field Markers & Unknown Fields
When `FooView::from_bytes(buf)` is called:
- It performs a single $O(n)$ linear scan over the top-level fields in `buf` via a runtime `FieldIter` (`for field in FieldIter::new(buf)`).
- For each recognized field tag, the view stores a compact **marker** (start/end byte offsets into `buf`, or presence/span markers) and validates required fields up front.
- Accessor methods use these markers for $O(1)$ lookup into `buf` to extract primitive values or return borrowed slices (`&'a str`, `&'a [u8]`) without re-scanning the message.
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
    RepeatedVarintIter, UnknownFieldsIter, WireType, decode_varint_at,
};

// ============================================================================
// Location (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct LocationView<'a> {
    buf: &'a [u8],
    lat_e7: FieldMarker,
    lon_e7: FieldMarker,
}

impl<'a> LocationView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

    /// Performs a single O(n) shallow scan over the top-level fields in `buf`,
    /// recording markers to locate fields and extract values on access.
    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut lat_e7 = FieldMarker::none();
        let mut lon_e7 = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::Varint) => lat_e7 = field.marker,
                (2, WireType::Varint) => lon_e7 = field.marker,
                _ => {} // Unknown fields remain in `buf` and are accessible via `unknown_fields()`
            }
        }

        if !lat_e7.is_present() {
            return Err(DecodeError::MissingRequiredField("Location.lat_e7"));
        }
        if !lon_e7.is_present() {
            return Err(DecodeError::MissingRequiredField("Location.lon_e7"));
        }

        Ok(Self { buf, lat_e7, lon_e7 })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    /// Extracts field 1 (`required int32 lat_e7`) from the marked offset in `self.buf`.
    #[inline]
    pub fn lat_e7(&self) -> Result<i32, DecodeError> {
        Ok(decode_varint_at(self.buf, self.lat_e7)? as i32)
    }

    /// Extracts field 2 (`required int32 lon_e7`) from the marked offset in `self.buf`.
    #[inline]
    pub fn lon_e7(&self) -> Result<i32, DecodeError> {
        Ok(decode_varint_at(self.buf, self.lon_e7)? as i32)
    }

    /// Iterates over raw wire spans (tag + payload) of unknown fields in `self.buf`.
    #[inline]
    pub fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

pub struct LocationBuilder<'b> {
    pub writer: BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> LocationBuilder<'b> {
    #[inline]
    pub fn new(buf: &'b mut [u8]) -> Self {
        Self {
            writer: BackwardWriter::new(buf),
            set_fields: [0u64; 1],
        }
    }

    #[inline]
    pub const fn from_writer(writer: BackwardWriter<'b>) -> Self {
        Self {
            writer,
            set_fields: [0u64; 1],
        }
    }

    #[inline]
    pub const fn writer(&self) -> &BackwardWriter<'b> {
        &self.writer
    }

    #[inline]
    pub fn writer_mut(&mut self) -> &mut BackwardWriter<'b> {
        &mut self.writer
    }

    /// Returns `Err(EncodeError::BufferOverflow)` if any write on the underlying
    /// writer has overflowed the buffer.
    #[inline]
    pub const fn check(&self) -> Result<(), EncodeError> {
        self.writer.check()
    }

    #[inline]
    pub fn finish(self) -> Result<&'b [u8], EncodeError> {
        self.writer.finish()
    }

    /// Copies all unknown fields preserved in `view` directly into the writer.
    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &LocationView<'_>) -> &mut Self {
        let _ = self
            .writer
            .copy_filtered_fields(view.as_bytes(), |tag| !LocationView::KNOWN_TAGS.contains(&tag));
        self
    }

    /// Copies all unset schema fields and unknown fields from `view` into the writer.
    #[inline]
    pub fn copy_unset_fields(&mut self, view: &LocationView<'_>) -> &mut Self {
        let prev_set = self.set_fields;
        let _ = self
            .writer
            .copy_filtered_fields(view.as_bytes(), |tag| match tag {
                1 => {
                    self.set_fields[0] |= 1;
                    (prev_set[0] & 1) == 0
                }
                2 => {
                    self.set_fields[0] |= 2;
                    (prev_set[0] & 2) == 0
                }
                _ => true,
            });
        self
    }

    #[inline]
    pub fn write_lon_e7(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(2, WireType::Varint);
        self
    }

    #[inline]
    pub fn write_lat_e7(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(1, WireType::Varint);
        self
    }
}

// ============================================================================
// SensorReading (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct SensorReadingView<'a> {
    buf: &'a [u8],
    timestamp_ms: FieldMarker,
    device_id: FieldMarker,
    location: FieldMarker,
}

impl<'a> SensorReadingView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4];

    /// Performs a single O(n) shallow scan over `buf` using the runtime field iterator.
    /// Nested messages (`location`) are not parsed here; only their byte span marker is stored.
    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut timestamp_ms = FieldMarker::none();
        let mut device_id = FieldMarker::none();
        let mut location = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::Varint) => timestamp_ms = field.marker,
                (2, WireType::LengthDelimited) => device_id = field.marker,
                (3, WireType::LengthDelimited) => location = field.marker,
                _ => {} // Repeated and unknown fields remain in `buf`
            }
        }

        if !timestamp_ms.is_present() {
            return Err(DecodeError::MissingRequiredField("SensorReading.timestamp_ms"));
        }

        Ok(Self {
            buf,
            timestamp_ms,
            device_id,
            location,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    /// Reads `required uint64 timestamp_ms = 1` using its stored marker.
    #[inline]
    pub fn timestamp_ms(&self) -> Result<u64, DecodeError> {
        decode_varint_at(self.buf, self.timestamp_ms)
    }

    /// Reads `optional string device_id = 2 [default = "unknown"]`.
    /// Zero-copy: slices `self.buf` directly using `self.device_id` span marker.
    #[inline]
    pub fn device_id(&self) -> Result<&'a str, DecodeError> {
        if !self.device_id.is_present() {
            return Ok("unknown");
        }
        let bytes = self.device_id.slice(self.buf);
        core::str::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8)
    }

    /// Reads `optional Location location = 3`.
    /// Shallow/on-demand: `LocationView::from_bytes` is only invoked when `.location()` is called.
    #[inline]
    pub fn location(&self) -> Result<Option<LocationView<'a>>, DecodeError> {
        if !self.location.is_present() {
            return Ok(None);
        }
        let sub_slice = self.location.slice(self.buf);
        LocationView::from_bytes(sub_slice).map(Some)
    }

    /// Returns a zero-allocation iterator over `repeated uint32 samples = 4`.
    #[inline]
    pub fn samples(&self) -> RepeatedVarintIter<'a, u32> {
        RepeatedVarintIter::new(self.buf, 4)
    }

    /// Iterates over raw wire spans (tag + payload) of unknown fields in `self.buf`.
    #[inline]
    pub fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

pub struct SensorReadingBuilder<'b> {
    pub writer: BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> SensorReadingBuilder<'b> {
    #[inline]
    pub fn new(buf: &'b mut [u8]) -> Self {
        Self {
            writer: BackwardWriter::new(buf),
            set_fields: [0u64; 1],
        }
    }

    #[inline]
    pub const fn from_writer(writer: BackwardWriter<'b>) -> Self {
        Self {
            writer,
            set_fields: [0u64; 1],
        }
    }

    #[inline]
    pub const fn writer(&self) -> &BackwardWriter<'b> {
        &self.writer
    }

    #[inline]
    pub fn writer_mut(&mut self) -> &mut BackwardWriter<'b> {
        &mut self.writer
    }

    /// Returns `Err(EncodeError::BufferOverflow)` if any write on the underlying
    /// writer has overflowed the buffer.
    #[inline]
    pub const fn check(&self) -> Result<(), EncodeError> {
        self.writer.check()
    }

    #[inline]
    pub fn finish(self) -> Result<&'b [u8], EncodeError> {
        self.writer.finish()
    }

    /// Copies all unknown fields preserved in `view` directly into the writer.
    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &SensorReadingView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| {
            !SensorReadingView::KNOWN_TAGS.contains(&tag)
        });
        self
    }

    /// Copies all unset schema fields and unknown fields from `view` into the writer.
    #[inline]
    pub fn copy_unset_fields(&mut self, view: &SensorReadingView<'_>) -> &mut Self {
        let prev_set = self.set_fields;
        let _ = self
            .writer
            .copy_filtered_fields(view.as_bytes(), |tag| match tag {
                1 => {
                    self.set_fields[0] |= 1;
                    (prev_set[0] & 1) == 0
                }
                2 => {
                    self.set_fields[0] |= 2;
                    (prev_set[0] & 2) == 0
                }
                3 => {
                    self.set_fields[0] |= 4;
                    (prev_set[0] & 4) == 0
                }
                4 => {
                    self.set_fields[0] |= 8;
                    (prev_set[0] & 8) == 0
                }
                _ => true,
            });
        self
    }

    #[inline]
    pub fn write_sample(&mut self, sample: u32) -> &mut Self {
        self.set_fields[0] |= 8;
        let _ = self.writer.write_varint(sample as u64);
        let _ = self.writer.write_tag(4, WireType::Varint);
        self
    }

    /// Writes a nested `Location` message in a single backward pass.
    /// The closure populates the child message first; once complete, the exact
    /// byte length is known and the length + tag varints are written immediately before it.
    #[inline]
    pub fn write_location<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut LocationBuilder<'_>),
    {
        self.set_fields[0] |= 4;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = LocationBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(3, WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_device_id(&mut self, device_id: &str) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_length_delimited(device_id.as_bytes());
        let _ = self.writer.write_tag(2, WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_timestamp_ms(&mut self, ts: u64) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_varint(ts);
        let _ = self.writer.write_tag(1, WireType::Varint);
        self
    }
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

### 4. Codegen Workflow (`protoc` Plugin & Metadata Helper)
Code generation uses `protoc` with two components:
1. **`protoc-gen-roto` (`protoc` plugin)**: Reads a serialized `CodeGeneratorRequest` from `stdin` and writes a `CodeGeneratorResponse` to `stdout`.
2. **`roto-build` (Metadata Helper)**: Prepares and invokes `protoc` for a crate by inspecting `Cargo.toml` dependencies.

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
3. **Invoke `protoc`**: Runs `protoc` with `--plugin=protoc-gen-roto`, passing the collected `-I` include flags and `extern_path` mappings so references to imported types emit `::google_protobuf_roto::google::protobuf::TimestampView<'a>` instead of regenerating imported schemas.

---

## CLI Usage (`cargo-roto`, `roto-build`, and `protoc-gen-roto`)

### 1. Generating a Crate with `cargo roto` or `roto-build`

Install the binaries locally or run them directly from the workspace:

```bash
# Install `cargo-roto` (`cargo roto`), `roto-build`, and `protoc-gen-roto`
cargo install --path cargo-roto --path protoc-gen-roto
```

Run inside any proto crate directory (or pass `--manifest-path`):

```bash
# Invoke as a Cargo subcommand (defaults to ./Cargo.toml, ./proto, and ./src)
cargo roto
cargo roto build

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
| `--mode <lib\|mod>` | Emit single-file `lib.rs` (`lib`) or multi-file `mod.rs` + per-package files (`mod`) | `lib` |
| `--offline` | Forward `--offline` to `cargo metadata` | `false` |
| `--locked` | Forward `--locked` to `cargo metadata` | `false` |
| `--frozen` | Forward `--frozen` to `cargo metadata` | `false` |
| `-h`, `--help` | Print CLI usage summary | — |

#### Environment Variables & Binary Discovery
- **`PROTOC`**: Explicit path to `protoc` (falls back to `protoc` on `$PATH`).
- **`PROTOC_GEN_ROTO`**: Explicit path to `protoc-gen-roto` (resolved in order: `--plugin <PATH>` $\to$ `$PROTOC_GEN_ROTO` $\to$ sibling binary next to `cargo-roto`/`roto-build` $\to$ `$PATH`).

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
| [`google-protobuf-roto-bootstrap`](google-protobuf-roto-bootstrap) | Thin self-hosted `#![no_std]` compiler bootstrap views and builders for `google/protobuf/descriptor.proto` and `google/protobuf/compiler/plugin.proto` |
| [`google-protobuf-roto`](google-protobuf-roto) | Full `#![no_std]` `google/protobuf` Well-Known Types (`Timestamp`, `Duration`, `Any`, `Empty`, `FieldMask`, `SourceContext`, `Struct`/`Value`/`ListValue`, `Type`, `Api`, wrappers, descriptors, plugin, and edition feature sets) |
| [`protoc-gen-roto`](protoc-gen-roto) | Zero-dependency `protoc` code generator plugin emitting `#![no_std]` `*View<'a>` and `*Builder<'b>` types |
| [`cargo-roto`](cargo-roto) | Zero-dependency Cargo subcommand (`cargo roto`) and `roto-build` CLI driver for `[package.metadata.roto]` resolution |
| [`tests/no_std_check`](tests/no_std_check) | Bare-metal `#![no_std]` + `no_alloc` verification crate cross-compiled for `thumbv7em-none-eabihf` |

---

## Development & Workspace Commands

```bash
# Run full verification suite (issue tracker, Rust unit/integration/e2e tests, and thumbv7em-none-eabihf no_std check)
make test

# Run only bare-metal thumbv7em-none-eabihf #![no_std] verification
make test-no-std

# List open issues ready for pickup
make issues-ready

# Launch local read-only issue viewer UI
make issues-ui
```

