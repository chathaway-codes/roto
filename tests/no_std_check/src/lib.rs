#![no_std]
#![forbid(unsafe_code)]

use google_protobuf_roto::google::protobuf::{
    compiler::{
        code_generator_response::{Feature, FileBuilder, FileView},
        CodeGeneratorRequestBuilder, CodeGeneratorRequestView, CodeGeneratorResponseBuilder,
        CodeGeneratorResponseView, VersionBuilder, VersionView,
    },
    field_descriptor_proto::{Label, Type},
    generated_code_info::annotation::Semantic,
    Edition, FileDescriptorSetBuilder, FileDescriptorSetView, SymbolVisibility,
};
use roto::runtime::{
    decode_varint_at, BackwardWriter, DecodeError, EncodeError, FieldIter, FieldMarker,
    RepeatedVarintIter, UnknownFieldsIter, WireType,
};
use roto_runtime::{
    decode_bool_at, decode_bytes_at, decode_f32_at, decode_f64_at, decode_fixed32_at,
    decode_fixed64_at, decode_sfixed32_at, decode_sfixed64_at, decode_sint32_at, decode_sint64_at,
    decode_str_at, decode_varint, decode_zigzag32, decode_zigzag64, encode_zigzag32,
    encode_zigzag64, Field, MessageView, RepeatedBytesIter, RepeatedFixed32Iter,
    RepeatedFixed64Iter, RepeatedMessageIter, RepeatedStrIter, RepeatedZigZagIter,
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

impl<'a> MessageView<'a> for LocationView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct LocationBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> LocationBuilder<'b> {
    #[inline]
    pub fn new(buf: &'b mut [u8]) -> Self {
        Self {
            writer: ::roto::runtime::BackwardWriter::new(buf),
            set_fields: [0u64; 1],
        }
    }

    #[inline]
    pub const fn from_writer(writer: ::roto::runtime::BackwardWriter<'b>) -> Self {
        Self {
            writer,
            set_fields: [0u64; 1],
        }
    }

    #[inline]
    pub const fn writer(&self) -> &::roto::runtime::BackwardWriter<'b> {
        &self.writer
    }

    #[inline]
    pub fn writer_mut(&mut self) -> &mut ::roto::runtime::BackwardWriter<'b> {
        &mut self.writer
    }

    #[inline]
    pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.check()
    }

    #[inline]
    pub fn finish(self) -> Result<&'b [u8], ::roto::runtime::EncodeError> {
        self.writer.finish()
    }

    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &LocationView<'_>) -> &mut Self {
        let _ = self
            .writer
            .copy_filtered_fields(view.as_bytes(), |tag| !LocationView::KNOWN_TAGS.contains(&tag));
        self
    }

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
    pub fn write_lat_e7(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_lon_e7(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
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
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> SensorReadingBuilder<'b> {
    #[inline]
    pub fn new(buf: &'b mut [u8]) -> Self {
        Self {
            writer: ::roto::runtime::BackwardWriter::new(buf),
            set_fields: [0u64; 1],
        }
    }

    #[inline]
    pub const fn from_writer(writer: ::roto::runtime::BackwardWriter<'b>) -> Self {
        Self {
            writer,
            set_fields: [0u64; 1],
        }
    }

    #[inline]
    pub const fn writer(&self) -> &::roto::runtime::BackwardWriter<'b> {
        &self.writer
    }

    #[inline]
    pub fn writer_mut(&mut self) -> &mut ::roto::runtime::BackwardWriter<'b> {
        &mut self.writer
    }

    #[inline]
    pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.check()
    }

    #[inline]
    pub fn finish(self) -> Result<&'b [u8], ::roto::runtime::EncodeError> {
        self.writer.finish()
    }

    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &SensorReadingView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| {
            !SensorReadingView::KNOWN_TAGS.contains(&tag)
        });
        self
    }

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
    pub fn write_timestamp_ms(&mut self, val: u64) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_varint(val);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_device_id(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self
            .writer
            .write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }

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
        let _ = self
            .writer
            .write_tag(3, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_samples(&mut self, val: u32) -> &mut Self {
        self.set_fields[0] |= 8;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_sample(&mut self, val: u32) -> &mut Self {
        self.write_samples(val)
    }

    #[inline]
    pub fn write_packed_samples(&mut self, vals: &[u32]) -> &mut Self {
        self.set_fields[0] |= 8;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_varint(val as u64);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self
            .writer
            .write_tag(4, ::roto::runtime::WireType::LengthDelimited);
        self
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

// ============================================================================
// Generated Enum (with allow_alias = true) & Oneof Constructs
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DeviceMode {
    ModeIdle = 0,
    ModeActive = 1,
    ModeSleep = 2,
}

impl DeviceMode {
    #[allow(non_upper_case_globals)]
    pub const ModeRunning: Self = Self::ModeActive;
    #[allow(non_upper_case_globals)]
    pub const ModeStandby: Self = Self::ModeSleep;

    #[inline]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::ModeIdle),
            1 => Some(Self::ModeActive),
            2 => Some(Self::ModeSleep),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

impl core::convert::TryFrom<i32> for DeviceMode {
    type Error = i32;

    #[inline]
    fn try_from(val: i32) -> Result<Self, Self::Error> {
        Self::from_i32(val).ok_or(val)
    }
}

impl ::roto::runtime::VarintValue for DeviceMode {
    #[inline]
    fn from_raw_varint(val: u64) -> Self {
        Self::from_i32(val as i32).unwrap_or(Self::ModeIdle)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SensorPayload<'a> {
    Text(&'a str),
    RawBytes(&'a [u8]),
    Location(LocationView<'a>),
    Code(i32),
    Mode(DeviceMode),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AlertThreshold {
    MaxCount(u32),
    DeltaNs(i64),
    MaxTempC(f32),
    MaxRatio(f64),
    Enabled(bool),
    TargetMode(DeviceMode),
}

// ============================================================================
// AllTypes (All 18 Field Types, Unpacked & Packed Repeated, Nested & Unknowns)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct AllTypesView<'a> {
    buf: &'a [u8],
}

impl<'a> AllTypesView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
    ];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        for item in FieldIter::new(buf) {
            let _field = item?;
        }
        Ok(Self { buf })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn f_double(&self) -> RepeatedFixed64Iter<'a, f64> {
        RepeatedFixed64Iter::new(self.buf, 1)
    }

    #[inline]
    pub const fn f_float(&self) -> RepeatedFixed32Iter<'a, f32> {
        RepeatedFixed32Iter::new(self.buf, 2)
    }

    #[inline]
    pub const fn f_int64(&self) -> RepeatedVarintIter<'a, i64> {
        RepeatedVarintIter::new(self.buf, 3)
    }

    #[inline]
    pub const fn f_uint64(&self) -> RepeatedVarintIter<'a, u64> {
        RepeatedVarintIter::new(self.buf, 4)
    }

    #[inline]
    pub const fn f_int32(&self) -> RepeatedVarintIter<'a, i32> {
        RepeatedVarintIter::new(self.buf, 5)
    }

    #[inline]
    pub const fn f_fixed64(&self) -> RepeatedFixed64Iter<'a, u64> {
        RepeatedFixed64Iter::new(self.buf, 6)
    }

    #[inline]
    pub const fn f_fixed32(&self) -> RepeatedFixed32Iter<'a, u32> {
        RepeatedFixed32Iter::new(self.buf, 7)
    }

    #[inline]
    pub const fn f_bool(&self) -> RepeatedVarintIter<'a, bool> {
        RepeatedVarintIter::new(self.buf, 8)
    }

    #[inline]
    pub const fn f_string(&self) -> RepeatedStrIter<'a> {
        RepeatedStrIter::new(self.buf, 9)
    }

    #[inline]
    pub const fn f_group(&self) -> RepeatedMessageIter<'a, LocationView<'a>> {
        RepeatedMessageIter::new(self.buf, 10)
    }

    #[inline]
    pub const fn locations(&self) -> RepeatedMessageIter<'a, LocationView<'a>> {
        RepeatedMessageIter::new(self.buf, 11)
    }

    #[inline]
    pub const fn f_bytes(&self) -> RepeatedBytesIter<'a> {
        RepeatedBytesIter::new(self.buf, 12)
    }

    #[inline]
    pub const fn f_uint32(&self) -> RepeatedVarintIter<'a, u32> {
        RepeatedVarintIter::new(self.buf, 13)
    }

    #[inline]
    pub const fn f_enum(&self) -> RepeatedVarintIter<'a, DeviceMode> {
        RepeatedVarintIter::new(self.buf, 14)
    }

    #[inline]
    pub const fn f_sfixed32(&self) -> RepeatedFixed32Iter<'a, i32> {
        RepeatedFixed32Iter::new(self.buf, 15)
    }

    #[inline]
    pub const fn f_sfixed64(&self) -> RepeatedFixed64Iter<'a, i64> {
        RepeatedFixed64Iter::new(self.buf, 16)
    }

    #[inline]
    pub const fn f_sint32(&self) -> RepeatedZigZagIter<'a, i32> {
        RepeatedZigZagIter::new(self.buf, 17)
    }

    #[inline]
    pub const fn f_sint64(&self) -> RepeatedZigZagIter<'a, i64> {
        RepeatedZigZagIter::new(self.buf, 18)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

