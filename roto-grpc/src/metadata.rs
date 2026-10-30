use core::fmt;

use roto_runtime::{BackwardWriter, EncodeError};

use crate::hpack::{HeaderField, HpackError};
use crate::status::GrpcStatus;

/// Default maximum number of custom metadata entries in a [`FixedMetadata`] map.
pub const DEFAULT_METADATA_ENTRIES: usize = 16;

/// Default byte capacity for custom metadata keys and values in a [`FixedMetadata`] map.
pub const DEFAULT_METADATA_CAP: usize = 512;

const B64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

#[inline]
const fn b64_val(b: u8) -> u8 {
    match b {
        b'A'..=b'Z' => b - b'A',
        b'a'..=b'z' => b - b'a' + 26,
        b'0'..=b'9' => b - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => 0xFF,
    }
}

/// Returns `true` if `name` is a binary gRPC metadata header name (`*-bin` with a non-empty prefix).
#[must_use]
#[inline]
pub fn is_binary_metadata_name(name: &[u8]) -> bool {
    name.len() > 4 && name.ends_with(b"-bin")
}

/// Returns `true` if `name` is a syntactically valid HTTP/2 & gRPC custom metadata header name:
/// non-empty, does not start with `':'` (pseudo-header), consists exclusively of lowercase ASCII
/// alphanumeric characters (`0-9`, `a-z`), `'-'`, `'_'`, or `'.'`, and if suffixed with `"-bin"`
/// has a non-empty prefix before `"-bin"`.
#[must_use]
#[inline]
pub fn is_valid_metadata_name(name: &[u8]) -> bool {
    if name.is_empty() || name.starts_with(b":") {
        return false;
    }
    if name.ends_with(b"-bin") && name.len() <= 4 {
        return false;
    }
    name.iter()
        .all(|&b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.'))
}

/// Returns `true` if `value` is a valid gRPC ASCII metadata value (`0x20..=0x7E`).
#[must_use]
#[inline]
pub fn is_valid_ascii_metadata_value(value: &[u8]) -> bool {
    value.iter().all(|&b| matches!(b, 0x20..=0x7E))
}

/// Returns `true` if `name` is a reserved HTTP/2 or gRPC transport header (`:*`, `grpc-*`,
/// `content-type`, `te`, `user-agent`) managed by the framing layer rather than user metadata.
#[must_use]
#[inline]
pub fn is_reserved_grpc_header(name: &[u8]) -> bool {
    name.starts_with(b":")
        || (name.len() >= 5 && name[..5].eq_ignore_ascii_case(b"grpc-"))
        || name.eq_ignore_ascii_case(b"content-type")
        || name.eq_ignore_ascii_case(b"te")
        || name.eq_ignore_ascii_case(b"user-agent")
}

/// Returns the padded RFC 4648 standard Base64 encoded byte length for a raw byte slice of
/// length `raw_len`.
#[must_use]
#[inline]
pub const fn base64_encoded_len(raw_len: usize) -> usize {
    raw_len.div_ceil(3) * 4
}

/// Encodes `raw` as standard padded RFC 4648 Base64 into `out`, returning the number of bytes
/// written.
pub fn base64_encode(raw: &[u8], out: &mut [u8]) -> Result<usize, EncodeError> {
    let enc_len = base64_encoded_len(raw.len());
    if out.len() < enc_len {
        return Err(EncodeError::BufferOverflow);
    }
    let full_chunks = raw.len() / 3;
    let mut in_pos = 0;
    let mut out_pos = 0;
    for _ in 0..full_chunks {
        let b0 = raw[in_pos];
        let b1 = raw[in_pos + 1];
        let b2 = raw[in_pos + 2];
        out[out_pos] = B64_CHARS[(b0 >> 2) as usize];
        out[out_pos + 1] = B64_CHARS[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize];
        out[out_pos + 2] = B64_CHARS[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize];
        out[out_pos + 3] = B64_CHARS[(b2 & 0x3f) as usize];
        in_pos += 3;
        out_pos += 4;
    }
    let rem = raw.len() - in_pos;
    if rem == 1 {
        let b0 = raw[in_pos];
        out[out_pos] = B64_CHARS[(b0 >> 2) as usize];
        out[out_pos + 1] = B64_CHARS[((b0 & 0x03) << 4) as usize];
        out[out_pos + 2] = b'=';
        out[out_pos + 3] = b'=';
        out_pos += 4;
    } else if rem == 2 {
        let b0 = raw[in_pos];
        let b1 = raw[in_pos + 1];
        out[out_pos] = B64_CHARS[(b0 >> 2) as usize];
        out[out_pos + 1] = B64_CHARS[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize];
        out[out_pos + 2] = B64_CHARS[((b1 & 0x0f) << 2) as usize];
        out[out_pos + 3] = b'=';
        out_pos += 4;
    }
    debug_assert_eq!(out_pos, enc_len);
    Ok(out_pos)
}

/// Encodes `raw` as standard padded RFC 4648 Base64 backward onto `writer`.
pub fn base64_encode_backward(raw: &[u8], writer: &mut BackwardWriter<'_>) {
    let n = raw.len();
    let rem = n % 3;
    if rem == 1 {
        let b0 = raw[n - 1];
        let tail = [
            B64_CHARS[(b0 >> 2) as usize],
            B64_CHARS[((b0 & 0x03) << 4) as usize],
            b'=',
            b'=',
        ];
        let _ = writer.write_bytes(&tail);
    } else if rem == 2 {
        let b0 = raw[n - 2];
        let b1 = raw[n - 1];
        let tail = [
            B64_CHARS[(b0 >> 2) as usize],
            B64_CHARS[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize],
            B64_CHARS[((b1 & 0x0f) << 2) as usize],
            b'=',
        ];
        let _ = writer.write_bytes(&tail);
    }
    let mut i = n - rem;
    while i >= 3 {
        i -= 3;
        let b0 = raw[i];
        let b1 = raw[i + 1];
        let b2 = raw[i + 2];
        let group = [
            B64_CHARS[(b0 >> 2) as usize],
            B64_CHARS[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize],
            B64_CHARS[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize],
            B64_CHARS[(b2 & 0x3f) as usize],
        ];
        let _ = writer.write_bytes(&group);
    }
}

/// Computes the exact decoded byte length of a padded or unpadded RFC 4648 Base64 slice.
pub fn base64_decoded_len(encoded: &[u8]) -> Result<usize, HpackError> {
    if encoded.is_empty() {
        return Ok(0);
    }
    let unpadded_len = if let Some(prefix) = encoded.strip_suffix(b"==") {
        if !encoded.len().is_multiple_of(4) {
            return Err(HpackError::InvalidHeaderValue);
        }
        prefix.len()
    } else if let Some(prefix) = encoded.strip_suffix(b"=") {
        if !encoded.len().is_multiple_of(4) {
            return Err(HpackError::InvalidHeaderValue);
        }
        prefix.len()
    } else {
        encoded.len()
    };
    let rem = unpadded_len % 4;
    if rem == 1 {
        return Err(HpackError::InvalidHeaderValue);
    }
    let extra = match rem {
        0 => 0,
        2 => 1,
        3 => 2,
        _ => unreachable!(),
    };
    Ok((unpadded_len / 4) * 3 + extra)
}

/// Decodes a padded or unpadded RFC 4648 standard Base64 slice into `out`, returning the number of
/// decoded bytes written.
pub fn base64_decode(encoded: &[u8], out: &mut [u8]) -> Result<usize, HpackError> {
    let decoded_len = base64_decoded_len(encoded)?;
    if out.len() < decoded_len {
        return Err(HpackError::BufferOverflow);
    }
    if encoded.is_empty() {
        return Ok(0);
    }
    let unpadded = if let Some(prefix) = encoded.strip_suffix(b"==") {
        prefix
    } else if let Some(prefix) = encoded.strip_suffix(b"=") {
        prefix
    } else {
        encoded
    };
    let full_chunks = unpadded.len() / 4;
    let mut in_pos = 0;
    let mut out_pos = 0;
    for _ in 0..full_chunks {
        let c0 = b64_val(unpadded[in_pos]);
        let c1 = b64_val(unpadded[in_pos + 1]);
        let c2 = b64_val(unpadded[in_pos + 2]);
        let c3 = b64_val(unpadded[in_pos + 3]);
        if (c0 | c1 | c2 | c3) & 0x80 != 0 {
            return Err(HpackError::InvalidHeaderValue);
        }
        out[out_pos] = (c0 << 2) | (c1 >> 4);
        out[out_pos + 1] = (c1 << 4) | (c2 >> 2);
        out[out_pos + 2] = (c2 << 6) | c3;
        in_pos += 4;
        out_pos += 3;
    }
    let rem = unpadded.len() - in_pos;
    if rem == 2 {
        let c0 = b64_val(unpadded[in_pos]);
        let c1 = b64_val(unpadded[in_pos + 1]);
        if (c0 | c1) & 0x80 != 0 {
            return Err(HpackError::InvalidHeaderValue);
        }
        out[out_pos] = (c0 << 2) | (c1 >> 4);
        out_pos += 1;
    } else if rem == 3 {
        let c0 = b64_val(unpadded[in_pos]);
        let c1 = b64_val(unpadded[in_pos + 1]);
        let c2 = b64_val(unpadded[in_pos + 2]);
        if (c0 | c1 | c2) & 0x80 != 0 {
            return Err(HpackError::InvalidHeaderValue);
        }
        out[out_pos] = (c0 << 2) | (c1 >> 4);
        out[out_pos + 1] = (c1 << 4) | (c2 >> 2);
        out_pos += 2;
    }
    debug_assert_eq!(out_pos, decoded_len);
    Ok(out_pos)
}

/// Storage interface for custom gRPC metadata header/trailer key-value pairs.
///
/// Implementations include:
/// - [`FixedMetadata`]: `#![no_std]` zero-allocation fixed-capacity storage that fails loudly with
///   [`HpackError::BufferOverflow`] when capacity is exceeded.
/// - `DynamicMetadata` (with the `std` feature): heap-allocated growable storage.
pub trait MetadataMap: Send + Sync {
    /// Appends a metadata entry `(name, value)` to the store, where `value` is raw decoded bytes
    /// for `*-bin` headers or printable ASCII (`0x20..=0x7E`) for ASCII headers.
    ///
    /// Fails loudly with [`HpackError::BufferOverflow`] if the underlying storage cannot fit the
    /// entry, or [`HpackError::InvalidHeaderValue`] if `name` is invalid/reserved or an ASCII
    /// header value contains non-printable bytes.
    fn push(&mut self, name: &[u8], value: &[u8]) -> Result<(), HpackError>;

    /// Appends an inbound wire-encoded metadata entry `(name, wire_value)`, Base64-decoding
    /// `wire_value` when `name` ends with `"-bin"` (accepting both padded and unpadded Base64 per
    /// the gRPC HTTP/2 specification).
    fn push_wire(&mut self, name: &[u8], wire_value: &[u8]) -> Result<(), HpackError>;

    /// Removes all entries from the store.
    fn clear(&mut self);

    /// Returns the number of metadata entries currently stored.
    fn len(&self) -> usize;

    /// Returns `true` if the store contains no entries.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the metadata entry at `index` (`0..self.len()`).
    fn get_entry(&self, index: usize) -> Option<HeaderField<'_>>;

    /// Looks up the first value whose header name matches `name` (case-insensitively).
    #[inline]
    fn get(&self, name: &[u8]) -> Option<&[u8]> {
        let len = self.len();
        for i in 0..len {
            if let Some(field) = self.get_entry(i) {
                if field.name.eq_ignore_ascii_case(name) {
                    return Some(field.value);
                }
            }
        }
        None
    }

    /// Looks up the first value whose header name matches `name` (case-insensitively) as raw bytes.
    #[inline]
    fn get_bytes(&self, name: &str) -> Option<&[u8]> {
        self.get(name.as_bytes())
    }

    /// Looks up the first value whose header name matches `name` (case-insensitively) and
    /// validates that the value is UTF-8.
    #[inline]
    fn get_str(&self, name: &str) -> Option<&str> {
        let bytes = self.get(name.as_bytes())?;
        core::str::from_utf8(bytes).ok()
    }

    /// Inserts a validated custom metadata header `(name, value)`.
    ///
    /// Rejects reserved gRPC/HTTP2 headers, invalid header names, and non-printable ASCII values
    /// with [`HpackError::InvalidHeaderValue`], and fails with [`HpackError::BufferOverflow`] if full.
    #[inline]
    fn insert(&mut self, name: &str, value: &str) -> Result<(), HpackError> {
        self.insert_bytes(name.as_bytes(), value.as_bytes())
    }

    /// Inserts a validated custom metadata header `(name, value)` with raw value bytes.
    ///
    /// For `*-bin` headers, `value` is stored as raw binary bytes and Base64-encoded on the wire.
    /// For ASCII headers, `value` must consist of printable ASCII (`0x20..=0x7E`).
    #[inline]
    fn insert_bytes(&mut self, name: &[u8], value: &[u8]) -> Result<(), HpackError> {
        self.push(name, value)
    }

    /// Replaces the contents of `self` with all entries from `other`.
    #[inline]
    fn copy_from(&mut self, other: &dyn MetadataMap) -> Result<(), HpackError> {
        self.clear();
        let n = other.len();
        for i in 0..n {
            if let Some(field) = other.get_entry(i) {
                self.push(field.name, field.value)?;
            }
        }
        Ok(())
    }
}

impl<'a> dyn MetadataMap + 'a {
    /// Returns an iterator over all [`HeaderField`] entries in this metadata map.
    #[inline]
    pub fn iter(&self) -> MetadataIter<'_> {
        MetadataIter {
            map: self,
            index: 0,
        }
    }
}

