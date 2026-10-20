use crate::error::EncodeError;
use crate::wire::WireType;

/// Encodes a signed 32-bit integer into a 32-bit ZigZag-encoded unsigned integer.
#[inline]
pub const fn encode_zigzag32(val: i32) -> u32 {
    ((val << 1) ^ (val >> 31)) as u32
}

/// Encodes a signed 64-bit integer into a 64-bit ZigZag-encoded unsigned integer.
#[inline]
pub const fn encode_zigzag64(val: i64) -> u64 {
    ((val << 1) ^ (val >> 63)) as u64
}

/// Single-pass backward writer that serializes Protocol Buffers wire data from
/// the end of a fixed-size `&'b mut [u8]` buffer toward the front.
#[derive(Debug)]
pub struct BackwardWriter<'b> {
    buf: &'b mut [u8],
    cursor: usize,
    err: Option<EncodeError>,
}

impl<'b> BackwardWriter<'b> {
    /// Creates a new [`BackwardWriter`] initialized with its cursor at `buf.len()`.
    #[inline]
    pub fn new(buf: &'b mut [u8]) -> Self {
        let cursor = buf.len();
        Self {
            buf,
            cursor,
            err: None,
        }
    }

    /// Returns the current write cursor offset into the backing buffer.
    #[inline]
    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    /// Returns the number of bytes written so far (`buf.len() - cursor`).
    #[inline]
    pub const fn written_len(&self) -> usize {
        self.buf.len() - self.cursor
    }

    /// Returns the remaining unwritten capacity in bytes (`cursor`).
    #[inline]
    pub const fn remaining_capacity(&self) -> usize {
        self.cursor
    }

    /// Returns `Err(err)` if a sticky [`EncodeError`] has occurred on this writer, or `Ok(())`.
    #[inline]
    pub const fn check(&self) -> Result<(), EncodeError> {
        match self.err {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    /// Consumes the writer and returns the contiguous written tail slice `&buf[cursor..]`,
    /// or `Err(err)` if any write overflowed the buffer.
    #[inline]
    pub fn finish(self) -> Result<&'b [u8], EncodeError> {
        self.check()?;
        Ok(&self.buf[self.cursor..])
    }

    /// Returns a view of the bytes written so far (`&buf[cursor..]`).
    #[inline]
    pub fn as_written_slice(&self) -> &[u8] {
        &self.buf[self.cursor..]
    }

    /// Prepends `bytes` immediately before the current cursor.
    ///
    /// If a previous write failed, or if `bytes.len() > self.cursor`, records a sticky
    /// [`EncodeError::BufferOverflow`] and returns `Err(EncodeError::BufferOverflow)`
    /// without modifying the buffer or advancing the cursor.
    #[inline]
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        if let Some(err) = self.err {
            return Err(err);
        }
        let len = bytes.len();
        if len > self.cursor {
            self.err = Some(EncodeError::BufferOverflow);
            return Err(EncodeError::BufferOverflow);
        }
        let new_cursor = self.cursor - len;
        self.buf[new_cursor..self.cursor].copy_from_slice(bytes);
        self.cursor = new_cursor;
        Ok(())
    }

    /// Encodes `val` as a base-128 varint (1 to 10 bytes) and prepends it before the cursor.
    #[inline]
    pub fn write_varint(&mut self, mut val: u64) -> Result<(), EncodeError> {
        let mut tmp = [0u8; 10];
        let mut len = 0;
        while val >= 0x80 {
            tmp[len] = (val as u8) | 0x80;
            val >>= 7;
            len += 1;
        }
        tmp[len] = val as u8;
        len += 1;
        self.write_bytes(&tmp[..len])
    }

    /// Encodes a field tag `(field_number << 3) | wire_type` as a varint.
    #[inline]
    pub fn write_tag(&mut self, field_number: u32, wire_type: WireType) -> Result<(), EncodeError> {
        let raw = (u64::from(field_number) << 3) | u64::from(wire_type as u8);
        self.write_varint(raw)
    }

    /// Encodes a boolean (`0` or `1`) as a single-byte varint.
    #[inline]
    pub fn write_bool(&mut self, val: bool) -> Result<(), EncodeError> {
        self.write_varint(u64::from(val))
    }

