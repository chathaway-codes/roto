use core::iter::FusedIterator;
use core::marker::PhantomData;

use crate::error::DecodeError;
use crate::iter::FieldIter;
use crate::wire::{WireType, decode_varint, decode_zigzag32, decode_zigzag64};

/// Conversion trait for scalar types encoded as Protocol Buffers base-128 varints.
pub trait VarintValue: Sized + Copy {
    fn from_raw_varint(val: u64) -> Self;
}

impl VarintValue for u32 {
    #[inline]
    fn from_raw_varint(val: u64) -> Self {
        val as u32
    }
}

impl VarintValue for u64 {
    #[inline]
    fn from_raw_varint(val: u64) -> Self {
        val
    }
}

impl VarintValue for i32 {
    #[inline]
    fn from_raw_varint(val: u64) -> Self {
        val as i32
    }
}

impl VarintValue for i64 {
    #[inline]
    fn from_raw_varint(val: u64) -> Self {
        val as i64
    }
}

impl VarintValue for bool {
    #[inline]
    fn from_raw_varint(val: u64) -> Self {
        val != 0
    }
}

/// Zero-allocation iterator over a repeated varint field, supporting both unpacked
/// (`WireType::Varint`) and packed (`WireType::LengthDelimited`) encodings.
#[derive(Clone, Debug)]
pub struct RepeatedVarintIter<'a, T> {
    buf: &'a [u8],
    iter: FieldIter<'a>,
    tag: u32,
    packed: &'a [u8],
    fused: bool,
    _marker: PhantomData<T>,
}

impl<'a, T: VarintValue> RepeatedVarintIter<'a, T> {
    /// Creates a new [`RepeatedVarintIter`] scanning `buf` for occurrences of `tag`.
    #[inline]
    pub const fn new(buf: &'a [u8], tag: u32) -> Self {
        Self {
            buf,
            iter: FieldIter::new(buf),
            tag,
            packed: &[],
            fused: false,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: VarintValue> Iterator for RepeatedVarintIter<'a, T> {
    type Item = Result<T, DecodeError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.fused {
            return None;
        }

        loop {
            if !self.packed.is_empty() {
                return match decode_varint(self.packed) {
                    Ok((raw, len)) => {
                        self.packed = &self.packed[len..];
                        Some(Ok(T::from_raw_varint(raw)))
                    }
                    Err(err) => {
                        self.packed = &[];
                        self.fused = true;
                        Some(Err(err))
                    }
                };
            }

            let field = match self.iter.next()? {
                Ok(field) => field,
                Err(err) => {
                    self.fused = true;
                    return Some(Err(err));
                }
            };

            if field.tag != self.tag {
                continue;
            }

            match field.wire_type {
                WireType::Varint => {
                    return Some(Ok(T::from_raw_varint(field.val)));
                }
                WireType::LengthDelimited => {
                    self.packed = field.marker.slice(self.buf);
                }
                actual => {
                    self.fused = true;
                    return Some(Err(DecodeError::UnexpectedWireType {
                        expected: WireType::Varint,
                        actual,
                    }));
                }
            }
        }
    }
}

impl<T: VarintValue> FusedIterator for RepeatedVarintIter<'_, T> {}

/// Conversion trait for ZigZag-encoded signed integer types (`sint32`, `sint64`).
pub trait ZigZagValue: Sized + Copy {
    fn from_raw_varint(val: u64) -> Self;
}

impl ZigZagValue for i32 {
    #[inline]
    fn from_raw_varint(val: u64) -> Self {
        decode_zigzag32(val as u32)
    }
}

impl ZigZagValue for i64 {
    #[inline]
    fn from_raw_varint(val: u64) -> Self {
        decode_zigzag64(val)
    }
}

/// Zero-allocation iterator over a repeated ZigZag-encoded varint field (`sint32`, `sint64`),
/// supporting both unpacked (`WireType::Varint`) and packed (`WireType::LengthDelimited`) encodings.
#[derive(Clone, Debug)]
pub struct RepeatedZigZagIter<'a, T> {
    buf: &'a [u8],
    iter: FieldIter<'a>,
    tag: u32,
    packed: &'a [u8],
    fused: bool,
    _marker: PhantomData<T>,
}