/// Iterator over [`HeaderField`] entries in a [`MetadataMap`].
#[must_use]
pub struct MetadataIter<'a> {
    map: &'a (dyn MetadataMap + 'a),
    index: usize,
}

impl<'a> Iterator for MetadataIter<'a> {
    type Item = HeaderField<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let field = self.map.get_entry(self.index)?;
        self.index += 1;
        Some(field)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.map.len().saturating_sub(self.index);
        (rem, Some(rem))
    }
}

impl ExactSizeIterator for MetadataIter<'_> {}

#[inline]
fn validate_entry_header(name: &[u8]) -> Result<bool, HpackError> {
    if !is_valid_metadata_name(name) || is_reserved_grpc_header(name) {
        return Err(HpackError::InvalidHeaderValue);
    }
    Ok(is_binary_metadata_name(name))
}

/// Zero-sized empty [`MetadataMap`] that rejects all valid insertions with [`HpackError::BufferOverflow`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EmptyMetadata;

/// Shared static instance of [`EmptyMetadata`].
pub static EMPTY_METADATA: EmptyMetadata = EmptyMetadata;

impl MetadataMap for EmptyMetadata {
    #[inline]
    fn push(&mut self, name: &[u8], value: &[u8]) -> Result<(), HpackError> {
        let is_bin = validate_entry_header(name)?;
        if !is_bin && !is_valid_ascii_metadata_value(value) {
            return Err(HpackError::InvalidHeaderValue);
        }
        Err(HpackError::BufferOverflow)
    }

