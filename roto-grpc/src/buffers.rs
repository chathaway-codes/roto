//! Byte storage abstraction for scratch/reassembly buffers.

/// A contiguous byte buffer that may or may not be able to grow.
pub trait ByteStore {
    /// Return a slice of at least `need` bytes, preserving existing contents.
    /// Never shrinks. Fixed-capacity stores return `None` if `need` exceeds
    /// their capacity; growable stores always succeed. The returned slice may be
    /// longer than `need` (fixed stores return the whole buffer, so
    /// `len >= need`); callers must slice to `need`.
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
