//! Byte storage abstraction for scratch/reassembly buffers.

/// A contiguous byte buffer that may or may not be able to grow.
pub trait ByteStore {
    /// Return a slice of at least `need` bytes, preserving existing contents.
    /// Never shrinks. Fixed-capacity stores return `None` if `need` exceeds
    /// their capacity; growable stores always succeed. The returned slice may be
    /// longer than `need` (fixed stores return the whole buffer and a `Vec` keeps
    /// its peak length), so callers needing exactly `need` bytes must slice.
    fn ensure(&mut self, need: usize) -> Option<&mut [u8]>;
}

impl<const N: usize> ByteStore for [u8; N] {
    fn ensure(&mut self, need: usize) -> Option<&mut [u8]> {
        if need <= N {
            Some(self)
        } else {
            None
        }
    }
}

impl ByteStore for &mut [u8] {
    fn ensure(&mut self, need: usize) -> Option<&mut [u8]> {
        if need <= self.len() {
            Some(self)
        } else {
            None
        }
    }
}

#[cfg(feature = "std")]
impl ByteStore for Vec<u8> {
    fn ensure(&mut self, need: usize) -> Option<&mut [u8]> {
        if self.len() < need {
            self.resize(need, 0);
        }
        Some(self)
    }
}

/// Default HPACK scratch capacity of [`FixedBuffers`]: room for the Huffman-decoded strings of
/// one header block.
pub const DEFAULT_HPACK_SCRATCH: usize = 512;

/// The caller-selectable buffer storage of a connection driver
/// ([`GrpcServer`](crate::GrpcServer), [`AsyncGrpcServer`](crate::AsyncGrpcServer),
/// [`GrpcClient`](crate::GrpcClient)).
///
/// A `BufferSet` is a value as well as a type: pass one to the drivers' `with_buffers` to move
/// caller-owned stores into the connection. [`FixedBuffers`] is the default.
///
/// This trait is deliberately unsealed but **unstable**: later releases add associated types
/// for further per-connection buffers, which is a breaking change for external implementors.
/// Prefer the provided [`FixedBuffers`], [`SliceBuffers`] and `GrowableBuffers`.
pub trait BufferSet {
    /// Scratch for Huffman-decoded HPACK strings; one is kept per connection because header
    /// blocks are decoded one at a time.
    type HpackScratch: ByteStore;

    /// Surrenders the HPACK scratch store to the connection.
    fn into_hpack_scratch(self) -> Self::HpackScratch;
}

/// `#![no_std]` default: inline [`DEFAULT_HPACK_SCRATCH`]-byte HPACK scratch, no allocation.
#[derive(Debug, Default, Clone, Copy)]
pub struct FixedBuffers;

impl BufferSet for FixedBuffers {
    type HpackScratch = [u8; DEFAULT_HPACK_SCRATCH];

    fn into_hpack_scratch(self) -> Self::HpackScratch {
        [0; DEFAULT_HPACK_SCRATCH]
    }
}

/// Caller-provided buffers: the connection borrows `hpack` for its lifetime, so `no_std` users
/// can size HPACK scratch to their peers (see `docs/grpc.md`).
#[derive(Debug)]
pub struct SliceBuffers<'s> {
    hpack: &'s mut [u8],
}

impl<'s> SliceBuffers<'s> {
    /// Uses `hpack` as the connection's HPACK scratch; blocks whose Huffman strings decode to
    /// more than `hpack.len()` bytes are rejected.
    pub fn new(hpack: &'s mut [u8]) -> Self {
        Self { hpack }
    }
}

impl<'s> BufferSet for SliceBuffers<'s> {
    type HpackScratch = &'s mut [u8];

    fn into_hpack_scratch(self) -> Self::HpackScratch {
        self.hpack
    }
}

/// Heap-backed buffers: HPACK scratch is sized once per header block with [`max_decoded_len`],
/// so it never overflows; growth is bounded by the inbound frame limits that cap block size.
/// The scratch keeps its peak capacity for the connection's lifetime.
#[cfg(feature = "std")]
#[derive(Debug, Default, Clone, Copy)]
pub struct GrowableBuffers;

#[cfg(feature = "std")]
impl BufferSet for GrowableBuffers {
    type HpackScratch = Vec<u8>;

    fn into_hpack_scratch(self) -> Self::HpackScratch {
        Vec::new()
    }
}

/// Scratch slice for decoding a header block of `block_len` bytes.
///
/// Sized to [`max_decoded_len`] when the store can provide it; otherwise the whole store, so
/// blocks that actually decode to less still succeed (the decoder reports
/// `HpackError::BufferOverflow` for those that do not).
pub(crate) fn hpack_scratch(store: &mut impl ByteStore, block_len: usize) -> &mut [u8] {
    let need = max_decoded_len(block_len);
    // Two calls (not one `match`) because the borrow checker rejects falling back to `store`
    // while the first result is still live; `ensure(0)` always yields the whole store.
    let fits = store.ensure(need).is_some();
    store.ensure(if fits { need } else { 0 }).unwrap_or_default()
}

/// Upper bound on the bytes a stateless HPACK decoder can produce from
/// Huffman strings in a header block of `len` bytes.
///
/// The shortest Huffman code is 5 bits, so `len` bytes decode to at most
/// `len * 8 / 5` bytes (rounded up here). Static-table lookups borrow
/// `'static` and raw literals borrow the block, so only Huffman strings
/// consume scratch; sizing scratch to this bound once per block never fails.
///
/// Saturates instead of overflowing for huge `len`.
#[must_use]
pub const fn max_decoded_len(len: usize) -> usize {
    len.saturating_mul(8).div_ceil(5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hpack::{huffman_decode, huffman_encode};

    #[test]
    fn fixed_array_never_exceeds_capacity() {
        let mut a = [7u8; 8];
        assert_eq!(a.ensure(8), Some(&mut [7u8; 8][..]));
        assert!(a.ensure(9).is_none());
    }

    #[test]
    fn slice_never_exceeds_capacity() {
        let mut backing = [1u8, 2, 3, 4];
        let mut s: &mut [u8] = &mut backing[..];
        assert_eq!(s.ensure(4).unwrap(), &[1, 2, 3, 4]);
        assert!(s.ensure(5).is_none());
    }

    #[cfg(feature = "std")]
    #[test]
    fn vec_grows_preserves_and_reuses_capacity() {
        let mut v: Vec<u8> = Vec::new();
        v.ensure(4).unwrap().copy_from_slice(&[1, 2, 3, 4]);
        let s = v.ensure(8).unwrap();
        assert_eq!(&s[..4], &[1, 2, 3, 4]);
        assert_eq!(s.len(), 8);
        let cap = v.capacity();
        assert_eq!(v.ensure(2).unwrap().len(), 8); // never shrinks
        assert_eq!(v.capacity(), cap);
    }

    #[test]
    fn max_decoded_len_bounds_worst_case_huffman() {
        // '0' has a 5-bit code: the densest possible input.
        for n in 1..=64usize {
            let all = [b'0'; 64];
            let src = &all[..n];
            let mut enc = [0u8; 128];
            let elen = huffman_encode(src, &mut enc).unwrap();
            let mut out = [0u8; 128];
            let dec = huffman_decode(&enc[..elen], &mut out).unwrap();
            assert_eq!(dec, src);
            assert!(dec.len() <= max_decoded_len(elen));
        }
        assert_eq!(max_decoded_len(0), 0);
        assert_eq!(max_decoded_len(5), 8);
        assert_eq!(max_decoded_len(1), 2);
    }
}