pub struct AllTypesBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> AllTypesBuilder<'b> {
    #[inline]
    pub fn new(buf: &'b mut [u8]) -> Self {
        Self {
            writer: ::roto::runtime::BackwardWriter::new(buf),
            set_fields: [0u64; 1],
        }
    }

    #[inline]
    pub const fn from_writer(writer: ::roto::runtime::BackwardWriter<'b>) -> Self {
        Self {
            writer,
            set_fields: [0u64; 1],
        }
    }

    #[inline]
    pub const fn writer(&self) -> &::roto::runtime::BackwardWriter<'b> {
        &self.writer
    }

    #[inline]
    pub fn writer_mut(&mut self) -> &mut ::roto::runtime::BackwardWriter<'b> {
        &mut self.writer
    }

    #[inline]
    pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.check()
    }

    #[inline]
    pub fn finish(self) -> Result<&'b [u8], ::roto::runtime::EncodeError> {
        self.writer.finish()
    }

    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &AllTypesView<'_>) -> &mut Self {
        let _ = self
            .writer
            .copy_filtered_fields(view.as_bytes(), |tag| !AllTypesView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &AllTypesView<'_>) -> &mut Self {
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
                5 => {
                    self.set_fields[0] |= 16;
                    (prev_set[0] & 16) == 0
                }
                6 => {
                    self.set_fields[0] |= 32;
                    (prev_set[0] & 32) == 0
                }
                7 => {
                    self.set_fields[0] |= 64;
                    (prev_set[0] & 64) == 0
                }
                8 => {
                    self.set_fields[0] |= 128;
                    (prev_set[0] & 128) == 0
                }
                9 => {
                    self.set_fields[0] |= 256;
                    (prev_set[0] & 256) == 0
                }
                10 => {
                    self.set_fields[0] |= 512;
                    (prev_set[0] & 512) == 0
                }
                11 => {
                    self.set_fields[0] |= 1024;
                    (prev_set[0] & 1024) == 0
                }
                12 => {
                    self.set_fields[0] |= 2048;
                    (prev_set[0] & 2048) == 0
                }
                13 => {
                    self.set_fields[0] |= 4096;
                    (prev_set[0] & 4096) == 0
                }
                14 => {
                    self.set_fields[0] |= 8192;
                    (prev_set[0] & 8192) == 0
                }
                15 => {
                    self.set_fields[0] |= 16384;
                    (prev_set[0] & 16384) == 0
                }
                16 => {
                    self.set_fields[0] |= 32768;
                    (prev_set[0] & 32768) == 0
                }
                17 => {
                    self.set_fields[0] |= 65536;
                    (prev_set[0] & 65536) == 0
                }
                18 => {
                    self.set_fields[0] |= 131072;
                    (prev_set[0] & 131072) == 0
                }
                _ => true,
            });
        self
    }

    #[inline]
    pub fn write_f_double(&mut self, val: f64) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_f64(val);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Fixed64);
        self
    }

    #[inline]
    pub fn write_packed_f_double(&mut self, vals: &[f64]) -> &mut Self {
        self.set_fields[0] |= 1;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_f64(val);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self
            .writer
            .write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_f_float(&mut self, val: f32) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_f32(val);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Fixed32);
        self
    }

    #[inline]
    pub fn write_packed_f_float(&mut self, vals: &[f32]) -> &mut Self {
        self.set_fields[0] |= 2;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_f32(val);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self
            .writer
            .write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_f_int64(&mut self, val: i64) -> &mut Self {
        self.set_fields[0] |= 4;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_packed_f_int64(&mut self, vals: &[i64]) -> &mut Self {
        self.set_fields[0] |= 4;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_varint(val as u64);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self
            .writer
            .write_tag(3, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_f_uint64(&mut self, val: u64) -> &mut Self {
        self.set_fields[0] |= 8;
        let _ = self.writer.write_varint(val);
        let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_packed_f_uint64(&mut self, vals: &[u64]) -> &mut Self {
        self.set_fields[0] |= 8;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_varint(val);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self
            .writer
            .write_tag(4, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_f_int32(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 16;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_packed_f_int32(&mut self, vals: &[i32]) -> &mut Self {
        self.set_fields[0] |= 16;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_varint(val as u64);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self
            .writer
            .write_tag(5, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_f_fixed64(&mut self, val: u64) -> &mut Self {
        self.set_fields[0] |= 32;
        let _ = self.writer.write_fixed64(val);
        let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Fixed64);
        self
    }

    #[inline]
    pub fn write_packed_f_fixed64(&mut self, vals: &[u64]) -> &mut Self {
        self.set_fields[0] |= 32;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_fixed64(val);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self
            .writer
            .write_tag(6, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_f_fixed32(&mut self, val: u32) -> &mut Self {
        self.set_fields[0] |= 64;
        let _ = self.writer.write_fixed32(val);
        let _ = self.writer.write_tag(7, ::roto::runtime::WireType::Fixed32);
        self
    }

    #[inline]
    pub fn write_packed_f_fixed32(&mut self, vals: &[u32]) -> &mut Self {
        self.set_fields[0] |= 64;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_fixed32(val);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self
            .writer
            .write_tag(7, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_f_bool(&mut self, val: bool) -> &mut Self {
        self.set_fields[0] |= 128;
        let _ = self.writer.write_bool(val);
        let _ = self.writer.write_tag(8, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_packed_f_bool(&mut self, vals: &[bool]) -> &mut Self {
        self.set_fields[0] |= 128;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_bool(val);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self
            .writer
            .write_tag(8, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_f_string(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 256;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self
            .writer
            .write_tag(9, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_f_group<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut LocationBuilder<'_>),
    {
        self.set_fields[0] |= 512;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = LocationBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self
            .writer
            .write_tag(10, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_locations<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut LocationBuilder<'_>),
    {
        self.set_fields[0] |= 1024;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = LocationBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self
            .writer
            .write_tag(11, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_location<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut LocationBuilder<'_>),
    {
        self.write_locations(f)
    }

    #[inline]
    pub fn write_f_bytes(&mut self, val: &[u8]) -> &mut Self {
        self.set_fields[0] |= 2048;
        let _ = self.writer.write_length_delimited(val);
        let _ = self
            .writer
            .write_tag(12, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_f_uint32(&mut self, val: u32) -> &mut Self {
        self.set_fields[0] |= 4096;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(13, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_packed_f_uint32(&mut self, vals: &[u32]) -> &mut Self {
        self.set_fields[0] |= 4096;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_varint(val as u64);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self
            .writer
            .write_tag(13, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_f_enum(&mut self, val: DeviceMode) -> &mut Self {
        self.set_fields[0] |= 8192;
        let _ = self.writer.write_varint(val.as_i32() as u64);
        let _ = self.writer.write_tag(14, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_packed_f_enum(&mut self, vals: &[DeviceMode]) -> &mut Self {
        self.set_fields[0] |= 8192;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_varint(val.as_i32() as u64);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self
            .writer
            .write_tag(14, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_f_sfixed32(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 16384;
        let _ = self.writer.write_sfixed32(val);
        let _ = self
            .writer
            .write_tag(15, ::roto::runtime::WireType::Fixed32);
        self
    }

    #[inline]
    pub fn write_packed_f_sfixed32(&mut self, vals: &[i32]) -> &mut Self {
        self.set_fields[0] |= 16384;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_sfixed32(val);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self
            .writer
            .write_tag(15, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_f_sfixed64(&mut self, val: i64) -> &mut Self {
        self.set_fields[0] |= 32768;
        let _ = self.writer.write_sfixed64(val);
        let _ = self
            .writer
            .write_tag(16, ::roto::runtime::WireType::Fixed64);
        self
    }

    #[inline]
    pub fn write_packed_f_sfixed64(&mut self, vals: &[i64]) -> &mut Self {
        self.set_fields[0] |= 32768;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_sfixed64(val);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self
            .writer
            .write_tag(16, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_f_sint32(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 65536;
        let _ = self.writer.write_zigzag32(val);
        let _ = self.writer.write_tag(17, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_packed_f_sint32(&mut self, vals: &[i32]) -> &mut Self {
        self.set_fields[0] |= 65536;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_zigzag32(val);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self
            .writer
            .write_tag(17, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_f_sint64(&mut self, val: i64) -> &mut Self {
        self.set_fields[0] |= 131072;
        let _ = self.writer.write_zigzag64(val);
        let _ = self.writer.write_tag(18, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_packed_f_sint64(&mut self, vals: &[i64]) -> &mut Self {
        self.set_fields[0] |= 131072;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_zigzag64(val);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self
            .writer
            .write_tag(18, ::roto::runtime::WireType::LengthDelimited);
        self
    }
}

// ============================================================================
// Generated AllTypesView<'a> (18 Field Types, Defaults, Oneofs, Nested Types)
// ============================================================================

pub mod demo {
    #[derive(Clone, Copy, Debug)]
    pub struct AllTypesView<'a> {
        buf: &'a [u8],
        f_1: ::roto::runtime::FieldMarker,
        opt_2: ::roto::runtime::FieldMarker,
        opt_3: ::roto::runtime::FieldMarker,
        opt_4: ::roto::runtime::FieldMarker,
        opt_5: ::roto::runtime::FieldMarker,
        opt_6: ::roto::runtime::FieldMarker,
        opt_7: ::roto::runtime::FieldMarker,
        opt_8: ::roto::runtime::FieldMarker,
        opt_9: ::roto::runtime::FieldMarker,
        opt_10: ::roto::runtime::FieldMarker,
        opt_11: ::roto::runtime::FieldMarker,
        opt_12: ::roto::runtime::FieldMarker,
        opt_13: ::roto::runtime::FieldMarker,
        opt_14: ::roto::runtime::FieldMarker,
        opt_15: ::roto::runtime::FieldMarker,
        opt_16: ::roto::runtime::FieldMarker,
        opt_17: ::roto::runtime::FieldMarker,
        opt_18: ::roto::runtime::FieldMarker,
        choice_str: ::roto::runtime::FieldMarker,
        choice_inner: ::roto::runtime::FieldMarker,
        thresh_code: ::roto::runtime::FieldMarker,
        thresh_kind: ::roto::runtime::FieldMarker,
        req_id: ::roto::runtime::FieldMarker,
        oneof_0: u32,
        oneof_1: u32,
    }

    impl<'a> AllTypesView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41,
        ];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            let mut f_1 = ::roto::runtime::FieldMarker::none();
            let mut opt_2 = ::roto::runtime::FieldMarker::none();
            let mut opt_3 = ::roto::runtime::FieldMarker::none();
            let mut opt_4 = ::roto::runtime::FieldMarker::none();
            let mut opt_5 = ::roto::runtime::FieldMarker::none();
            let mut opt_6 = ::roto::runtime::FieldMarker::none();
            let mut opt_7 = ::roto::runtime::FieldMarker::none();
            let mut opt_8 = ::roto::runtime::FieldMarker::none();
            let mut opt_9 = ::roto::runtime::FieldMarker::none();
            let mut opt_10 = ::roto::runtime::FieldMarker::none();
            let mut opt_11 = ::roto::runtime::FieldMarker::none();
            let mut opt_12 = ::roto::runtime::FieldMarker::none();
            let mut opt_13 = ::roto::runtime::FieldMarker::none();
            let mut opt_14 = ::roto::runtime::FieldMarker::none();
            let mut opt_15 = ::roto::runtime::FieldMarker::none();
            let mut opt_16 = ::roto::runtime::FieldMarker::none();
            let mut opt_17 = ::roto::runtime::FieldMarker::none();
            let mut opt_18 = ::roto::runtime::FieldMarker::none();
            let mut choice_str = ::roto::runtime::FieldMarker::none();
            let mut choice_inner = ::roto::runtime::FieldMarker::none();
            let mut thresh_code = ::roto::runtime::FieldMarker::none();
            let mut thresh_kind = ::roto::runtime::FieldMarker::none();
            let mut req_id = ::roto::runtime::FieldMarker::none();
            let mut oneof_0: u32 = 0;
            let mut oneof_1: u32 = 0;

            for item in ::roto::runtime::FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (1, ::roto::runtime::WireType::Fixed64) => f_1 = field.marker,
                    (2, ::roto::runtime::WireType::Fixed32) => opt_2 = field.marker,
                    (3, ::roto::runtime::WireType::Varint) => opt_3 = field.marker,
                    (4, ::roto::runtime::WireType::Varint) => opt_4 = field.marker,
                    (5, ::roto::runtime::WireType::Varint) => opt_5 = field.marker,
                    (6, ::roto::runtime::WireType::Fixed64) => opt_6 = field.marker,
                    (7, ::roto::runtime::WireType::Fixed32) => opt_7 = field.marker,
                    (8, ::roto::runtime::WireType::Varint) => opt_8 = field.marker,
                    (9, ::roto::runtime::WireType::LengthDelimited) => opt_9 = field.marker,
                    (10, ::roto::runtime::WireType::LengthDelimited) => opt_10 = field.marker,
                    (11, ::roto::runtime::WireType::LengthDelimited) => opt_11 = field.marker,
                    (12, ::roto::runtime::WireType::LengthDelimited) => opt_12 = field.marker,
                    (13, ::roto::runtime::WireType::Varint) => opt_13 = field.marker,
                    (14, ::roto::runtime::WireType::Varint) => opt_14 = field.marker,
                    (15, ::roto::runtime::WireType::Fixed32) => opt_15 = field.marker,
                    (16, ::roto::runtime::WireType::Fixed64) => opt_16 = field.marker,
                    (17, ::roto::runtime::WireType::Varint) => opt_17 = field.marker,
                    (18, ::roto::runtime::WireType::Varint) => opt_18 = field.marker,
                    (37, ::roto::runtime::WireType::LengthDelimited) => {
                        choice_str = field.marker;
                        oneof_0 = 37;
                    }
                    (38, ::roto::runtime::WireType::LengthDelimited) => {
                        choice_inner = field.marker;
                        oneof_0 = 38;
                    }
                    (39, ::roto::runtime::WireType::Varint) => {
                        thresh_code = field.marker;
                        oneof_1 = 39;
                    }
                    (40, ::roto::runtime::WireType::Varint) => {
                        thresh_kind = field.marker;
                        oneof_1 = 40;
                    }
                    (41, ::roto::runtime::WireType::Varint) => req_id = field.marker,
                    _ => {}
                }
            }

            if !req_id.is_present() {
                return Err(::roto::runtime::DecodeError::MissingRequiredField(
                    "AllTypes.req_id",
                ));
            }

            Ok(Self {
                buf,
                f_1,
                opt_2,
                opt_3,
                opt_4,
                opt_5,
                opt_6,
                opt_7,
                opt_8,
                opt_9,
                opt_10,
                opt_11,
                opt_12,
                opt_13,
                opt_14,
                opt_15,
                opt_16,
                opt_17,
                opt_18,
                choice_str,
                choice_inner,
                thresh_code,
                thresh_kind,
                req_id,
                oneof_0,
                oneof_1,
            })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn has_buf(&self) -> bool {
            self.f_1.is_present()
        }

        #[inline]
        pub fn buf(&self) -> Result<f64, ::roto::runtime::DecodeError> {
            if !self.f_1.is_present() {
                return Ok(3.5f64);
            }
            ::roto::runtime::decode_f64_at(self.buf, self.f_1)
        }

        #[inline]
        pub const fn has_opt_2(&self) -> bool {
            self.opt_2.is_present()
        }

        #[inline]
        pub fn opt_2(&self) -> Result<f32, ::roto::runtime::DecodeError> {
            if !self.opt_2.is_present() {
                return Ok(f32::INFINITY);
            }
            ::roto::runtime::decode_f32_at(self.buf, self.opt_2)
        }

        #[inline]
        pub const fn has_opt_3(&self) -> bool {
            self.opt_3.is_present()
        }

        #[inline]
        pub fn opt_3(&self) -> Result<i64, ::roto::runtime::DecodeError> {
            if !self.opt_3.is_present() {
                return Ok(-900i64);
            }
            Ok(::roto::runtime::decode_varint_at(self.buf, self.opt_3)? as i64)
        }

        #[inline]
        pub const fn has_opt_4(&self) -> bool {
            self.opt_4.is_present()
        }

        #[inline]
        pub fn opt_4(&self) -> Result<u64, ::roto::runtime::DecodeError> {
            if !self.opt_4.is_present() {
                return Ok(400u64);
            }
            ::roto::runtime::decode_varint_at(self.buf, self.opt_4)
        }

        #[inline]
        pub const fn has_opt_5(&self) -> bool {
            self.opt_5.is_present()
        }

        #[inline]
        pub fn opt_5(&self) -> Result<i32, ::roto::runtime::DecodeError> {
            if !self.opt_5.is_present() {
                return Ok(-50i32);
            }
            Ok(::roto::runtime::decode_varint_at(self.buf, self.opt_5)? as i32)
        }

        #[inline]
        pub const fn has_opt_6(&self) -> bool {
            self.opt_6.is_present()
        }

        #[inline]
        pub fn opt_6(&self) -> Result<u64, ::roto::runtime::DecodeError> {
            if !self.opt_6.is_present() {
                return Ok(600u64);
            }
            ::roto::runtime::decode_fixed64_at(self.buf, self.opt_6)
        }

        #[inline]
        pub const fn has_opt_7(&self) -> bool {
            self.opt_7.is_present()
        }

        #[inline]
        pub fn opt_7(&self) -> Result<u32, ::roto::runtime::DecodeError> {
            if !self.opt_7.is_present() {
                return Ok(70u32);
            }
            ::roto::runtime::decode_fixed32_at(self.buf, self.opt_7)
        }

        #[inline]
        pub const fn has_opt_8(&self) -> bool {
            self.opt_8.is_present()
        }

        #[inline]
        pub fn opt_8(&self) -> Result<bool, ::roto::runtime::DecodeError> {
            if !self.opt_8.is_present() {
                return Ok(true);
            }
            ::roto::runtime::decode_bool_at(self.buf, self.opt_8)
        }

        #[inline]
        pub const fn has_opt_9(&self) -> bool {
            self.opt_9.is_present()
        }

        #[inline]
        pub fn opt_9(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
            if !self.opt_9.is_present() {
                return Ok("default_str");
            }
            ::roto::runtime::decode_str_at(self.buf, self.opt_9)
        }

        #[inline]
        pub const fn has_opt_10(&self) -> bool {
            self.opt_10.is_present()
        }

        #[inline]
        pub fn opt_10(
            &self,
        ) -> Result<Option<crate::demo::all_types::InnerView<'a>>, ::roto::runtime::DecodeError>
        {
            if !self.opt_10.is_present() {
                return Ok(None);
            }
            crate::demo::all_types::InnerView::from_bytes(self.opt_10.slice(self.buf)).map(Some)
        }

        #[inline]
        pub const fn has_opt_11(&self) -> bool {
            self.opt_11.is_present()
        }

        #[inline]
        pub fn opt_11(
            &self,
        ) -> Result<Option<crate::demo::all_types::InnerView<'a>>, ::roto::runtime::DecodeError>
        {
            if !self.opt_11.is_present() {
                return Ok(None);
            }
            crate::demo::all_types::InnerView::from_bytes(self.opt_11.slice(self.buf)).map(Some)
        }

        #[inline]
        pub const fn has_opt_12(&self) -> bool {
            self.opt_12.is_present()
        }

        #[inline]
        pub fn opt_12(&self) -> Result<&'a [u8], ::roto::runtime::DecodeError> {
            if !self.opt_12.is_present() {
                return Ok(&b"\x00\xfeok"[..]);
            }
            ::roto::runtime::decode_bytes_at(self.buf, self.opt_12)
        }

        #[inline]
        pub const fn has_opt_13(&self) -> bool {
            self.opt_13.is_present()
        }

        #[inline]
        pub fn opt_13(&self) -> Result<u32, ::roto::runtime::DecodeError> {
            if !self.opt_13.is_present() {
                return Ok(130u32);
            }
            Ok(::roto::runtime::decode_varint_at(self.buf, self.opt_13)? as u32)
        }

        #[inline]
        pub const fn has_opt_14(&self) -> bool {
            self.opt_14.is_present()
        }

        #[inline]
        pub fn opt_14(&self) -> Result<crate::demo::all_types::Kind, ::roto::runtime::DecodeError> {
            if !self.opt_14.is_present() {
                return Ok(crate::demo::all_types::Kind::KindB);
            }
            Ok(
                <crate::demo::all_types::Kind as ::roto::runtime::VarintValue>::from_raw_varint(
                    ::roto::runtime::decode_varint_at(self.buf, self.opt_14)?,
                ),
            )
        }

        #[inline]
        pub const fn has_opt_15(&self) -> bool {
            self.opt_15.is_present()
        }

        #[inline]
        pub fn opt_15(&self) -> Result<i32, ::roto::runtime::DecodeError> {
            if !self.opt_15.is_present() {
                return Ok(-150i32);
            }
            ::roto::runtime::decode_sfixed32_at(self.buf, self.opt_15)
        }

        #[inline]
        pub const fn has_opt_16(&self) -> bool {
            self.opt_16.is_present()
        }

        #[inline]
        pub fn opt_16(&self) -> Result<i64, ::roto::runtime::DecodeError> {
            if !self.opt_16.is_present() {
                return Ok(-1600i64);
            }
            ::roto::runtime::decode_sfixed64_at(self.buf, self.opt_16)
        }

        #[inline]
        pub const fn has_opt_17(&self) -> bool {
            self.opt_17.is_present()
        }

        #[inline]
        pub fn opt_17(&self) -> Result<i32, ::roto::runtime::DecodeError> {
            if !self.opt_17.is_present() {
                return Ok(-170i32);
            }
            ::roto::runtime::decode_sint32_at(self.buf, self.opt_17)
        }

        #[inline]
        pub const fn has_opt_18(&self) -> bool {
            self.opt_18.is_present()
        }

        #[inline]
        pub fn opt_18(&self) -> Result<i64, ::roto::runtime::DecodeError> {
            if !self.opt_18.is_present() {
                return Ok(-1800i64);
            }
            ::roto::runtime::decode_sint64_at(self.buf, self.opt_18)
        }

        #[inline]
        pub const fn rep_1(&self) -> ::roto::runtime::RepeatedFixed64Iter<'a, f64> {
            ::roto::runtime::RepeatedFixed64Iter::new(self.buf, 19)
        }

        #[inline]
        pub const fn rep_2(&self) -> ::roto::runtime::RepeatedFixed32Iter<'a, f32> {
            ::roto::runtime::RepeatedFixed32Iter::new(self.buf, 20)
        }

        #[inline]
        pub const fn rep_3(&self) -> ::roto::runtime::RepeatedVarintIter<'a, i64> {
            ::roto::runtime::RepeatedVarintIter::new(self.buf, 21)
        }

        #[inline]
        pub const fn rep_4(&self) -> ::roto::runtime::RepeatedVarintIter<'a, u64> {
            ::roto::runtime::RepeatedVarintIter::new(self.buf, 22)
        }

        #[inline]
        pub const fn rep_5(&self) -> ::roto::runtime::RepeatedVarintIter<'a, i32> {
            ::roto::runtime::RepeatedVarintIter::new(self.buf, 23)
        }

        #[inline]
        pub const fn rep_6(&self) -> ::roto::runtime::RepeatedFixed64Iter<'a, u64> {
            ::roto::runtime::RepeatedFixed64Iter::new(self.buf, 24)
        }

        #[inline]
        pub const fn rep_7(&self) -> ::roto::runtime::RepeatedFixed32Iter<'a, u32> {
            ::roto::runtime::RepeatedFixed32Iter::new(self.buf, 25)
        }

        #[inline]
        pub const fn rep_8(&self) -> ::roto::runtime::RepeatedVarintIter<'a, bool> {
            ::roto::runtime::RepeatedVarintIter::new(self.buf, 26)
        }

        #[inline]
        pub const fn rep_9(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
            ::roto::runtime::RepeatedStrIter::new(self.buf, 27)
        }

        #[inline]
        pub const fn rep_10(
            &self,
        ) -> ::roto::runtime::RepeatedMessageIter<'a, crate::demo::all_types::InnerView<'a>>
        {
            ::roto::runtime::RepeatedMessageIter::new(self.buf, 28)
        }

        #[inline]
        pub const fn rep_11(
            &self,
        ) -> ::roto::runtime::RepeatedMessageIter<'a, crate::demo::all_types::InnerView<'a>>
        {
            ::roto::runtime::RepeatedMessageIter::new(self.buf, 29)
        }

        #[inline]
        pub const fn rep_12(&self) -> ::roto::runtime::RepeatedBytesIter<'a> {
            ::roto::runtime::RepeatedBytesIter::new(self.buf, 30)
        }

        #[inline]
        pub const fn rep_13(&self) -> ::roto::runtime::RepeatedVarintIter<'a, u32> {
            ::roto::runtime::RepeatedVarintIter::new(self.buf, 31)
        }

        #[inline]
        pub const fn rep_14(
            &self,
        ) -> ::roto::runtime::RepeatedVarintIter<'a, crate::demo::all_types::Kind> {
            ::roto::runtime::RepeatedVarintIter::new(self.buf, 32)
        }

        #[inline]
        pub const fn rep_15(&self) -> ::roto::runtime::RepeatedFixed32Iter<'a, i32> {
            ::roto::runtime::RepeatedFixed32Iter::new(self.buf, 33)
        }

        #[inline]
        pub const fn rep_16(&self) -> ::roto::runtime::RepeatedFixed64Iter<'a, i64> {
            ::roto::runtime::RepeatedFixed64Iter::new(self.buf, 34)
        }

        #[inline]
        pub const fn rep_17(&self) -> ::roto::runtime::RepeatedZigZagIter<'a, i32> {
            ::roto::runtime::RepeatedZigZagIter::new(self.buf, 35)
        }

        #[inline]
        pub const fn rep_18(&self) -> ::roto::runtime::RepeatedZigZagIter<'a, i64> {
            ::roto::runtime::RepeatedZigZagIter::new(self.buf, 36)
        }

        #[inline]
        pub const fn has_choice_str(&self) -> bool {
            self.choice_str.is_present()
        }

        #[inline]
        pub fn choice_str(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
            if !self.choice_str.is_present() {
                return Ok("");
            }
            ::roto::runtime::decode_str_at(self.buf, self.choice_str)
        }

        #[inline]
        pub const fn has_choice_inner(&self) -> bool {
            self.choice_inner.is_present()
        }

        #[inline]
        pub fn choice_inner(
            &self,
        ) -> Result<Option<crate::demo::all_types::InnerView<'a>>, ::roto::runtime::DecodeError>
        {
            if !self.choice_inner.is_present() {
                return Ok(None);
            }
            crate::demo::all_types::InnerView::from_bytes(self.choice_inner.slice(self.buf))
                .map(Some)
        }

        #[inline]
        pub const fn has_thresh_code(&self) -> bool {
            self.thresh_code.is_present()
        }

        #[inline]
        pub fn thresh_code(&self) -> Result<i32, ::roto::runtime::DecodeError> {
            if !self.thresh_code.is_present() {
                return Ok(0i32);
            }
            ::roto::runtime::decode_sint32_at(self.buf, self.thresh_code)
        }

        #[inline]
        pub const fn has_thresh_kind(&self) -> bool {
            self.thresh_kind.is_present()
        }

        #[inline]
        pub fn thresh_kind(
            &self,
        ) -> Result<crate::demo::all_types::Kind, ::roto::runtime::DecodeError> {
            if !self.thresh_kind.is_present() {
                return Ok(crate::demo::all_types::Kind::KindA);
            }
            Ok(
                <crate::demo::all_types::Kind as ::roto::runtime::VarintValue>::from_raw_varint(
                    ::roto::runtime::decode_varint_at(self.buf, self.thresh_kind)?,
                ),
            )
        }

        #[inline]
        pub const fn has_req_id(&self) -> bool {
            self.req_id.is_present()
        }

        #[inline]
        pub fn req_id(&self) -> Result<u32, ::roto::runtime::DecodeError> {
            Ok(::roto::runtime::decode_varint_at(self.buf, self.req_id)? as u32)
        }

        #[inline]
        pub fn choice(
            &self,
        ) -> Result<Option<all_types::Choice<'a>>, ::roto::runtime::DecodeError> {
            match self.oneof_0 {
                37 => Ok(Some(all_types::Choice::ChoiceStr(
                    ::roto::runtime::decode_str_at(self.buf, self.choice_str)?,
                ))),
                38 => Ok(Some(all_types::Choice::ChoiceInner(
                    crate::demo::all_types::InnerView::from_bytes(
                        self.choice_inner.slice(self.buf),
                    )?,
                ))),
                _ => Ok(None),
            }
        }

        #[inline]
        pub fn threshold(
            &self,
        ) -> Result<Option<all_types::Threshold>, ::roto::runtime::DecodeError> {
            match self.oneof_1 {
                39 => Ok(Some(all_types::Threshold::ThreshCode(
                    ::roto::runtime::decode_sint32_at(self.buf, self.thresh_code)?,
                ))),
                40 => Ok(Some(all_types::Threshold::ThreshKind(
                    <crate::demo::all_types::Kind as ::roto::runtime::VarintValue>::from_raw_varint(
                        ::roto::runtime::decode_varint_at(self.buf, self.thresh_kind)?,
                    ),
                ))),
                _ => Ok(None),
            }
        }

        #[inline]
        pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
            ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> ::roto::runtime::MessageView<'a> for AllTypesView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub mod all_types {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(i32)]
        pub enum Kind {
            KindA = 0,
            KindB = 1,
        }

        impl Kind {
            #[inline]
            pub const fn from_i32(val: i32) -> Option<Self> {
                match val {
                    0 => Some(Self::KindA),
                    1 => Some(Self::KindB),
                    _ => None,
                }
            }

            #[inline]
            pub const fn as_i32(self) -> i32 {
                self as i32
            }
        }

        impl core::convert::TryFrom<i32> for Kind {
            type Error = i32;

            #[inline]
            fn try_from(val: i32) -> Result<Self, Self::Error> {
                Self::from_i32(val).ok_or(val)
            }
        }

        impl ::roto::runtime::VarintValue for Kind {
            #[inline]
            fn from_raw_varint(val: u64) -> Self {
                Self::from_i32(val as i32).unwrap_or(Self::KindA)
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub enum Choice<'a> {
            ChoiceStr(&'a str),
            ChoiceInner(crate::demo::all_types::InnerView<'a>),
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum Threshold {
            ThreshCode(i32),
            ThreshKind(crate::demo::all_types::Kind),
        }

        #[derive(Clone, Copy, Debug)]
        pub struct InnerView<'a> {
            buf: &'a [u8],
            val: ::roto::runtime::FieldMarker,
        }

        impl<'a> InnerView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut val = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => val = field.marker,
                        _ => {}
                    }
                }

                Ok(Self { buf, val })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_val(&self) -> bool {
                self.val.is_present()
            }

            #[inline]
            pub fn val(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                if !self.val.is_present() {
                    return Ok(0i32);
                }
                Ok(::roto::runtime::decode_varint_at(self.buf, self.val)? as i32)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for InnerView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct InnerBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> InnerBuilder<'b> {
            #[inline]
            pub fn new(buf: &'b mut [u8]) -> Self {
                Self {
                    writer: ::roto::runtime::BackwardWriter::new(buf),
                    set_fields: [0; 1],
                }
            }

            #[inline]
            pub const fn from_writer(writer: ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self {
                    writer,
                    set_fields: [0; 1],
                }
            }

            #[inline]
            pub const fn writer(&self) -> &::roto::runtime::BackwardWriter<'b> {
                &self.writer
            }

            #[inline]
            pub fn writer_mut(&mut self) -> &mut ::roto::runtime::BackwardWriter<'b> {
                &mut self.writer
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn finish(self) -> Result<&'b [u8], ::roto::runtime::EncodeError> {
                self.writer.finish()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &InnerView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(
                    view.as_bytes(),
                    |tag| !InnerView::KNOWN_TAGS.contains(&tag),
                );
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &InnerView<'_>) -> &mut Self {
                let prev_set = self.set_fields;
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
                    1 => {
                        self.set_fields[0] |= 1;
                        (prev_set[0] & 1) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_val(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_varint(val as u64);
                let _ = self
                    .writer
                    .write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }
        }
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
        || prim_writer.check().is_err()
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
    let Ok(prim_finished) = prim_writer.finish() else {
        return false;
    };
    if prim_finished.is_empty() {
        return false;
    }

    // Exercise sticky BufferOverflow and builder.check() loop guard
    let mut tiny_buf = [0u8; 4];
    let mut loop_iterations = 0usize;
    let mut tiny_builder = SensorReadingBuilder::new(&mut tiny_buf);
    for s in [10u32, 20, 30, 40, 50] {
        tiny_builder.write_sample(s);
        loop_iterations += 1;
        if tiny_builder.check().is_err() {
            break;
        }
    }
    if loop_iterations != 3 || tiny_builder.finish() != Err(EncodeError::BufferOverflow) {
        return false;
    }

    // Full end-to-end Location / SensorReading roundtrip with unknown field forwarding
    let mut backing1 = [0u8; 128];
    let mut writer1 = BackwardWriter::new(&mut backing1);
    // Unknown field 77 on SensorReading
    if writer1.write_varint(999).is_err() || writer1.write_tag(77, WireType::Varint).is_err() {
        return false;
    }

    let mut builder = SensorReadingBuilder::from_writer(writer1);
    builder
        .write_sample(108)
        .write_sample(42)
        .write_location(|loc| {
            // Unknown field 55 on Location
            let _ = loc.writer_mut().write_varint(777);
            let _ = loc.writer_mut().write_tag(55, WireType::Varint);
            loc.write_lon_e7(-1224194000).write_lat_e7(377749000);
        })
        .write_device_id("sensor-node-01")
        .write_timestamp_ms(1_700_000_000_000);
    if builder.check().is_err() || builder.writer().check().is_err() {
        return false;
    }
    let Ok(wire_bytes1) = builder.finish() else {
        return false;
    };

    let Ok(reading1) = SensorReadingView::from_bytes(wire_bytes1) else {
        return false;
    };
    let Ok(Some(loc1)) = reading1.location() else {
        return false;
    };

    // Re-encode with unknown fields forwarded
    let mut backing2 = [0u8; 128];
    let mut builder = SensorReadingBuilder::new(&mut backing2);
    builder
        .copy_unknown_fields(&reading1)
        .write_sample(108)
        .write_sample(42)
        .write_location(|loc| {
            loc.copy_unknown_fields(&loc1)
                .write_lon_e7(-1224194000)
                .write_lat_e7(377749000);
        })
        .write_device_id("sensor-node-01")
        .write_timestamp_ms(1_700_000_000_000);
    let Ok(wire_bytes2) = builder.finish() else {
        return false;
    };

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

    // Exercise google_protobuf_roto descriptor Views, Builders, Options, and Metadata in #![no_std]
    let mut desc_backing = [0u8; 512];
    let mut fds = FileDescriptorSetBuilder::new(&mut desc_backing);
    fds.write_file(|file| {
        file.write_edition(Edition::Edition2023)
            .write_syntax("editions")
            .write_source_code_info(|sci| {
                sci.write_location(|loc| {
                    loc.write_leading_comments("SensorReading doc")
                        .write_packed_span(&[10, 0, 15, 1])
                        .write_packed_path(&[4, 0]);
                });
            })
            .write_service(|svc| {
                svc.write_method(|m| {
                    m.write_server_streaming(true)
                        .write_client_streaming(false)
                        .write_output_type(".demo.SensorReading")
                        .write_input_type(".demo.Location")
                        .write_name("StreamReadings");
                })
                .write_name("SensorService");
            })
            .write_enum_type(|en| {
                en.write_visibility(SymbolVisibility::VisibilityExport)
                    .write_value(|v| {
                        v.write_number(0).write_name("STATUS_OK");
                    })
                    .write_name("Status");
            })
            .write_message_type(|msg| {
                msg.write_visibility(SymbolVisibility::VisibilityLocal)
                    .write_options(|mopt| {
                        mopt.write_uninterpreted_option(|uo| {
                            uo.write_identifier_value("TRUE").write_name(|np| {
                                np.write_is_extension(true)
                                    .write_name_part("demo.custom_msg_opt");
                            });
                        })
                        .write_map_entry(false)
                        .write_deprecated(true);
                    })
                    .write_field(|fld| {
                        fld.write_proto3_optional(true)
                            .write_json_name("timestampMs")
                            .write_options(|fopt| {
                                fopt.write_deprecated(false).write_packed(true);
                            })
                            .write_default_value("0")
                            .write_type(Type::TypeUint64)
                            .write_label(Label::LabelRequired)
                            .write_number(1)
                            .write_name("timestamp_ms");
                    })
                    .write_name("SensorReading");
            })
            .write_package("demo")
            .write_name("sensor.proto");
    });
    if fds.check().is_err() {
        return false;
    }
    let Ok(desc_bytes) = fds.finish() else {
        return false;
    };
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

    // Verify SourceCodeInfo and LocationView with packed path/span
    let Ok(Some(sci_view)) = file_view.source_code_info() else {
        return false;
    };
    let mut locs = sci_view.location();
    let Some(Ok(loc_view)) = locs.next() else {
        return false;
    };
    if locs.next().is_some() || loc_view.leading_comments() != Ok("SensorReading doc") {
        return false;
    }
    let mut loc_path = loc_view.path();
    if loc_path.next() != Some(Ok(4)) || loc_path.next() != Some(Ok(0)) || loc_path.next().is_some()
    {
        return false;
    }
    let mut loc_span = loc_view.span();
    if loc_span.next() != Some(Ok(10))
        || loc_span.next() != Some(Ok(0))
        || loc_span.next() != Some(Ok(15))
        || loc_span.next() != Some(Ok(1))
        || loc_span.next().is_some()
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

    // Verify MessageOptionsView and uninterpreted_option::NamePartView
    let Ok(Some(msg_opt_view)) = msg_view.options() else {
        return false;
    };
    if msg_opt_view.deprecated() != Ok(true) || msg_opt_view.map_entry() != Ok(false) {
        return false;
    }
    let mut uos = msg_opt_view.uninterpreted_option();
    let Some(Ok(uo_view)) = uos.next() else {
        return false;
    };
    if uos.next().is_some() || uo_view.identifier_value() != Ok("TRUE") {
        return false;
    }
    let mut name_parts = uo_view.name();
    let Some(Ok(np_view)) = name_parts.next() else {
        return false;
    };
    if name_parts.next().is_some()
        || np_view.name_part() != Ok("demo.custom_msg_opt")
        || np_view.is_extension() != Ok(true)
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

    // Verify FieldOptionsView
    let Ok(Some(field_opt_view)) = field_view.options() else {
        return false;
    };
    if field_opt_view.packed() != Ok(true) || field_opt_view.deprecated() != Ok(false) {
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

    // Exercise google::protobuf::compiler::plugin types in #![no_std]
    let mut ver_backing = [0u8; 64];
    let mut vb = VersionBuilder::new(&mut ver_backing);
    vb.write_suffix("rc1")
        .write_patch(1)
        .write_minor(28)
        .write_major(4);
    let Ok(ver_bytes) = vb.finish() else {
        return false;
    };
    let Ok(standalone_ver) = VersionView::from_bytes(ver_bytes) else {
        return false;
    };
    if standalone_ver.major() != Ok(4)
        || standalone_ver.minor() != Ok(28)
        || standalone_ver.patch() != Ok(1)
        || standalone_ver.suffix() != Ok("rc1")
    {
        return false;
    }

    let mut req_backing = [0u8; 512];
    let mut req = CodeGeneratorRequestBuilder::new(&mut req_backing);
    req.write_compiler_version(|ver| {
        ver.write_suffix("")
            .write_patch(0)
            .write_minor(28)
            .write_major(4);
    })
    .write_source_file_descriptors(|fdp| {
        fdp.write_syntax("proto2")
            .write_package("demo")
            .write_name("sensor.proto");
    })
    .write_proto_file(|fdp| {
        let _ = fdp.writer.write_bytes(file_view.as_bytes());
    })
    .write_parameter("no_std")
    .write_file_to_generate("sensor.proto");
    let Ok(req_bytes) = req.finish() else {
        return false;
    };
    let Ok(req_view) = CodeGeneratorRequestView::from_bytes(req_bytes) else {
        return false;
    };
    let mut ftg = req_view.file_to_generate();
    if ftg.next() != Some(Ok("sensor.proto"))
        || ftg.next().is_some()
        || req_view.parameter() != Ok("no_std")
    {
        return false;
    }
    let Ok(Some(cv)) = req_view.compiler_version() else {
        return false;
    };
    if cv.major() != Ok(4) || cv.minor() != Ok(28) || cv.patch() != Ok(0) || cv.suffix() != Ok("") {
        return false;
    }
    let mut pfiles = req_view.proto_file();
    let Some(Ok(pf)) = pfiles.next() else {
        return false;
    };
    if pfiles.next().is_some() || pf.name() != Ok("sensor.proto") {
        return false;
    }
    let mut sfiles = req_view.source_file_descriptors();
    let Some(Ok(sf)) = sfiles.next() else {
        return false;
    };
    if sfiles.next().is_some() || sf.name() != Ok("sensor.proto") || sf.syntax() != Ok("proto2") {
        return false;
    }

    let mut file_backing = [0u8; 128];
    let mut fb = FileBuilder::new(&mut file_backing);
    fb.write_content("// standalone")
        .write_name("standalone.rs");
    let Ok(file_bytes) = fb.finish() else {
        return false;
    };
    let Ok(standalone_file) = FileView::from_bytes(file_bytes) else {
        return false;
    };
    if standalone_file.name() != Ok("standalone.rs")
        || standalone_file.content() != Ok("// standalone")
    {
        return false;
    }

    let mut resp_backing = [0u8; 256];
    let mut resp = CodeGeneratorResponseBuilder::new(&mut resp_backing);
    let features = (Feature::FeatureProto3Optional.as_i32()
        | Feature::FeatureSupportsEditions.as_i32()) as u64;
    resp.write_file(|f| {
        f.write_generated_code_info(|gci| {
            gci.write_annotation(|ann| {
                ann.write_semantic(Semantic::Set)
                    .write_end(20)
                    .write_begin(5)
                    .write_source_file("sensor.proto")
                    .write_packed_path(&[4, 0]);
            });
        })
        .write_content("pub mod demo {}\n")
        .write_insertion_point("scope")
        .write_name("sensor.rs");
    })
    .write_maximum_edition(Edition::Edition2024.as_i32())
    .write_minimum_edition(Edition::EditionProto2.as_i32())
    .write_supported_features(features);
    let Ok(resp_bytes) = resp.finish() else {
        return false;
    };
    let Ok(resp_view) = CodeGeneratorResponseView::from_bytes(resp_bytes) else {
        return false;
    };
    if resp_view.has_error()
        || resp_view.error() != Ok("")
        || resp_view.supported_features() != Ok(3)
        || resp_view.minimum_edition() != Ok(Edition::EditionProto2.as_i32())
        || resp_view.maximum_edition() != Ok(Edition::Edition2024.as_i32())
        || Feature::from_i32(0) != Some(Feature::FeatureNone)
    {
        return false;
    }
    let mut rfiles = resp_view.file();
    let Some(Ok(rf)) = rfiles.next() else {
        return false;
    };
    if rfiles.next().is_some()
        || rf.name() != Ok("sensor.rs")
        || rf.insertion_point() != Ok("scope")
        || rf.content() != Ok("pub mod demo {}\n")
    {
        return false;
    }
    let Ok(Some(gci)) = rf.generated_code_info() else {
        return false;
    };
    let Some(Ok(ann)) = gci.annotation().next() else {
        return false;
    };
    if ann.source_file() != Ok("sensor.proto")
        || ann.begin() != Ok(5)
        || ann.end() != Ok(20)
        || ann.semantic() != Ok(Semantic::Set)
    {
        return false;
    }

    // Exercise generated enum (with allow_alias = true) and oneof constructs in #![no_std]
    if DeviceMode::ModeRunning != DeviceMode::ModeActive
        || DeviceMode::ModeStandby != DeviceMode::ModeSleep
        || DeviceMode::ModeRunning.as_i32() != 1
        || DeviceMode::ModeStandby.as_i32() != 2
        || DeviceMode::from_i32(0) != Some(DeviceMode::ModeIdle)
        || DeviceMode::from_i32(1) != Some(DeviceMode::ModeActive)
        || DeviceMode::from_i32(2) != Some(DeviceMode::ModeSleep)
        || DeviceMode::from_i32(99).is_some()
        || DeviceMode::try_from(1) != Ok(DeviceMode::ModeActive)
        || DeviceMode::try_from(99) != Err(99)
        || <DeviceMode as ::roto::runtime::VarintValue>::from_raw_varint(2) != DeviceMode::ModeSleep
        || <DeviceMode as ::roto::runtime::VarintValue>::from_raw_varint(99) != DeviceMode::ModeIdle
    {
        return false;
    }

    let payloads = [
        SensorPayload::Text("sensor-node-01"),
        SensorPayload::RawBytes(b"\xAA\xBB"),
        SensorPayload::Location(loc2),
        SensorPayload::Code(-42),
        SensorPayload::Mode(DeviceMode::ModeRunning),
    ];
    let SensorPayload::Text("sensor-node-01") = payloads[0] else {
        return false;
    };
    let SensorPayload::RawBytes(b"\xAA\xBB") = payloads[1] else {
        return false;
    };
    let SensorPayload::Location(p_loc) = payloads[2] else {
        return false;
    };
    if p_loc.lat_e7() != Ok(377749000) {
        return false;
    };
    let SensorPayload::Code(-42) = payloads[3] else {
        return false;
    };
    let SensorPayload::Mode(DeviceMode::ModeActive) = payloads[4] else {
        return false;
    };

    let thresholds = [
        AlertThreshold::MaxCount(100),
        AlertThreshold::DeltaNs(-500),
        AlertThreshold::MaxTempC(85.5),
        AlertThreshold::MaxRatio(0.75),
        AlertThreshold::Enabled(true),
        AlertThreshold::TargetMode(DeviceMode::ModeStandby),
    ];
    if thresholds[0] != AlertThreshold::MaxCount(100)
        || thresholds[1] != AlertThreshold::DeltaNs(-500)
        || thresholds[2] != AlertThreshold::MaxTempC(85.5)
        || thresholds[3] != AlertThreshold::MaxRatio(0.75)
        || thresholds[4] != AlertThreshold::Enabled(true)
        || thresholds[5] != AlertThreshold::TargetMode(DeviceMode::ModeSleep)
    {
        return false;
    }

    // Exercise AllTypesBuilder (all 18 field types, unpacked & packed repeated writers,
    // singularized aliases, nested message closures, and copy_unknown_fields)
    let mut all_buf1 = [0u8; 512];
    let mut all_writer1 = BackwardWriter::new(&mut all_buf1);
    // Unknown field 99 on AllTypes
    if all_writer1.write_varint(12345).is_err()
        || all_writer1.write_tag(99, WireType::Varint).is_err()
    {
        return false;
    }
    let mut b = AllTypesBuilder::from_writer(all_writer1);
    b.write_packed_f_sint64(&[-200, 300])
        .write_f_sint64(-100)
        .write_packed_f_sint32(&[-20, 30])
        .write_f_sint32(-10)
        .write_packed_f_sfixed64(&[-2000, 3000])
        .write_f_sfixed64(-1000)
        .write_packed_f_sfixed32(&[-200, 300])
        .write_f_sfixed32(-100)
        .write_packed_f_enum(&[DeviceMode::ModeActive, DeviceMode::ModeSleep])
        .write_f_enum(DeviceMode::ModeIdle)
        .write_packed_f_uint32(&[20, 30])
        .write_f_uint32(10)
        .write_f_bytes(b"\x01\x02")
        .write_location(|loc| {
            loc.write_lon_e7(22).write_lat_e7(11);
        })
        .write_locations(|loc| {
            loc.write_lon_e7(44).write_lat_e7(33);
        })
        .write_f_group(|grp| {
            grp.write_lon_e7(66).write_lat_e7(55);
        })
        .write_f_string("roto")
        .write_packed_f_bool(&[false, true])
        .write_f_bool(true)
        .write_packed_f_fixed32(&[200, 300])
        .write_f_fixed32(100)
        .write_packed_f_fixed64(&[2000, 3000])
        .write_f_fixed64(1000)
        .write_packed_f_int32(&[-2, 3])
        .write_f_int32(-1)
        .write_packed_f_uint64(&[200, 300])
        .write_f_uint64(100)
        .write_packed_f_int64(&[-20, 30])
        .write_f_int64(-10)
        .write_packed_f_float(&[2.5, 3.5])
        .write_f_float(1.5)
        .write_packed_f_double(&[2.25, 3.25])
        .write_f_double(1.25);
    if b.check().is_err() || b.writer().check().is_err() {
        return false;
    }
    let _ = b.writer_mut();
    let Ok(all_bytes1) = b.finish() else {
        return false;
    };
    let Ok(all_view1) = AllTypesView::from_bytes(all_bytes1) else {
        return false;
    };
    if all_view1.unknown_fields().count() != 1 {
        return false;
    }

    // Forward unknown fields into a new AllTypesBuilder and verify
    let mut all_buf2 = [0u8; 64];
    let mut b2 = AllTypesBuilder::new(&mut all_buf2);
    b2.copy_unknown_fields(&all_view1);
    let Ok(all_bytes2) = b2.finish() else {
        return false;
    };
    let Ok(all_view2) = AllTypesView::from_bytes(all_bytes2) else {
        return false;
    };
    if all_view2.unknown_fields().count() != 1 {
        return false;
    }

    let mut d_it = all_view1.f_double();
    if d_it.next() != Some(Ok(1.25))
        || d_it.next() != Some(Ok(2.25))
        || d_it.next() != Some(Ok(3.25))
        || d_it.next().is_some()
    {
        return false;
    }
    let mut fl_it = all_view1.f_float();
    if fl_it.next() != Some(Ok(1.5))
        || fl_it.next() != Some(Ok(2.5))
        || fl_it.next() != Some(Ok(3.5))
        || fl_it.next().is_some()
    {
        return false;
    }
    let mut i64_it = all_view1.f_int64();
    if i64_it.next() != Some(Ok(-10))
        || i64_it.next() != Some(Ok(-20))
        || i64_it.next() != Some(Ok(30))
        || i64_it.next().is_some()
    {
        return false;
    }
    let mut u64_it = all_view1.f_uint64();
    if u64_it.next() != Some(Ok(100))
        || u64_it.next() != Some(Ok(200))
        || u64_it.next() != Some(Ok(300))
        || u64_it.next().is_some()
    {
        return false;
    }
    let mut i32_it = all_view1.f_int32();
    if i32_it.next() != Some(Ok(-1))
        || i32_it.next() != Some(Ok(-2))
        || i32_it.next() != Some(Ok(3))
        || i32_it.next().is_some()
    {
        return false;
    }
    let mut fx64_it = all_view1.f_fixed64();
    if fx64_it.next() != Some(Ok(1000))
        || fx64_it.next() != Some(Ok(2000))
        || fx64_it.next() != Some(Ok(3000))
        || fx64_it.next().is_some()
    {
        return false;
    }
    let mut fx32_it = all_view1.f_fixed32();
    if fx32_it.next() != Some(Ok(100))
        || fx32_it.next() != Some(Ok(200))
        || fx32_it.next() != Some(Ok(300))
        || fx32_it.next().is_some()
    {
        return false;
    }
    let mut b_it = all_view1.f_bool();
    if b_it.next() != Some(Ok(true))
        || b_it.next() != Some(Ok(false))
        || b_it.next() != Some(Ok(true))
        || b_it.next().is_some()
    {
        return false;
    }
    let mut s_it = all_view1.f_string();
    if s_it.next() != Some(Ok("roto")) || s_it.next().is_some() {
        return false;
    }
    let mut grp_it = all_view1.f_group();
    let Some(Ok(grp_loc)) = grp_it.next() else {
        return false;
    };
    if grp_loc.lat_e7() != Ok(55) || grp_loc.lon_e7() != Ok(66) || grp_it.next().is_some() {
        return false;
    }
    let mut loc_it = all_view1.locations();
    let (Some(Ok(l1)), Some(Ok(l2))) = (loc_it.next(), loc_it.next()) else {
        return false;
    };
    if l1.lat_e7() != Ok(33)
        || l1.lon_e7() != Ok(44)
        || l2.lat_e7() != Ok(11)
        || l2.lon_e7() != Ok(22)
        || loc_it.next().is_some()
    {
        return false;
    }
    let mut by_it = all_view1.f_bytes();
    if by_it.next() != Some(Ok(&b"\x01\x02"[..])) || by_it.next().is_some() {
        return false;
    }
    let mut u32_it = all_view1.f_uint32();
    if u32_it.next() != Some(Ok(10))
        || u32_it.next() != Some(Ok(20))
        || u32_it.next() != Some(Ok(30))
        || u32_it.next().is_some()
    {
        return false;
    }
    let mut en_it = all_view1.f_enum();
    if en_it.next() != Some(Ok(DeviceMode::ModeIdle))
        || en_it.next() != Some(Ok(DeviceMode::ModeActive))
        || en_it.next() != Some(Ok(DeviceMode::ModeSleep))
        || en_it.next().is_some()
    {
        return false;
    }
    let mut sfx32_it = all_view1.f_sfixed32();
    if sfx32_it.next() != Some(Ok(-100))
        || sfx32_it.next() != Some(Ok(-200))
        || sfx32_it.next() != Some(Ok(300))
        || sfx32_it.next().is_some()
    {
        return false;
    }
    let mut sfx64_it = all_view1.f_sfixed64();
    if sfx64_it.next() != Some(Ok(-1000))
        || sfx64_it.next() != Some(Ok(-2000))
        || sfx64_it.next() != Some(Ok(3000))
        || sfx64_it.next().is_some()
    {
        return false;
    }
    let mut si32_it = all_view1.f_sint32();
    if si32_it.next() != Some(Ok(-10))
        || si32_it.next() != Some(Ok(-20))
        || si32_it.next() != Some(Ok(30))
        || si32_it.next().is_some()
    {
        return false;
    }
    let mut si64_it = all_view1.f_sint64();
    if si64_it.next() != Some(Ok(-100))
        || si64_it.next() != Some(Ok(-200))
        || si64_it.next() != Some(Ok(300))
        || si64_it.next().is_some()
    {
        return false;
    }

    // Also exercise SensorReadingBuilder::write_packed_samples and write_samples
    let mut sr_packed_buf = [0u8; 64];
    let mut srb = SensorReadingBuilder::new(&mut sr_packed_buf);
    srb.write_packed_samples(&[20, 30])
        .write_samples(10)
        .write_timestamp_ms(77);
    let Ok(sr_packed_bytes) = srb.finish() else {
        return false;
    };
    let Ok(sr_packed_view) = SensorReadingView::from_bytes(sr_packed_bytes) else {
        return false;
    };
    let mut sr_samples = sr_packed_view.samples();
    if sr_samples.next() != Some(Ok(10))
        || sr_samples.next() != Some(Ok(20))
        || sr_samples.next() != Some(Ok(30))
        || sr_samples.next().is_some()
    {
        return false;
    }

    // Exercise generated demo::AllTypesView<'a> (18 field types, required field checks,
    // proto2 defaults, oneof accessors with "last on wire wins", repeated iterators,
    // nested messages, and unknown fields)
    {
        use demo::all_types::{Choice, InnerView, Kind, Threshold};
        use demo::AllTypesView;

        // 1. Missing required field check
        if !matches!(
            AllTypesView::from_bytes(&[]),
            Err(DecodeError::MissingRequiredField("AllTypes.req_id"))
        ) {
            return false;
        }

        // 2. Minimal message (only required `req_id = 41` set) -> test all proto2 defaults
        let mut min_backing = [0u8; 32];
        let mut min_writer = BackwardWriter::new(&mut min_backing);
        if min_writer.write_varint(77).is_err()
            || min_writer.write_tag(41, WireType::Varint).is_err()
        {
            return false;
        }
        let Ok(min_bytes) = min_writer.finish() else {
            return false;
        };
        let Ok(min_view) = <AllTypesView<'_> as MessageView<'_>>::from_bytes(min_bytes) else {
            return false;
        };
        if min_view.as_bytes() != min_bytes
            || !min_view.has_req_id()
            || min_view.req_id() != Ok(77)
            || min_view.has_buf()
            || min_view.buf() != Ok(3.5f64)
            || min_view.has_opt_2()
            || min_view.opt_2() != Ok(f32::INFINITY)
            || min_view.has_opt_3()
            || min_view.opt_3() != Ok(-900i64)
            || min_view.has_opt_4()
            || min_view.opt_4() != Ok(400u64)
            || min_view.has_opt_5()
            || min_view.opt_5() != Ok(-50i32)
            || min_view.has_opt_6()
            || min_view.opt_6() != Ok(600u64)
            || min_view.has_opt_7()
            || min_view.opt_7() != Ok(70u32)
            || min_view.has_opt_8()
            || min_view.opt_8() != Ok(true)
            || min_view.has_opt_9()
            || min_view.opt_9() != Ok("default_str")
            || min_view.has_opt_10()
            || min_view.opt_10().ok().flatten().is_some()
            || min_view.has_opt_11()
            || min_view.opt_11().ok().flatten().is_some()
            || min_view.has_opt_12()
            || min_view.opt_12() != Ok(&b"\x00\xfeok"[..])
            || min_view.has_opt_13()
            || min_view.opt_13() != Ok(130u32)
            || min_view.has_opt_14()
            || min_view.opt_14() != Ok(Kind::KindB)
            || min_view.has_opt_15()
            || min_view.opt_15() != Ok(-150i32)
            || min_view.has_opt_16()
            || min_view.opt_16() != Ok(-1600i64)
            || min_view.has_opt_17()
            || min_view.opt_17() != Ok(-170i32)
            || min_view.has_opt_18()
            || min_view.opt_18() != Ok(-1800i64)
            || min_view.choice().ok().flatten().is_some()
            || min_view.threshold().ok().flatten().is_some()
            || min_view.unknown_fields().next().is_some()
        {
            return false;
        }

        // 3. Full message with all 18 singular fields, all 18 repeated fields, nested Inner,
        // unknown field (tag 99), and multiple oneof fields on the wire (verifying last-wins)
        let mut inner_backing = [0u8; 16];
        let mut ib = demo::all_types::InnerBuilder::new(&mut inner_backing);
        ib.write_val(321);
        if ib.check().is_err() || ib.writer().check().is_err() {
            return false;
        }
        let _ = ib.writer_mut();
        let Ok(inner_bytes) = ib.finish() else {
            return false;
        };

        let mut full_backing = [0u8; 512];
        let mut fw = BackwardWriter::new(&mut full_backing);
        // Note: BackwardWriter writes from back to front, so items written first appear LAST on the wire!
        // Unknown field tag 99
        if fw.write_varint(999).is_err() || fw.write_tag(99, WireType::Varint).is_err() {
            return false;
        }
        // Required field 41 = 1001
        if fw.write_varint(1001).is_err() || fw.write_tag(41, WireType::Varint).is_err() {
            return false;
        }
        // Oneof 1 (`threshold`): tag 39 written after tag 40 in backward writer -> tag 40 is last on wire!
        if fw.write_varint(Kind::KindB.as_i32() as u64).is_err()
            || fw.write_tag(40, WireType::Varint).is_err()
            || fw.write_zigzag32(-55).is_err()
            || fw.write_tag(39, WireType::Varint).is_err()
        {
            return false;
        }
        // Oneof 0 (`choice`): tag 37 written after tag 38 in backward writer -> tag 38 is last on wire!
        if fw.write_length_delimited(inner_bytes).is_err()
            || fw.write_tag(38, WireType::LengthDelimited).is_err()
            || fw.write_length_delimited(b"first_choice").is_err()
            || fw.write_tag(37, WireType::LengthDelimited).is_err()
        {
            return false;
        }
        // Repeated fields 19..=36
        if fw.write_zigzag64(-180).is_err()
            || fw.write_tag(36, WireType::Varint).is_err()
            || fw.write_zigzag32(-170).is_err()
            || fw.write_tag(35, WireType::Varint).is_err()
            || fw.write_sfixed64(-160).is_err()
            || fw.write_tag(34, WireType::Fixed64).is_err()
            || fw.write_sfixed32(-150).is_err()
            || fw.write_tag(33, WireType::Fixed32).is_err()
            || fw.write_varint(Kind::KindB.as_i32() as u64).is_err()
            || fw.write_tag(32, WireType::Varint).is_err()
            || fw.write_varint(13).is_err()
            || fw.write_tag(31, WireType::Varint).is_err()
            || fw.write_length_delimited(b"rep_bytes").is_err()
            || fw.write_tag(30, WireType::LengthDelimited).is_err()
            || fw.write_length_delimited(inner_bytes).is_err()
            || fw.write_tag(29, WireType::LengthDelimited).is_err()
            || fw.write_length_delimited(inner_bytes).is_err()
            || fw.write_tag(28, WireType::LengthDelimited).is_err()
            || fw.write_length_delimited(b"rep_str").is_err()
            || fw.write_tag(27, WireType::LengthDelimited).is_err()
            || fw.write_bool(true).is_err()
            || fw.write_tag(26, WireType::Varint).is_err()
            || fw.write_fixed32(77).is_err()
            || fw.write_tag(25, WireType::Fixed32).is_err()
            || fw.write_fixed64(66).is_err()
            || fw.write_tag(24, WireType::Fixed64).is_err()
            || fw.write_varint(55).is_err()
            || fw.write_tag(23, WireType::Varint).is_err()
            || fw.write_varint(44).is_err()
            || fw.write_tag(22, WireType::Varint).is_err()
            || fw.write_varint(33).is_err()
            || fw.write_tag(21, WireType::Varint).is_err()
            || fw.write_f32(2.25).is_err()
            || fw.write_tag(20, WireType::Fixed32).is_err()
            || fw.write_f64(1.25).is_err()
            || fw.write_tag(19, WireType::Fixed64).is_err()
        {
            return false;
        }
        // Singular fields 1..=18
        if fw.write_zigzag64(-18).is_err()
            || fw.write_tag(18, WireType::Varint).is_err()
            || fw.write_zigzag32(-17).is_err()
            || fw.write_tag(17, WireType::Varint).is_err()
            || fw.write_sfixed64(-16).is_err()
            || fw.write_tag(16, WireType::Fixed64).is_err()
            || fw.write_sfixed32(-15).is_err()
            || fw.write_tag(15, WireType::Fixed32).is_err()
            || fw.write_varint(Kind::KindA.as_i32() as u64).is_err()
            || fw.write_tag(14, WireType::Varint).is_err()
            || fw.write_varint(13).is_err()
            || fw.write_tag(13, WireType::Varint).is_err()
            || fw.write_length_delimited(b" custom_bytes").is_err()
            || fw.write_tag(12, WireType::LengthDelimited).is_err()
            || fw.write_length_delimited(inner_bytes).is_err()
            || fw.write_tag(11, WireType::LengthDelimited).is_err()
            || fw.write_length_delimited(inner_bytes).is_err()
            || fw.write_tag(10, WireType::LengthDelimited).is_err()
            || fw.write_length_delimited(b"custom_str").is_err()
            || fw.write_tag(9, WireType::LengthDelimited).is_err()
            || fw.write_bool(false).is_err()
            || fw.write_tag(8, WireType::Varint).is_err()
            || fw.write_fixed32(7).is_err()
            || fw.write_tag(7, WireType::Fixed32).is_err()
            || fw.write_fixed64(6).is_err()
            || fw.write_tag(6, WireType::Fixed64).is_err()
            || fw.write_varint(5).is_err()
            || fw.write_tag(5, WireType::Varint).is_err()
            || fw.write_varint(4).is_err()
            || fw.write_tag(4, WireType::Varint).is_err()
            || fw.write_varint(3).is_err()
            || fw.write_tag(3, WireType::Varint).is_err()
            || fw.write_f32(2.5).is_err()
            || fw.write_tag(2, WireType::Fixed32).is_err()
            || fw.write_f64(1.5).is_err()
            || fw.write_tag(1, WireType::Fixed64).is_err()
        {
            return false;
        }

        let Ok(full_bytes) = fw.finish() else {
            return false;
        };
        let Ok(full) = AllTypesView::from_bytes(full_bytes) else {
            return false;
        };
        if full.buf() != Ok(1.5)
            || full.opt_2() != Ok(2.5)
            || full.opt_3() != Ok(3)
            || full.opt_4() != Ok(4)
            || full.opt_5() != Ok(5)
            || full.opt_6() != Ok(6)
            || full.opt_7() != Ok(7)
            || full.opt_8() != Ok(false)
            || full.opt_9() != Ok("custom_str")
            || full.opt_12() != Ok(&b" custom_bytes"[..])
            || full.opt_13() != Ok(13)
            || full.opt_14() != Ok(Kind::KindA)
            || full.opt_15() != Ok(-15)
            || full.opt_16() != Ok(-16)
            || full.opt_17() != Ok(-17)
            || full.opt_18() != Ok(-18)
            || full.req_id() != Ok(1001)
        {
            return false;
        }
        let Ok(Some(in10)) = full.opt_10() else {
            return false;
        };
        let Ok(Some(in11)) = full.opt_11() else {
            return false;
        };
        if !in10.has_val()
            || in10.val() != Ok(321)
            || in11.val() != Ok(321)
            || in10.as_bytes() != inner_bytes
            || in10.unknown_fields().next().is_some()
            || <InnerView<'_> as MessageView<'_>>::from_bytes(&[])
                .and_then(|empty_in| empty_in.val())
                != Ok(0)
        {
            return false;
        }

        // Verify all 18 repeated field iterators
        if full.rep_1().next() != Some(Ok(1.25))
            || full.rep_2().next() != Some(Ok(2.25))
            || full.rep_3().next() != Some(Ok(33))
            || full.rep_4().next() != Some(Ok(44))
            || full.rep_5().next() != Some(Ok(55))
            || full.rep_6().next() != Some(Ok(66))
            || full.rep_7().next() != Some(Ok(77))
            || full.rep_8().next() != Some(Ok(true))
            || full.rep_9().next() != Some(Ok("rep_str"))
            || full.rep_10().next().map(|r| r.and_then(|v| v.val())) != Some(Ok(321))
            || full.rep_11().next().map(|r| r.and_then(|v| v.val())) != Some(Ok(321))
            || full.rep_12().next() != Some(Ok(&b"rep_bytes"[..]))
            || full.rep_13().next() != Some(Ok(13))
            || full.rep_14().next() != Some(Ok(Kind::KindB))
            || full.rep_15().next() != Some(Ok(-150))
            || full.rep_16().next() != Some(Ok(-160))
            || full.rep_17().next() != Some(Ok(-170))
            || full.rep_18().next() != Some(Ok(-180))
        {
            return false;
        }

        // Verify "last oneof field on the wire wins" for both borrowed and scalar oneofs
        let Ok(Some(Choice::ChoiceInner(choice_in))) = full.choice() else {
            return false;
        };
        if choice_in.val() != Ok(321)
            || !full.has_choice_str()
            || full.choice_str() != Ok("first_choice")
            || !full.has_choice_inner()
            || full.choice_inner().ok().flatten().is_none()
        {
            return false;
        }
        if full.threshold() != Ok(Some(Threshold::ThreshKind(Kind::KindB)))
            || !full.has_thresh_code()
            || full.thresh_code() != Ok(-55)
            || !full.has_thresh_kind()
            || full.thresh_kind() != Ok(Kind::KindB)
            || Kind::try_from(1) != Ok(Kind::KindB)
            || Kind::try_from(9) != Err(9)
        {
            return false;
        }

        // Verify unknown field preservation on AllTypesView
        if full.unknown_fields().count() != 1 {
            return false;
        }

        // Verify InnerBuilder::from_writer, copy_unknown_fields, and copy_unset_fields
        let mut inner_backing2 = [0u8; 32];
        let mut ib2 = demo::all_types::InnerBuilder::from_writer(BackwardWriter::new(&mut inner_backing2));
        ib2.copy_unknown_fields(&in10).copy_unset_fields(&in10);
        let Ok(inner_bytes2) = ib2.finish() else {
            return false;
        };
        let Ok(in10_copy) = InnerView::from_bytes(inner_bytes2) else {
            return false;
        };
        if in10_copy.val() != Ok(321) {
            return false;
        }
    }

    // Exercise copy_unset_fields on SensorReadingBuilder, LocationBuilder, AllTypesBuilder, and VersionBuilder
    let mut partial_sr_buf = [0u8; 128];
    let mut partial_srb = SensorReadingBuilder::new(&mut partial_sr_buf);
    partial_srb
        .write_location(|loc| {
            loc.write_lat_e7(111111111).copy_unset_fields(&loc1);
        })
        .write_device_id("sensor-node-updated")
        .copy_unset_fields(&reading1);
    let Ok(partial_sr_bytes) = partial_srb.finish() else {
        return false;
    };
    let Ok(partial_sr) = SensorReadingView::from_bytes(partial_sr_bytes) else {
        return false;
    };
    let Ok(Some(partial_loc)) = partial_sr.location() else {
        return false;
    };
    let mut partial_samples = partial_sr.samples();
    if partial_sr.timestamp_ms() != Ok(1_700_000_000_000)
        || partial_sr.device_id() != Ok("sensor-node-updated")
        || partial_loc.lat_e7() != Ok(111111111)
        || partial_loc.lon_e7() != Ok(-1224194000)
        || partial_loc.unknown_fields().count() != 1
        || partial_samples.next() != Some(Ok(42))
        || partial_samples.next() != Some(Ok(108))
        || partial_samples.next().is_some()
        || partial_sr.unknown_fields().count() != 1
    {
        return false;
    }

    let mut partial_all_buf = [0u8; 512];
    let mut partial_all = AllTypesBuilder::new(&mut partial_all_buf);
    partial_all
        .write_f_string("updated-roto")
        .copy_unset_fields(&all_view1);
    let Ok(partial_all_bytes) = partial_all.finish() else {
        return false;
    };
    let Ok(partial_all_view) = AllTypesView::from_bytes(partial_all_bytes) else {
        return false;
    };
    let mut partial_str_it = partial_all_view.f_string();
    let mut partial_u32_it = partial_all_view.f_uint32();
    if partial_str_it.next() != Some(Ok("updated-roto"))
        || partial_str_it.next().is_some()
        || partial_u32_it.next() != Some(Ok(10))
        || partial_u32_it.next() != Some(Ok(20))
        || partial_u32_it.next() != Some(Ok(30))
        || partial_u32_it.next().is_some()
        || partial_all_view.unknown_fields().count() != 1
    {
        return false;
    }

    let mut partial_ver_buf = [0u8; 64];
    let mut partial_vb = VersionBuilder::new(&mut partial_ver_buf);
    partial_vb
        .write_patch(9)
        .copy_unset_fields(&standalone_ver);
    let Ok(partial_ver_bytes) = partial_vb.finish() else {
        return false;
    };
    let Ok(partial_ver) = VersionView::from_bytes(partial_ver_bytes) else {
        return false;
    };
    if partial_ver.major() != Ok(4)
        || partial_ver.minor() != Ok(28)
        || partial_ver.patch() != Ok(9)
        || partial_ver.suffix() != Ok("rc1")
    {
        return false;
    }

    // Exercise google-protobuf-roto-bootstrap and google-protobuf-roto WKTs in #![no_std]
    let mut boot_buf = [0u8; 64];
    let mut boot_vb =
        google_protobuf_roto_bootstrap::google::protobuf::compiler::VersionBuilder::new(
            &mut boot_buf,
        );
    boot_vb.write_patch(2).write_minor(28).write_major(4);
    let Ok(boot_bytes) = boot_vb.finish() else {
        return false;
    };
    let Ok(boot_ver) =
        google_protobuf_roto_bootstrap::google::protobuf::compiler::VersionView::from_bytes(
            boot_bytes,
        )
    else {
        return false;
    };
    if boot_ver.major() != Ok(4) || boot_ver.minor() != Ok(28) || boot_ver.patch() != Ok(2) {
        return false;
    }

    let mut wkt_ts_buf = [0u8; 64];
    let mut wkt_ts =
        google_protobuf_roto::google::protobuf::TimestampBuilder::new(&mut wkt_ts_buf);
    wkt_ts.write_nanos(999).write_seconds(1_700_000_000);
    let Ok(wkt_ts_bytes) = wkt_ts.finish() else {
        return false;
    };
    let Ok(wkt_ts_view) =
        google_protobuf_roto::google::protobuf::TimestampView::from_bytes(wkt_ts_bytes)
    else {
        return false;
    };
    if wkt_ts_view.seconds() != Ok(1_700_000_000) || wkt_ts_view.nanos() != Ok(999) {
        return false;
    }

    if !verify_roto_grpc_no_std() {
        return false;
    }

    true
}

pub fn verify_roto_grpc_no_std() -> bool {
    use roto_grpc::framing::{
        decode_grpc_frame, iter_grpc_frames, write_grpc_frame, DEFAULT_MAX_MESSAGE_SIZE,
        GRPC_HEADER_LEN,
    };
    use roto_grpc::status::{
        percent_decode, percent_encode, percent_encode_backward, GrpcCode, GrpcStatus,
    };

    if GrpcCode::from_header_str("0") != Some(GrpcCode::Ok)
        || GrpcCode::from_header_str("16") != Some(GrpcCode::Unauthenticated)
        || GrpcCode::from_header_str("00").is_some()
        || GrpcCode::from_header_str("17").is_some()
        || !GrpcStatus::ok().is_ok()
        || GrpcStatus::unimplemented("not ready").is_ok()
    {
        return false;
    }

    let mut enc_buf = [0u8; 64];
    let Ok(encoded_msg) = percent_encode("err: 100% \n", &mut enc_buf) else {
        return false;
    };
    if encoded_msg != "err: 100%25 %0A" {
        return false;
    }

    let mut bw_buf = [0u8; 64];
    let mut bw = BackwardWriter::new(&mut bw_buf);
    percent_encode_backward("err: 100% \n", &mut bw);
    let Ok(bw_slice) = bw.finish() else {
        return false;
    };
    if bw_slice != encoded_msg.as_bytes() {
        return false;
    }

    let mut dec_buf = [0u8; 64];
    let Ok(decoded_msg) = percent_decode(bw_slice, &mut dec_buf) else {
        return false;
    };
    if decoded_msg != "err: 100% \n" {
        return false;
    }

    let mut frame_buf = [0u8; 128];
    let mut frame_writer = BackwardWriter::new(&mut frame_buf);
    // Write two consecutive frames backwards (so the second call ends up at the front of the wire buffer)
    if write_grpc_frame(&mut frame_writer, true, |w| {
        w.with_sub_writer(|sub| {
            let mut lb = LocationBuilder::from_writer(sub);
            lb.write_lon_e7(200).write_lat_e7(100);
            lb.writer
        });
    })
    .is_err()
    {
        return false;
    }
    if write_grpc_frame(&mut frame_writer, false, |w| {
        w.with_sub_writer(|sub| {
            let mut lb = LocationBuilder::from_writer(sub);
            lb.write_lon_e7(400).write_lat_e7(300);
            lb.writer
        });
    })
    .is_err()
    {
        return false;
    }

    let Ok(wire_frames) = frame_writer.finish() else {
        return false;
    };
    if wire_frames.len() <= GRPC_HEADER_LEN * 2 {
        return false;
    }

    let Ok(Some((first_frame, rest))) = decode_grpc_frame(wire_frames, DEFAULT_MAX_MESSAGE_SIZE)
    else {
        return false;
    };
    if first_frame.compressed || rest.is_empty() {
        return false;
    }
    let Ok(loc_first) = LocationView::from_bytes(first_frame.payload) else {
        return false;
    };
    if loc_first.lat_e7() != Ok(300) || loc_first.lon_e7() != Ok(400) {
        return false;
    }

    let mut iter = iter_grpc_frames(wire_frames, DEFAULT_MAX_MESSAGE_SIZE);
    let Some(Ok(f1)) = iter.next() else {
        return false;
    };
    let Some(Ok(f2)) = iter.next() else {
        return false;
    };
    if f1.compressed || !f2.compressed || iter.next().is_some() || !iter.remainder().is_empty() {
        return false;
    }

    // Exercise roto_grpc::hpack in bare-metal #![no_std] mode
    use roto_grpc::hpack::{
        decode_prefix_int, encode_prefix_int, huffman_decode, huffman_encode, static_table_get,
        GrpcRequestHeaders, GrpcResponseHeaders, GrpcTrailers, HpackDecoder, HpackEncoder,
        GRPC_CONTENT_TYPE_ROTO, INDEX_METHOD_POST,
    };

    if static_table_get(INDEX_METHOD_POST) != Some((&b":method"[..], &b"POST"[..])) {
        return false;
    }

    let mut pint_buf = [0u8; 8];
    let Ok(pint_len) = encode_prefix_int(5, 0x00, 1337, &mut pint_buf) else {
        return false;
    };
    if pint_buf[..pint_len] != [0x1f, 0x9a, 0x0a]
        || decode_prefix_int(5, &pint_buf[..pint_len]) != Ok((1337, 3))
    {
        return false;
    }

    let mut huff_wire = [0u8; 32];
    let mut huff_out = [0u8; 32];
    let Ok(huff_len) = huffman_encode(b"www.example.com", &mut huff_wire) else {
        return false;
    };
    let Ok(huff_dec) = huffman_decode(&huff_wire[..huff_len], &mut huff_out) else {
        return false;
    };
    if huff_dec != b"www.example.com" {
        return false;
    }

    let hpack_enc = HpackEncoder::with_huffman(true);
    let mut hpack_wire = [0u8; 128];
    let mut hpack_scratch = [0u8; 128];
    let mut hpack_dec = HpackDecoder::<256>::new();

    let Ok(req_len) = hpack_enc.encode_grpc_request_headers(
        b"http",
        b"localhost:50051",
        b"/sensor.Node/Read",
        GRPC_CONTENT_TYPE_ROTO,
        &mut hpack_wire,
    ) else {
        return false;
    };
    let Ok(req_hdrs) =
        GrpcRequestHeaders::decode(&mut hpack_dec, &hpack_wire[..req_len], &mut hpack_scratch)
    else {
        return false;
    };
    if !req_hdrs.is_valid_grpc_post() || req_hdrs.path_str() != Some("/sensor.Node/Read") {
        return false;
    }

    let Ok(resp_len) =
        hpack_enc.encode_grpc_response_headers(GRPC_CONTENT_TYPE_ROTO, &mut hpack_wire)
    else {
        return false;
    };
    let Ok(resp_hdrs) =
        GrpcResponseHeaders::decode(&mut hpack_dec, &hpack_wire[..resp_len], &mut hpack_scratch)
    else {
        return false;
    };
    if resp_hdrs.status != Some(&b"200"[..])
        || resp_hdrs.content_type != Some(GRPC_CONTENT_TYPE_ROTO)
    {
        return false;
    }

    let expected_status = GrpcStatus::internal("bus error: 100%");
    let Ok(tr_len) = hpack_enc.encode_grpc_trailers(expected_status, &mut hpack_wire) else {
        return false;
    };
    let Ok(trailers) =
        GrpcTrailers::decode(&mut hpack_dec, &hpack_wire[..tr_len], &mut hpack_scratch)
    else {
        return false;
    };
    let mut msg_scratch = [0u8; 64];
    if trailers.to_grpc_status(&mut msg_scratch) != Ok(expected_status) {
        return false;
    }

    // Exercise roto_grpc::h2 Sans-I/O HTTP/2 engine in bare-metal #![no_std] mode
    use roto_grpc::h2::{H2Connection, H2Event, StreamState};

    let mut h2_client = H2Connection::<2, 256>::new_client();
    let mut h2_server = H2Connection::<2, 256>::new_server();
    let mut h2_buf = [0u8; 256];

    // 1. Client -> Server preface + SETTINGS
    let c_init_len = h2_client.poll_transmit(&mut h2_buf);
    let mut pos = 0;
    while pos < c_init_len {
        let Ok((n, _)) = h2_server.recv(&h2_buf[pos..c_init_len]) else {
            return false;
        };
        pos += n;
    }
    if !h2_server.is_handshake_complete() {
        return false;
    }

    // 2. Server -> Client SETTINGS + SETTINGS ACK
    let s_init_len = h2_server.poll_transmit(&mut h2_buf);
    pos = 0;
    while pos < s_init_len {
        let Ok((n, _)) = h2_client.recv(&h2_buf[pos..s_init_len]) else {
            return false;
        };
        pos += n;
    }
    if !h2_client.is_handshake_complete() || !h2_client.peer_acked_settings() {
        return false;
    }

    // 3. Client -> Server SETTINGS ACK
    let c_ack_len = h2_client.poll_transmit(&mut h2_buf);
    if h2_server.recv(&h2_buf[..c_ack_len]) != Ok((c_ack_len, Some(H2Event::SettingsAck))) {
        return false;
    }

    // 4. Client sends gRPC request HEADERS + single-pass DATA frame on stream 1
    let Ok((sid, req_h2_len)) = h2_client.send_grpc_request_headers(
        b"http",
        b"localhost:50051",
        b"/sensor.Node/Read",
        GRPC_CONTENT_TYPE_ROTO,
        false,
        &mut h2_buf,
    ) else {
        return false;
    };
    if sid != 1 {
        return false;
    }
    let Ok((_, Some(H2Event::Headers { stream_id: 1, .. }))) =
        h2_server.recv(&h2_buf[..req_h2_len])
    else {
        return false;
    };

    let Ok(data_wire) = h2_client.send_grpc_data_frame(1, false, true, &mut h2_buf, |w| {
        w.with_sub_writer(|sub| {
            let mut lb = LocationBuilder::from_writer(sub);
            lb.write_lon_e7(888).write_lat_e7(777);
            lb.writer
        });
    }) else {
        return false;
    };
    let Ok((
        _,
        Some(H2Event::GrpcMessage {
            stream_id: 1,
            compressed: false,
            payload: msg_payload,
            end_stream: true,
        }),
    )) = h2_server.recv(data_wire)
    else {
        return false;
    };
    let Ok(h2_loc) = LocationView::from_bytes(msg_payload) else {
        return false;
    };
    if h2_loc.lat_e7() != Ok(777)
        || h2_loc.lon_e7() != Ok(888)
        || h2_server.stream_state(1) != StreamState::HalfClosedRemote
    {
        return false;
    }

    // 5. Server sends Trailers-Only OK response to close stream 1
    let Ok(tr_h2_len) = h2_server.send_grpc_trailers_only(
        1,
        GrpcStatus::ok(),
        GRPC_CONTENT_TYPE_ROTO,
        &mut h2_buf,
    ) else {
        return false;
    };
    if h2_server.stream_state(1) != StreamState::Closed {
        return false;
    }
    let Ok((
        _,
        Some(H2Event::Headers {
            stream_id: 1,
            end_stream: true,
            ..
        }),
    )) = h2_client.recv(&h2_buf[..tr_h2_len])
    else {
        return false;
    };
    if h2_client.stream_state(1) != StreamState::Closed {
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
        assert!(verify_roto_grpc_no_std());
    }
}