    #[inline]
    fn push_wire(&mut self, name: &[u8], wire_value: &[u8]) -> Result<(), HpackError> {
        let is_bin = validate_entry_header(name)?;
        if is_bin {
            let _ = base64_decoded_len(wire_value)?;
        } else if !is_valid_ascii_metadata_value(wire_value) {
            return Err(HpackError::InvalidHeaderValue);
        }
        Err(HpackError::BufferOverflow)
    }

    #[inline]
    fn clear(&mut self) {}

    #[inline]
    fn len(&self) -> usize {
        0
    }

    #[inline]
    fn get_entry(&self, _index: usize) -> Option<HeaderField<'_>> {
        None
    }
}

#[derive(Clone, Copy, Default)]
struct MetaSpan {
    name_start: u16,
    name_len: u16,
    val_start: u16,
    val_len: u16,
}

/// `#![no_std]` zero-allocation fixed-capacity metadata map.
///
/// Stores up to `MAX_ENTRIES` key-value pairs whose combined key and value byte lengths do not
/// exceed `CAP` bytes. Fails loudly with [`HpackError::BufferOverflow`] when either limit is
/// exceeded.
#[derive(Clone, Copy)]
pub struct FixedMetadata<
    const MAX_ENTRIES: usize = DEFAULT_METADATA_ENTRIES,
    const CAP: usize = DEFAULT_METADATA_CAP,
> {
    buf: [u8; CAP],
    spans: [MetaSpan; MAX_ENTRIES],
    len: usize,
    used: usize,
}

impl<const MAX_ENTRIES: usize, const CAP: usize> Default for FixedMetadata<MAX_ENTRIES, CAP> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX_ENTRIES: usize, const CAP: usize> fmt::Debug for FixedMetadata<MAX_ENTRIES, CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        for field in self.iter() {
            list.entry(&field);
        }
        list.finish()
    }
}

impl<const MAX_ENTRIES: usize, const CAP: usize> PartialEq for FixedMetadata<MAX_ENTRIES, CAP> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        for i in 0..self.len {
            if self.get_entry(i) != other.get_entry(i) {
                return false;
            }
        }
        true
    }
}

impl<const MAX_ENTRIES: usize, const CAP: usize> Eq for FixedMetadata<MAX_ENTRIES, CAP> {}