impl<'a, T: ZigZagValue> RepeatedZigZagIter<'a, T> {
    /// Creates a new [`RepeatedZigZagIter`] scanning `buf` for occurrences of `tag`.
    #[inline]
    pub const fn new(buf: &'a [u8], tag: u32) -> Self {
        Self {
            buf,
            iter: FieldIter::new(buf),
            tag,
            packed: &[],
            fused: false,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: ZigZagValue> Iterator for RepeatedZigZagIter<'a, T> {
    type Item = Result<T, DecodeError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.fused {
            return None;
        }

        loop {
            if !self.packed.is_empty() {
                return match decode_varint(self.packed) {
                    Ok((raw, len)) => {
                        self.packed = &self.packed[len..];
                        Some(Ok(T::from_raw_varint(raw)))
                    }
                    Err(err) => {
                        self.packed = &[];
                        self.fused = true;
                        Some(Err(err))
                    }
                };
            }

            let field = match self.iter.next()? {
                Ok(field) => field,
                Err(err) => {
                    self.fused = true;
                    return Some(Err(err));
                }
            };

            if field.tag != self.tag {
                continue;
            }

            match field.wire_type {
                WireType::Varint => {
                    return Some(Ok(T::from_raw_varint(field.val)));
                }
                WireType::LengthDelimited => {
                    self.packed = field.marker.slice(self.buf);
                }
                actual => {
                    self.fused = true;
                    return Some(Err(DecodeError::UnexpectedWireType {
                        expected: WireType::Varint,
                        actual,
                    }));
                }
            }
        }
    }
}

impl<T: ZigZagValue> FusedIterator for RepeatedZigZagIter<'_, T> {}

/// Conversion trait for 4-byte little-endian fixed-width scalar types (`fixed32`, `sfixed32`, `float`).
pub trait Fixed32Value: Sized + Copy {
    fn from_le_bytes(bytes: [u8; 4]) -> Self;
}

impl Fixed32Value for u32 {
    #[inline]
    fn from_le_bytes(bytes: [u8; 4]) -> Self {
        u32::from_le_bytes(bytes)
    }
}

impl Fixed32Value for i32 {
    #[inline]
    fn from_le_bytes(bytes: [u8; 4]) -> Self {
        i32::from_le_bytes(bytes)
    }
}

impl Fixed32Value for f32 {
    #[inline]
    fn from_le_bytes(bytes: [u8; 4]) -> Self {
        f32::from_le_bytes(bytes)
    }
}

/// Zero-allocation iterator over a repeated 4-byte fixed-width field, supporting both
/// unpacked (`WireType::Fixed32`) and packed (`WireType::LengthDelimited`) encodings.
#[derive(Clone, Debug)]
pub struct RepeatedFixed32Iter<'a, T> {
    buf: &'a [u8],
    iter: FieldIter<'a>,
    tag: u32,
    packed: &'a [u8],
    fused: bool,
    _marker: PhantomData<T>,
}

impl<'a, T: Fixed32Value> RepeatedFixed32Iter<'a, T> {
    /// Creates a new [`RepeatedFixed32Iter`] scanning `buf` for occurrences of `tag`.
    #[inline]
    pub const fn new(buf: &'a [u8], tag: u32) -> Self {
        Self {
            buf,
            iter: FieldIter::new(buf),
            tag,
            packed: &[],
            fused: false,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: Fixed32Value> Iterator for RepeatedFixed32Iter<'a, T> {
    type Item = Result<T, DecodeError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.fused {
            return None;
        }

        loop {
            if !self.packed.is_empty() {
                let Ok(bytes) = self.packed.get(..4).and_then(|s| s.try_into().ok()).ok_or(()) else {
                    self.packed = &[];
                    self.fused = true;
                    return Some(Err(DecodeError::UnexpectedEof));
                };
                self.packed = &self.packed[4..];
                return Some(Ok(T::from_le_bytes(bytes)));
            }

            let field = match self.iter.next()? {
                Ok(field) => field,
                Err(err) => {
                    self.fused = true;
                    return Some(Err(err));
                }
            };

            if field.tag != self.tag {
                continue;
            }

            match field.wire_type {
                WireType::Fixed32 => {
                    return Some(Ok(T::from_le_bytes((field.val as u32).to_le_bytes())));
                }
                WireType::LengthDelimited => {
                    self.packed = field.marker.slice(self.buf);
                }
                actual => {
                    self.fused = true;
                    return Some(Err(DecodeError::UnexpectedWireType {
                        expected: WireType::Fixed32,
                        actual,
                    }));
                }
            }
        }
    }
}

impl<T: Fixed32Value> FusedIterator for RepeatedFixed32Iter<'_, T> {}

/// Conversion trait for 8-byte little-endian fixed-width scalar types (`fixed64`, `sfixed64`, `double`).
pub trait Fixed64Value: Sized + Copy {
    fn from_le_bytes(bytes: [u8; 8]) -> Self;
}

impl Fixed64Value for u64 {
    #[inline]
    fn from_le_bytes(bytes: [u8; 8]) -> Self {
        u64::from_le_bytes(bytes)
    }
}

impl Fixed64Value for i64 {
    #[inline]
    fn from_le_bytes(bytes: [u8; 8]) -> Self {
        i64::from_le_bytes(bytes)
    }
}

impl Fixed64Value for f64 {
    #[inline]
    fn from_le_bytes(bytes: [u8; 8]) -> Self {
        f64::from_le_bytes(bytes)
    }
}

/// Zero-allocation iterator over a repeated 8-byte fixed-width field, supporting both
/// unpacked (`WireType::Fixed64`) and packed (`WireType::LengthDelimited`) encodings.
#[derive(Clone, Debug)]
pub struct RepeatedFixed64Iter<'a, T> {
    buf: &'a [u8],
    iter: FieldIter<'a>,
    tag: u32,
    packed: &'a [u8],
    fused: bool,
    _marker: PhantomData<T>,
}

