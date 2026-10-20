#![no_std]
#![forbid(unsafe_code)]

use google_protobuf_roto::google::protobuf::{
    Edition, FileDescriptorSetBuilder, FileDescriptorSetView, SymbolVisibility,
    field_descriptor_proto::{Label, Type},
};
use roto::runtime::{
    BackwardWriter, DecodeError, EncodeError, FieldIter, FieldMarker, RepeatedVarintIter,
    UnknownFieldsIter, WireType, decode_varint_at,
};
use roto_runtime::{
    Field, MessageView, RepeatedBytesIter, RepeatedFixed32Iter, RepeatedFixed64Iter,
    RepeatedMessageIter, RepeatedStrIter, RepeatedZigZagIter, decode_bool_at, decode_bytes_at,
    decode_f32_at, decode_f64_at, decode_fixed32_at, decode_fixed64_at, decode_sfixed32_at,
    decode_sfixed64_at, decode_sint32_at, decode_sint64_at, decode_str_at, decode_varint,
    decode_zigzag32, decode_zigzag64, encode_zigzag32, encode_zigzag64,
};

#[cfg(all(not(test), target_os = "none"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

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

struct NoStdSubView<'a> {
    raw: &'a [u8],
}

impl<'a> MessageView<'a> for NoStdSubView<'a> {
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Ok(Self { raw: buf })
    }
}