impl<const MAX_ENTRIES: usize, const CAP: usize> FixedMetadata<MAX_ENTRIES, CAP> {
    /// Creates an empty [`FixedMetadata`] store.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            buf: [0u8; CAP],
            spans: [MetaSpan {
                name_start: 0,
                name_len: 0,
                val_start: 0,
                val_len: 0,
            }; MAX_ENTRIES],
            len: 0,
            used: 0,
        }
    }

    /// Returns the maximum number of entries this store can hold (`MAX_ENTRIES`).
    #[must_use]
    #[inline]
    pub const fn max_entries(&self) -> usize {
        MAX_ENTRIES
    }

    /// Returns the total byte capacity of this store (`CAP`).
    #[must_use]
    #[inline]
    pub const fn capacity_bytes(&self) -> usize {
        CAP
    }

    /// Returns the number of bytes currently used in the backing buffer.
    #[must_use]
    #[inline]
    pub const fn used_bytes(&self) -> usize {
        self.used
    }

    /// Returns the number of entries currently stored.
    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the store contains no entries.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Removes all entries from the store.
    #[inline]
    pub fn clear(&mut self) {
        <Self as MetadataMap>::clear(self);
    }

    /// Inserts a validated custom ASCII metadata header `(name, value)`.
    #[inline]
    pub fn insert(&mut self, name: &str, value: &str) -> Result<(), HpackError> {
        <Self as MetadataMap>::insert(self, name, value)
    }

    /// Inserts a validated custom metadata header `(name, value)` with raw value bytes.
    #[inline]
    pub fn insert_bytes(&mut self, name: &[u8], value: &[u8]) -> Result<(), HpackError> {
        <Self as MetadataMap>::insert_bytes(self, name, value)
    }

    /// Looks up the first value whose header name matches `name` (case-insensitively).
    #[must_use]
    #[inline]
    pub fn get(&self, name: &[u8]) -> Option<&[u8]> {
        <Self as MetadataMap>::get(self, name)
    }

    /// Looks up the first value whose header name matches `name` (case-insensitively) as raw bytes.
    #[must_use]
    #[inline]
    pub fn get_bytes(&self, name: &str) -> Option<&[u8]> {
        <Self as MetadataMap>::get_bytes(self, name)
    }

    /// Looks up the first value whose header name matches `name` (case-insensitively) as a UTF-8 `&str`.
    #[must_use]
    #[inline]
    pub fn get_str(&self, name: &str) -> Option<&str> {
        <Self as MetadataMap>::get_str(self, name)
    }

    /// Returns an iterator over all [`HeaderField`] entries in this map.
    #[inline]
    pub fn iter(&self) -> MetadataIter<'_> {
        MetadataIter {
            map: self,
            index: 0,
        }
    }

    fn push_raw(&mut self, name: &[u8], value: &[u8]) -> Result<(), HpackError> {
        if self.len >= MAX_ENTRIES {
            return Err(HpackError::BufferOverflow);
        }
        let entry_bytes = name
            .len()
            .checked_add(value.len())
            .ok_or(HpackError::BufferOverflow)?;
        let new_used = self
            .used
            .checked_add(entry_bytes)
            .ok_or(HpackError::BufferOverflow)?;
        if new_used > CAP {
            return Err(HpackError::BufferOverflow);
        }

        let name_start = self.used;
        let val_start = name_start + name.len();

        let span = MetaSpan {
            name_start: u16::try_from(name_start).map_err(|_| HpackError::BufferOverflow)?,
            name_len: u16::try_from(name.len()).map_err(|_| HpackError::BufferOverflow)?,
            val_start: u16::try_from(val_start).map_err(|_| HpackError::BufferOverflow)?,
            val_len: u16::try_from(value.len()).map_err(|_| HpackError::BufferOverflow)?,
        };

        self.buf[name_start..val_start].copy_from_slice(name);
        self.buf[val_start..new_used].copy_from_slice(value);
        self.spans[self.len] = span;
        self.len += 1;
        self.used = new_used;
        Ok(())
    }
}

impl<const MAX_ENTRIES: usize, const CAP: usize> MetadataMap for FixedMetadata<MAX_ENTRIES, CAP> {
    fn push(&mut self, name: &[u8], value: &[u8]) -> Result<(), HpackError> {
        let is_bin = validate_entry_header(name)?;
        if !is_bin && !is_valid_ascii_metadata_value(value) {
            return Err(HpackError::InvalidHeaderValue);
        }
        self.push_raw(name, value)
    }

    fn push_wire(&mut self, name: &[u8], wire_value: &[u8]) -> Result<(), HpackError> {
        let is_bin = validate_entry_header(name)?;
        if !is_bin {
            if !is_valid_ascii_metadata_value(wire_value) {
                return Err(HpackError::InvalidHeaderValue);
            }
            return self.push_raw(name, wire_value);
        }

        let decoded_len = base64_decoded_len(wire_value)?;
        if self.len >= MAX_ENTRIES {
            return Err(HpackError::BufferOverflow);
        }
        let entry_bytes = name
            .len()
            .checked_add(decoded_len)
            .ok_or(HpackError::BufferOverflow)?;
        let new_used = self
            .used
            .checked_add(entry_bytes)
            .ok_or(HpackError::BufferOverflow)?;
        if new_used > CAP {
            return Err(HpackError::BufferOverflow);
        }

        let name_start = self.used;
        let val_start = name_start + name.len();

        let span = MetaSpan {
            name_start: u16::try_from(name_start).map_err(|_| HpackError::BufferOverflow)?,
            name_len: u16::try_from(name.len()).map_err(|_| HpackError::BufferOverflow)?,
            val_start: u16::try_from(val_start).map_err(|_| HpackError::BufferOverflow)?,
            val_len: u16::try_from(decoded_len).map_err(|_| HpackError::BufferOverflow)?,
        };

        base64_decode(wire_value, &mut self.buf[val_start..new_used])?;
        self.buf[name_start..val_start].copy_from_slice(name);
        self.spans[self.len] = span;
        self.len += 1;
        self.used = new_used;
        Ok(())
    }

