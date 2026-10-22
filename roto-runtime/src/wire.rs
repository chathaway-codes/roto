use crate::error::DecodeError;

/// Protocol Buffers wire types.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WireType {
    Varint = 0,
    Fixed64 = 1,
    LengthDelimited = 2,
    StartGroup = 3,
    EndGroup = 4,
    Fixed32 = 5,
}

impl WireType {
    /// Converts a raw 3-bit wire type value (`0..=5`) into a [`WireType`].
    #[inline]
    pub const fn from_u8(val: u8) -> Result<Self, DecodeError> {
        match val {
            0 => Ok(Self::Varint),
            1 => Ok(Self::Fixed64),
            2 => Ok(Self::LengthDelimited),
            3 => Ok(Self::StartGroup),
            4 => Ok(Self::EndGroup),
            5 => Ok(Self::Fixed32),
            _ => Err(DecodeError::InvalidWireType(val)),
        }
    }
}

/// Compact 8-byte span marker storing start and end byte offsets into a message slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldMarker {
    start: u32,
    end: u32,
}

impl Default for FieldMarker {
    #[inline]
    fn default() -> Self {
        Self::none()
    }
}

impl FieldMarker {
    /// Returns an absent marker sentinel (`start = u32::MAX, end = u32::MAX`).
    #[inline]
    pub const fn none() -> Self {
        Self {
            start: u32::MAX,
            end: u32::MAX,
        }
    }

    /// Creates a present marker spanning `start..end`.
    #[inline]
    pub const fn new(start: usize, end: usize) -> Self {
        debug_assert!(start <= end && end < u32::MAX as usize);
        Self {
            start: start as u32,
            end: end as u32,
        }
    }

    /// Returns `true` if this marker records a present field span.
    #[inline]
    pub const fn is_present(&self) -> bool {
        self.start != u32::MAX
    }

    /// Returns the starting byte offset of the span.
    #[inline]
    pub const fn start(&self) -> usize {
        self.start as usize
    }

    /// Returns the ending byte offset of the span.
    #[inline]
    pub const fn end(&self) -> usize {
        self.end as usize
    }

    /// Returns the byte length of the span, or `0` if absent.
    #[inline]
    pub const fn len(&self) -> usize {
        if self.is_present() {
            (self.end - self.start) as usize
        } else {
            0
        }
    }

    /// Returns `true` if the span is absent or has zero byte length.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Extends this marker to cover `span`. If `self` is currently absent, sets `*self = span`;
    /// otherwise updates `self.end = span.end`.
    #[inline]
    pub fn extend(&mut self, span: FieldMarker) {
        if !self.is_present() {
            *self = span;
        } else {
            debug_assert!(span.is_present() && self.start <= span.start && self.end <= span.end);
            self.end = span.end;
        }
    }

    /// Returns the sub-slice of `buf` covered by this marker, or `&[]` if absent.
    #[inline]
    pub const fn slice<'a>(&self, buf: &'a [u8]) -> &'a [u8] {
        if !self.is_present() {
            &[]
        } else {
            buf.split_at(self.end()).0.split_at(self.start()).1
        }
    }
}

/// Decodes a base-128 varint (up to 10 bytes) from the start of `buf`,
/// returning `(value, bytes_consumed)`.
#[inline]
pub fn decode_varint(buf: &[u8]) -> Result<(u64, usize), DecodeError> {
    let mut result: u64 = 0;
    let max_len = if buf.len() < 10 { buf.len() } else { 10 };
    let mut i = 0;
    while i < max_len {
        let byte = buf[i];
        if i == 9 && byte > 0x01 {
            return Err(DecodeError::InvalidVarint);
        }
        result |= ((byte & 0x7F) as u64) << (i * 7);
        if (byte & 0x80) == 0 {
            return Ok((result, i + 1));
        }
        i += 1;
    }
    Err(DecodeError::UnexpectedEof)
}

/// Decodes a base-128 varint from `buf` at the span recorded by `marker`.
#[inline]
pub fn decode_varint_at(buf: &[u8], marker: FieldMarker) -> Result<u64, DecodeError> {
    let slice = decode_bytes_at(buf, marker)?;
    let (val, _) = decode_varint(slice)?;
    Ok(val)
}

