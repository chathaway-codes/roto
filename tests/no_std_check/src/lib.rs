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


pub struct LocationBuilder<'a, 'b> {
    pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
}

impl<'a, 'b> LocationBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &LocationView<'_>,
    ) -> Result<(), ::roto::runtime::EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_lat_e7(&mut self, val: i32) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(1, ::roto::runtime::WireType::Varint)
    }

    #[inline]
    pub fn write_lon_e7(&mut self, val: i32) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(2, ::roto::runtime::WireType::Varint)
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
    pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
}

impl<'a, 'b> SensorReadingBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &SensorReadingView<'_>,
    ) -> Result<(), ::roto::runtime::EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_timestamp_ms(&mut self, val: u64) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_varint(val)?;
        self.writer.write_tag(1, ::roto::runtime::WireType::Varint)
    }

    #[inline]
    pub fn write_device_id(&mut self, val: &str) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer
            .write_tag(2, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_location<F>(&mut self, f: F) -> Result<(), ::roto::runtime::EncodeError>
    where
        F: FnOnce(&mut LocationBuilder<'_, 'b>) -> Result<(), ::roto::runtime::EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = LocationBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer
            .write_tag(3, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_samples(&mut self, val: u32) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(4, ::roto::runtime::WireType::Varint)
    }

    #[inline]
    pub fn write_sample(&mut self, val: u32) -> Result<(), ::roto::runtime::EncodeError> {
        self.write_samples(val)
    }

    #[inline]
    pub fn write_packed_samples(
        &mut self,
        vals: &[u32],
    ) -> Result<(), ::roto::runtime::EncodeError> {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            self.writer.write_varint(val as u64)?;
        }
        let packed_len = end_pos - self.writer.cursor();
        self.writer.write_varint(packed_len as u64)?;
        self.writer
            .write_tag(4, ::roto::runtime::WireType::LengthDelimited)
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

pub struct AllTypesBuilder<'a, 'b> {
    pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
}

impl<'a, 'b> AllTypesBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &AllTypesView<'_>,
    ) -> Result<(), ::roto::runtime::EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_f_double(&mut self, val: f64) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_f64(val)?;
        self.writer.write_tag(1, ::roto::runtime::WireType::Fixed64)
    }

    #[inline]
    pub fn write_packed_f_double(
        &mut self,
        vals: &[f64],
    ) -> Result<(), ::roto::runtime::EncodeError> {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            self.writer.write_f64(val)?;
        }
        let packed_len = end_pos - self.writer.cursor();
        self.writer.write_varint(packed_len as u64)?;
        self.writer
            .write_tag(1, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_f_float(&mut self, val: f32) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_f32(val)?;
        self.writer.write_tag(2, ::roto::runtime::WireType::Fixed32)
    }

    #[inline]
    pub fn write_packed_f_float(
        &mut self,
        vals: &[f32],
    ) -> Result<(), ::roto::runtime::EncodeError> {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            self.writer.write_f32(val)?;
        }
        let packed_len = end_pos - self.writer.cursor();
        self.writer.write_varint(packed_len as u64)?;
        self.writer
            .write_tag(2, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_f_int64(&mut self, val: i64) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(3, ::roto::runtime::WireType::Varint)
    }

    #[inline]
    pub fn write_packed_f_int64(
        &mut self,
        vals: &[i64],
    ) -> Result<(), ::roto::runtime::EncodeError> {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            self.writer.write_varint(val as u64)?;
        }
        let packed_len = end_pos - self.writer.cursor();
        self.writer.write_varint(packed_len as u64)?;
        self.writer
            .write_tag(3, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_f_uint64(&mut self, val: u64) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_varint(val)?;
        self.writer.write_tag(4, ::roto::runtime::WireType::Varint)
    }

    #[inline]
    pub fn write_packed_f_uint64(
        &mut self,
        vals: &[u64],
    ) -> Result<(), ::roto::runtime::EncodeError> {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            self.writer.write_varint(val)?;
        }
        let packed_len = end_pos - self.writer.cursor();
        self.writer.write_varint(packed_len as u64)?;
        self.writer
            .write_tag(4, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_f_int32(&mut self, val: i32) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(5, ::roto::runtime::WireType::Varint)
    }

    #[inline]
    pub fn write_packed_f_int32(
        &mut self,
        vals: &[i32],
    ) -> Result<(), ::roto::runtime::EncodeError> {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            self.writer.write_varint(val as u64)?;
        }
        let packed_len = end_pos - self.writer.cursor();
        self.writer.write_varint(packed_len as u64)?;
        self.writer
            .write_tag(5, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_f_fixed64(&mut self, val: u64) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_fixed64(val)?;
        self.writer.write_tag(6, ::roto::runtime::WireType::Fixed64)
    }

    #[inline]
    pub fn write_packed_f_fixed64(
        &mut self,
        vals: &[u64],
    ) -> Result<(), ::roto::runtime::EncodeError> {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            self.writer.write_fixed64(val)?;
        }
        let packed_len = end_pos - self.writer.cursor();
        self.writer.write_varint(packed_len as u64)?;
        self.writer
            .write_tag(6, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_f_fixed32(&mut self, val: u32) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_fixed32(val)?;
        self.writer.write_tag(7, ::roto::runtime::WireType::Fixed32)
    }

    #[inline]
    pub fn write_packed_f_fixed32(
        &mut self,
        vals: &[u32],
    ) -> Result<(), ::roto::runtime::EncodeError> {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            self.writer.write_fixed32(val)?;
        }
        let packed_len = end_pos - self.writer.cursor();
        self.writer.write_varint(packed_len as u64)?;
        self.writer
            .write_tag(7, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_f_bool(&mut self, val: bool) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(8, ::roto::runtime::WireType::Varint)
    }

    #[inline]
    pub fn write_packed_f_bool(
        &mut self,
        vals: &[bool],
    ) -> Result<(), ::roto::runtime::EncodeError> {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            self.writer.write_bool(val)?;
        }
        let packed_len = end_pos - self.writer.cursor();
        self.writer.write_varint(packed_len as u64)?;
        self.writer
            .write_tag(8, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_f_string(&mut self, val: &str) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer
            .write_tag(9, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_f_group<F>(&mut self, f: F) -> Result<(), ::roto::runtime::EncodeError>
    where
        F: FnOnce(&mut LocationBuilder<'_, 'b>) -> Result<(), ::roto::runtime::EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = LocationBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer
            .write_tag(10, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_locations<F>(&mut self, f: F) -> Result<(), ::roto::runtime::EncodeError>
    where
        F: FnOnce(&mut LocationBuilder<'_, 'b>) -> Result<(), ::roto::runtime::EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = LocationBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer
            .write_tag(11, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_location<F>(&mut self, f: F) -> Result<(), ::roto::runtime::EncodeError>
    where
        F: FnOnce(&mut LocationBuilder<'_, 'b>) -> Result<(), ::roto::runtime::EncodeError>,
    {
        self.write_locations(f)
    }

    #[inline]
    pub fn write_f_bytes(&mut self, val: &[u8]) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_length_delimited(val)?;
        self.writer
            .write_tag(12, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_f_uint32(&mut self, val: u32) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(13, ::roto::runtime::WireType::Varint)
    }

    #[inline]
    pub fn write_packed_f_uint32(
        &mut self,
        vals: &[u32],
    ) -> Result<(), ::roto::runtime::EncodeError> {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            self.writer.write_varint(val as u64)?;
        }
        let packed_len = end_pos - self.writer.cursor();
        self.writer.write_varint(packed_len as u64)?;
        self.writer
            .write_tag(13, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_f_enum(&mut self, val: DeviceMode) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_varint(val.as_i32() as u64)?;
        self.writer.write_tag(14, ::roto::runtime::WireType::Varint)
    }

    #[inline]
    pub fn write_packed_f_enum(
        &mut self,
        vals: &[DeviceMode],
    ) -> Result<(), ::roto::runtime::EncodeError> {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            self.writer.write_varint(val.as_i32() as u64)?;
        }
        let packed_len = end_pos - self.writer.cursor();
        self.writer.write_varint(packed_len as u64)?;
        self.writer
            .write_tag(14, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_f_sfixed32(&mut self, val: i32) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_sfixed32(val)?;
        self.writer
            .write_tag(15, ::roto::runtime::WireType::Fixed32)
    }

    #[inline]
    pub fn write_packed_f_sfixed32(
        &mut self,
        vals: &[i32],
    ) -> Result<(), ::roto::runtime::EncodeError> {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            self.writer.write_sfixed32(val)?;
        }
        let packed_len = end_pos - self.writer.cursor();
        self.writer.write_varint(packed_len as u64)?;
        self.writer
            .write_tag(15, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_f_sfixed64(&mut self, val: i64) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_sfixed64(val)?;
        self.writer
            .write_tag(16, ::roto::runtime::WireType::Fixed64)
    }

    #[inline]
    pub fn write_packed_f_sfixed64(
        &mut self,
        vals: &[i64],
    ) -> Result<(), ::roto::runtime::EncodeError> {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            self.writer.write_sfixed64(val)?;
        }
        let packed_len = end_pos - self.writer.cursor();
        self.writer.write_varint(packed_len as u64)?;
        self.writer
            .write_tag(16, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_f_sint32(&mut self, val: i32) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_zigzag32(val)?;
        self.writer.write_tag(17, ::roto::runtime::WireType::Varint)
    }

    #[inline]
    pub fn write_packed_f_sint32(
        &mut self,
        vals: &[i32],
    ) -> Result<(), ::roto::runtime::EncodeError> {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            self.writer.write_zigzag32(val)?;
        }
        let packed_len = end_pos - self.writer.cursor();
        self.writer.write_varint(packed_len as u64)?;
        self.writer
            .write_tag(17, ::roto::runtime::WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_f_sint64(&mut self, val: i64) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.write_zigzag64(val)?;
        self.writer.write_tag(18, ::roto::runtime::WireType::Varint)
    }

    #[inline]
    pub fn write_packed_f_sint64(
        &mut self,
        vals: &[i64],
    ) -> Result<(), ::roto::runtime::EncodeError> {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            self.writer.write_zigzag64(val)?;
        }
        let packed_len = end_pos - self.writer.cursor();
        self.writer.write_varint(packed_len as u64)?;
        self.writer
            .write_tag(18, ::roto::runtime::WireType::LengthDelimited)
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

    // Exercise google_protobuf_roto descriptor Views, Builders, Options, and Metadata in #![no_std]
    let mut desc_backing = [0u8; 512];
    let mut desc_writer = BackwardWriter::new(&mut desc_backing);
    {
        let mut fds = FileDescriptorSetBuilder::new(&mut desc_writer);
        if fds
            .write_file(|file| {
                file.write_edition(Edition::Edition2023)?;
                file.write_syntax("editions")?;
                file.write_source_code_info(|sci| {
                    sci.write_location(|loc| {
                        loc.write_leading_comments("SensorReading doc")?;
                        loc.write_packed_span(&[10, 0, 15, 1])?;
                        loc.write_packed_path(&[4, 0])
                    })
                })?;
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
                    msg.write_options(|mopt| {
                        mopt.write_uninterpreted_option(|uo| {
                            uo.write_identifier_value("TRUE")?;
                            uo.write_name(|np| {
                                np.write_is_extension(true)?;
                                np.write_name_part("demo.custom_msg_opt")
                            })
                        })?;
                        mopt.write_map_entry(false)?;
                        mopt.write_deprecated(true)
                    })?;
                    msg.write_field(|fld| {
                        fld.write_proto3_optional(true)?;
                        fld.write_json_name("timestampMs")?;
                        fld.write_options(|fopt| {
                            fopt.write_deprecated(false)?;
                            fopt.write_packed(true)
                        })?;
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
    let mut ver_writer = BackwardWriter::new(&mut ver_backing);
    {
        let mut vb = VersionBuilder::new(&mut ver_writer);
        if vb.write_suffix("rc1").is_err()
            || vb.write_patch(1).is_err()
            || vb.write_minor(28).is_err()
            || vb.write_major(4).is_err()
        {
            return false;
        }
    }
    let Ok(standalone_ver) = VersionView::from_bytes(ver_writer.finish()) else {
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
    let mut req_writer = BackwardWriter::new(&mut req_backing);
    {
        let mut req = CodeGeneratorRequestBuilder::new(&mut req_writer);
        if req
            .write_compiler_version(|ver| {
                ver.write_suffix("")?;
                ver.write_patch(0)?;
                ver.write_minor(28)?;
                ver.write_major(4)
            })
            .is_err()
            || req
                .write_source_file_descriptors(|fdp| {
                    fdp.write_syntax("proto2")?;
                    fdp.write_package("demo")?;
                    fdp.write_name("sensor.proto")
                })
                .is_err()
            || req
                .write_proto_file(|fdp| fdp.writer.write_bytes(file_view.as_bytes()))
                .is_err()
            || req.write_parameter("no_std").is_err()
            || req.write_file_to_generate("sensor.proto").is_err()
        {
            return false;
        }
    }
    let Ok(req_view) = CodeGeneratorRequestView::from_bytes(req_writer.finish()) else {
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
    let mut file_writer = BackwardWriter::new(&mut file_backing);
    {
        let mut fb = FileBuilder::new(&mut file_writer);
        if fb.write_content("// standalone").is_err() || fb.write_name("standalone.rs").is_err() {
            return false;
        }
    }
    let Ok(standalone_file) = FileView::from_bytes(file_writer.finish()) else {
        return false;
    };
    if standalone_file.name() != Ok("standalone.rs")
        || standalone_file.content() != Ok("// standalone")
    {
        return false;
    }

    let mut resp_backing = [0u8; 256];
    let mut resp_writer = BackwardWriter::new(&mut resp_backing);
    {
        let mut resp = CodeGeneratorResponseBuilder::new(&mut resp_writer);
        let features = (Feature::FeatureProto3Optional.as_i32()
            | Feature::FeatureSupportsEditions.as_i32()) as u64;
        if resp
            .write_file(|f| {
                f.write_generated_code_info(|gci| {
                    gci.write_annotation(|ann| {
                        ann.write_semantic(Semantic::Set)?;
                        ann.write_end(20)?;
                        ann.write_begin(5)?;
                        ann.write_source_file("sensor.proto")?;
                        ann.write_packed_path(&[4, 0])
                    })
                })?;
                f.write_content("pub mod demo {}\n")?;
                f.write_insertion_point("scope")?;
                f.write_name("sensor.rs")
            })
            .is_err()
            || resp
                .write_maximum_edition(Edition::Edition2024.as_i32())
                .is_err()
            || resp
                .write_minimum_edition(Edition::EditionProto2.as_i32())
                .is_err()
            || resp.write_supported_features(features).is_err()
        {
            return false;
        }
    }
    let Ok(resp_view) = CodeGeneratorResponseView::from_bytes(resp_writer.finish()) else {
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
    {
        // Unknown field 99 on AllTypes
        if all_writer1.write_varint(12345).is_err()
            || all_writer1.write_tag(99, WireType::Varint).is_err()
        {
            return false;
        }
        let mut b = AllTypesBuilder::new(&mut all_writer1);
        if b.write_packed_f_sint64(&[-200, 300]).is_err()
            || b.write_f_sint64(-100).is_err()
            || b.write_packed_f_sint32(&[-20, 30]).is_err()
            || b.write_f_sint32(-10).is_err()
            || b.write_packed_f_sfixed64(&[-2000, 3000]).is_err()
            || b.write_f_sfixed64(-1000).is_err()
            || b.write_packed_f_sfixed32(&[-200, 300]).is_err()
            || b.write_f_sfixed32(-100).is_err()
            || b.write_packed_f_enum(&[DeviceMode::ModeActive, DeviceMode::ModeSleep])
                .is_err()
            || b.write_f_enum(DeviceMode::ModeIdle).is_err()
            || b.write_packed_f_uint32(&[20, 30]).is_err()
            || b.write_f_uint32(10).is_err()
            || b.write_f_bytes(b"\x01\x02").is_err()
            || b.write_location(|loc| {
                loc.write_lon_e7(22)?;
                loc.write_lat_e7(11)
            })
            .is_err()
            || b.write_locations(|loc| {
                loc.write_lon_e7(44)?;
                loc.write_lat_e7(33)
            })
            .is_err()
            || b.write_f_group(|grp| {
                grp.write_lon_e7(66)?;
                grp.write_lat_e7(55)
            })
            .is_err()
            || b.write_f_string("roto").is_err()
            || b.write_packed_f_bool(&[false, true]).is_err()
            || b.write_f_bool(true).is_err()
            || b.write_packed_f_fixed32(&[200, 300]).is_err()
            || b.write_f_fixed32(100).is_err()
            || b.write_packed_f_fixed64(&[2000, 3000]).is_err()
            || b.write_f_fixed64(1000).is_err()
            || b.write_packed_f_int32(&[-2, 3]).is_err()
            || b.write_f_int32(-1).is_err()
            || b.write_packed_f_uint64(&[200, 300]).is_err()
            || b.write_f_uint64(100).is_err()
            || b.write_packed_f_int64(&[-20, 30]).is_err()
            || b.write_f_int64(-10).is_err()
            || b.write_packed_f_float(&[2.5, 3.5]).is_err()
            || b.write_f_float(1.5).is_err()
            || b.write_packed_f_double(&[2.25, 3.25]).is_err()
            || b.write_f_double(1.25).is_err()
        {
            return false;
        }
    }
    let Ok(all_view1) = AllTypesView::from_bytes(all_writer1.finish()) else {
        return false;
    };
    if all_view1.unknown_fields().count() != 1 {
        return false;
    }

    // Forward unknown fields into a new AllTypesBuilder and verify
    let mut all_buf2 = [0u8; 64];
    let mut all_writer2 = BackwardWriter::new(&mut all_buf2);
    {
        let mut b2 = AllTypesBuilder::new(&mut all_writer2);
        if b2.copy_unknown_fields(&all_view1).is_err() {
            return false;
        }
    }
    let Ok(all_view2) = AllTypesView::from_bytes(all_writer2.finish()) else {
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
    let mut sr_packed_writer = BackwardWriter::new(&mut sr_packed_buf);
    {
        let mut srb = SensorReadingBuilder::new(&mut sr_packed_writer);
        if srb.write_packed_samples(&[20, 30]).is_err()
            || srb.write_samples(10).is_err()
            || srb.write_timestamp_ms(77).is_err()
        {
            return false;
        }
    }
    let Ok(sr_packed_view) = SensorReadingView::from_bytes(sr_packed_writer.finish()) else {
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