    #[inline]
    fn clear(&mut self) {
        self.len = 0;
        self.used = 0;
    }

    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    fn get_entry(&self, index: usize) -> Option<HeaderField<'_>> {
        if index >= self.len {
            return None;
        }
        let span = self.spans[index];
        let ns = span.name_start as usize;
        let ne = ns + span.name_len as usize;
        let vs = span.val_start as usize;
        let ve = vs + span.val_len as usize;
        Some(HeaderField {
            name: &self.buf[ns..ne],
            value: &self.buf[vs..ve],
        })
    }
}

#[cfg(feature = "std")]
#[derive(Clone, Copy, Default)]
struct DynMetaSpan {
    name_start: usize,
    name_len: usize,
    val_start: usize,
    val_len: usize,
}

/// Growable heap-backed gRPC metadata map available when the `std` feature is enabled.
///
/// Does not allocate on construction (`DynamicMetadata::new()`); grows dynamically as custom
/// metadata entries are pushed.
#[cfg(feature = "std")]
#[derive(Clone, Default)]
pub struct DynamicMetadata {
    buf: std::vec::Vec<u8>,
    spans: std::vec::Vec<DynMetaSpan>,
}

#[cfg(feature = "std")]
impl fmt::Debug for DynamicMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        for field in self.iter() {
            list.entry(&field);
        }
        list.finish()
    }
}

#[cfg(feature = "std")]
impl PartialEq for DynamicMetadata {
    fn eq(&self, other: &Self) -> bool {
        if self.spans.len() != other.spans.len() {
            return false;
        }
        for i in 0..self.spans.len() {
            if self.get_entry(i) != other.get_entry(i) {
                return false;
            }
        }
        true
    }
}

#[cfg(feature = "std")]
impl Eq for DynamicMetadata {}

#[cfg(feature = "std")]
impl DynamicMetadata {
    /// Creates an empty [`DynamicMetadata`] without allocating.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            buf: std::vec::Vec::new(),
            spans: std::vec::Vec::new(),
        }
    }

    /// Creates an empty [`DynamicMetadata`] with pre-allocated capacity for `entries` and `bytes`.
    #[must_use]
    #[inline]
    pub fn with_capacity(entries: usize, bytes: usize) -> Self {
        Self {
            buf: std::vec::Vec::with_capacity(bytes),
            spans: std::vec::Vec::with_capacity(entries),
        }
    }

    /// Returns the number of bytes currently stored in the backing buffer.
    #[must_use]
    #[inline]
    pub fn used_bytes(&self) -> usize {
        self.buf.len()
    }

    /// Returns the number of entries currently stored.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.spans.len()
    }

    /// Returns `true` if the store contains no entries.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    /// Removes all entries from the store.
    #[inline]
    pub fn clear(&mut self) {
        <Self as MetadataMap>::clear(self);
    }

    /// Inserts a validated custom ASCII metadata header `(name, value)`.
    #[inline]
    pub fn insert(&mut self, name: &str, value: &str) -> Result<(), HpackError> {
        <Self as MetadataMap>::insert(self, name, value)
    }

    /// Inserts a validated custom metadata header `(name, value)` with raw value bytes.
    #[inline]
    pub fn insert_bytes(&mut self, name: &[u8], value: &[u8]) -> Result<(), HpackError> {
        <Self as MetadataMap>::insert_bytes(self, name, value)
    }

    /// Looks up the first value whose header name matches `name` (case-insensitively).
    #[must_use]
    #[inline]
    pub fn get(&self, name: &[u8]) -> Option<&[u8]> {
        <Self as MetadataMap>::get(self, name)
    }

    /// Looks up the first value whose header name matches `name` (case-insensitively) as raw bytes.
    #[must_use]
    #[inline]
    pub fn get_bytes(&self, name: &str) -> Option<&[u8]> {
        <Self as MetadataMap>::get_bytes(self, name)
    }

    /// Looks up the first value whose header name matches `name` (case-insensitively) as a UTF-8 `&str`.
    #[must_use]
    #[inline]
    pub fn get_str(&self, name: &str) -> Option<&str> {
        <Self as MetadataMap>::get_str(self, name)
    }

    /// Returns an iterator over all [`HeaderField`] entries in this map.
    #[inline]
    pub fn iter(&self) -> MetadataIter<'_> {
        MetadataIter {
            map: self,
            index: 0,
        }
    }

    fn push_raw(&mut self, name: &[u8], value: &[u8]) {
        let name_start = self.buf.len();
        self.buf.reserve(name.len().saturating_add(value.len()));
        self.buf.extend_from_slice(name);
        let val_start = self.buf.len();
        self.buf.extend_from_slice(value);
        self.spans.push(DynMetaSpan {
            name_start,
            name_len: name.len(),
            val_start,
            val_len: value.len(),
        });
    }
}

#[cfg(feature = "std")]
impl MetadataMap for DynamicMetadata {
    fn push(&mut self, name: &[u8], value: &[u8]) -> Result<(), HpackError> {
        let is_bin = validate_entry_header(name)?;
        if !is_bin && !is_valid_ascii_metadata_value(value) {
            return Err(HpackError::InvalidHeaderValue);
        }
        self.push_raw(name, value);
        Ok(())
    }

    fn push_wire(&mut self, name: &[u8], wire_value: &[u8]) -> Result<(), HpackError> {
        let is_bin = validate_entry_header(name)?;
        if !is_bin {
            if !is_valid_ascii_metadata_value(wire_value) {
                return Err(HpackError::InvalidHeaderValue);
            }
            self.push_raw(name, wire_value);
            return Ok(());
        }

        let decoded_len = base64_decoded_len(wire_value)?;
        let name_start = self.buf.len();
        let val_start = name_start + name.len();
        let new_used = val_start + decoded_len;
        self.buf.resize(new_used, 0);
        if let Err(err) = base64_decode(wire_value, &mut self.buf[val_start..new_used]) {
            self.buf.truncate(name_start);
            return Err(err);
        }
        self.buf[name_start..val_start].copy_from_slice(name);
        self.spans.push(DynMetaSpan {
            name_start,
            name_len: name.len(),
            val_start,
            val_len: decoded_len,
        });
        Ok(())
    }