/// Decodes a boolean varint from `buf` at `marker`.
#[inline]
pub fn decode_bool_at(buf: &[u8], marker: FieldMarker) -> Result<bool, DecodeError> {
    Ok(decode_varint_at(buf, marker)? != 0)
}

/// Decodes a 32-bit ZigZag-encoded unsigned integer into a signed `i32`.
#[inline]
pub const fn decode_zigzag32(val: u32) -> i32 {
    ((val >> 1) as i32) ^ (-((val & 1) as i32))
}

/// Decodes a 64-bit ZigZag-encoded unsigned integer into a signed `i64`.
#[inline]
pub const fn decode_zigzag64(val: u64) -> i64 {
    ((val >> 1) as i64) ^ (-((val & 1) as i64))
}

/// Decodes a ZigZag-encoded `sint32` field from `buf` at `marker`.
#[inline]
pub fn decode_sint32_at(buf: &[u8], marker: FieldMarker) -> Result<i32, DecodeError> {
    Ok(decode_zigzag32(decode_varint_at(buf, marker)? as u32))
}

/// Decodes a ZigZag-encoded `sint64` field from `buf` at `marker`.
#[inline]
pub fn decode_sint64_at(buf: &[u8], marker: FieldMarker) -> Result<i64, DecodeError> {
    Ok(decode_zigzag64(decode_varint_at(buf, marker)?))
}

/// Decodes a little-endian 4-byte `fixed32` field from `buf` at `marker`.
#[inline]
pub fn decode_fixed32_at(buf: &[u8], marker: FieldMarker) -> Result<u32, DecodeError> {
    let slice = decode_bytes_at(buf, marker)?;
    let bytes: [u8; 4] = slice.try_into().map_err(|_| DecodeError::UnexpectedEof)?;
    Ok(u32::from_le_bytes(bytes))
}

/// Decodes a little-endian 8-byte `fixed64` field from `buf` at `marker`.
#[inline]
pub fn decode_fixed64_at(buf: &[u8], marker: FieldMarker) -> Result<u64, DecodeError> {
    let slice = decode_bytes_at(buf, marker)?;
    let bytes: [u8; 8] = slice.try_into().map_err(|_| DecodeError::UnexpectedEof)?;
    Ok(u64::from_le_bytes(bytes))
}

/// Decodes a little-endian 4-byte signed `sfixed32` field from `buf` at `marker`.
#[inline]
pub fn decode_sfixed32_at(buf: &[u8], marker: FieldMarker) -> Result<i32, DecodeError> {
    Ok(decode_fixed32_at(buf, marker)? as i32)
}

/// Decodes a little-endian 8-byte signed `sfixed64` field from `buf` at `marker`.
#[inline]
pub fn decode_sfixed64_at(buf: &[u8], marker: FieldMarker) -> Result<i64, DecodeError> {
    Ok(decode_fixed64_at(buf, marker)? as i64)
}

/// Decodes an IEEE-754 little-endian 4-byte `float` field from `buf` at `marker`.
#[inline]
pub fn decode_f32_at(buf: &[u8], marker: FieldMarker) -> Result<f32, DecodeError> {
    Ok(f32::from_bits(decode_fixed32_at(buf, marker)?))
}

/// Decodes an IEEE-754 little-endian 8-byte `double` field from `buf` at `marker`.
#[inline]
pub fn decode_f64_at(buf: &[u8], marker: FieldMarker) -> Result<f64, DecodeError> {
    Ok(f64::from_bits(decode_fixed64_at(buf, marker)?))
}

/// Extracts a borrowed byte slice from `buf` at `marker`.
#[inline]
pub fn decode_bytes_at(buf: &[u8], marker: FieldMarker) -> Result<&[u8], DecodeError> {
    if !marker.is_present() {
        return Err(DecodeError::UnexpectedEof);
    }
    buf.get(marker.start()..marker.end())
        .ok_or(DecodeError::UnexpectedEof)
}

/// Extracts and validates a borrowed UTF-8 string slice from `buf` at `marker`.
#[inline]
pub fn decode_str_at(buf: &[u8], marker: FieldMarker) -> Result<&str, DecodeError> {
    let bytes = decode_bytes_at(buf, marker)?;
    core::str::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8)
}
