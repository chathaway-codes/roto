use core::iter::FusedIterator;

use crate::error::DecodeError;
use crate::wire::{FieldMarker, WireType, decode_varint};

const MAX_GROUP_DEPTH: usize = 64;

/// A scanned top-level wire field yielding byte spans into the message buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Field {
    pub tag: u32,
    pub wire_type: WireType,
    /// Byte span of the field's value payload in `buf`.
    pub marker: FieldMarker,
    /// Byte span of the entire field (tag + length prefix + payload) in `buf`.
    pub raw_span: FieldMarker,
}

/// Single-pass $O(n)$ linear iterator over top-level Protocol Buffers wire fields in a slice.
#[derive(Clone, Debug)]
pub struct FieldIter<'a> {
    buf: &'a [u8],
    cursor: usize,
}

impl<'a> FieldIter<'a> {
    /// Creates a new [`FieldIter`] scanning `buf` from byte offset `0`.
    #[inline]
    pub const fn new(buf: &'a [u8]) -> Self {
        Self { buf, cursor: 0 }
    }

    #[inline]
    fn next_field(&mut self) -> Result<Field, DecodeError> {
        let field_start = self.cursor;
        let (tag, wire_type, tag_len) = decode_tag(&self.buf[field_start..])?;
        let after_tag = field_start + tag_len;

        match wire_type {
            WireType::Varint => {
                let (_, varint_len) = decode_varint(&self.buf[after_tag..])?;
                let field_end = after_tag + varint_len;
                self.cursor = field_end;
                Ok(Field {
                    tag,
                    wire_type,
                    marker: FieldMarker::new(after_tag, field_end),
                    raw_span: FieldMarker::new(field_start, field_end),
                })
            }
            WireType::Fixed64 => {
                let field_end = after_tag
                    .checked_add(8)
                    .filter(|&end| end <= self.buf.len())
                    .ok_or(DecodeError::UnexpectedEof)?;
                self.cursor = field_end;
                Ok(Field {
                    tag,
                    wire_type,
                    marker: FieldMarker::new(after_tag, field_end),
                    raw_span: FieldMarker::new(field_start, field_end),
                })
            }
            WireType::Fixed32 => {
                let field_end = after_tag
                    .checked_add(4)
                    .filter(|&end| end <= self.buf.len())
                    .ok_or(DecodeError::UnexpectedEof)?;
                self.cursor = field_end;
                Ok(Field {
                    tag,
                    wire_type,
                    marker: FieldMarker::new(after_tag, field_end),
                    raw_span: FieldMarker::new(field_start, field_end),
                })
            }
            WireType::LengthDelimited => {
                let (len_u64, len_varint_len) = decode_varint(&self.buf[after_tag..])?;
                let len = usize::try_from(len_u64).map_err(|_| DecodeError::UnexpectedEof)?;
                let payload_start = after_tag + len_varint_len;
                let payload_end = payload_start
                    .checked_add(len)
                    .filter(|&end| end <= self.buf.len())
                    .ok_or(DecodeError::UnexpectedEof)?;
                self.cursor = payload_end;
                Ok(Field {
                    tag,
                    wire_type,
                    marker: FieldMarker::new(payload_start, payload_end),
                    raw_span: FieldMarker::new(field_start, payload_end),
                })
            }
            WireType::StartGroup => {
                let group_body_start = after_tag;
                let (end_group_tag_start, after_end_group_tag) =
                    skip_group(self.buf, group_body_start, tag)?;
                self.cursor = after_end_group_tag;
                Ok(Field {
                    tag,
                    wire_type,
                    marker: FieldMarker::new(group_body_start, end_group_tag_start),
                    raw_span: FieldMarker::new(field_start, after_end_group_tag),
                })
            }
            WireType::EndGroup => Err(DecodeError::InvalidWireType(4)),
        }
    }
}

impl<'a> Iterator for FieldIter<'a> {
    type Item = Result<Field, DecodeError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.buf.len() {
            return None;
        }
        match self.next_field() {
            Ok(field) => Some(Ok(field)),
            Err(err) => {
                self.cursor = self.buf.len();
                Some(Err(err))
            }
        }
    }
}

impl FusedIterator for FieldIter<'_> {}

#[inline]
fn decode_tag(buf: &[u8]) -> Result<(u32, WireType, usize), DecodeError> {
    let (raw, tag_len) = decode_varint(buf)?;
    let field_number_u64 = raw >> 3;
    if field_number_u64 == 0 || field_number_u64 > u32::MAX as u64 {
        return Err(DecodeError::InvalidVarint);
    }
    let tag = field_number_u64 as u32;
    let wire_type = WireType::from_u8((raw & 0x07) as u8)?;
    Ok((tag, wire_type, tag_len))
}

fn skip_group(
    buf: &[u8],
    group_body_start: usize,
    outer_tag: u32,
) -> Result<(usize, usize), DecodeError> {
    let mut stack = [0u32; MAX_GROUP_DEPTH];
    stack[0] = outer_tag;
    let mut depth = 1usize;
    let mut pos = group_body_start;

    loop {
        if pos >= buf.len() {
            return Err(DecodeError::UnexpectedEof);
        }
        let tag_start = pos;
        let (tag, wire_type, tag_len) = decode_tag(&buf[pos..])?;
        pos += tag_len;

        match wire_type {
            WireType::Varint => {
                let (_, varint_len) = decode_varint(&buf[pos..])?;
                pos += varint_len;
            }
            WireType::Fixed64 => {
                pos = pos
                    .checked_add(8)
                    .filter(|&end| end <= buf.len())
                    .ok_or(DecodeError::UnexpectedEof)?;
            }
            WireType::Fixed32 => {
                pos = pos
                    .checked_add(4)
                    .filter(|&end| end <= buf.len())
                    .ok_or(DecodeError::UnexpectedEof)?;
            }
            WireType::LengthDelimited => {
                let (len_u64, len_varint_len) = decode_varint(&buf[pos..])?;
                let len = usize::try_from(len_u64).map_err(|_| DecodeError::UnexpectedEof)?;
                pos = (pos + len_varint_len)
                    .checked_add(len)
                    .filter(|&end| end <= buf.len())
                    .ok_or(DecodeError::UnexpectedEof)?;
            }
            WireType::StartGroup => {
                if depth >= MAX_GROUP_DEPTH {
                    return Err(DecodeError::InvalidWireType(3));
                }
                stack[depth] = tag;
                depth += 1;
            }
            WireType::EndGroup => {
                depth -= 1;
                if stack[depth] != tag {
                    return Err(DecodeError::InvalidWireType(4));
                }
                if depth == 0 {
                    return Ok((tag_start, pos));
                }
            }
        }
    }
}
