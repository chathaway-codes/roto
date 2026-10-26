use core::fmt;

use roto_runtime::{BackwardWriter, EncodeError};

use crate::metadata::{
    base64_encode, base64_encode_backward, base64_encoded_len, is_binary_metadata_name,
    is_reserved_grpc_header, MetadataMap,
};
use crate::status::{
    percent_decode, percent_encode, percent_encode_backward, percent_encoded_len,
    try_percent_decode_in_place, GrpcCode, GrpcStatus,
};

/// Default capacity in bytes for the stack-allocated HPACK dynamic table.
pub const DEFAULT_DYNAMIC_TABLE_CAP: usize = 512;

/// Overhead in bytes per dynamic table entry as specified by RFC 7541 §4.1.
pub const DYNAMIC_ENTRY_OVERHEAD: usize = 32;

/// Total number of entries in the RFC 7541 Appendix A static table.
pub const STATIC_TABLE_LEN: usize = 61;

/// Default gRPC `content-type` header value for Roto messages.
pub const GRPC_CONTENT_TYPE_ROTO: &[u8] = b"application/grpc+roto";

/// Canonical base gRPC `content-type` header value.
pub const GRPC_CONTENT_TYPE: &[u8] = b"application/grpc";

// Common RFC 7541 Appendix A 1-based static table indices used by gRPC over HTTP/2.
pub const INDEX_AUTHORITY: usize = 1;
pub const INDEX_METHOD_GET: usize = 2;
pub const INDEX_METHOD_POST: usize = 3;
pub const INDEX_PATH_ROOT: usize = 4;
pub const INDEX_PATH_INDEX_HTML: usize = 5;
pub const INDEX_SCHEME_HTTP: usize = 6;
pub const INDEX_SCHEME_HTTPS: usize = 7;
pub const INDEX_STATUS_200: usize = 8;
pub const INDEX_STATUS_204: usize = 9;
pub const INDEX_STATUS_206: usize = 10;
pub const INDEX_STATUS_304: usize = 11;
pub const INDEX_STATUS_400: usize = 12;
pub const INDEX_STATUS_404: usize = 13;
pub const INDEX_STATUS_500: usize = 14;
pub const INDEX_ACCEPT_ENCODING: usize = 16;
pub const INDEX_CONTENT_ENCODING: usize = 26;
pub const INDEX_CONTENT_LENGTH: usize = 28;
pub const INDEX_CONTENT_TYPE: usize = 31;
pub const INDEX_USER_AGENT: usize = 58;

/// Errors that can occur while decoding an HPACK header block (RFC 7541).
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HpackError {
    /// Input buffer ended before a complete integer, string literal, or header representation.
    UnexpectedEof,
    /// A prefix integer exceeded `usize::MAX` or had an invalid prefix bit width.
    IntegerOverflow,
    /// A static or dynamic table index was `0` or exceeded the available table entries.
    InvalidIndex,
    /// A Huffman-encoded string contained invalid padding (`> 7` bits or non-`1` padding bits)
    /// or the 30-bit EOS symbol (`256`).
    InvalidHuffman,
    /// The caller-provided scratch or output buffer was too small.
    BufferOverflow,
    /// A dynamic table size update exceeded the maximum allowed table size (`SETTINGS_HEADER_TABLE_SIZE`).
    DynamicTableSizeExceeded,
    /// A dynamic table size update occurred after a header field representation in a header block (RFC 7541 §4.2).
    InvalidTableSizeUpdate,
    /// A gRPC status or message header was malformed.
    InvalidHeaderValue,
}

impl fmt::Display for HpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => f.write_str("unexpected end of HPACK input"),
            Self::IntegerOverflow => f.write_str("HPACK prefix integer overflow"),
            Self::InvalidIndex => f.write_str("invalid HPACK table index"),
            Self::InvalidHuffman => f.write_str("invalid HPACK canonical Huffman sequence"),
            Self::BufferOverflow => f.write_str("HPACK scratch buffer overflow"),
            Self::DynamicTableSizeExceeded => {
                f.write_str("HPACK dynamic table size update exceeds allowed maximum")
            }
            Self::InvalidTableSizeUpdate => {
                f.write_str("HPACK dynamic table size update after header emission")
            }
            Self::InvalidHeaderValue => f.write_str("invalid gRPC header value"),
        }
    }
}

impl core::error::Error for HpackError {}

impl From<HpackError> for GrpcStatus<'static> {
    #[inline]
    fn from(err: HpackError) -> Self {
        match err {
            HpackError::BufferOverflow => {
                Self::resource_exhausted("HPACK decode scratch buffer overflow")
            }
            _ => Self::internal("HPACK header decode error"),
        }
    }
}

// ============================================================================
// RFC 7541 §5.1 Prefix Integer Codec
// ============================================================================

/// Encodes `value` as an RFC 7541 §5.1 integer with `prefix_bits` (`1..=8`) and bitwise-ORs
/// `tag_mask` into the high bits of the first byte, writing into `out`.
///
/// Returns the number of bytes written (`>= 1`), or [`EncodeError::BufferOverflow`] if `out`
/// is too small or `prefix_bits` is outside `1..=8`.
pub fn encode_prefix_int(
    prefix_bits: u8,
    tag_mask: u8,
    value: usize,
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    if !(1..=8).contains(&prefix_bits) || out.is_empty() {
        return Err(EncodeError::BufferOverflow);
    }
    let max_prefix = ((1u16 << prefix_bits) - 1) as usize;
    let prefix_mask = max_prefix as u8;
    let tag = tag_mask & !prefix_mask;

    if value < max_prefix {
        out[0] = tag | (value as u8);
        return Ok(1);
    }

    out[0] = tag | prefix_mask;
    let mut rem = value - max_prefix;
    let mut pos = 1;
    while rem >= 128 {
        if pos >= out.len() {
            return Err(EncodeError::BufferOverflow);
        }
        out[pos] = ((rem & 0x7F) as u8) | 0x80;
        rem >>= 7;
        pos += 1;
    }
    if pos >= out.len() {
        return Err(EncodeError::BufferOverflow);
    }
    out[pos] = rem as u8;
    Ok(pos + 1)
}

/// Encodes `value` as an RFC 7541 §5.1 prefix integer onto `writer` in a single backward step.
///
/// If `writer` lacks capacity, [`EncodeError::BufferOverflow`] is latched onto `writer`.
pub fn encode_prefix_int_backward(
    prefix_bits: u8,
    tag_mask: u8,
    value: usize,
    writer: &mut BackwardWriter<'_>,
) {
    let mut tmp = [0u8; 16];
    match encode_prefix_int(prefix_bits, tag_mask, value, &mut tmp) {
        Ok(len) => {
            let _ = writer.write_bytes(&tmp[..len]);
        }
        Err(_) => {
            let rem = writer.remaining_capacity();
            for _ in 0..=rem {
                let _ = writer.write_bytes(&[0]);
            }
        }
    }
}

/// Decodes an RFC 7541 §5.1 prefix integer with `prefix_bits` (`1..=8`) from the start of `buf`.
///
/// Returns `Ok((value, bytes_consumed))`, [`HpackError::UnexpectedEof`] if `buf` ends prematurely,
/// or [`HpackError::IntegerOverflow`] if the encoded integer overflows `usize` or `prefix_bits`
/// is outside `1..=8`.
pub fn decode_prefix_int(prefix_bits: u8, buf: &[u8]) -> Result<(usize, usize), HpackError> {
    if !(1..=8).contains(&prefix_bits) {
        return Err(HpackError::IntegerOverflow);
    }
    let Some(&first) = buf.first() else {
        return Err(HpackError::UnexpectedEof);
    };
    let max_prefix = ((1u16 << prefix_bits) - 1) as usize;
    let mut value = (first as usize) & max_prefix;
    if value < max_prefix {
        return Ok((value, 1));
    }

    let mut shift: u32 = 0;
    let mut idx = 1;
    loop {
        if idx >= buf.len() {
            return Err(HpackError::UnexpectedEof);
        }
        let b = buf[idx];
        idx += 1;

        let low = (b & 0x7F) as usize;
        let Some(shifted) = low.checked_shl(shift) else {
            return Err(HpackError::IntegerOverflow);
        };
        if (shifted >> shift) != low {
            return Err(HpackError::IntegerOverflow);
        }
        value = value
            .checked_add(shifted)
            .ok_or(HpackError::IntegerOverflow)?;

        if (b & 0x80) == 0 {
            return Ok((value, idx));
        }
        shift = shift.saturating_add(7);
    }
}

// ============================================================================
// RFC 7541 Appendix A Static Table (61 Entries)
// ============================================================================