impl<'a, T: Fixed64Value> RepeatedFixed64Iter<'a, T> {
    /// Creates a new [`RepeatedFixed64Iter`] scanning `buf` for occurrences of `tag`.
    #[inline]
    pub const fn new(buf: &'a [u8], tag: u32) -> Self {
        Self {
            buf,
            iter: FieldIter::new(buf),
            tag,
            packed: &[],
            fused: false,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: Fixed64Value> Iterator for RepeatedFixed64Iter<'a, T> {
    type Item = Result<T, DecodeError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.fused {
            return None;
        }

        loop {
            if !self.packed.is_empty() {
                let Ok(bytes) = self.packed.get(..8).and_then(|s| s.try_into().ok()).ok_or(()) else {
                    self.packed = &[];
                    self.fused = true;
                    return Some(Err(DecodeError::UnexpectedEof));
                };
                self.packed = &self.packed[8..];
                return Some(Ok(T::from_le_bytes(bytes)));
            }

            let field = match self.iter.next()? {
                Ok(field) => field,
                Err(err) => {
                    self.fused = true;
                    return Some(Err(err));
                }
            };

            if field.tag != self.tag {
                continue;
            }

            match field.wire_type {
                WireType::Fixed64 => {
                    return Some(Ok(T::from_le_bytes(field.val.to_le_bytes())));
                }
                WireType::LengthDelimited => {
                    self.packed = field.marker.slice(self.buf);
                }
                actual => {
                    self.fused = true;
                    return Some(Err(DecodeError::UnexpectedWireType {
                        expected: WireType::Fixed64,
                        actual,
                    }));
                }
            }
        }
    }
}

impl<T: Fixed64Value> FusedIterator for RepeatedFixed64Iter<'_, T> {}

/// Zero-allocation iterator over a repeated `bytes` field (`WireType::LengthDelimited`).
#[derive(Clone, Debug)]
pub struct RepeatedBytesIter<'a> {
    buf: &'a [u8],
    iter: FieldIter<'a>,
    tag: u32,
    fused: bool,
}

impl<'a> RepeatedBytesIter<'a> {
    /// Creates a new [`RepeatedBytesIter`] scanning `buf` for occurrences of `tag`.
    #[inline]
    pub const fn new(buf: &'a [u8], tag: u32) -> Self {
        Self {
            buf,
            iter: FieldIter::new(buf),
            tag,
            fused: false,
        }
    }
}

impl<'a> Iterator for RepeatedBytesIter<'a> {
    type Item = Result<&'a [u8], DecodeError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.fused {
            return None;
        }

        loop {
            let field = match self.iter.next()? {
                Ok(field) => field,
                Err(err) => {
                    self.fused = true;
                    return Some(Err(err));
                }
            };

            if field.tag != self.tag {
                continue;
            }

            if field.wire_type != WireType::LengthDelimited {
                self.fused = true;
                return Some(Err(DecodeError::UnexpectedWireType {
                    expected: WireType::LengthDelimited,
                    actual: field.wire_type,
                }));
            }

            return Some(Ok(field.marker.slice(self.buf)));
        }
    }
}

impl FusedIterator for RepeatedBytesIter<'_> {}

/// Zero-allocation iterator over a repeated UTF-8 `string` field (`WireType::LengthDelimited`).
#[derive(Clone, Debug)]
pub struct RepeatedStrIter<'a> {
    inner: RepeatedBytesIter<'a>,
    fused: bool,
}

impl<'a> RepeatedStrIter<'a> {
    /// Creates a new [`RepeatedStrIter`] scanning `buf` for occurrences of `tag`.
    #[inline]
    pub const fn new(buf: &'a [u8], tag: u32) -> Self {
        Self {
            inner: RepeatedBytesIter::new(buf, tag),
            fused: false,
        }
    }
}