    #[inline]
    fn clear(&mut self) {
        self.buf.clear();
        self.spans.clear();
    }

    #[inline]
    fn len(&self) -> usize {
        self.spans.len()
    }

    #[inline]
    fn get_entry(&self, index: usize) -> Option<HeaderField<'_>> {
        let span = *self.spans.get(index)?;
        let ns = span.name_start;
        let ne = ns + span.name_len;
        let vs = span.val_start;
        let ve = vs + span.val_len;
        Some(HeaderField {
            name: &self.buf[ns..ne],
            value: &self.buf[vs..ve],
        })
    }
}

/// Per-RPC metadata context passed to generated service trait handlers.
///
/// Provides read access to inbound request metadata and mutable access to outbound initial
/// response headers and trailing metadata.
pub struct RpcMetadata<'a> {
    request: &'a dyn MetadataMap,
    response_headers: &'a mut dyn MetadataMap,
    response_trailers: &'a mut dyn MetadataMap,
}

impl fmt::Debug for RpcMetadata<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RpcMetadata")
            .field("request_len", &self.request.len())
            .field("response_headers_len", &self.response_headers.len())
            .field("response_trailers_len", &self.response_trailers.len())
            .finish()
    }
}

impl<'a> RpcMetadata<'a> {
    /// Creates an [`RpcMetadata`] context binding inbound `request` metadata and outbound
    /// `response_headers` and `response_trailers` stores.
    #[must_use]
    #[inline]
    pub fn new(
        request: &'a dyn MetadataMap,
        response_headers: &'a mut dyn MetadataMap,
        response_trailers: &'a mut dyn MetadataMap,
    ) -> Self {
        Self {
            request,
            response_headers,
            response_trailers,
        }
    }

    /// Returns the inbound request [`MetadataMap`].
    #[must_use]
    #[inline]
    pub fn request(&self) -> &'a (dyn MetadataMap + 'a) {
        self.request
    }

    /// Looks up an inbound request metadata header value by `name` (case-insensitively) as raw bytes.
    #[must_use]
    #[inline]
    pub fn get(&self, name: &str) -> Option<&'a [u8]> {
        self.request.get(name.as_bytes())
    }

    /// Looks up an inbound request metadata header value by raw `name` bytes (case-insensitively).
    #[must_use]
    #[inline]
    pub fn get_bytes(&self, name: &[u8]) -> Option<&'a [u8]> {
        self.request.get(name)
    }

    /// Looks up an inbound request metadata header value by `name` (case-insensitively) as a UTF-8 `&str`.
    #[must_use]
    #[inline]
    pub fn get_str(&self, name: &str) -> Option<&'a str> {
        self.request.get_str(name)
    }

    /// Returns an iterator over all inbound request metadata [`HeaderField`] entries.
    #[inline]
    pub fn iter(&self) -> MetadataIter<'a> {
        self.request.iter()
    }

    /// Returns an immutable view of the outbound response initial metadata map.
    #[must_use]
    #[inline]
    pub fn response_headers(&self) -> &dyn MetadataMap {
        self.response_headers
    }

    /// Returns a mutable view of the outbound response initial metadata map.
    #[inline]
    pub fn response_headers_mut(&mut self) -> &mut dyn MetadataMap {
        self.response_headers
    }

    /// Appends a custom ASCII header to the outbound initial response metadata.
    ///
    /// Fails loudly with `GrpcCode::ResourceExhausted` if the backing [`MetadataMap`] overflows,
    /// or `GrpcCode::Internal` if `name` or `value` is invalid or reserved.
    #[inline]
    pub fn insert_header(&mut self, name: &str, value: &str) -> Result<(), GrpcStatus<'static>> {
        self.response_headers
            .insert(name, value)
            .map_err(GrpcStatus::from)
    }

    /// Appends a custom header with raw value bytes to the outbound initial response metadata.
    #[inline]
    pub fn insert_header_bytes(
        &mut self,
        name: &[u8],
        value: &[u8],
    ) -> Result<(), GrpcStatus<'static>> {
        self.response_headers
            .insert_bytes(name, value)
            .map_err(GrpcStatus::from)
    }

    /// Returns an immutable view of the outbound response trailing metadata map.
    #[must_use]
    #[inline]
    pub fn response_trailers(&self) -> &dyn MetadataMap {
        self.response_trailers
    }

    /// Returns a mutable view of the outbound response trailing metadata map.
    #[inline]
    pub fn response_trailers_mut(&mut self) -> &mut dyn MetadataMap {
        self.response_trailers
    }

    /// Appends a custom ASCII trailer to the outbound response trailing metadata.
    ///
    /// Fails loudly with `GrpcCode::ResourceExhausted` if the backing [`MetadataMap`] overflows,
    /// or `GrpcCode::Internal` if `name` or `value` is invalid or reserved.
    #[inline]
    pub fn insert_trailer(&mut self, name: &str, value: &str) -> Result<(), GrpcStatus<'static>> {
        self.response_trailers
            .insert(name, value)
            .map_err(GrpcStatus::from)
    }

    /// Appends a custom trailer with raw value bytes to the outbound response trailing metadata.
    #[inline]
    pub fn insert_trailer_bytes(
        &mut self,
        name: &[u8],
        value: &[u8],
    ) -> Result<(), GrpcStatus<'static>> {
        self.response_trailers
            .insert_bytes(name, value)
            .map_err(GrpcStatus::from)
    }
}

/// Self-contained metadata storage triple (`request`, `response_headers`, `response_trailers`)
/// for direct unit testing or in-memory service invocation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RpcContext<M: MetadataMap = FixedMetadata> {
    pub request: M,
    pub response_headers: M,
    pub response_trailers: M,
}