/// Complete 61-entry RFC 7541 Appendix A static table stored in ROM/flash (`1`-indexed via [`static_table_get`]).
pub static STATIC_TABLE: [(&[u8], &[u8]); STATIC_TABLE_LEN] = [
    (b":authority", b""),                   // 1
    (b":method", b"GET"),                   // 2
    (b":method", b"POST"),                  // 3
    (b":path", b"/"),                       // 4
    (b":path", b"/index.html"),             // 5
    (b":scheme", b"http"),                  // 6
    (b":scheme", b"https"),                 // 7
    (b":status", b"200"),                   // 8
    (b":status", b"204"),                   // 9
    (b":status", b"206"),                   // 10
    (b":status", b"304"),                   // 11
    (b":status", b"400"),                   // 12
    (b":status", b"404"),                   // 14 - 1 = 13
    (b":status", b"500"),                   // 14
    (b"accept-charset", b""),               // 15
    (b"accept-encoding", b"gzip, deflate"), // 16
    (b"accept-language", b""),              // 17
    (b"accept-ranges", b""),                // 18
    (b"accept", b""),                       // 19
    (b"access-control-allow-origin", b""),  // 20
    (b"age", b""),                          // 21
    (b"allow", b""),                        // 22
    (b"authorization", b""),                // 23
    (b"cache-control", b""),                // 24
    (b"content-disposition", b""),          // 25
    (b"content-encoding", b""),             // 26
    (b"content-language", b""),             // 27
    (b"content-length", b""),               // 28
    (b"content-location", b""),             // 29
    (b"content-range", b""),                // 30
    (b"content-type", b""),                 // 31
    (b"cookie", b""),                       // 32
    (b"date", b""),                         // 33
    (b"etag", b""),                         // 34
    (b"expect", b""),                       // 35
    (b"expires", b""),                      // 36
    (b"from", b""),                         // 37
    (b"host", b""),                         // 38
    (b"if-match", b""),                     // 39
    (b"if-modified-since", b""),            // 40
    (b"if-none-match", b""),                // 41
    (b"if-range", b""),                     // 42
    (b"if-unmodified-since", b""),          // 43
    (b"last-modified", b""),                // 44
    (b"link", b""),                         // 45
    (b"location", b""),                     // 46
    (b"max-forwards", b""),                 // 47
    (b"proxy-authenticate", b""),           // 48
    (b"proxy-authorization", b""),          // 49
    (b"range", b""),                        // 50
    (b"referer", b""),                      // 51
    (b"refresh", b""),                      // 52
    (b"retry-after", b""),                  // 53
    (b"server", b""),                       // 54
    (b"set-cookie", b""),                   // 55
    (b"strict-transport-security", b""),    // 56
    (b"transfer-encoding", b""),            // 57
    (b"user-agent", b""),                   // 58
    (b"vary", b""),                         // 59
    (b"via", b""),                          // 60
    (b"www-authenticate", b""),             // 61
];

/// Looks up a 1-based index (`1..=61`) in the RFC 7541 Appendix A static table.
#[inline]
pub const fn static_table_get(index: usize) -> Option<(&'static [u8], &'static [u8])> {
    if index >= 1 && index <= STATIC_TABLE_LEN {
        Some(STATIC_TABLE[index - 1])
    } else {
        None
    }
}

// ============================================================================
// RFC 7541 Appendix B Canonical Huffman Table & Codec
// ============================================================================

/// Canonical Huffman bit lengths for symbols `0..=255` and `EOS (256)` from RFC 7541 Appendix B.
pub static HUFFMAN_BIT_LENS: [u8; 257] = [
    // 0..=31
    13, 23, 28, 28, 28, 28, 28, 28, 28, 24, 30, 28, 28, 30, 28, 28, 28, 28, 28, 28, 28, 28, 30, 28,
    28, 28, 28, 28, 28, 28, 28, 28, // 32..=63
    6, 10, 10, 12, 13, 6, 8, 11, 10, 10, 8, 11, 8, 6, 6, 6, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 7, 8, 15,
    6, 12, 10, // 64..=95
    13, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 8, 7, 8, 13, 19, 13,
    14, 6, // 96..=127
    15, 5, 6, 5, 6, 5, 6, 6, 6, 5, 7, 7, 6, 6, 6, 5, 6, 7, 6, 5, 5, 6, 7, 7, 7, 7, 7, 15, 11, 14,
    13, 28, // 128..=159
    20, 22, 20, 20, 22, 22, 22, 23, 22, 23, 23, 23, 23, 23, 24, 23, 24, 24, 22, 23, 24, 23, 23, 23,
    23, 21, 22, 23, 22, 23, 23, 24, // 160..=191
    22, 21, 20, 22, 22, 23, 23, 21, 23, 22, 22, 24, 21, 22, 23, 23, 21, 21, 22, 21, 23, 22, 23, 23,
    20, 22, 22, 22, 23, 22, 22, 23, // 192..=223
    26, 26, 20, 19, 22, 23, 22, 25, 26, 26, 26, 27, 27, 26, 24, 25, 19, 21, 26, 27, 27, 26, 27, 24,
    21, 21, 26, 26, 28, 27, 27, 27, // 224..=256 (256 is EOS = 30)
    20, 24, 20, 21, 22, 21, 21, 23, 22, 22, 25, 25, 24, 24, 26, 23, 26, 27, 26, 26, 27, 27, 27, 27,
    27, 28, 27, 27, 27, 27, 27, 26, 30,
];

const fn build_huffman_codes() -> [(u32, u8); 257] {
    let mut table = [(0u32, 0u8); 257];
    let mut code: u32 = 0;
    let mut prev_len: u8 = 5;
    let mut len: u8 = 5;
    while len <= 30 {
        let mut sym = 0usize;
        while sym < 257 {
            if HUFFMAN_BIT_LENS[sym] == len {
                code <<= len - prev_len;
                table[sym] = (code, len);
                code += 1;
                prev_len = len;
            }
            sym += 1;
        }
        len += 1;
    }
    table
}

/// Complete RFC 7541 Appendix B canonical Huffman code table `(msb_aligned_code, bit_len)` for
/// symbols `0..=255` and `EOS (256)`.
pub static HUFFMAN_CODES: [(u32, u8); 257] = build_huffman_codes();

// Compile-time verification that the canonical table ends with EOS = (0x3fff_ffff, 30),
// proving Kraft's inequality holds with exact equality across all 257 symbols.
const _: () = assert!(HUFFMAN_CODES[256].0 == 0x3fff_ffff && HUFFMAN_CODES[256].1 == 30);

const HUFFMAN_LEAF_BIT: u16 = 0x8000;
const HUFFMAN_EOS_SYM: u16 = 256;

const fn build_huffman_trie() -> [(u16, u16); 256] {
    let mut nodes = [(u16::MAX, u16::MAX); 256];
    let mut next_node = 1u16;
    let mut sym = 0usize;
    while sym < 257 {
        let (code, len) = HUFFMAN_CODES[sym];
        let mut curr = 0usize;
        let mut bit_idx = len;
        while bit_idx > 1 {
            bit_idx -= 1;
            let bit = ((code >> bit_idx) & 1) as u16;
            let child = if bit == 0 {
                nodes[curr].0
            } else {
                nodes[curr].1
            };
            let next = if child == u16::MAX {
                let allocated = next_node;
                next_node += 1;
                if bit == 0 {
                    nodes[curr].0 = allocated;
                } else {
                    nodes[curr].1 = allocated;
                }
                allocated
            } else {
                child
            };
            curr = next as usize;
        }
        let last_bit = (code & 1) as u16;
        let leaf = HUFFMAN_LEAF_BIT | (sym as u16);
        if last_bit == 0 {
            nodes[curr].0 = leaf;
        } else {
            nodes[curr].1 = leaf;
        }
        sym += 1;
    }
    assert!(next_node == 256);
    nodes
}

/// Compact 1 KiB binary trie in ROM/flash for decoding RFC 7541 Appendix B Huffman strings.
static HUFFMAN_TRIE: [(u16, u16); 256] = build_huffman_trie();

/// Computes the exact encoded byte length of `src` when compressed with RFC 7541 Appendix B
/// canonical Huffman coding (including any trailing `1`-bit padding to an octet boundary).
#[inline]
pub const fn huffman_encoded_len(src: &[u8]) -> usize {
    let mut i = 0;
    let mut total_bits = 0usize;
    while i < src.len() {
        total_bits += HUFFMAN_BIT_LENS[src[i] as usize] as usize;
        i += 1;
    }
    total_bits.div_ceil(8)
}

/// Encodes `src` using the RFC 7541 Appendix B canonical Huffman code into `out`, padding any
/// final partial octet with `1` bits (the MSB prefix of EOS).
///
/// Returns the number of bytes written to `out`, or [`EncodeError::BufferOverflow`] if `out`
/// is too small.
pub fn huffman_encode(src: &[u8], out: &mut [u8]) -> Result<usize, EncodeError> {
    let mut acc: u64 = 0;
    let mut bits: u8 = 0;
    let mut write_pos = 0;

    for &b in src {
        let (code, len) = HUFFMAN_CODES[b as usize];
        acc = (acc << len) | (code as u64);
        bits += len;
        while bits >= 8 {
            bits -= 8;
            if write_pos >= out.len() {
                return Err(EncodeError::BufferOverflow);
            }
            out[write_pos] = (acc >> bits) as u8;
            write_pos += 1;
        }
    }

    if bits > 0 {
        if write_pos >= out.len() {
            return Err(EncodeError::BufferOverflow);
        }
        let pad = 8 - bits;
        let pad_mask = (1u64 << pad) - 1;
        out[write_pos] = ((acc << pad) | pad_mask) as u8;
        write_pos += 1;
    }

    Ok(write_pos)
}