impl<'a> Iterator for RepeatedStrIter<'a> {
    type Item = Result<&'a str, DecodeError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.fused {
            return None;
        }

        match self.inner.next()? {
            Ok(bytes) => match core::str::from_utf8(bytes) {
                Ok(s) => Some(Ok(s)),
                Err(_) => {
                    self.fused = true;
                    Some(Err(DecodeError::InvalidUtf8))
                }
            },
            Err(err) => {
                self.fused = true;
                Some(Err(err))
            }
        }
    }
}

impl FusedIterator for RepeatedStrIter<'_> {}

/// Trait implemented by zero-copy message views that can be decoded from a borrowed wire slice.
pub trait MessageView<'a>: Sized {
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError>;
}

/// Zero-allocation iterator over a repeated nested message field (`WireType::LengthDelimited`).
#[derive(Clone, Debug)]
pub struct RepeatedMessageIter<'a, V> {
    inner: RepeatedBytesIter<'a>,
    fused: bool,
    _marker: PhantomData<V>,
}

impl<'a, V: MessageView<'a>> RepeatedMessageIter<'a, V> {
    /// Creates a new [`RepeatedMessageIter`] scanning `buf` for occurrences of `tag`.
    #[inline]
    pub const fn new(buf: &'a [u8], tag: u32) -> Self {
        Self {
            inner: RepeatedBytesIter::new(buf, tag),
            fused: false,
            _marker: PhantomData,
        }
    }
}

impl<'a, V: MessageView<'a>> Iterator for RepeatedMessageIter<'a, V> {
    type Item = Result<V, DecodeError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.fused {
            return None;
        }

        match self.inner.next()? {
            Ok(bytes) => match V::from_bytes(bytes) {
                Ok(view) => Some(Ok(view)),
                Err(err) => {
                    self.fused = true;
                    Some(Err(err))
                }
            },
            Err(err) => {
                self.fused = true;
                Some(Err(err))
            }
        }
    }
}

impl<'a, V: MessageView<'a>> FusedIterator for RepeatedMessageIter<'a, V> {}

/// Zero-allocation iterator over a repeated `group` field (`WireType::StartGroup`).
#[derive(Clone, Debug)]
pub struct RepeatedGroupIter<'a, V> {
    buf: &'a [u8],
    iter: FieldIter<'a>,
    tag: u32,
    fused: bool,
    _marker: PhantomData<V>,
}

impl<'a, V: MessageView<'a>> RepeatedGroupIter<'a, V> {
    /// Creates a new [`RepeatedGroupIter`] scanning `buf` for occurrences of `tag`.
    #[inline]
    pub const fn new(buf: &'a [u8], tag: u32) -> Self {
        Self {
            buf,
            iter: FieldIter::new(buf),
            tag,
            fused: false,
            _marker: PhantomData,
        }
    }
}

impl<'a, V: MessageView<'a>> Iterator for RepeatedGroupIter<'a, V> {
    type Item = Result<V, DecodeError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.fused {
            return None;
        }

        loop {
            let field = match self.iter.next()? {
                Ok(field) => field,
                Err(err) => {
                    self.fused = true;
                    return Some(Err(err));
                }
            };

            if field.tag != self.tag {
                continue;
            }

            if field.wire_type != WireType::StartGroup {
                self.fused = true;
                return Some(Err(DecodeError::UnexpectedWireType {
                    expected: WireType::StartGroup,
                    actual: field.wire_type,
                }));
            }

            return match V::from_bytes(field.marker.slice(self.buf)) {
                Ok(view) => Some(Ok(view)),
                Err(err) => {
                    self.fused = true;
                    Some(Err(err))
                }
            };
        }
    }
}

impl<'a, V: MessageView<'a>> FusedIterator for RepeatedGroupIter<'a, V> {}

/// Zero-allocation iterator yielding the raw wire slices (tag + length prefix + payload)
/// of all top-level fields whose tag is not in `known_tags`.
#[derive(Clone, Debug)]
pub struct UnknownFieldsIter<'a> {
    buf: &'a [u8],
    iter: FieldIter<'a>,
    known_tags: &'static [u32],
}

impl<'a> UnknownFieldsIter<'a> {
    /// Creates a new [`UnknownFieldsIter`] yielding unrecognized fields from `buf`.
    #[inline]
    pub const fn new(buf: &'a [u8], known_tags: &'static [u32]) -> Self {
        Self {
            buf,
            iter: FieldIter::new(buf),
            known_tags,
        }
    }
}

impl<'a> Iterator for UnknownFieldsIter<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(Ok(field)) = self.iter.next() {
            if !self.known_tags.contains(&field.tag) {
                return Some(field.raw_span.slice(self.buf));
            }
        }
        None
    }
}

impl FusedIterator for UnknownFieldsIter<'_> {}
