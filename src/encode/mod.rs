#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use crate::num::{SignedVarIntTarget, VarIntTarget};

/// Encodes a single number to a varint. Requires SSE2 support.
///
/// Produces a tuple, with the encoded data followed by the number of bytes used to encode the
/// varint.
///
/// # Examples
/// ```
/// use varint_simd::encode;
///
/// let encoded = encode::<u32>(1337);
/// assert_eq!(encoded, ([185, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2));
/// ```
#[inline]
#[cfg(any(target_feature = "sse2", doc))]
#[cfg_attr(rustc_nightly, doc(cfg(target_feature = "sse2")))]
pub fn encode<T: VarIntTarget>(num: T) -> ([u8; 16], u8) {
    unsafe { encode_unsafe(num) }
}

/// Convenience function for encoding a single signed integer in ZigZag format to a varint.
/// See also: [`encode`]
///
/// # Examples
/// ```
/// use varint_simd::encode_zigzag;
///
/// let encoded = encode_zigzag::<i32>(-20);
/// assert_eq!(encoded, ([39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1));
/// ```
#[inline]
#[cfg(any(target_feature = "sse2", doc))]
#[cfg_attr(rustc_nightly, doc(cfg(target_feature = "sse2")))]
pub fn encode_zigzag<T: SignedVarIntTarget>(num: T) -> ([u8; 16], u8) {
    unsafe { encode_unsafe(T::Unsigned::zigzag(num)) }
}

/// Encodes a single number to a varint, and writes the resulting data to the slice. Returns the
/// number of bytes written (maximum 10 bytes).
///
/// See also: [`encode`]
///
/// **Panics:** if the slice is too small to contain the varint.
#[inline]
#[cfg(any(target_feature = "sse2", doc))]
#[cfg_attr(rustc_nightly, doc(cfg(target_feature = "sse2")))]
pub fn encode_to_slice<T: VarIntTarget>(num: T, slice: &mut [u8]) -> u8 {
    let (data, size) = encode(num);
    slice[..size as usize].copy_from_slice(&data[..size as usize]);

    size
}

/// Encodes a single number to a varint. Requires SSE2 support.
///
/// Produces a tuple, with the encoded data followed by the number of bytes used to encode the
/// varint.
///
/// # Safety
/// This should not have any unsafe behavior with any input. However, it still calls a large number
/// of unsafe functions.
#[inline]
#[cfg(any(target_feature = "sse2", doc))]
#[cfg_attr(rustc_nightly, doc(cfg(target_feature = "sse2")))]
pub unsafe fn encode_unsafe<T: VarIntTarget>(num: T) -> ([u8; 16], u8) {
    if T::MAX_VARINT_BYTES <= 5 {
        // We could kick off a lzcnt here on the original number but that makes the math complicated and slow

        let stage1 = num.num_to_scalar_stage1();

        // We could OR the data with 1 to avoid undefined behavior, but for some reason it's still faster to take the branch
        let leading = stage1.leading_zeros();

        let unused_bytes = (leading - 1) / 8;
        let bytes_needed = 8 - unused_bytes;

        // set all but the last MSBs
        let msbs = 0x8080808080808080;
        let msbmask = 0xFFFFFFFFFFFFFFFF >> ((8 - bytes_needed + 1) * 8 - 1);

        let merged = stage1 | (msbs & msbmask);

        (core::mem::transmute::<[u64; 2], [u8; 16]>([merged, 0]), bytes_needed as u8)
    } else {
        // Break the number into 7-bit parts and spread them out into a vector
        let stage1: __m128i = core::mem::transmute(num.num_to_vector_stage1());

        // Create a mask for where there exist values
        // This signed comparison works because all MSBs should be cleared at this point
        // Also handle the special case when num == 0
        let minimum = _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xffu8 as i8);
        let exists = _mm_or_si128(_mm_cmpgt_epi8(stage1, _mm_setzero_si128()), minimum);
        let bits = _mm_movemask_epi8(exists);

        // Count the number of bytes used
        let bytes = 32 - bits.leading_zeros() as u8; // lzcnt on supported CPUs
                                                     // TODO: Compiler emits an unnecessary branch here when using bsr/bsl fallback

        // Fill that many bytes into a vector
        let ascend = _mm_setr_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let mask = _mm_cmplt_epi8(ascend, _mm_set1_epi8(bytes as i8));

        // Shift it down 1 byte so the last MSB is the only one set, and make sure only the MSB is set
        let shift = _mm_bsrli_si128(mask, 1);
        let msbmask = _mm_and_si128(shift, _mm_set1_epi8(128u8 as i8));

        // Merge the MSB bits into the vector
        let merged = _mm_or_si128(stage1, msbmask);

        (core::mem::transmute::<__m128i, [u8; 16]>(merged), bytes)
    }
}