/// Decodes an RFC 7541 Appendix B canonical Huffman-encoded byte slice `src` into `out`,
/// returning the decoded slice `&'b [u8]`.
///
/// Strictly enforces RFC 7541 §5.2:
/// - Any trailing padding after the last symbol must be `<= 7` bits and consist solely of `1` bits.
/// - Any appearance of the 30-bit EOS symbol (`256`) inside `src` returns [`HpackError::InvalidHuffman`].
/// - Returns [`HpackError::BufferOverflow`] if `out` is too small for the decoded output.
pub fn huffman_decode<'b>(src: &[u8], out: &'b mut [u8]) -> Result<&'b [u8], HpackError> {
    let mut node = 0usize;
    let mut current_bits = 0u8;
    let mut all_ones = true;
    let mut write_pos = 0usize;

    for &byte in src {
        let mut shift = 8u8;
        while shift > 0 {
            shift -= 1;
            let bit = (byte >> shift) & 1;
            current_bits += 1;
            let next = if bit == 0 {
                all_ones = false;
                HUFFMAN_TRIE[node].0
            } else {
                HUFFMAN_TRIE[node].1
            };

            if (next & HUFFMAN_LEAF_BIT) != 0 {
                let sym = next & !HUFFMAN_LEAF_BIT;
                if sym == HUFFMAN_EOS_SYM {
                    return Err(HpackError::InvalidHuffman);
                }
                if write_pos >= out.len() {
                    return Err(HpackError::BufferOverflow);
                }
                out[write_pos] = sym as u8;
                write_pos += 1;
                node = 0;
                current_bits = 0;
                all_ones = true;
            } else {
                node = next as usize;
            }
        }
    }

    if current_bits > 7 || !all_ones {
        return Err(HpackError::InvalidHuffman);
    }

    Ok(&out[..write_pos])
}

// ============================================================================
// RFC 7541 §4 Fixed-Capacity Zero-Allocation Dynamic Table
// ============================================================================

#[derive(Clone, Copy, Debug, Default)]
struct DynEntryMeta {
    offset: usize,
    name_len: usize,
    val_len: usize,
}

impl DynEntryMeta {
    #[inline]
    const fn rfc_size(&self) -> usize {
        self.name_len + self.val_len + DYNAMIC_ENTRY_OVERHEAD
    }

    #[inline]
    const fn raw_len(&self) -> usize {
        self.name_len + self.val_len
    }
}

/// Maximum number of dynamic table metadata slots tracked by [`DynamicTable`].
///
/// Since every entry has an RFC 7541 §4.1 overhead of `32` bytes, `64` slots can track up to
/// `64 * 32 = 2048` bytes of pure overhead (enough for any table up to `2 KiB` at minimum entry
/// size, or larger tables with realistic header lengths).
const MAX_DYNAMIC_ENTRIES: usize = 64;

/// Fixed-capacity, zero-allocation HPACK dynamic table (RFC 7541 §4).
///
/// Stores header names and values contiguously inside an inline `[u8; CAP]` array so every
/// lookup returns direct borrowed slices `(&[u8], &[u8])` with zero heap allocation.
#[derive(Clone, Debug)]
pub struct DynamicTable<const CAP: usize = DEFAULT_DYNAMIC_TABLE_CAP> {
    storage: [u8; CAP],
    entries: [DynEntryMeta; MAX_DYNAMIC_ENTRIES],
    entry_count: usize,
    used_raw_bytes: usize,
    current_rfc_size: usize,
    max_rfc_size: usize,
}

impl<const CAP: usize> Default for DynamicTable<CAP> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP: usize> DynamicTable<CAP> {
    /// Creates an empty [`DynamicTable`] with `max_size` initialized to `CAP`.
    #[inline]
    pub const fn new() -> Self {
        Self::with_max_size(CAP)
    }

    /// Creates an empty [`DynamicTable`] with the given RFC 7541 maximum table size.
    #[inline]
    pub const fn with_max_size(max_size: usize) -> Self {
        Self {
            storage: [0u8; CAP],
            entries: [DynEntryMeta {
                offset: 0,
                name_len: 0,
                val_len: 0,
            }; MAX_DYNAMIC_ENTRIES],
            entry_count: 0,
            used_raw_bytes: 0,
            current_rfc_size: 0,
            max_rfc_size: max_size,
        }
    }

    /// Returns the number of entries currently in the dynamic table.
    #[inline]
    pub const fn len(&self) -> usize {
        self.entry_count
    }

    /// Returns `true` if the dynamic table contains no entries.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.entry_count == 0
    }

    /// Returns the current RFC 7541 §4.1 size of the dynamic table in octets
    /// (sum of `name.len() + value.len() + 32` for all entries).
    #[inline]
    pub const fn size(&self) -> usize {
        self.current_rfc_size
    }

    /// Returns the configured maximum RFC 7541 dynamic table size in octets.
    #[inline]
    pub const fn max_size(&self) -> usize {
        self.max_rfc_size
    }

    /// Clears all entries from the dynamic table.
    #[inline]
    pub fn clear(&mut self) {
        self.entry_count = 0;
        self.used_raw_bytes = 0;
        self.current_rfc_size = 0;
    }

    /// Updates the maximum RFC 7541 dynamic table size, evicting oldest entries as needed
    /// per RFC 7541 §4.3.
    pub fn set_max_size(&mut self, new_max_size: usize) {
        self.max_rfc_size = new_max_size;
        while self.current_rfc_size > self.max_rfc_size && self.entry_count > 0 {
            self.evict_oldest();
        }
    }

    /// Looks up a 0-based dynamic index (`0` is the most recently inserted entry, corresponding
    /// to HPACK index `62`).
    #[inline]
    pub fn get(&self, dyn_index: usize) -> Option<(&[u8], &[u8])> {
        if dyn_index >= self.entry_count {
            return None;
        }
        //Newest entry is at `entries[entry_count - 1 - dyn_index]`.
        let meta = self.entries[self.entry_count - 1 - dyn_index];
        let name_end = meta.offset + meta.name_len;
        let val_end = name_end + meta.val_len;
        Some((
            &self.storage[meta.offset..name_end],
            &self.storage[name_end..val_end],
        ))
    }

    /// Inserts `(name, value)` as the newest entry in the dynamic table following RFC 7541 §4.4.
    ///
    /// If the entry's RFC size (`name.len() + value.len() + 32`) exceeds `max_size` (or if
    /// `name.len() + value.len() > CAP`), all existing entries are evicted and the table is
    /// left empty per RFC 7541 §4.4.
    pub fn insert(&mut self, name: &[u8], value: &[u8]) {
        let raw_len = name.len().saturating_add(value.len());
        let entry_rfc_size = raw_len.saturating_add(DYNAMIC_ENTRY_OVERHEAD);

        if entry_rfc_size > self.max_rfc_size || raw_len > CAP {
            self.clear();
            return;
        }

        while (self.current_rfc_size + entry_rfc_size > self.max_rfc_size
            || self.used_raw_bytes + raw_len > CAP
            || self.entry_count >= MAX_DYNAMIC_ENTRIES)
            && self.entry_count > 0
        {
            self.evict_oldest();
        }

        let offset = self.used_raw_bytes;
        self.storage[offset..offset + name.len()].copy_from_slice(name);
        self.storage[offset + name.len()..offset + raw_len].copy_from_slice(value);

        self.entries[self.entry_count] = DynEntryMeta {
            offset,
            name_len: name.len(),
            val_len: value.len(),
        };
        self.entry_count += 1;
        self.used_raw_bytes += raw_len;
        self.current_rfc_size += entry_rfc_size;
    }

    fn evict_oldest(&mut self) {
        if self.entry_count == 0 {
            return;
        }
        let oldest = self.entries[0];
        let drop_raw = oldest.raw_len();
        let drop_rfc = oldest.rfc_size();

        if self.entry_count == 1 {
            self.clear();
            return;
        }

        self.storage.copy_within(drop_raw..self.used_raw_bytes, 0);
        for i in 1..self.entry_count {
            let mut m = self.entries[i];
            m.offset -= drop_raw;
            self.entries[i - 1] = m;
        }
        self.entry_count -= 1;
        self.used_raw_bytes -= drop_raw;
        self.current_rfc_size -= drop_rfc;
    }
}

// ============================================================================
// Zero-Allocation HPACK Decoder (RFC 7541 §6)
// ============================================================================

/// A decoded HTTP/2 header field `(name, value)` borrowing directly from the static table,
/// the input header block, or the caller-supplied scratch buffer.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeaderField<'a> {
    pub name: &'a [u8],
    pub value: &'a [u8],
}

impl<'a> HeaderField<'a> {
    /// Creates a new [`HeaderField`] from borrowed `name` and `value` byte slices.
    #[inline]
    pub const fn new(name: &'a [u8], value: &'a [u8]) -> Self {
        Self { name, value }
    }

    /// Returns `true` if `self.name` is an HTTP/2 pseudo-header (starts with `b':'`).
    #[inline]
    pub const fn is_pseudo(&self) -> bool {
        matches!(self.name, [b':', ..])
    }

    /// Attempts to view `self.name` as a UTF-8 `&str`.
    #[inline]
    pub const fn name_str(&self) -> Option<&'a str> {
        match core::str::from_utf8(self.name) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }

    /// Attempts to view `self.value` as a UTF-8 `&str`.
    #[inline]
    pub const fn value_str(&self) -> Option<&'a str> {
        match core::str::from_utf8(self.value) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }
}

