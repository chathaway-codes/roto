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
4. When finished, the slice `&buf[cursor..]` contains the contiguous, valid `proto2` wire message.

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

pub struct LocationBuilder<'a, 'b> {
    writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> LocationBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    /// Copies all unknown fields preserved in `view` directly into the writer.
    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &LocationView<'_>) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_lon_e7(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(2, WireType::Varint)
    }

    #[inline]
    pub fn write_lat_e7(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(1, WireType::Varint)
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

pub struct SensorReadingBuilder<'a, 'b> {
    writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> SensorReadingBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    /// Copies all unknown fields preserved in `view` directly into the writer.
    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &SensorReadingView<'_>) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_sample(&mut self, sample: u32) -> Result<(), EncodeError> {
        self.writer.write_varint(sample as u64)?;
        self.writer.write_tag(4, WireType::Varint)
    }

    /// Writes a nested `Location` message in a single backward pass.
    /// The closure populates the child message first; once complete, the exact
    /// byte length is known and the length + tag varints are written immediately before it.
    #[inline]
    pub fn write_location<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut LocationBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = LocationBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(3, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_device_id(&mut self, device_id: &str) -> Result<(), EncodeError> {
        self.writer.write_bytes(device_id.as_bytes())?;
        self.writer.write_varint(device_id.len() as u64)?;
        self.writer.write_tag(2, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_timestamp_ms(&mut self, ts: u64) -> Result<(), EncodeError> {
        self.writer.write_varint(ts)?;
        self.writer.write_tag(1, WireType::Varint)
    }
}
```

### Usage Example: Single-Pass Write & Shallow Read

```rust
use roto::runtime::BackwardWriter;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    // 1. Serialize into a fixed-size stack buffer (no heap allocation, single pass)
    let mut backing = [0u8; 128];
    let mut writer = BackwardWriter::new(&mut backing);

    {
        let mut builder = SensorReadingBuilder::new(&mut writer);

        // Because the buffer fills backwards, fields written first appear at the
        // end of the wire buffer (or in any order valid in Protobuf).
        builder.write_sample(42)?;
        builder.write_sample(108)?;

        // Nested message is serialized inline; its byte length is prefixed automatically
        builder.write_location(|loc| {
            loc.write_lon_e7(-1224194000)?;
            loc.write_lat_e7(377749000)?;
            Ok(())
        })?;

        builder.write_device_id("sensor-node-01")?;
        builder.write_timestamp_ms(1_700_000_000_000)?;
    }

    // `wire_bytes` is the written tail slice `&backing[cursor..128]`
    let wire_bytes: &[u8] = writer.finish();

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

## Development & Workspace Commands

```bash
# Run verification suite
make test

# List open issues ready for pickup
make issues-ready

# Launch local read-only issue viewer UI
make issues-ui
```
