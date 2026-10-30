//! Pluggable byte storage for HPACK scratch, header/stream reassembly and the
//! TX queue.
//!
//! [`FixedBuffers`] (default) is inline and allocation-free, [`SliceBuffers`]
//! borrows caller-provided slices (still zero-alloc), and `GrowableBuffers`
//! (feature `std`) grows `Vec<u8>`s on demand.

use crate::h2::DEFAULT_REASM_CAP;
use core::marker::PhantomData;

/// Default HPACK Huffman scratch size in bytes.
pub const DEFAULT_HPACK_SCRATCH: usize = 512;
/// Default TX queue size in bytes.
pub const DEFAULT_TX_QUEUE_CAP: usize = 512;

/// A contiguous byte buffer that may or may not be able to grow.
pub trait ByteStore {
    /// Current contents/capacity as a slice.
    fn as_slice(&self) -> &[u8];
    /// Current contents/capacity as a mutable slice.
    fn as_mut_slice(&mut self) -> &mut [u8];
    /// Ensure at least `need` bytes are available, preserving existing bytes.
    /// Returns `false` (leaving the store unchanged) if the store cannot
    /// provide `need` bytes. Never shrinks.
    fn ensure(&mut self, need: usize) -> bool;
}

impl<const N: usize> ByteStore for [u8; N] {
    fn as_slice(&self) -> &[u8] {
        self
    }
    fn as_mut_slice(&mut self) -> &mut [u8] {
        self
    }
    fn ensure(&mut self, need: usize) -> bool {
        need <= N
    }
}

impl ByteStore for &mut [u8] {
    fn as_slice(&self) -> &[u8] {
        self
    }
    fn as_mut_slice(&mut self) -> &mut [u8] {
        self
    }
    fn ensure(&mut self, need: usize) -> bool {
        need <= self.len()
    }
}

#[cfg(feature = "std")]
impl ByteStore for Vec<u8> {
    fn as_slice(&self) -> &[u8] {
        self
    }
    fn as_mut_slice(&mut self) -> &mut [u8] {
        self
    }
    fn ensure(&mut self, need: usize) -> bool {
        if self.len() < need {
            self.resize(need, 0);
        }
        true
    }
}

/// Bundle of the stores used by a connection, so public types need a single
/// parameter instead of one per buffer.
pub trait BufferSet {
    /// Scratch for Huffman-decoded HPACK strings (see [`max_decoded_len`]).
    type HpackScratch: ByteStore;
    /// Header block (HEADERS/CONTINUATION) reassembly.
    type HeaderReasm: ByteStore;
    /// Outbound frame queue.
    type TxQueue: ByteStore;
    /// Per-stream message reassembly.
    type StreamReasm: ByteStore;
}

/// Inline fixed-size buffers; the default sizes match the historical layout.
#[derive(Debug, Clone, Copy, Default)]
pub struct FixedBuffers<
    const HPACK: usize = DEFAULT_HPACK_SCRATCH,
    const REASM: usize = DEFAULT_REASM_CAP,
    const TX: usize = DEFAULT_TX_QUEUE_CAP,
>;

impl<const HPACK: usize, const REASM: usize, const TX: usize> BufferSet
    for FixedBuffers<HPACK, REASM, TX>
{
    type HpackScratch = [u8; HPACK];
    type HeaderReasm = [u8; REASM];
    type TxQueue = [u8; TX];
    type StreamReasm = [u8; REASM];
}

/// Caller-provided slices; no allocation, capacity fixed by the slice lengths.
#[derive(Debug, Default)]
pub struct SliceBuffers<'a>(PhantomData<&'a mut [u8]>);

impl<'a> BufferSet for SliceBuffers<'a> {
    type HpackScratch = &'a mut [u8];
    type HeaderReasm = &'a mut [u8];
    type TxQueue = &'a mut [u8];
    type StreamReasm = &'a mut [u8];
}

/// Heap buffers that grow on demand.
#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, Default)]
pub struct GrowableBuffers;

#[cfg(feature = "std")]
impl BufferSet for GrowableBuffers {
    type HpackScratch = Vec<u8>;
    type HeaderReasm = Vec<u8>;
    type TxQueue = Vec<u8>;
    type StreamReasm = Vec<u8>;
}

/// Upper bound on the bytes a stateless HPACK decoder can produce from
/// Huffman strings in `block`.
///
/// The shortest Huffman code is 5 bits, so a block of `n` bytes decodes to at
/// most `n * 8 / 5` bytes. Static-table lookups borrow `'static` and raw
/// literals borrow the block, so only Huffman strings consume scratch; sizing
/// scratch to this bound once per block therefore never fails.
pub const fn max_decoded_len(block: &[u8]) -> usize {
    (block.len() * 8).div_ceil(5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hpack::{huffman_decode, huffman_encode};

    #[test]
    fn fixed_defaults_match_current_sizes() {
        assert_eq!(
            core::mem::size_of::<<FixedBuffers as BufferSet>::HpackScratch>(),
            512
        );
        assert_eq!(
            core::mem::size_of::<<FixedBuffers as BufferSet>::HeaderReasm>(),
            DEFAULT_REASM_CAP
        );
        assert_eq!(
            core::mem::size_of::<<FixedBuffers as BufferSet>::TxQueue>(),
            512
        );
    }

    #[test]
    fn fixed_array_never_exceeds_capacity() {
        let mut a = [7u8; 8];
        assert!(a.ensure(8));
        assert!(!a.ensure(9));
        assert_eq!(a.as_slice().len(), 8);
        assert_eq!(a.as_mut_slice(), &[7u8; 8]);
    }

    #[test]
    fn slice_never_exceeds_capacity() {
        let mut backing = [1u8, 2, 3, 4];
        let mut s: <SliceBuffers as BufferSet>::TxQueue = &mut backing[..];
        assert!(s.ensure(4));
        assert!(!s.ensure(5));
        assert_eq!(ByteStore::as_slice(&s), &[1, 2, 3, 4]);
    }

    #[cfg(feature = "std")]
    #[test]
    fn vec_grows_preserves_and_reuses_capacity() {
        let mut v: <GrowableBuffers as BufferSet>::HpackScratch = Vec::new();
        assert!(v.ensure(4));
        assert_eq!(v.as_slice().len(), 4);
        v.as_mut_slice().copy_from_slice(&[1, 2, 3, 4]);
        assert!(v.ensure(8));
        assert_eq!(&v.as_slice()[..4], &[1, 2, 3, 4]);
        assert_eq!(v.as_slice().len(), 8);
        let cap = v.capacity();
        assert!(v.ensure(2)); // never shrinks
        assert_eq!(v.as_slice().len(), 8);
        assert_eq!(v.capacity(), cap);
    }

    #[test]
    fn max_decoded_len_bounds_worst_case_huffman() {
        // '0' has a 5-bit code: the densest possible input.
        for n in 1..=64usize {
            let src = [b'0'; 64];
            let src = &src[..n];
            let mut enc = [0u8; 128];
            let elen = huffman_encode(src, &mut enc).unwrap();
            let block = &enc[..elen];
            let mut out = [0u8; 128];
            let dec = huffman_decode(block, &mut out).unwrap();
            assert_eq!(dec, src);
            assert!(dec.len() <= max_decoded_len(block));
        }
        assert_eq!(max_decoded_len(&[]), 0);
        assert_eq!(max_decoded_len(&[0; 5]), 8);
        assert_eq!(max_decoded_len(&[0; 1]), 2);
    }
}