/// Zero-allocation HPACK decoder (RFC 7541) with an inline fixed-capacity dynamic table.
///
/// Supports both stateless zero-table mode (`max_allowed_table_size == 0`, matching
/// `SETTINGS_HEADER_TABLE_SIZE = 0`) and stateful dynamic table decoding up to `DYN_CAP` bytes.
#[derive(Clone, Debug)]
pub struct HpackDecoder<const DYN_CAP: usize = DEFAULT_DYNAMIC_TABLE_CAP> {
    dynamic_table: DynamicTable<DYN_CAP>,
    max_allowed_table_size: usize,
}

impl<const DYN_CAP: usize> Default for HpackDecoder<DYN_CAP> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const DYN_CAP: usize> HpackDecoder<DYN_CAP> {
    /// Creates a new [`HpackDecoder`] with `max_allowed_table_size` set to `DYN_CAP`.
    #[inline]
    pub const fn new() -> Self {
        Self::with_max_table_size(DYN_CAP)
    }

    /// Creates a stateless [`HpackDecoder`] with `max_allowed_table_size = 0`
    /// (for connections advertising `SETTINGS_HEADER_TABLE_SIZE = 0`).
    #[inline]
    pub const fn stateless() -> Self {
        Self::with_max_table_size(0)
    }

    /// Creates a new [`HpackDecoder`] with the specified maximum allowed dynamic table size.
    #[inline]
    pub const fn with_max_table_size(max_allowed_table_size: usize) -> Self {
        Self {
            dynamic_table: DynamicTable::with_max_size(max_allowed_table_size),
            max_allowed_table_size,
        }
    }

    /// Returns a shared reference to the underlying [`DynamicTable`].
    #[inline]
    pub const fn dynamic_table(&self) -> &DynamicTable<DYN_CAP> {
        &self.dynamic_table
    }

    /// Returns the current RFC 7541 dynamic table size in octets.
    #[inline]
    pub const fn dynamic_table_size(&self) -> usize {
        self.dynamic_table.size()
    }

    /// Returns the configured maximum allowed dynamic table size (`SETTINGS_HEADER_TABLE_SIZE`).
    #[inline]
    pub const fn max_allowed_table_size(&self) -> usize {
        self.max_allowed_table_size
    }

    /// Updates the maximum allowed dynamic table size (`SETTINGS_HEADER_TABLE_SIZE`) and
    /// immediately clamps the active dynamic table if needed.
    #[inline]
    pub fn set_max_allowed_table_size(&mut self, max_allowed: usize) {
        self.max_allowed_table_size = max_allowed;
        if self.dynamic_table.max_size() > max_allowed {
            self.dynamic_table.set_max_size(max_allowed);
        }
    }

    /// Decodes an HPACK header block `block`, invoking `on_header` for each decoded [`HeaderField`].
    ///
    /// - Static table strings borrow `'static`.
    /// - Unencoded string literals borrow directly from `block` with zero copying.
    /// - Huffman-encoded string literals and dynamic table lookups are written into `scratch`
    ///   so every yielded [`HeaderField<'a>`] has the full `'a` lifetime of `block` and `scratch`.
    pub fn decode<'a, F>(
        &mut self,
        block: &'a [u8],
        scratch: &'a mut [u8],
        mut on_header: F,
    ) -> Result<(), HpackError>
    where
        F: FnMut(HeaderField<'a>) -> Result<(), HpackError>,
    {
        let mut cursor = 0usize;
        let mut scratch_rem: &'a mut [u8] = scratch;
        let mut seen_header = false;

        while cursor < block.len() {
            let first = block[cursor];

            if (first & 0x80) != 0 {
                // §6.1 Indexed Header Field Representation (prefix = 7 bits: 1xxxxxxx)
                seen_header = true;
                let (index, consumed) = decode_prefix_int(7, &block[cursor..])?;
                cursor += consumed;
                let (name, value) = self.resolve_indexed_pair(index, &mut scratch_rem)?;
                on_header(HeaderField { name, value })?;
            } else if (first & 0xE0) == 0x20 {
                // §6.3 Dynamic Table Size Update (prefix = 5 bits: 001xxxxx)
                if seen_header {
                    return Err(HpackError::InvalidTableSizeUpdate);
                }
                let (new_size, consumed) = decode_prefix_int(5, &block[cursor..])?;
                cursor += consumed;
                if new_size > self.max_allowed_table_size {
                    return Err(HpackError::DynamicTableSizeExceeded);
                }
                self.dynamic_table.set_max_size(new_size);
            } else {
                // §6.2.1 Literal Header Field with Incremental Indexing (01xxxxxx, 6-bit prefix)
                // §6.2.2 Literal Header Field without Indexing (0000xxxx, 4-bit prefix)
                // §6.2.3 Literal Header Field Never Indexed (0001xxxx, 4-bit prefix)
                seen_header = true;
                let incremental = (first & 0xC0) == 0x40;
                let prefix_bits = if incremental { 6 } else { 4 };
                let (name_index, consumed) = decode_prefix_int(prefix_bits, &block[cursor..])?;
                cursor += consumed;
                let name = if name_index == 0 {
                    let (n, c) = decode_string_literal(&block[cursor..], &mut scratch_rem)?;
                    cursor += c;
                    n
                } else {
                    self.resolve_indexed_name(name_index, &mut scratch_rem)?
                };
                let (value, val_consumed) =
                    decode_string_literal(&block[cursor..], &mut scratch_rem)?;
                cursor += val_consumed;
                if incremental {
                    self.dynamic_table.insert(name, value);
                }
                on_header(HeaderField { name, value })?;
            }
        }

        Ok(())
    }

    fn resolve_indexed_pair<'a>(
        &self,
        index: usize,
        scratch: &mut &'a mut [u8],
    ) -> Result<(&'a [u8], &'a [u8]), HpackError> {
        if index == 0 {
            return Err(HpackError::InvalidIndex);
        }
        if let Some((name, val)) = static_table_get(index) {
            return Ok((name, val));
        }
        let dyn_idx = index - STATIC_TABLE_LEN - 1;
        let Some((name, val)) = self.dynamic_table.get(dyn_idx) else {
            return Err(HpackError::InvalidIndex);
        };
        let name_copied = copy_to_scratch(name, scratch)?;
        let val_copied = copy_to_scratch(val, scratch)?;
        Ok((name_copied, val_copied))
    }

    fn resolve_indexed_name<'a>(
        &self,
        index: usize,
        scratch: &mut &'a mut [u8],
    ) -> Result<&'a [u8], HpackError> {
        if index == 0 {
            return Err(HpackError::InvalidIndex);
        }
        if let Some((name, _)) = static_table_get(index) {
            return Ok(name);
        }
        let dyn_idx = index - STATIC_TABLE_LEN - 1;
        let Some((name, _)) = self.dynamic_table.get(dyn_idx) else {
            return Err(HpackError::InvalidIndex);
        };
        copy_to_scratch(name, scratch)
    }
}

/// Stateless helper that decodes an HPACK header block with `SETTINGS_HEADER_TABLE_SIZE = 0`.
#[inline]
pub fn decode_header_block<'a, F>(
    block: &'a [u8],
    scratch: &'a mut [u8],
    on_header: F,
) -> Result<(), HpackError>
where
    F: FnMut(HeaderField<'a>) -> Result<(), HpackError>,
{
    let mut decoder = HpackDecoder::<0>::stateless();
    decoder.decode(block, scratch, on_header)
}

fn copy_to_scratch<'a>(src: &[u8], scratch: &mut &'a mut [u8]) -> Result<&'a [u8], HpackError> {
    if src.len() > scratch.len() {
        return Err(HpackError::BufferOverflow);
    }
    let (head, tail) = core::mem::take(scratch).split_at_mut(src.len());
    head.copy_from_slice(src);
    *scratch = tail;
    Ok(head)
}

/// Decodes an RFC 7541 §5.2 string literal from the start of `buf`, returning
/// `Ok((decoded_slice, bytes_consumed))`.
///
/// Unencoded strings borrow directly from `buf`; Huffman-encoded strings (`H = 1`) are decoded
/// into `scratch`.
pub fn decode_string_literal<'a>(
    buf: &'a [u8],
    scratch: &mut &'a mut [u8],
) -> Result<(&'a [u8], usize), HpackError> {
    let Some(&first) = buf.first() else {
        return Err(HpackError::UnexpectedEof);
    };
    let is_huffman = (first & 0x80) != 0;
    let (str_len, int_consumed) = decode_prefix_int(7, buf)?;
    let total_consumed = int_consumed
        .checked_add(str_len)
        .ok_or(HpackError::IntegerOverflow)?;
    if buf.len() < total_consumed {
        return Err(HpackError::UnexpectedEof);
    }
    let raw = &buf[int_consumed..total_consumed];
    if !is_huffman {
        return Ok((raw, total_consumed));
    }

    let decoded_len = huffman_decode(raw, scratch)?.len();
    let (head, tail) = core::mem::take(scratch).split_at_mut(decoded_len);
    *scratch = tail;
    Ok((head, total_consumed))
}

// ============================================================================
// High-Level gRPC Header Extractors
// ============================================================================

/// Returns `true` if `content_type` is a valid gRPC `content-type` header value
/// (`"application/grpc"` or `"application/grpc+..."` or `"application/grpc;..."`).
#[inline]
pub fn is_grpc_content_type(content_type: &[u8]) -> bool {
    if !content_type.starts_with(GRPC_CONTENT_TYPE) {
        return false;
    }
    matches!(
        content_type.get(GRPC_CONTENT_TYPE.len()),
        None | Some(b'+' | b';')
    )
}