impl<M: MetadataMap + Default> RpcContext<M> {
    /// Creates an empty [`RpcContext`] using `M::default()`.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<M: MetadataMap> RpcContext<M> {
    /// Borrows this context as an [`RpcMetadata`] handle suitable for passing to service methods.
    #[must_use]
    #[inline]
    pub fn metadata(&mut self) -> RpcMetadata<'_> {
        RpcMetadata::new(
            &self.request,
            &mut self.response_headers,
            &mut self.response_trailers,
        )
    }

    /// Alias for [`Self::metadata`].
    #[must_use]
    #[inline]
    pub fn as_metadata(&mut self) -> RpcMetadata<'_> {
        self.metadata()
    }

    /// Clears all three metadata maps (`request`, `response_headers`, and `response_trailers`).
    #[inline]
    pub fn clear(&mut self) {
        self.request.clear();
        self.response_headers.clear();
        self.response_trailers.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::GrpcCode;

    #[test]
    fn metadata_name_and_value_validation_and_reserved_filtering() {
        assert!(is_valid_metadata_name(b"x-custom-1_2.3"));
        assert!(is_valid_metadata_name(b"x-trace-bin"));
        assert!(is_binary_metadata_name(b"x-trace-bin"));
        assert!(!is_binary_metadata_name(b"-bin"));
        assert!(!is_binary_metadata_name(b"x-trace"));

        // Invalid names: empty, pseudo-header, bare "-bin", uppercase, spaces/control/punctuation
        assert!(!is_valid_metadata_name(b""));
        assert!(!is_valid_metadata_name(b":path"));
        assert!(!is_valid_metadata_name(b"-bin"));
        assert!(!is_valid_metadata_name(b"X-Upper"));
        assert!(!is_valid_metadata_name(b"bad header"));
        assert!(!is_valid_metadata_name(b"bad:header"));
        assert!(!is_valid_metadata_name(b"bad/header"));
        assert!(!is_valid_metadata_name(b"bad\nheader"));

        // ASCII values: printable ASCII + space (0x20..=0x7E)
        assert!(is_valid_ascii_metadata_value(b""));
        assert!(is_valid_ascii_metadata_value(b"hello world ~!@#"));
        assert!(!is_valid_ascii_metadata_value(b"line\nbreak"));
        assert!(!is_valid_ascii_metadata_value(b"null\0byte"));
        assert!(!is_valid_ascii_metadata_value(b"\x7f"));
        assert!(!is_valid_ascii_metadata_value(b"\x80\xff"));

        // Reserved gRPC / HTTP/2 headers
        for reserved in [
            b":method".as_slice(),
            b":status",
            b"content-type",
            b"Content-Type",
            b"te",
            b"user-agent",
            b"grpc-status",
            b"grpc-message",
            b"grpc-encoding",
            b"grpc-accept-encoding",
            b"grpc-timeout",
            b"grpc-custom-anything",
        ] {
            assert!(is_reserved_grpc_header(reserved));
        }
        assert!(!is_reserved_grpc_header(b"x-grpc-custom"));
    }

    #[test]
    fn base64_forward_backward_padded_unpadded_and_invalid_inputs() {
        for raw in [
            b"".as_slice(),
            b"\x00",
            b"\x01\x02",
            b"\x00\xFF\x42",
            b"\x01\x02\x03\x04",
            b"\xDE\xAD\xBE\xEF\xCA\xFE",
            b"roto-grpc binary metadata payload!",
        ] {
            let enc_len = base64_encoded_len(raw.len());
            let mut enc_buf = [0u8; 128];
            let n = base64_encode(raw, &mut enc_buf).unwrap();
            assert_eq!(n, enc_len);

            let mut bw_buf = [0u8; 128];
            let mut writer = BackwardWriter::new(&mut bw_buf);
            base64_encode_backward(raw, &mut writer);
            let bw_slice = writer.finish().unwrap();
            assert_eq!(bw_slice, &enc_buf[..n]);

            // Decode padded
            let mut dec_buf = [0u8; 128];
            let dec_n = base64_decode(&enc_buf[..n], &mut dec_buf).unwrap();
            assert_eq!(&dec_buf[..dec_n], raw);

            // Decode unpadded (strip '=' padding)
            let unpadded = enc_buf[..n].trim_ascii_end_matches_eq();
            let mut dec_unpadded = [0u8; 128];
            let dec_un_n = base64_decode(unpadded, &mut dec_unpadded).unwrap();
            assert_eq!(&dec_unpadded[..dec_un_n], raw);
        }

        // Buffer overflow on encode and decode
        let mut tiny = [0u8; 2];
        assert_eq!(
            base64_encode(b"abc", &mut tiny),
            Err(EncodeError::BufferOverflow)
        );
        assert_eq!(
            base64_decode(b"YWJj", &mut tiny),
            Err(HpackError::BufferOverflow)
        );

        // Invalid base64 inputs
        for bad in [
            b"A".as_slice(),
            b"ABCDE",
            b"=",
            b"==",
            b"===",
            b"====",
            b"A===",
            b"AB=C",
            b"not!base64",
            b"YWJj\n",
        ] {
            let mut out = [0u8; 32];
            assert_eq!(
                base64_decode(bad, &mut out),
                Err(HpackError::InvalidHeaderValue),
                "expected InvalidHeaderValue for {bad:?}"
            );
        }
    }

    trait TrimEq {
        fn trim_ascii_end_matches_eq(&self) -> &[u8];
    }
    impl TrimEq for [u8] {
        fn trim_ascii_end_matches_eq(&self) -> &[u8] {
            let mut s = self;
            while let Some(prefix) = s.strip_suffix(b"=") {
                s = prefix;
            }
            s
        }
    }

    #[test]
    fn empty_fixed_and_dynamic_metadata_methods_and_iterators() {
        // 1. EmptyMetadata
        let mut empty = EmptyMetadata;
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
        assert_eq!(empty.get_entry(0), None);
        assert_eq!(empty.get(b"x-key"), None);
        assert_eq!((&empty as &dyn MetadataMap).iter().len(), 0);
        assert_eq!((&empty as &dyn MetadataMap).iter().next(), None);
        empty.clear();
        assert_eq!(empty.insert("x-key", "val"), Err(HpackError::BufferOverflow));
        assert_eq!(
            empty.insert("grpc-status", "0"),
            Err(HpackError::InvalidHeaderValue)
        );
        assert_eq!(
            empty.insert("X-Upper", "val"),
            Err(HpackError::InvalidHeaderValue)
        );
        assert_eq!(
            empty.insert("x-key", "bad\nval"),
            Err(HpackError::InvalidHeaderValue)
        );
        assert_eq!(
            empty.push_wire(b"x-bin", b"AQID"),
            Err(HpackError::BufferOverflow)
        );
        assert_eq!(
            empty.push_wire(b"x-bin", b"A"),
            Err(HpackError::InvalidHeaderValue)
        );

        // 2. FixedMetadata
        let mut fixed: FixedMetadata<3, 32> = FixedMetadata::default();
        assert_eq!(fixed.max_entries(), 3);
        assert_eq!(fixed.capacity_bytes(), 32);
        assert_eq!(fixed.used_bytes(), 0);
        assert_eq!(fixed.len(), 0);
        assert!(fixed.is_empty());

        fixed.insert("x-dup", "first").unwrap();
        fixed.insert("x-dup", "second").unwrap();
        fixed.push_wire(b"x-bin", b"AP9C").unwrap(); // decodes to [0x00, 0xFF, 0x42]
        assert_eq!(fixed.len(), 3);
        assert!(!fixed.is_empty());
        assert_eq!(fixed.used_bytes(), 5 + 5 + 5 + 6 + 5 + 3);
        assert_eq!(fixed.get_str("x-dup"), Some("first"));
        assert_eq!(fixed.get_str("X-DUP"), Some("first"));
        assert_eq!(fixed.get_bytes("x-bin"), Some(&[0x00, 0xFF, 0x42][..]));
        assert_eq!(fixed.get(b"x-bin"), Some(&[0x00, 0xFF, 0x42][..]));

        // ExactSizeIterator on FixedMetadata and &dyn MetadataMap
        let mut it = fixed.iter();
        assert_eq!(it.len(), 3);
        assert_eq!(
            it.next(),
            Some(HeaderField {
                name: b"x-dup",
                value: b"first"
            })
        );
        assert_eq!(it.len(), 2);
        assert_eq!(
            it.next(),
            Some(HeaderField {
                name: b"x-dup",
                value: b"second"
            })
        );
        assert_eq!(it.len(), 1);
        assert_eq!(
            it.next(),
            Some(HeaderField {
                name: b"x-bin",
                value: &[0x00, 0xFF, 0x42]
            })
        );
        assert_eq!(it.len(), 0);
        assert_eq!(it.next(), None);

        // MAX_ENTRIES overflow on 4th entry
        assert_eq!(
            fixed.insert("a", "b"),
            Err(HpackError::BufferOverflow)
        );

        // CAP overflow in copy_from
        let mut small_fixed: FixedMetadata<4, 12> = FixedMetadata::new();
        assert_eq!(
            small_fixed.copy_from(&fixed),
            Err(HpackError::BufferOverflow)
        );

        // 3. DynamicMetadata + RpcContext + RpcMetadata
        #[cfg(feature = "std")]
        {
            let mut dyn_meta = DynamicMetadata::with_capacity(4, 64);
            assert!(dyn_meta.is_empty());
            assert_eq!(dyn_meta.len(), 0);
            assert_eq!(dyn_meta.used_bytes(), 0);
            dyn_meta.copy_from(&fixed).unwrap();
            assert_eq!(dyn_meta.len(), 3);
            assert_eq!(dyn_meta.used_bytes(), fixed.used_bytes());
            assert_eq!(dyn_meta.get_str("x-dup"), Some("first"));
            assert_eq!(dyn_meta.get_bytes("x-bin"), Some(&[0x00, 0xFF, 0x42][..]));

            let collected: std::vec::Vec<_> = (&dyn_meta as &dyn MetadataMap).iter().collect();
            assert_eq!(collected.len(), 3);

            dyn_meta.clear();
            assert!(dyn_meta.is_empty());
            assert_eq!(dyn_meta.used_bytes(), 0);
        }

        // 4. RpcContext & RpcMetadata methods and error mapping
        let mut ctx: RpcContext<FixedMetadata<2, 24>> = RpcContext::new();
        ctx.request.insert("x-req", "rval").unwrap();
        {
            let mut meta = ctx.as_metadata();
            assert_eq!(meta.request().len(), 1);
            assert_eq!(meta.get("x-req"), Some(&b"rval"[..]));
            assert_eq!(meta.get_bytes(b"x-req"), Some(&b"rval"[..]));
            assert_eq!(meta.get_str("x-req"), Some("rval"));
            assert_eq!(meta.iter().len(), 1);

            meta.insert_header("x-h", "hv").unwrap();
            meta.insert_header_bytes(b"x-hb-bin", &[1, 2]).unwrap();
            assert_eq!(meta.response_headers().len(), 2);
            // Overflow on 3rd header -> ResourceExhausted
            assert_eq!(
                meta.insert_header("x-h3", "v").unwrap_err().code,
                GrpcCode::ResourceExhausted
            );

            meta.insert_trailer("x-t", "tv").unwrap();
            meta.insert_trailer_bytes(b"x-tb-bin", &[3, 4]).unwrap();
            assert_eq!(meta.response_trailers().len(), 2);
            // Invalid header -> Internal
            assert_eq!(
                meta.insert_trailer("grpc-status", "0").unwrap_err().code,
                GrpcCode::Internal
            );
        }
        ctx.clear();
        assert!(ctx.request.is_empty());
        assert!(ctx.response_headers.is_empty());
        assert!(ctx.response_trailers.is_empty());
    }
}