/// Exercises `roto` and `roto-runtime` in a `#![no_std]`, zero-allocation environment.
pub fn verify_no_std() -> bool {
    if !(roto::runtime::runtime_ready() && roto_runtime::runtime_ready()) {
        return false;
    }

    let Ok(WireType::Varint) = WireType::from_u8(0) else {
        return false;
    };
    let Err(DecodeError::InvalidWireType(7)) = WireType::from_u8(7) else {
        return false;
    };
    let _enc_err = EncodeError::BufferOverflow;

    let none = FieldMarker::none();
    if none.is_present() || !none.is_empty() || !none.slice(&[1, 2]).is_empty() {
        return false;
    }

    let buf: &[u8] = &[
        0xAC, 0x02, // 0..2: varint 300
        0x01, // 2..3: bool true / zigzag -1
        0x78, 0x56, 0x34, 0x12, // 3..7: fixed32 0x12345678
        0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01, // 7..15: fixed64
        b'r', b'o', b't', b'o', // 15..19: "roto"
    ];

    let Ok((300, 2)) = decode_varint(buf) else {
        return false;
    };
    let Ok(300) = decode_varint_at(buf, FieldMarker::new(0, 2)) else {
        return false;
    };
    let Ok(true) = decode_bool_at(buf, FieldMarker::new(2, 3)) else {
        return false;
    };
    if decode_zigzag32(1) != -1
        || decode_zigzag64(2) != 1
        || encode_zigzag32(-1) != 1
        || encode_zigzag64(1) != 2
    {
        return false;
    }
    let Ok(-1) = decode_sint32_at(buf, FieldMarker::new(2, 3)) else {
        return false;
    };
    let Ok(-1) = decode_sint64_at(buf, FieldMarker::new(2, 3)) else {
        return false;
    };
    let Ok(0x12345678) = decode_fixed32_at(buf, FieldMarker::new(3, 7)) else {
        return false;
    };
    let Ok(0x12345678) = decode_sfixed32_at(buf, FieldMarker::new(3, 7)) else {
        return false;
    };
    let Ok(0x0123456789ABCDEF) = decode_fixed64_at(buf, FieldMarker::new(7, 15)) else {
        return false;
    };
    let Ok(0x0123456789ABCDEF) = decode_sfixed64_at(buf, FieldMarker::new(7, 15)) else {
        return false;
    };
    if decode_f32_at(buf, FieldMarker::new(3, 7)).is_err() {
        return false;
    }
    if decode_f64_at(buf, FieldMarker::new(7, 15)).is_err() {
        return false;
    }
    let Ok(b"roto") = decode_bytes_at(buf, FieldMarker::new(15, 19)) else {
        return false;
    };
    let Ok("roto") = decode_str_at(buf, FieldMarker::new(15, 19)) else {
        return false;
    };

    let wire_msg: &[u8] = &[
        0x08, 0x2A, // tag 1, Varint 42 (0..2)
        0x12, 0x04, b'r', b'o', b't', b'o', // tag 2, LengthDelimited "roto" (2..8)
        0x1B, 0x20, 0x01, 0x1C, // tag 3, StartGroup { tag 4, Varint 1 } EndGroup (8..12)
    ];
    let mut iter = FieldIter::new(wire_msg);
    let Some(Ok(Field {
        tag: 1,
        wire_type: WireType::Varint,
        marker: m1,
        raw_span: r1,
    })) = iter.next()
    else {
        return false;
    };
    if m1 != FieldMarker::new(1, 2) || r1 != FieldMarker::new(0, 2) {
        return false;
    }
    let Some(Ok(f2)) = iter.next() else {
        return false;
    };
    if f2.tag != 2 || f2.marker.slice(wire_msg) != b"roto" {
        return false;
    }
    let Some(Ok(f3)) = iter.next() else {
        return false;
    };
    if f3.tag != 3 || f3.wire_type != WireType::StartGroup || f3.marker != FieldMarker::new(9, 11) {
        return false;
    }
    if iter.next().is_some() {
        return false;
    }

    // Exercise repeated iterators and UnknownFieldsIter
    let repeated_buf: &[u8] = &[
        0x08, 0x01, // tag 1, Varint 1
        0x0A, 0x02, 0x02, 0x03, // tag 1, packed Varint [2, 3]
        0x10, 0x01, // tag 2, ZigZag -1
        0x1D, 1, 0, 0, 0, // tag 3, Fixed32 1
        0x21, 2, 0, 0, 0, 0, 0, 0, 0, // tag 4, Fixed64 2
        0x2A, 0x02, b'h', b'i', // tag 5, LengthDelimited "hi"
        0x38, 0x2A, // tag 7 (unknown), Varint 42
    ];

    let mut varints = RepeatedVarintIter::<u32>::new(repeated_buf, 1);
    if varints.next() != Some(Ok(1))
        || varints.next() != Some(Ok(2))
        || varints.next() != Some(Ok(3))
        || varints.next().is_some()
    {
        return false;
    }

    let mut zigzags = RepeatedZigZagIter::<i32>::new(repeated_buf, 2);
    if zigzags.next() != Some(Ok(-1)) || zigzags.next().is_some() {
        return false;
    }

    let mut fixed32s = RepeatedFixed32Iter::<u32>::new(repeated_buf, 3);
    if fixed32s.next() != Some(Ok(1)) || fixed32s.next().is_some() {
        return false;
    }

    let mut fixed64s = RepeatedFixed64Iter::<u64>::new(repeated_buf, 4);
    if fixed64s.next() != Some(Ok(2)) || fixed64s.next().is_some() {
        return false;
    }

    let mut bytes_iter = RepeatedBytesIter::new(repeated_buf, 5);
    if bytes_iter.next() != Some(Ok(&b"hi"[..])) || bytes_iter.next().is_some() {
        return false;
    }

    let mut str_iter = RepeatedStrIter::new(repeated_buf, 5);
    if str_iter.next() != Some(Ok("hi")) || str_iter.next().is_some() {
        return false;
    }

    let mut msg_iter = RepeatedMessageIter::<NoStdSubView<'_>>::new(repeated_buf, 5);
    let Some(Ok(sub)) = msg_iter.next() else {
        return false;
    };
    if sub.raw != b"hi" || msg_iter.next().is_some() {
        return false;
    }

    let mut unknown = UnknownFieldsIter::new(repeated_buf, &[1, 2, 3, 4, 5]);
    if unknown.next() != Some(&[0x38, 0x2A][..]) || unknown.next().is_some() {
        return false;
    }

    // Exercise all BackwardWriter methods directly
    let mut prim_buf = [0u8; 64];
    let mut prim_writer = BackwardWriter::new(&mut prim_buf);
    if prim_writer.cursor() != 64
        || prim_writer.written_len() != 0
        || prim_writer.remaining_capacity() != 64
        || !prim_writer.as_written_slice().is_empty()
    {
        return false;
    }
    if prim_writer.write_length_delimited(b"ok").is_err()
        || prim_writer.write_f64(1.5).is_err()
        || prim_writer.write_f32(2.5).is_err()
        || prim_writer.write_sfixed64(-10).is_err()
        || prim_writer.write_sfixed32(-20).is_err()
        || prim_writer.write_fixed64(10).is_err()
        || prim_writer.write_fixed32(20).is_err()
        || prim_writer.write_zigzag64(-30).is_err()
        || prim_writer.write_zigzag32(-40).is_err()
        || prim_writer.write_bool(true).is_err()
    {
        return false;
    }
    if prim_writer.written_len() == 0 || prim_writer.finish().is_empty() {
        return false;
    }

    // Full end-to-end Location / SensorReading roundtrip with unknown field forwarding
    let mut backing1 = [0u8; 128];
    let mut writer1 = BackwardWriter::new(&mut backing1);
    {
        // Unknown field 77 on SensorReading
        if writer1.write_varint(999).is_err() || writer1.write_tag(77, WireType::Varint).is_err() {
            return false;
        }

        let mut builder = SensorReadingBuilder::new(&mut writer1);
        if builder.write_sample(108).is_err() || builder.write_sample(42).is_err() {
            return false;
        }
        if builder
            .write_location(|loc| {
                // Unknown field 55 on Location
                loc.writer.write_varint(777)?;
                loc.writer.write_tag(55, WireType::Varint)?;
                loc.write_lon_e7(-1224194000)?;
                loc.write_lat_e7(377749000)?;
                Ok(())
            })
            .is_err()
        {
            return false;
        }
        if builder.write_device_id("sensor-node-01").is_err()
            || builder.write_timestamp_ms(1_700_000_000_000).is_err()
        {
            return false;
        }
    }
    let wire_bytes1 = writer1.finish();

    let Ok(reading1) = SensorReadingView::from_bytes(wire_bytes1) else {
        return false;
    };
    let Ok(Some(loc1)) = reading1.location() else {
        return false;
    };

    // Re-encode with unknown fields forwarded
    let mut backing2 = [0u8; 128];
    let mut writer2 = BackwardWriter::new(&mut backing2);
    {
        let mut builder = SensorReadingBuilder::new(&mut writer2);
        if builder.copy_unknown_fields(&reading1).is_err()
            || builder.write_sample(108).is_err()
            || builder.write_sample(42).is_err()
        {
            return false;
        }
        if builder
            .write_location(|loc| {
                loc.copy_unknown_fields(&loc1)?;
                loc.write_lon_e7(-1224194000)?;
                loc.write_lat_e7(377749000)?;
                Ok(())
            })
            .is_err()
        {
            return false;
        }
        if builder.write_device_id("sensor-node-01").is_err()
            || builder.write_timestamp_ms(1_700_000_000_000).is_err()
        {
            return false;
        }
    }
    let wire_bytes2 = writer2.finish();

    let Ok(reading2) = SensorReadingView::from_bytes(wire_bytes2) else {
        return false;
    };
    if reading2.timestamp_ms() != Ok(1_700_000_000_000)
        || reading2.device_id() != Ok("sensor-node-01")
    {
        return false;
    }
    let Ok(Some(loc2)) = reading2.location() else {
        return false;
    };
    if loc2.lat_e7() != Ok(377749000) || loc2.lon_e7() != Ok(-1224194000) {
        return false;
    }
    let mut samples = reading2.samples();
    if samples.next() != Some(Ok(42)) || samples.next() != Some(Ok(108)) || samples.next().is_some()
    {
        return false;
    }
    if reading2.unknown_fields().count() != 1 || loc2.unknown_fields().count() != 1 {
        return false;
    }

    // Exercise google_protobuf_roto descriptor Views, Builders, and Enums in #![no_std]
    let mut desc_backing = [0u8; 256];
    let mut desc_writer = BackwardWriter::new(&mut desc_backing);
    {
        let mut fds = FileDescriptorSetBuilder::new(&mut desc_writer);
        if fds
            .write_file(|file| {
                file.write_edition(Edition::Edition2023)?;
                file.write_syntax("editions")?;
                file.write_service(|svc| {
                    svc.write_method(|m| {
                        m.write_server_streaming(true)?;
                        m.write_client_streaming(false)?;
                        m.write_output_type(".demo.SensorReading")?;
                        m.write_input_type(".demo.Location")?;
                        m.write_name("StreamReadings")
                    })?;
                    svc.write_name("SensorService")
                })?;
                file.write_enum_type(|en| {
                    en.write_visibility(SymbolVisibility::VisibilityExport)?;
                    en.write_value(|v| {
                        v.write_number(0)?;
                        v.write_name("STATUS_OK")
                    })?;
                    en.write_name("Status")
                })?;
                file.write_message_type(|msg| {
                    msg.write_visibility(SymbolVisibility::VisibilityLocal)?;
                    msg.write_field(|fld| {
                        fld.write_proto3_optional(true)?;
                        fld.write_json_name("timestampMs")?;
                        fld.write_default_value("0")?;
                        fld.write_type(Type::TypeUint64)?;
                        fld.write_label(Label::LabelRequired)?;
                        fld.write_number(1)?;
                        fld.write_name("timestamp_ms")
                    })?;
                    msg.write_name("SensorReading")
                })?;
                file.write_package("demo")?;
                file.write_name("sensor.proto")
            })
            .is_err()
        {
            return false;
        }
    }
    let desc_bytes = desc_writer.finish();
    let Ok(fds_view) = FileDescriptorSetView::from_bytes(desc_bytes) else {
        return false;
    };
    let mut files = fds_view.file();
    let Some(Ok(file_view)) = files.next() else {
        return false;
    };
    if files.next().is_some()
        || file_view.name() != Ok("sensor.proto")
        || file_view.package() != Ok("demo")
        || file_view.syntax() != Ok("editions")
        || file_view.edition() != Ok(Edition::Edition2023)
    {
        return false;
    }
    let mut msgs = file_view.message_type();
    let Some(Ok(msg_view)) = msgs.next() else {
        return false;
    };
    if msgs.next().is_some()
        || msg_view.name() != Ok("SensorReading")
        || msg_view.visibility() != Ok(SymbolVisibility::VisibilityLocal)
    {
        return false;
    }
    let mut fields = msg_view.field();
    let Some(Ok(field_view)) = fields.next() else {
        return false;
    };
    if fields.next().is_some()
        || field_view.name() != Ok("timestamp_ms")
        || field_view.number() != Ok(1)
        || field_view.label() != Ok(Label::LabelRequired)
        || field_view.r#type() != Ok(Type::TypeUint64)
        || field_view.default_value() != Ok("0")
        || field_view.json_name() != Ok("timestampMs")
        || field_view.proto3_optional() != Ok(true)
    {
        return false;
    }
    let mut enums = file_view.enum_type();
    let Some(Ok(enum_view)) = enums.next() else {
        return false;
    };
    if enums.next().is_some()
        || enum_view.name() != Ok("Status")
        || enum_view.visibility() != Ok(SymbolVisibility::VisibilityExport)
    {
        return false;
    }
    let mut svcs = file_view.service();
    let Some(Ok(svc_view)) = svcs.next() else {
        return false;
    };
    if svcs.next().is_some() || svc_view.name() != Ok("SensorService") {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_std_harness_smoke_test() {
        assert!(verify_no_std());
    }
}