/// Extracted gRPC HTTP/2 request headers from an inbound `HEADERS` block.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GrpcRequestHeaders<'a> {
    pub method: Option<&'a [u8]>,
    pub scheme: Option<&'a [u8]>,
    pub authority: Option<&'a [u8]>,
    pub path: Option<&'a [u8]>,
    pub content_type: Option<&'a [u8]>,
    pub te: Option<&'a [u8]>,
    pub grpc_encoding: Option<&'a [u8]>,
    pub grpc_accept_encoding: Option<&'a [u8]>,
    pub grpc_timeout: Option<&'a [u8]>,
    pub user_agent: Option<&'a [u8]>,
}

impl<'a> GrpcRequestHeaders<'a> {
    /// Decodes an HPACK header block into [`GrpcRequestHeaders`] using `decoder`.
    pub fn decode<const DYN_CAP: usize>(
        decoder: &mut HpackDecoder<DYN_CAP>,
        block: &'a [u8],
        scratch: &'a mut [u8],
    ) -> Result<Self, HpackError> {
        let mut headers = Self::default();
        decoder.decode(block, scratch, |field| {
            headers.on_header(field);
            Ok(())
        })?;
        Ok(headers)
    }

    /// Decodes an HPACK header block into [`GrpcRequestHeaders`] and stores custom metadata headers
    /// in `metadata`.
    pub fn decode_with_metadata<const DYN_CAP: usize, M: MetadataMap + ?Sized>(
        decoder: &mut HpackDecoder<DYN_CAP>,
        block: &'a [u8],
        scratch: &'a mut [u8],
        metadata: &mut M,
    ) -> Result<Self, HpackError> {
        let mut headers = Self::default();
        decoder.decode(block, scratch, |field| {
            headers.on_header_with_metadata(field, metadata)
        })?;
        Ok(headers)
    }

    /// Updates this [`GrpcRequestHeaders`] with a single [`HeaderField`].
    #[inline]
    pub fn on_header(&mut self, field: HeaderField<'a>) {
        match field.name {
            b":method" => self.method = Some(field.value),
            b":scheme" => self.scheme = Some(field.value),
            b":authority" => self.authority = Some(field.value),
            b":path" => self.path = Some(field.value),
            b"content-type" => self.content_type = Some(field.value),
            b"te" => self.te = Some(field.value),
            b"grpc-encoding" => self.grpc_encoding = Some(field.value),
            b"grpc-accept-encoding" => self.grpc_accept_encoding = Some(field.value),
            b"grpc-timeout" => self.grpc_timeout = Some(field.value),
            b"user-agent" => self.user_agent = Some(field.value),
            _ => {}
        }
    }

    /// Updates this [`GrpcRequestHeaders`] with a single [`HeaderField`], pushing any custom
    /// non-pseudo metadata headers into `metadata`.
    #[inline]
    pub fn on_header_with_metadata<M: MetadataMap + ?Sized>(
        &mut self,
        field: HeaderField<'a>,
        metadata: &mut M,
    ) -> Result<(), HpackError> {
        match field.name {
            b":method" => self.method = Some(field.value),
            b":scheme" => self.scheme = Some(field.value),
            b":authority" => self.authority = Some(field.value),
            b":path" => self.path = Some(field.value),
            b"content-type" => self.content_type = Some(field.value),
            b"te" => self.te = Some(field.value),
            b"grpc-encoding" => self.grpc_encoding = Some(field.value),
            b"grpc-accept-encoding" => self.grpc_accept_encoding = Some(field.value),
            b"grpc-timeout" => self.grpc_timeout = Some(field.value),
            b"user-agent" => self.user_agent = Some(field.value),
            name if name.iter().any(u8::is_ascii_uppercase) => {
                return Err(HpackError::InvalidHeaderValue);
            }
            name if !name.starts_with(b":") && !is_reserved_grpc_header(name) => {
                metadata.push_wire(name, field.value)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Returns `true` if `:method == "POST"`, `:path` starts with `'/'`, `te == "trailers"`,
    /// and `content-type` is a valid gRPC content type.
    #[inline]
    pub fn is_valid_grpc_post(&self) -> bool {
        self.method == Some(b"POST")
            && self.path.is_some_and(|p| p.starts_with(b"/"))
            && self.te == Some(b"trailers")
            && self.content_type.is_some_and(is_grpc_content_type)
    }

    /// Returns `:path` as a UTF-8 `&str` if present and valid UTF-8.
    #[inline]
    pub const fn path_str(&self) -> Option<&'a str> {
        let Some(p) = self.path else {
            return None;
        };
        match core::str::from_utf8(p) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }
}

/// Extracted gRPC HTTP/2 response headers (including optional Trailers-Only fields).
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GrpcResponseHeaders<'a> {
    pub status: Option<&'a [u8]>,
    pub content_type: Option<&'a [u8]>,
    pub grpc_encoding: Option<&'a [u8]>,
    pub grpc_accept_encoding: Option<&'a [u8]>,
    pub grpc_status: Option<GrpcCode>,
    pub grpc_message: Option<&'a [u8]>,
}

impl<'a> GrpcResponseHeaders<'a> {
    /// Decodes an HPACK header block into [`GrpcResponseHeaders`] using `decoder`.
    pub fn decode<const DYN_CAP: usize>(
        decoder: &mut HpackDecoder<DYN_CAP>,
        block: &'a [u8],
        scratch: &'a mut [u8],
    ) -> Result<Self, HpackError> {
        let mut headers = Self::default();
        decoder.decode(block, scratch, |field| headers.on_header(field))?;
        Ok(headers)
    }

    /// Decodes an HPACK header block into [`GrpcResponseHeaders`] and stores custom metadata
    /// headers in `metadata`.
    pub fn decode_with_metadata<const DYN_CAP: usize, M: MetadataMap + ?Sized>(
        decoder: &mut HpackDecoder<DYN_CAP>,
        block: &'a [u8],
        scratch: &'a mut [u8],
        metadata: &mut M,
    ) -> Result<Self, HpackError> {
        let mut headers = Self::default();
        decoder.decode(block, scratch, |field| {
            headers.on_header_with_metadata(field, metadata)
        })?;
        Ok(headers)
    }

    /// Updates this [`GrpcResponseHeaders`] with a single [`HeaderField`].
    #[inline]
    pub fn on_header(&mut self, field: HeaderField<'a>) -> Result<(), HpackError> {
        match field.name {
            b":status" => self.status = Some(field.value),
            b"content-type" => self.content_type = Some(field.value),
            b"grpc-encoding" => self.grpc_encoding = Some(field.value),
            b"grpc-accept-encoding" => self.grpc_accept_encoding = Some(field.value),
            b"grpc-status" => {
                self.grpc_status = Some(parse_grpc_status_header(field.value)?);
            }
            b"grpc-message" => self.grpc_message = Some(field.value),
            _ => {}
        }
        Ok(())
    }

    /// Updates this [`GrpcResponseHeaders`] with a single [`HeaderField`], pushing any custom
    /// non-pseudo metadata headers into `metadata`.
    #[inline]
    pub fn on_header_with_metadata<M: MetadataMap + ?Sized>(
        &mut self,
        field: HeaderField<'a>,
        metadata: &mut M,
    ) -> Result<(), HpackError> {
        match field.name {
            b":status" => self.status = Some(field.value),
            b"content-type" => self.content_type = Some(field.value),
            b"grpc-encoding" => self.grpc_encoding = Some(field.value),
            b"grpc-accept-encoding" => self.grpc_accept_encoding = Some(field.value),
            b"grpc-status" => {
                self.grpc_status = Some(parse_grpc_status_header(field.value)?);
            }
            b"grpc-message" => self.grpc_message = Some(field.value),
            name if name.iter().any(u8::is_ascii_uppercase) => {
                return Err(HpackError::InvalidHeaderValue);
            }
            name if !name.starts_with(b":") && !is_reserved_grpc_header(name) => {
                metadata.push_wire(name, field.value)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Returns `true` if this initial response header block is a gRPC Trailers-Only response
    /// (carries `grpc-status`).
    #[inline]
    pub const fn is_trailers_only(&self) -> bool {
        self.grpc_status.is_some()
    }

    /// If `grpc-status` is present, reconstructs a [`GrpcStatus`] by percent-decoding
    /// `grpc-message` into `msg_scratch` (or borrowing in-place if unescaped).
    pub fn to_grpc_status<'m>(
        &self,
        msg_scratch: &'m mut [u8],
    ) -> Result<Option<GrpcStatus<'m>>, HpackError>
    where
        'a: 'm,
    {
        let Some(code) = self.grpc_status else {
            return Ok(None);
        };
        let msg = decode_grpc_message_field(self.grpc_message, msg_scratch)?;
        Ok(Some(GrpcStatus::new(code, msg)))
    }
}

/// Extracted gRPC HTTP/2 response trailers (or Trailers-Only response block).
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GrpcTrailers<'a> {
    pub status: Option<&'a [u8]>,
    pub content_type: Option<&'a [u8]>,
    pub grpc_status: Option<GrpcCode>,
    pub grpc_message: Option<&'a [u8]>,
}

impl<'a> GrpcTrailers<'a> {
    /// Decodes an HPACK header block into [`GrpcTrailers`] using `decoder`.
    pub fn decode<const DYN_CAP: usize>(
        decoder: &mut HpackDecoder<DYN_CAP>,
        block: &'a [u8],
        scratch: &'a mut [u8],
    ) -> Result<Self, HpackError> {
        let mut trailers = Self::default();
        decoder.decode(block, scratch, |field| trailers.on_header(field))?;
        Ok(trailers)
    }