    /// Encodes a signed 32-bit integer using ZigZag encoding (`sint32`) and writes it as a varint.
    #[inline]
    pub fn write_zigzag32(&mut self, val: i32) -> Result<(), EncodeError> {
        self.write_varint(u64::from(encode_zigzag32(val)))
    }

    /// Encodes a signed 64-bit integer using ZigZag encoding (`sint64`) and writes it as a varint.
    #[inline]
    pub fn write_zigzag64(&mut self, val: i64) -> Result<(), EncodeError> {
        self.write_varint(encode_zigzag64(val))
    }

    /// Writes a 4-byte little-endian `fixed32` value.
    #[inline]
    pub fn write_fixed32(&mut self, val: u32) -> Result<(), EncodeError> {
        self.write_bytes(&val.to_le_bytes())
    }

    /// Writes an 8-byte little-endian `fixed64` value.
    #[inline]
    pub fn write_fixed64(&mut self, val: u64) -> Result<(), EncodeError> {
        self.write_bytes(&val.to_le_bytes())
    }

    /// Writes a 4-byte little-endian signed `sfixed32` value.
    #[inline]
    pub fn write_sfixed32(&mut self, val: i32) -> Result<(), EncodeError> {
        self.write_fixed32(val as u32)
    }

    /// Writes an 8-byte little-endian signed `sfixed64` value.
    #[inline]
    pub fn write_sfixed64(&mut self, val: i64) -> Result<(), EncodeError> {
        self.write_fixed64(val as u64)
    }

    /// Writes a 4-byte IEEE-754 little-endian `float` value.
    #[inline]
    pub fn write_f32(&mut self, val: f32) -> Result<(), EncodeError> {
        self.write_fixed32(val.to_bits())
    }

    /// Writes an 8-byte IEEE-754 little-endian `double` value.
    #[inline]
    pub fn write_f64(&mut self, val: f64) -> Result<(), EncodeError> {
        self.write_fixed64(val.to_bits())
    }

    /// Writes a length-delimited payload by first writing `payload` and then
    /// prepending its byte length as a varint.
    #[inline]
    pub fn write_length_delimited(&mut self, payload: &[u8]) -> Result<(), EncodeError> {
        self.write_bytes(payload)?;
        self.write_varint(payload.len() as u64)
    }

    /// Temporarily lends the unwritten prefix (`&mut self.buf[..self.cursor]`) as a
    /// sub-[`BackwardWriter`] to `f`, then updates `self.cursor` and propagates any
    /// sticky [`EncodeError`] back to `self`.
    #[inline]
    pub fn with_sub_writer<F>(&mut self, f: F)
    where
        F: FnOnce(BackwardWriter<'_>) -> BackwardWriter<'_>,
    {
        let cursor = self.cursor;
        let err = self.err;
        let sub = BackwardWriter {
            buf: &mut self.buf[..cursor],
            cursor,
            err,
        };
        let sub = f(sub);
        self.cursor = sub.cursor;
        if sub.err.is_some() {
            self.err = sub.err;
        }
    }

    /// Copies all top-level fields from `src_buf` whose tag satisfies `keep_tag(tag)`,
    /// preserving their original forward wire order.
    #[inline]
    pub fn copy_filtered_fields<F>(
        &mut self,
        src_buf: &[u8],
        mut keep_tag: F,
    ) -> Result<(), EncodeError>
    where
        F: FnMut(u32) -> bool,
    {
        if let Some(err) = self.err {
            return Err(err);
        }
        let mut total_len = 0usize;
        for item in crate::iter::FieldIter::new(src_buf) {
            let Ok(field) = item else { break };
            if keep_tag(field.tag) {
                total_len += field.raw_span.len();
            }
        }
        if total_len == 0 {
            return Ok(());
        }
        if total_len > self.cursor {
            self.err = Some(EncodeError::BufferOverflow);
            return Err(EncodeError::BufferOverflow);
        }
        let new_cursor = self.cursor - total_len;
        let mut write_pos = new_cursor;
        for item in crate::iter::FieldIter::new(src_buf) {
            let Ok(field) = item else { break };
            if keep_tag(field.tag) {
                let raw = field.raw_span.slice(src_buf);
                self.buf[write_pos..write_pos + raw.len()].copy_from_slice(raw);
                write_pos += raw.len();
            }
        }
        self.cursor = new_cursor;
        Ok(())
    }
}

