#![no_std]
#![forbid(unsafe_code)]

mod roto {
    pub use roto_runtime as runtime;
}

use roto::runtime::{
    BackwardWriter, DecodeError, EncodeError, FieldIter, FieldMarker, RepeatedVarintIter,
    UnknownFieldsIter, WireType, decode_varint_at,
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

        Ok(Self {
            buf,
            lat_e7,
            lon_e7,
        })
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

    #[inline]
    pub const fn check(&self) -> Result<(), EncodeError> {
        self.writer.check()
    }

    /// Copies all unknown fields preserved in `view` directly into the writer.
    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &LocationView<'_>) -> &mut Self {
        for raw_field in view.unknown_fields() {
            let _ = self.writer.write_bytes(raw_field);
        }
        self
    }

    #[inline]
    pub fn write_lon_e7(&mut self, val: i32) -> &mut Self {
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(2, WireType::Varint);
        self
    }

    #[inline]
    pub fn write_lat_e7(&mut self, val: i32) -> &mut Self {
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
            return Err(DecodeError::MissingRequiredField(
                "SensorReading.timestamp_ms",
            ));
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

    #[inline]
    pub const fn check(&self) -> Result<(), EncodeError> {
        self.writer.check()
    }

    /// Copies all unknown fields preserved in `view` directly into the writer.
    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &SensorReadingView<'_>) -> &mut Self {
        for raw_field in view.unknown_fields() {
            let _ = self.writer.write_bytes(raw_field);
        }
        self
    }

    #[inline]
    pub fn write_sample(&mut self, sample: u32) -> &mut Self {
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
        F: FnOnce(&mut LocationBuilder<'_, 'b>),
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = LocationBuilder::new(self.writer);
            f(&mut sub_builder);
        }
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(3, WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_device_id(&mut self, device_id: &str) -> &mut Self {
        let _ = self.writer.write_bytes(device_id.as_bytes());
        let _ = self.writer.write_varint(device_id.len() as u64);
        let _ = self.writer.write_tag(2, WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_timestamp_ms(&mut self, ts: u64) -> &mut Self {
        let _ = self.writer.write_varint(ts);
        let _ = self.writer.write_tag(1, WireType::Varint);
        self
    }
}

#[test]
fn readme_usage_example_roundtrip() {
    // 1a. Exact README.md usage example call sequence with method chaining
    {
        let mut backing = [0u8; 128];
        let mut writer = BackwardWriter::new(&mut backing);

        {
            let mut builder = SensorReadingBuilder::new(&mut writer);

            builder
                .write_sample(42)
                .write_sample(108)
                .write_location(|loc| {
                    loc.write_lon_e7(-1224194000)
                        .write_lat_e7(377749000);
                })
                .write_device_id("sensor-node-01")
                .write_timestamp_ms(1_700_000_000_000);
            assert_eq!(builder.check(), Ok(()));
        }

        let wire_bytes: &[u8] = writer.finish().unwrap();

        let reading = SensorReadingView::from_bytes(wire_bytes).unwrap();
        assert_eq!(reading.timestamp_ms(), Ok(1_700_000_000_000));
        assert_eq!(reading.device_id(), Ok("sensor-node-01"));

        let loc = reading.location().unwrap().expect("location present");
        assert_eq!(loc.lat_e7(), Ok(377749000));
        assert_eq!(loc.lon_e7(), Ok(-1224194000));

        // Because backward writing prepends each field, write_sample(42) then write_sample(108)
        // places 108 before 42 in wire order.
        let mut samples = reading.samples();
        assert_eq!(samples.next(), Some(Ok(108)));
        assert_eq!(samples.next(), Some(Ok(42)));
        assert_eq!(samples.next(), None);
    }

    // 1b. Writing samples in reverse order (108 then 42) so wire order is [42, 108]
    {
        let mut backing = [0u8; 128];
        let mut writer = BackwardWriter::new(&mut backing);

        {
            let mut builder = SensorReadingBuilder::new(&mut writer);

            builder
                .write_sample(108)
                .write_sample(42)
                .write_location(|loc| {
                    loc.write_lon_e7(-1224194000)
                        .write_lat_e7(377749000);
                })
                .write_device_id("sensor-node-01")
                .write_timestamp_ms(1_700_000_000_000);
        }

        let wire_bytes: &[u8] = writer.finish().unwrap();

        let reading = SensorReadingView::from_bytes(wire_bytes).unwrap();
        assert_eq!(reading.timestamp_ms(), Ok(1_700_000_000_000));
        assert_eq!(reading.device_id(), Ok("sensor-node-01"));

        let loc = reading.location().unwrap().expect("location present");
        assert_eq!(loc.lat_e7(), Ok(377749000));
        assert_eq!(loc.lon_e7(), Ok(-1224194000));

        let mut samples = reading.samples();
        assert_eq!(samples.next(), Some(Ok(42)));
        assert_eq!(samples.next(), Some(Ok(108)));
        assert_eq!(samples.next(), None);
        assert_eq!(reading.unknown_fields().next(), None);
        assert_eq!(loc.unknown_fields().next(), None);
    }
}

#[test]
fn required_field_enforcement() {
    // Missing Location.lat_e7 (only lon_e7 written)
    {
        let mut buf = [0u8; 32];
        let mut writer = BackwardWriter::new(&mut buf);
        let mut loc = LocationBuilder::new(&mut writer);
        loc.write_lon_e7(-1224194000);
        let wire = writer.finish().unwrap();
        assert_eq!(
            LocationView::from_bytes(wire).unwrap_err(),
            DecodeError::MissingRequiredField("Location.lat_e7")
        );
    }

    // Missing Location.lon_e7 (only lat_e7 written)
    {
        let mut buf = [0u8; 32];
        let mut writer = BackwardWriter::new(&mut buf);
        let mut loc = LocationBuilder::new(&mut writer);
        loc.write_lat_e7(377749000);
        let wire = writer.finish().unwrap();
        assert_eq!(
            LocationView::from_bytes(wire).unwrap_err(),
            DecodeError::MissingRequiredField("Location.lon_e7")
        );
    }

    // Missing SensorReading.timestamp_ms (only device_id written)
    {
        let mut buf = [0u8; 32];
        let mut writer = BackwardWriter::new(&mut buf);
        let mut builder = SensorReadingBuilder::new(&mut writer);
        builder.write_device_id("node-x");
        let wire = writer.finish().unwrap();
        assert_eq!(
            SensorReadingView::from_bytes(wire).unwrap_err(),
            DecodeError::MissingRequiredField("SensorReading.timestamp_ms")
        );
    }
}

#[test]
fn default_value_fallback_when_optional_fields_omitted() {
    let mut buf = [0u8; 32];
    let mut writer = BackwardWriter::new(&mut buf);
    {
        let mut builder = SensorReadingBuilder::new(&mut writer);
        builder.write_timestamp_ms(999);
    }
    let wire = writer.finish().unwrap();

    let reading = SensorReadingView::from_bytes(wire).unwrap();
    assert_eq!(reading.timestamp_ms(), Ok(999));
    assert_eq!(reading.device_id(), Ok("unknown"));
    assert!(reading.location().unwrap().is_none());
    assert_eq!(reading.samples().next(), None);
}

#[test]
fn lazy_nested_message_view_defers_inner_validation() {
    // Construct a SensorReading whose nested Location (field 3) is missing `lon_e7`
    let mut buf = [0u8; 64];
    let mut writer = BackwardWriter::new(&mut buf);
    {
        let mut builder = SensorReadingBuilder::new(&mut writer);
        builder
            .write_location(|loc| {
                // Only write lat_e7; omit required lon_e7!
                loc.write_lat_e7(377749000);
            })
            .write_timestamp_ms(12345);
    }
    let wire = writer.finish().unwrap();

    // Shallow top-level scan succeeds without parsing inner Location
    let reading = SensorReadingView::from_bytes(wire).unwrap();
    assert_eq!(reading.timestamp_ms(), Ok(12345));
    assert_eq!(reading.device_id(), Ok("unknown"));

    // Calling .location() triggers LocationView::from_bytes on demand and surfaces the inner error
    assert_eq!(
        reading.location().unwrap_err(),
        DecodeError::MissingRequiredField("Location.lon_e7")
    );
}

#[test]
fn repeated_varint_iteration_unpacked_and_packed() {
    let mut buf = [0u8; 64];
    let mut writer = BackwardWriter::new(&mut buf);

    // Unpacked sample = 400 (tag 4, Varint)
    {
        let mut builder = SensorReadingBuilder::new(&mut writer);
        builder.write_sample(400);
    }
    // Packed samples [200, 300] (tag 4, LengthDelimited)
    {
        let end_pos = writer.cursor();
        writer.write_varint(300).unwrap();
        writer.write_varint(200).unwrap();
        let packed_len = end_pos - writer.cursor();
        writer.write_varint(packed_len as u64).unwrap();
        writer.write_tag(4, WireType::LengthDelimited).unwrap();
    }
    // Unpacked sample = 100 (tag 4, Varint) and required timestamp_ms
    {
        let mut builder = SensorReadingBuilder::new(&mut writer);
        builder.write_sample(100).write_timestamp_ms(1);
    }

    let wire = writer.finish().unwrap();
    let reading = SensorReadingView::from_bytes(wire).unwrap();
    let mut samples = reading.samples();
    assert_eq!(samples.next(), Some(Ok(100)));
    assert_eq!(samples.next(), Some(Ok(200)));
    assert_eq!(samples.next(), Some(Ok(300)));
    assert_eq!(samples.next(), Some(Ok(400)));
    assert_eq!(samples.next(), None);
}

#[test]
fn unknown_fields_preservation_and_forwarding_roundtrip() {
    // 1. Build a message containing unknown fields in both Location and SensorReading
    let mut buf1 = [0u8; 128];
    let mut writer1 = BackwardWriter::new(&mut buf1);
    {
        // Unknown field 99 (Fixed32 = 0xCAFEBABE) at end of SensorReading
        writer1.write_fixed32(0xCAFEBABE).unwrap();
        writer1.write_tag(99, WireType::Fixed32).unwrap();

        let mut builder = SensorReadingBuilder::new(&mut writer1);
        builder
            .write_sample(7)
            .write_location(|loc| {
                // Unknown field 50 (Varint = 123456) inside Location
                loc.writer.write_varint(123456).unwrap();
                loc.writer.write_tag(50, WireType::Varint).unwrap();
                loc.write_lon_e7(-200).write_lat_e7(100);
            });
        // Unknown field 88 (LengthDelimited = b"extra") in SensorReading
        builder.writer.write_length_delimited(b"extra").unwrap();
        builder
            .writer
            .write_tag(88, WireType::LengthDelimited)
            .unwrap();
        builder.write_timestamp_ms(555);
    }
    let wire1 = writer1.finish().unwrap();

    // 2. Decode into views and verify unknown fields are visible
    let reading1 = SensorReadingView::from_bytes(wire1).unwrap();
    let loc1 = reading1.location().unwrap().unwrap();
    assert_eq!(loc1.lat_e7(), Ok(100));
    assert_eq!(loc1.lon_e7(), Ok(-200));
    assert_eq!(loc1.unknown_fields().count(), 1);
    assert_eq!(reading1.unknown_fields().count(), 2);

    // 3. Re-serialize into a new buffer forwarding all unknown fields
    let mut buf2 = [0u8; 128];
    let mut writer2 = BackwardWriter::new(&mut buf2);
    {
        let mut builder = SensorReadingBuilder::new(&mut writer2);
        builder
            .copy_unknown_fields(&reading1)
            .write_sample(7)
            .write_location(|loc| {
                loc.copy_unknown_fields(&loc1)
                    .write_lon_e7(loc1.lon_e7().unwrap())
                    .write_lat_e7(loc1.lat_e7().unwrap());
            })
            .write_timestamp_ms(reading1.timestamp_ms().unwrap());
    }
    let wire2 = writer2.finish().unwrap();

    // 4. Verify re-encoded message preserves all known and unknown fields
    let reading2 = SensorReadingView::from_bytes(wire2).unwrap();
    assert_eq!(reading2.timestamp_ms(), Ok(555));
    let loc2 = reading2.location().unwrap().unwrap();
    assert_eq!(loc2.lat_e7(), Ok(100));
    assert_eq!(loc2.lon_e7(), Ok(-200));

    let mut loc2_unk = loc2.unknown_fields();
    let loc_raw = loc2_unk.next().unwrap();
    assert_eq!(loc2_unk.next(), None);
    let mut loc_iter = FieldIter::new(loc_raw);
    let f50 = loc_iter.next().unwrap().unwrap();
    assert_eq!((f50.tag, f50.wire_type), (50, WireType::Varint));
    assert_eq!(decode_varint_at(loc_raw, f50.marker), Ok(123456));

    // Note: copy_unknown_fields writes unknown fields in forward iteration order into the backward
    // writer, so both unknown fields (88 and 99) are present in reading2.
    assert_eq!(reading2.unknown_fields().count(), 2);
}

#[test]
fn builder_buffer_overflow_handling() {
    // Buffer too small for LocationBuilder
    {
        let mut small = [0u8; 2];
        let mut writer = BackwardWriter::new(&mut small);
        {
            let mut loc = LocationBuilder::new(&mut writer);
            // Negative int32 varint requires 10 bytes + 1 byte tag = 11 bytes
            loc.write_lon_e7(-1224194000);
            assert_eq!(loc.check(), Err(EncodeError::BufferOverflow));
        }
        assert_eq!(writer.finish(), Err(EncodeError::BufferOverflow));
    }

    // Buffer too small for SensorReadingBuilder nested location prefix or device_id
    {
        let mut small = [0u8; 10];
        let mut writer = BackwardWriter::new(&mut small);
        {
            let mut builder = SensorReadingBuilder::new(&mut writer);
            builder.write_device_id("sensor-node-01");
            assert_eq!(builder.check(), Err(EncodeError::BufferOverflow));
        }
        assert_eq!(writer.finish(), Err(EncodeError::BufferOverflow));
    }

    // Buffer fits inner Location fields (6 bytes: 2 bytes for lat_e7=1 + 2 bytes for lon_e7=2 -> 4 bytes)
    // but overflows when writing outer length + tag (needs 6 bytes total, give 5 bytes)
    {
        let mut tight = [0u8; 5];
        let mut writer = BackwardWriter::new(&mut tight);
        {
            let mut builder = SensorReadingBuilder::new(&mut writer);
            builder.write_location(|loc| {
                loc.write_lon_e7(2).write_lat_e7(1);
            });
            assert_eq!(builder.check(), Err(EncodeError::BufferOverflow));
        }
        assert_eq!(writer.finish(), Err(EncodeError::BufferOverflow));
    }

    // Loop using builder.check() stops early on buffer overflow
    {
        let mut small = [0u8; 6];
        let mut writer = BackwardWriter::new(&mut small);
        let mut count = 0;
        let res: Result<&[u8], EncodeError> = (|| {
            let mut builder = SensorReadingBuilder::new(&mut writer);
            for sample in 1..=10 {
                count += 1;
                builder.write_sample(sample).check()?;
            }
            builder.write_timestamp_ms(1);
            writer.finish()
        })();
        assert_eq!(res, Err(EncodeError::BufferOverflow));
        assert_eq!(count, 4);
    }
}