    /// Decodes an HPACK header block into [`GrpcTrailers`] and stores custom metadata headers/trailers
    /// in `metadata`.
    pub fn decode_with_metadata<const DYN_CAP: usize, M: MetadataMap + ?Sized>(
        decoder: &mut HpackDecoder<DYN_CAP>,
        block: &'a [u8],
        scratch: &'a mut [u8],
        metadata: &mut M,
    ) -> Result<Self, HpackError> {
        let mut trailers = Self::default();
        decoder.decode(block, scratch, |field| {
            trailers.on_header_with_metadata(field, metadata)
        })?;
        Ok(trailers)
    }

    /// Updates this [`GrpcTrailers`] with a single [`HeaderField`].
    #[inline]
    pub fn on_header(&mut self, field: HeaderField<'a>) -> Result<(), HpackError> {
        match field.name {
            b":status" => self.status = Some(field.value),
            b"content-type" => self.content_type = Some(field.value),
            b"grpc-status" => {
                self.grpc_status = Some(parse_grpc_status_header(field.value)?);
            }
            b"grpc-message" => self.grpc_message = Some(field.value),
            _ => {}
        }
        Ok(())
    }

    /// Updates this [`GrpcTrailers`] with a single [`HeaderField`], pushing any custom non-pseudo
    /// metadata headers/trailers into `metadata`.
    #[inline]
    pub fn on_header_with_metadata<M: MetadataMap + ?Sized>(
        &mut self,
        field: HeaderField<'a>,
        metadata: &mut M,
    ) -> Result<(), HpackError> {
        match field.name {
            b":status" => self.status = Some(field.value),
            b"content-type" => self.content_type = Some(field.value),
            b"grpc-encoding" | b"grpc-accept-encoding" => {}
            b"grpc-status" => {
                self.grpc_status = Some(parse_grpc_status_header(field.value)?);
            }
            b"grpc-message" => self.grpc_message = Some(field.value),
            name if name.iter().any(u8::is_ascii_uppercase) => {
                return Err(HpackError::InvalidHeaderValue);
            }
            name if !name.starts_with(b":") && !is_reserved_grpc_header(name) => {
                metadata.push_wire(name, field.value)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Converts these trailers into a [`GrpcStatus`], percent-decoding `grpc-message` into
    /// `msg_scratch` (or borrowing directly when unescaped).
    pub fn to_grpc_status<'m>(
        &self,
        msg_scratch: &'m mut [u8],
    ) -> Result<GrpcStatus<'m>, HpackError>
    where
        'a: 'm,
    {
        let code = self.grpc_status.ok_or(HpackError::InvalidHeaderValue)?;
        let msg = decode_grpc_message_field(self.grpc_message, msg_scratch)?;
        Ok(GrpcStatus::new(code, msg))
    }
}

#[inline]
fn parse_grpc_status_header(value: &[u8]) -> Result<GrpcCode, HpackError> {
    if let Some(code) = GrpcCode::from_ascii_bytes(value) {
        return Ok(code);
    }
    if !value.is_empty() && !matches!(value, [b'0', ..]) && value.iter().all(u8::is_ascii_digit) {
        return Ok(GrpcCode::Unknown);
    }
    Err(HpackError::InvalidHeaderValue)
}

fn decode_grpc_message_field<'a, 'm>(
    raw_msg: Option<&'a [u8]>,
    msg_scratch: &'m mut [u8],
) -> Result<&'m str, HpackError>
where
    'a: 'm,
{
    let Some(raw) = raw_msg else {
        return Ok("");
    };
    if let Some(borrowed) = try_percent_decode_in_place(raw) {
        return Ok(borrowed);
    }
    percent_decode(raw, msg_scratch).map_err(|err| match err {
        roto_runtime::DecodeError::UnexpectedEof => HpackError::BufferOverflow,
        _ => HpackError::InvalidHeaderValue,
    })
}

// ============================================================================
// Zero-Allocation HPACK Encoder (RFC 7541 §6)
// ============================================================================

/// Encodes an RFC 7541 §5.2 string literal into `out`, optionally using canonical Huffman coding.
pub fn encode_string_literal(
    bytes: &[u8],
    use_huffman: bool,
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    if use_huffman {
        let huff_len = huffman_encoded_len(bytes);
        let prefix_len = encode_prefix_int(7, 0x80, huff_len, out)?;
        if out.len() < prefix_len + huff_len {
            return Err(EncodeError::BufferOverflow);
        }
        let written = huffman_encode(bytes, &mut out[prefix_len..prefix_len + huff_len])?;
        Ok(prefix_len + written)
    } else {
        let prefix_len = encode_prefix_int(7, 0x00, bytes.len(), out)?;
        if out.len() < prefix_len + bytes.len() {
            return Err(EncodeError::BufferOverflow);
        }
        out[prefix_len..prefix_len + bytes.len()].copy_from_slice(bytes);
        Ok(prefix_len + bytes.len())
    }
}

/// Encodes an RFC 7541 §5.2 string literal backward onto `writer`.
pub fn encode_string_literal_backward(bytes: &[u8], writer: &mut BackwardWriter<'_>) {
    let _ = writer.write_bytes(bytes);
    encode_prefix_int_backward(7, 0x00, bytes.len(), writer);
}

/// Encodes an RFC 7541 §6.1 Indexed Header Field (`0b1xxxxxxx`, 7-bit prefix) into `out`.
#[inline]
pub fn encode_indexed(index: usize, out: &mut [u8]) -> Result<usize, EncodeError> {
    if index == 0 {
        return Err(EncodeError::BufferOverflow);
    }
    encode_prefix_int(7, 0x80, index, out)
}

/// Encodes an RFC 7541 §6.1 Indexed Header Field onto `writer`.
#[inline]
pub fn encode_indexed_backward(index: usize, writer: &mut BackwardWriter<'_>) {
    encode_prefix_int_backward(7, 0x80, index, writer);
}

/// Encodes an RFC 7541 §6.2.2 / §6.2.3 Literal Header Field with an indexed name (`name_index >= 1`)
/// without dynamic table indexing into `out`.
///
/// When `never_index` is `true`, uses the `0001xxxx` Never Indexed representation (§6.2.3);
/// otherwise uses the `0000xxxx` Without Indexing representation (§6.2.2).
pub fn encode_literal_indexed_name(
    name_index: usize,
    value: &[u8],
    never_index: bool,
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    encode_literal_indexed_name_ext(name_index, value, never_index, false, false, out)
}

/// Encodes a Literal Header Field with an indexed name and full control over incremental indexing
/// and Huffman string encoding.
pub fn encode_literal_indexed_name_ext(
    name_index: usize,
    value: &[u8],
    never_index: bool,
    incremental_index: bool,
    use_huffman: bool,
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    if name_index == 0 {
        return Err(EncodeError::BufferOverflow);
    }
    let (prefix_bits, tag_mask) = if incremental_index {
        (6, 0x40)
    } else if never_index {
        (4, 0x10)
    } else {
        (4, 0x00)
    };
    let mut pos = encode_prefix_int(prefix_bits, tag_mask, name_index, out)?;
    pos += encode_string_literal(value, use_huffman, &mut out[pos..])?;
    Ok(pos)
}

/// Encodes a Literal Header Field with an indexed name onto `writer`.
pub fn encode_literal_indexed_name_backward(
    name_index: usize,
    value: &[u8],
    never_index: bool,
    writer: &mut BackwardWriter<'_>,
) {
    let tag_mask = if never_index { 0x10 } else { 0x00 };
    encode_string_literal_backward(value, writer);
    encode_prefix_int_backward(4, tag_mask, name_index, writer);
}

/// Encodes an RFC 7541 §6.2.2 / §6.2.3 Literal Header Field with a literal `name` and `value`
/// without dynamic table indexing into `out`.
pub fn encode_literal(
    name: &[u8],
    value: &[u8],
    never_index: bool,
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    encode_literal_ext(name, value, never_index, false, false, out)
}

/// Encodes a Literal Header Field with literal `name` and `value` and full control over
/// incremental indexing and Huffman string encoding.
pub fn encode_literal_ext(
    name: &[u8],
    value: &[u8],
    never_index: bool,
    incremental_index: bool,
    use_huffman: bool,
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    if out.is_empty() {
        return Err(EncodeError::BufferOverflow);
    }
    out[0] = if incremental_index {
        0x40
    } else if never_index {
        0x10
    } else {
        0x00
    };
    let mut pos = 1;
    pos += encode_string_literal(name, use_huffman, &mut out[pos..])?;
    pos += encode_string_literal(value, use_huffman, &mut out[pos..])?;
    Ok(pos)
}

/// Encodes a Literal Header Field with literal `name` and `value` onto `writer`.
pub fn encode_literal_backward(
    name: &[u8],
    value: &[u8],
    never_index: bool,
    writer: &mut BackwardWriter<'_>,
) {
    let tag = if never_index { 0x10u8 } else { 0x00u8 };
    encode_string_literal_backward(value, writer);
    encode_string_literal_backward(name, writer);
    let _ = writer.write_bytes(&[tag]);
}

/// Encodes an RFC 7541 §6.3 Dynamic Table Size Update (`001xxxxx`, 5-bit prefix) into `out`.
#[inline]
pub fn encode_dynamic_table_size_update(
    max_size: usize,
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    encode_prefix_int(5, 0x20, max_size, out)
}

#[inline]
fn hpack_int_len(prefix_bits: u8, mut val: usize) -> usize {
    let max_prefix = (1usize << prefix_bits) - 1;
    if val < max_prefix {
        return 1;
    }
    val -= max_prefix;
    let mut n = 2;
    while val >= 128 {
        val >>= 7;
        n += 1;
    }
    n
}

/// Returns the exact unencoded HPACK byte length of all entries in `metadata` (accounting for
/// RFC 4648 Base64 encoding of `*-bin` binary values).
#[must_use]
#[inline]
pub fn metadata_hpack_len<M: MetadataMap + ?Sized>(metadata: &M) -> usize {
    let mut total = 0;
    let len = metadata.len();
    for i in 0..len {
        if let Some(f) = metadata.get_entry(i) {
            let wire_val_len = if is_binary_metadata_name(f.name) {
                base64_encoded_len(f.value.len())
            } else {
                f.value.len()
            };
            total += 1
                + hpack_int_len(7, f.name.len())
                + f.name.len()
                + hpack_int_len(7, wire_val_len)
                + wire_val_len;
        }
    }
    total
}

/// Encodes all custom metadata entries in `metadata` into `out` as literal header fields.
pub fn encode_metadata<M: MetadataMap + ?Sized>(
    metadata: &M,
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    HpackEncoder::new().encode_metadata(metadata, out)
}

/// Encodes all custom metadata entries in `metadata` backward onto `writer` (Base64-encoding
/// `*-bin` binary values in-place).
pub fn encode_metadata_backward<M: MetadataMap + ?Sized>(
    metadata: &M,
    writer: &mut BackwardWriter<'_>,
) {
    let mut i = metadata.len();
    while i > 0 {
        i -= 1;
        if let Some(field) = metadata.get_entry(i) {
            if is_binary_metadata_name(field.name) {
                let b64_len = base64_encoded_len(field.value.len());
                base64_encode_backward(field.value, writer);
                encode_prefix_int_backward(7, 0x00, b64_len, writer);
                encode_string_literal_backward(field.name, writer);
                let _ = writer.write_bytes(&[0x00]);
            } else {
                encode_literal_backward(field.name, field.value, false, writer);
            }
        }
    }
}

/// Encodes outbound gRPC HTTP/2 request headers (`:method: POST`, `:scheme`, `:path`,
/// `:authority`, `content-type`, `te: trailers`) into `out`.
pub fn encode_grpc_request_headers(
    scheme: &[u8],
    authority: &[u8],
    path: &[u8],
    content_type: &[u8],
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    HpackEncoder::new().encode_grpc_request_headers(scheme, authority, path, content_type, out)
}

/// Encodes outbound gRPC HTTP/2 request headers followed by custom `metadata` entries into `out`.
pub fn encode_grpc_request_headers_with_metadata<M: MetadataMap + ?Sized>(
    scheme: &[u8],
    authority: &[u8],
    path: &[u8],
    content_type: &[u8],
    metadata: &M,
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    HpackEncoder::new()
        .encode_grpc_request_headers_with_metadata(scheme, authority, path, content_type, metadata, out)
}

/// Encodes outbound gRPC HTTP/2 response headers (`:status: 200`, `content-type`) into `out`.
pub fn encode_grpc_response_headers(
    content_type: &[u8],
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    HpackEncoder::new().encode_grpc_response_headers(content_type, out)
}

/// Encodes outbound gRPC HTTP/2 response headers followed by custom `metadata` entries into `out`.
pub fn encode_grpc_response_headers_with_metadata<M: MetadataMap + ?Sized>(
    content_type: &[u8],
    metadata: &M,
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    HpackEncoder::new().encode_grpc_response_headers_with_metadata(content_type, metadata, out)
}

/// Encodes outbound gRPC HTTP/2 response trailers (`grpc-status` and percent-encoded
/// `grpc-message` if non-empty) into `out`.
pub fn encode_grpc_trailers(status: GrpcStatus<'_>, out: &mut [u8]) -> Result<usize, EncodeError> {
    HpackEncoder::new().encode_grpc_trailers(status, out)
}

/// Encodes outbound gRPC HTTP/2 response trailers followed by custom `metadata` entries into `out`.
pub fn encode_grpc_trailers_with_metadata<M: MetadataMap + ?Sized>(
    status: GrpcStatus<'_>,
    metadata: &M,
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    HpackEncoder::new().encode_grpc_trailers_with_metadata(status, metadata, out)
}

/// Encodes a gRPC HTTP/2 Trailers-Only response (`:status: 200`, `content-type`, `grpc-status`,
/// and percent-encoded `grpc-message` if non-empty) into `out`.
pub fn encode_grpc_trailers_only(
    status: GrpcStatus<'_>,
    content_type: &[u8],
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    HpackEncoder::new().encode_grpc_trailers_only(status, content_type, out)
}

/// Encodes a gRPC HTTP/2 Trailers-Only response followed by custom `metadata` entries into `out`.
pub fn encode_grpc_trailers_only_with_metadata<M: MetadataMap + ?Sized>(
    status: GrpcStatus<'_>,
    content_type: &[u8],
    metadata: &M,
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    HpackEncoder::new().encode_grpc_trailers_only_with_metadata(status, content_type, metadata, out)
}

/// Zero-allocation HPACK encoder for HTTP/2 and gRPC headers.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HpackEncoder {
    /// Whether to Huffman-encode literal header strings.
    pub use_huffman: bool,
}

impl HpackEncoder {
    /// Creates a new [`HpackEncoder`] with `use_huffman = false`.
    #[inline]
    pub const fn new() -> Self {
        Self { use_huffman: false }
    }

    /// Creates a new [`HpackEncoder`] configured to Huffman-encode literal strings.
    #[inline]
    pub const fn with_huffman(use_huffman: bool) -> Self {
        Self { use_huffman }
    }

    /// Encodes an indexed header field (`1..=61`).
    #[inline]
    pub fn encode_indexed(&self, index: usize, out: &mut [u8]) -> Result<usize, EncodeError> {
        encode_indexed(index, out)
    }

    /// Encodes a literal header field with an indexed name (`1..=61`) without dynamic indexing.
    #[inline]
    pub fn encode_literal_indexed_name(
        &self,
        name_index: usize,
        value: &[u8],
        never_index: bool,
        out: &mut [u8],
    ) -> Result<usize, EncodeError> {
        encode_literal_indexed_name_ext(
            name_index,
            value,
            never_index,
            false,
            self.use_huffman,
            out,
        )
    }

    /// Encodes a literal header field with literal `name` and `value` without dynamic indexing.
    #[inline]
    pub fn encode_literal(
        &self,
        name: &[u8],
        value: &[u8],
        never_index: bool,
        out: &mut [u8],
    ) -> Result<usize, EncodeError> {
        encode_literal_ext(name, value, never_index, false, self.use_huffman, out)
    }

    /// Encodes all custom metadata entries in `metadata` into `out` as literal header fields
    /// (Base64-encoding `*-bin` binary values in-place).
    pub fn encode_metadata<M: MetadataMap + ?Sized>(
        &self,
        metadata: &M,
        out: &mut [u8],
    ) -> Result<usize, EncodeError> {
        let mut pos = 0;
        let n = metadata.len();
        for i in 0..n {
            if let Some(field) = metadata.get_entry(i) {
                if is_binary_metadata_name(field.name) {
                    if pos >= out.len() {
                        return Err(EncodeError::BufferOverflow);
                    }
                    out[pos] = 0x00;
                    pos += 1;
                    pos += encode_string_literal(field.name, self.use_huffman, &mut out[pos..])?;
                    let b64_len = base64_encoded_len(field.value.len());
                    pos += encode_prefix_int(7, 0x00, b64_len, &mut out[pos..])?;
                    if out.len() < pos + b64_len {
                        return Err(EncodeError::BufferOverflow);
                    }
                    let _ = base64_encode(field.value, &mut out[pos..pos + b64_len])?;
                    pos += b64_len;
                } else {
                    pos += self.encode_literal(field.name, field.value, false, &mut out[pos..])?;
                }
            }
        }
        Ok(pos)
    }

    /// Encodes outbound gRPC request headers into `out`:
    /// - `:method: POST` (static index 3)
    /// - `:scheme` (static index 6 for `"http"`, 7 for `"https"`, or literal with name index 6)
    /// - `:path` (static index 4 for `"/"`, or literal with name index 4)
    /// - `:authority` (literal with name index 1, if non-empty)
    /// - `content-type` (literal with name index 31)
    /// - `te: trailers` (literal)
    pub fn encode_grpc_request_headers(
        &self,
        scheme: &[u8],
        authority: &[u8],
        path: &[u8],
        content_type: &[u8],
        out: &mut [u8],
    ) -> Result<usize, EncodeError> {
        let mut pos = 0;
        // 1. :method: POST
        pos += self.encode_indexed(INDEX_METHOD_POST, &mut out[pos..])?;
        // 2. :scheme
        if scheme == b"http" {
            pos += self.encode_indexed(INDEX_SCHEME_HTTP, &mut out[pos..])?;
        } else if scheme == b"https" {
            pos += self.encode_indexed(INDEX_SCHEME_HTTPS, &mut out[pos..])?;
        } else {
            pos += self.encode_literal_indexed_name(
                INDEX_SCHEME_HTTP,
                scheme,
                false,
                &mut out[pos..],
            )?;
        }
        // 3. :path
        if path == b"/" {
            pos += self.encode_indexed(INDEX_PATH_ROOT, &mut out[pos..])?;
        } else {
            pos +=
                self.encode_literal_indexed_name(INDEX_PATH_ROOT, path, false, &mut out[pos..])?;
        }
        // 4. :authority
        if !authority.is_empty() {
            pos += self.encode_literal_indexed_name(
                INDEX_AUTHORITY,
                authority,
                false,
                &mut out[pos..],
            )?;
        }
        // 5. content-type
        pos += self.encode_literal_indexed_name(
            INDEX_CONTENT_TYPE,
            content_type,
            false,
            &mut out[pos..],
        )?;
        // 6. te: trailers
        pos += self.encode_literal(b"te", b"trailers", false, &mut out[pos..])?;
        Ok(pos)
    }

    /// Encodes outbound gRPC request headers followed by custom `metadata` entries into `out`.
    pub fn encode_grpc_request_headers_with_metadata<M: MetadataMap + ?Sized>(
        &self,
        scheme: &[u8],
        authority: &[u8],
        path: &[u8],
        content_type: &[u8],
        metadata: &M,
        out: &mut [u8],
    ) -> Result<usize, EncodeError> {
        let mut pos =
            self.encode_grpc_request_headers(scheme, authority, path, content_type, out)?;
        pos += self.encode_metadata(metadata, &mut out[pos..])?;
        Ok(pos)
    }

    /// Encodes outbound gRPC response headers (`:status: 200`, `content-type`) into `out`.
    pub fn encode_grpc_response_headers(
        &self,
        content_type: &[u8],
        out: &mut [u8],
    ) -> Result<usize, EncodeError> {
        let mut pos = 0;
        pos += self.encode_indexed(INDEX_STATUS_200, &mut out[pos..])?;
        pos += self.encode_literal_indexed_name(
            INDEX_CONTENT_TYPE,
            content_type,
            false,
            &mut out[pos..],
        )?;
        Ok(pos)
    }

    /// Encodes outbound HTTP/2 response headers (`:status`, and `content-type` if non-empty) into `out`.
    pub fn encode_http_response_headers(
        &self,
        status_code: u16,
        content_type: &[u8],
        out: &mut [u8],
    ) -> Result<usize, EncodeError> {
        let mut pos = 0;
        let status_idx = match status_code {
            200 => Some(INDEX_STATUS_200),
            204 => Some(INDEX_STATUS_204),
            206 => Some(INDEX_STATUS_206),
            304 => Some(INDEX_STATUS_304),
            400 => Some(INDEX_STATUS_400),
            404 => Some(INDEX_STATUS_404),
            500 => Some(INDEX_STATUS_500),
            _ => None,
        };
        if let Some(idx) = status_idx {
            pos += self.encode_indexed(idx, &mut out[pos..])?;
        } else {
            let d0 = b'0' + ((status_code / 100) % 10) as u8;
            let d1 = b'0' + ((status_code / 10) % 10) as u8;
            let d2 = b'0' + (status_code % 10) as u8;
            let code_bytes = [d0, d1, d2];
            pos += self.encode_literal_indexed_name(
                INDEX_STATUS_200,
                &code_bytes,
                false,
                &mut out[pos..],
            )?;
        }
        if !content_type.is_empty() {
            pos += self.encode_literal_indexed_name(
                INDEX_CONTENT_TYPE,
                content_type,
                false,
                &mut out[pos..],
            )?;
        }
        Ok(pos)
    }

    /// Encodes outbound gRPC response headers (`:status: 200`, `content-type`) followed by custom
    /// `metadata` entries into `out`.
    pub fn encode_grpc_response_headers_with_metadata<M: MetadataMap + ?Sized>(
        &self,
        content_type: &[u8],
        metadata: &M,
        out: &mut [u8],
    ) -> Result<usize, EncodeError> {
        let mut pos = self.encode_grpc_response_headers(content_type, out)?;
        pos += self.encode_metadata(metadata, &mut out[pos..])?;
        Ok(pos)
    }

    /// Encodes outbound gRPC response trailers (`grpc-status`, and percent-encoded `grpc-message`
    /// if `status.message` is non-empty) into `out`.
    pub fn encode_grpc_trailers(
        &self,
        status: GrpcStatus<'_>,
        out: &mut [u8],
    ) -> Result<usize, EncodeError> {
        self.encode_grpc_trailers_with_mode(status, false, GRPC_CONTENT_TYPE_ROTO, out)
    }

    /// Encodes outbound gRPC response trailers (`grpc-status`, `grpc-message`) followed by custom
    /// `metadata` entries into `out`.
    pub fn encode_grpc_trailers_with_metadata<M: MetadataMap + ?Sized>(
        &self,
        status: GrpcStatus<'_>,
        metadata: &M,
        out: &mut [u8],
    ) -> Result<usize, EncodeError> {
        self.encode_grpc_trailers_with_mode_and_metadata(
            status,
            false,
            GRPC_CONTENT_TYPE_ROTO,
            metadata,
            out,
        )
    }

    /// Encodes a gRPC Trailers-Only response (`:status: 200`, `content-type`, `grpc-status`,
    /// and percent-encoded `grpc-message` if non-empty) into `out`.
    pub fn encode_grpc_trailers_only(
        &self,
        status: GrpcStatus<'_>,
        content_type: &[u8],
        out: &mut [u8],
    ) -> Result<usize, EncodeError> {
        self.encode_grpc_trailers_with_mode(status, true, content_type, out)
    }

    /// Encodes a gRPC Trailers-Only response followed by custom `metadata` entries into `out`.
    pub fn encode_grpc_trailers_only_with_metadata<M: MetadataMap + ?Sized>(
        &self,
        status: GrpcStatus<'_>,
        content_type: &[u8],
        metadata: &M,
        out: &mut [u8],
    ) -> Result<usize, EncodeError> {
        self.encode_grpc_trailers_with_mode_and_metadata(status, true, content_type, metadata, out)
    }

    /// Encodes gRPC trailers into `out`, optionally prepending `:status: 200` and `content-type`
    /// when `trailers_only` is `true`.
    pub fn encode_grpc_trailers_with_mode(
        &self,
        status: GrpcStatus<'_>,
        trailers_only: bool,
        content_type: &[u8],
        out: &mut [u8],
    ) -> Result<usize, EncodeError> {
        let mut pos = 0;
        if trailers_only {
            pos += self.encode_grpc_response_headers(content_type, &mut out[pos..])?;
        }
        // grpc-status
        pos += self.encode_literal(
            b"grpc-status",
            status.code.as_str().as_bytes(),
            false,
            &mut out[pos..],
        )?;
        // grpc-message (if non-empty, percent-encoded in-place)
        if !status.message.is_empty() {
            if pos >= out.len() {
                return Err(EncodeError::BufferOverflow);
            }
            out[pos] = 0x00;
            pos += 1;
            pos += encode_string_literal(b"grpc-message", self.use_huffman, &mut out[pos..])?;
            let pct_len = percent_encoded_len(status.message);
            pos += encode_prefix_int(7, 0x00, pct_len, &mut out[pos..])?;
            if out.len() < pos + pct_len {
                return Err(EncodeError::BufferOverflow);
            }
            let _ = percent_encode(status.message, &mut out[pos..pos + pct_len])?;
            pos += pct_len;
        }
        Ok(pos)
    }

    /// Encodes gRPC trailers into `out` (with optional `:status: 200` and `content-type` when
    /// `trailers_only` is `true`) followed by custom `metadata` entries.
    pub fn encode_grpc_trailers_with_mode_and_metadata<M: MetadataMap + ?Sized>(
        &self,
        status: GrpcStatus<'_>,
        trailers_only: bool,
        content_type: &[u8],
        metadata: &M,
        out: &mut [u8],
    ) -> Result<usize, EncodeError> {
        let mut pos =
            self.encode_grpc_trailers_with_mode(status, trailers_only, content_type, out)?;
        pos += self.encode_metadata(metadata, &mut out[pos..])?;
        Ok(pos)
    }
}

/// Encodes gRPC trailers (or a Trailers-Only response when `trailers_only == true`) backward
/// onto `writer`.
pub fn encode_grpc_trailers_backward(
    status: GrpcStatus<'_>,
    trailers_only: bool,
    content_type: &[u8],
    writer: &mut BackwardWriter<'_>,
) {
    if !status.message.is_empty() {
        let pct_len = percent_encoded_len(status.message);
        percent_encode_backward(status.message, writer);
        encode_prefix_int_backward(7, 0x00, pct_len, writer);
        encode_string_literal_backward(b"grpc-message", writer);
        let _ = writer.write_bytes(&[0x00]);
    }
    encode_literal_backward(
        b"grpc-status",
        status.code.as_str().as_bytes(),
        false,
        writer,
    );
    if trailers_only {
        encode_literal_indexed_name_backward(INDEX_CONTENT_TYPE, content_type, false, writer);
        encode_indexed_backward(INDEX_STATUS_200, writer);
    }
}
