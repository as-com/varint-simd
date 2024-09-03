#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::cmp::min;

use crate::num::{SignedVarIntTarget, VarIntTarget};
use crate::VarIntDecodeError;

mod lookup;

/// Decodes a single varint from the input slice.
///
/// Produces a tuple containing the decoded number and the number of bytes read. For best
/// performance, provide a slice at least 16 bytes in length, or use the unsafe version directly.
///
/// # Examples
/// ```
/// use varint_simd::{decode, VarIntDecodeError};
///
/// fn main() -> Result<(), VarIntDecodeError> {
///     let decoded = decode::<u32>(&[185, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])?;
///     assert_eq!(decoded, (1337, 2));
///     Ok(())
/// }
/// ```
#[inline]
pub fn decode<T: VarIntTarget>(bytes: &[u8]) -> Result<(T, usize), VarIntDecodeError> {
    let result = if bytes.len() >= 16 {
        unsafe { decode_unsafe(bytes.as_ptr()) }
    } else if !bytes.is_empty() {
        let mut data = [0u8; 16];
        let len = min(16, bytes.len());
        // unsafe { core::ptr::copy_nonoverlapping(bytes.as_ptr(), data.as_mut_ptr(), len); }
        data[..len].copy_from_slice(&bytes[..len]);
        unsafe { decode_unsafe(data.as_ptr()) }
    } else {
        return Err(VarIntDecodeError::NotEnoughBytes);
    };

    // The ordering of conditions here is weird because of a performance regression (?) in rustc 1.49
    if bytes.len() >= T::MAX_VARINT_BYTES as usize
        // we perform a signed comparison here because a valid last byte is always positive
        && unsafe { *bytes.get_unchecked((T::MAX_VARINT_BYTES - 1) as usize) } > T::MAX_LAST_VARINT_BYTE
        && result.1 == T::MAX_VARINT_BYTES as usize
        || result.1 > T::MAX_VARINT_BYTES as usize
    {
        Err(VarIntDecodeError::Overflow)
    } else if result.1 > bytes.len() {
        Err(VarIntDecodeError::NotEnoughBytes)
    } else {
        Ok(result)
    }
}

/// Decodes only the length of a single variant from the input slice.
///
/// # Examples
/// ```
/// use varint_simd::{decode_len, VarIntDecodeError};
///
/// fn main() -> Result<(), VarIntDecodeError> {
///     let decoded = decode_len::<u32>(&[185, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])?;
///     assert_eq!(decoded, 2);
///     Ok(())
/// }
/// ```
#[inline]
pub fn decode_len<T: VarIntTarget>(bytes: &[u8]) -> Result<usize, VarIntDecodeError> {
    let result = if bytes.len() >= 16 {
        unsafe { decode_len_unsafe::<T>(bytes.as_ptr()) }
    } else if !bytes.is_empty() {
        let mut data = [0u8; 16];
        let len = min(16, bytes.len());
        // unsafe { core::ptr::copy_nonoverlapping(bytes.as_ptr(), data.as_mut_ptr(), len); }
        data[..len].copy_from_slice(&bytes[..len]);
        unsafe { decode_len_unsafe::<T>(data.as_ptr()) }
    } else {
        return Err(VarIntDecodeError::NotEnoughBytes);
    };

    Ok(result)
}

/// Convenience function for decoding a single varint in ZigZag format from the input slice.
/// See also: [`decode`]
///
/// # Examples
/// ```
/// use varint_simd::{decode_zigzag, VarIntDecodeError};
///
/// fn main() -> Result<(), VarIntDecodeError> {
///     let decoded = decode_zigzag::<i32>(&[39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])?;
///     assert_eq!(decoded, (-20, 1));
///     Ok(())
/// }
/// ```
#[inline]
pub fn decode_zigzag<T: SignedVarIntTarget>(bytes: &[u8]) -> Result<(T, usize), VarIntDecodeError> {
    decode::<T::Unsigned>(bytes).map(|r| (r.0.unzigzag(), r.1))
}

/// Decodes the length of the next integer
///
/// # Safety
/// Same as `decode_unsafe`
#[inline]
pub unsafe fn decode_len_unsafe<T: VarIntTarget>(bytes: *const u8) -> usize {
    if T::MAX_VARINT_BYTES <= 5 {
        let b = bytes.cast::<u64>().read_unaligned();
        let msbs = !b & !0x7f7f7f7f7f7f7f7f;
        let len = msbs.trailing_zeros() + 1; // in bits
        (len / 8) as usize
    } else {
        let b0 = bytes.cast::<u64>().read_unaligned();
        let b1 = bytes.cast::<u64>().add(1).read_unaligned();

        let msbs0 = !b0 & !0x7f7f7f7f7f7f7f7f;
        let msbs1 = !b1 & !0x7f7f7f7f7f7f7f7f;

        let len0 = msbs0.trailing_zeros() + 1;
        let len1 = msbs1.trailing_zeros() + 1;

        let len = if msbs0 == 0 { len1 + 64 } else { len0 };
        len as usize / 8
    }
}

/// Decodes a single varint from the input pointer. Returns a tuple containing the decoded number
/// and the number of bytes read.
///
/// # Safety
/// There must be at least 16 bytes of allocated memory after the beginning of the pointer.
/// Otherwise, there may be undefined behavior. Any data after the end of the varint are ignored.
/// A truncated value will be returned if the varint represents a number too large for the target
/// type.
///
/// You may prefer to use this unsafe interface if you know what you are doing and need a little
/// extra performance.
#[inline]
pub unsafe fn decode_unsafe<T: VarIntTarget>(bytes: *const u8) -> (T, usize) {
    // It looks like you're trying to understand what this code does. You should probably read
    // this first: https://developers.google.com/protocol-buffers/docs/encoding#varints

    if T::MAX_VARINT_BYTES <= 5 {
        // we can do everything in a normal 64-bit register
        let b = bytes.cast::<u64>().read_unaligned();
        // println!("{:#066b} b", b);

        // println!("{:#066b} op", !0x7f7f7f7f7f7f7f7fu64);
        let msbs = !b & !0x7f7f7f7f7f7f7f7f;
        // println!("{:#066b} msbs", msbs);
        /*
        TODO: theoretically, we could delay the `+1` and/or do it in parallel with other parts, but
         moving it downwards absolutely tanks performance and I have no idea why
        */
        let len = msbs.trailing_zeros() + 1; // in bits

        // println!("{}", len);

        // b & blsmsk(msbs)
        let varint_part = b & (msbs ^ msbs.wrapping_sub(1));
        // println!("{:#066b} varint_part", varint_part);

        let num = T::scalar_to_num(varint_part);

        (num, (len / 8) as usize)
    } else {
        let b0 = bytes.cast::<u64>().read_unaligned();
        let b1 = bytes.cast::<u64>().add(1).read_unaligned();

        let msbs0 = !b0 & !0x7f7f7f7f7f7f7f7f;
        let msbs1 = !b1 & !0x7f7f7f7f7f7f7f7f;

        // TODO: could this be faster on CPUs without fast tzcnt?
        // let blsi0 = msbs0.wrapping_neg() & msbs0;
        // let blsi1 = msbs1.wrapping_neg() & msbs1;
        //
        // let len0 = ((blsi0.wrapping_mul(0x20406080a0c0e1)) >> 60) & 15;
        // let len1 = ((blsi1.wrapping_mul(0x20406080a0c0e1)) >> 60) & 15;

        let len0 = msbs0.trailing_zeros() + 1;
        let len1 = msbs1.trailing_zeros() + 1;

        // doing this is faster than using len0, len1 because tzcnt has significant latency
        // and if the caller does not need the length, the call can be optimized out entirely
        // b0 & blsmsk(msbs0)
        let varint_part0 = b0 & (msbs0 ^ msbs0.wrapping_sub(1));
        // b1 & blsmsk(msbs1)
        let varint_part1 = (b1 & (msbs1 ^ msbs1.wrapping_sub(1))) * ((msbs0 == 0) as u64);

        // let varint_part0 = b0 & !(0xffffffffffffffff << len0.min(63));
        // let varint_part1 = b1 & !(0xffffffffffffffff << (((msbs0 == 0) as u32) * len1.min(63)));

        let num = T::vector_to_num(core::mem::transmute([varint_part0, varint_part1]));
        let len = if msbs0 == 0 { len1 + 64 } else { len0 } / 8;

        (num, len as usize)
    }
}

/// Decodes two adjacent varints simultaneously. Target types must fit within 16 bytes when varint
/// encoded. Requires SSSE3 support.
///
/// For example, it is permissible to decode `u32` and `u32`, and `u64` and `u32`, but it is not
/// possible to decode two `u64` values with this function simultaneously.
///
/// Returns a tuple containing the two decoded values and the two lengths of bytes read for each
/// value.
///
/// For best performance, ensure each target type is `u32` or smaller.
///
/// # Safety
/// There must be at least 16 bytes of allocated memory after the start of the pointer. Otherwise,
/// there may be undefined behavior. Any data after the two varints are ignored. Truncated values
/// will be returned if a varint exceeds the target type's limit.
#[inline]
#[cfg(any(target_feature = "ssse3", doc))]
#[cfg_attr(rustc_nightly, doc(cfg(target_feature = "ssse3")))]
pub unsafe fn decode_two_unsafe<T: VarIntTarget, U: VarIntTarget>(
    bytes: *const u8,
) -> (T, U, u8, u8) {
    if T::MAX_VARINT_BYTES + U::MAX_VARINT_BYTES > 16 {
        // check will be eliminated at compile time
        panic!(
            "exceeded length limit: cannot decode {} and {}, total length {} exceeds 16 bytes",
            core::any::type_name::<T>(),
            core::any::type_name::<U>(),
            T::MAX_VARINT_BYTES + U::MAX_VARINT_BYTES
        );
    }

    if T::MAX_VARINT_BYTES <= 5 && U::MAX_VARINT_BYTES <= 5 {
        // This will work with our lookup table, use that version
        return decode_two_u32_unsafe(bytes);
    }

    let b = _mm_loadu_si128(bytes as *const __m128i);

    // First find where the boundaries are
    let bitmask = _mm_movemask_epi8(b) as u32;

    // Find the number of bytes taken up by each varint
    let bm_not = !bitmask;
    let first_len = bm_not.trailing_zeros() + 1; // should compile to bsf or tzcnt
    let bm_not_2 = bm_not >> first_len;
    let second_len = bm_not_2.trailing_zeros() + 1;

    let ascend = _mm_setr_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);

    let first_len_vec = _mm_set1_epi8(first_len as i8);
    let first_mask = _mm_cmplt_epi8(ascend, first_len_vec);
    let first = _mm_and_si128(b, first_mask);

    let second_shuf = _mm_add_epi8(ascend, first_len_vec);
    let second_shuffled = _mm_shuffle_epi8(b, second_shuf);
    let second_mask = _mm_cmplt_epi8(ascend, _mm_set1_epi8(second_len as i8));
    let second = _mm_and_si128(second_shuffled, second_mask);

    let first_num;
    let second_num;

    // Only use "turbo" mode if the numbers fit in 64-bit lanes
    let should_turbo = T::MAX_VARINT_BYTES <= 8
        && U::MAX_VARINT_BYTES <= 8
        && cfg!(not(all(target_feature = "bmi2", very_fast_pdep)));
    if should_turbo {
        // const, so optimized out
        let comb = _mm_or_si128(first, _mm_bslli_si128(second, 8));

        let x = if T::MAX_VARINT_BYTES <= 2 && U::MAX_VARINT_BYTES <= 2 {
            dual_u8_stage2(comb)
        } else if T::MAX_VARINT_BYTES <= 3 && U::MAX_VARINT_BYTES <= 3 {
            dual_u16_stage2(comb)
        } else {
            dual_u32_stage2(comb)
        };

        let x: [u32; 4] = core::mem::transmute(x);
        // _mm_extract_epi32 requires SSE4.1
        first_num = T::cast_u32(x[0]);
        second_num = U::cast_u32(x[2]);
    } else {
        first_num = T::vector_to_num(core::mem::transmute(first));
        second_num = U::vector_to_num(core::mem::transmute(second));
    }

    (first_num, second_num, first_len as u8, second_len as u8)
}

#[inline]
#[cfg(any(target_feature = "ssse3", doc))]
unsafe fn decode_two_u32_unsafe<T: VarIntTarget, U: VarIntTarget>(
    bytes: *const u8,
) -> (T, U, u8, u8) {
    let b = _mm_loadu_si128(bytes as *const __m128i);

    // Get the movemask and mask out irrelevant parts
    let bitmask = _mm_movemask_epi8(b) as u32 & 0b1111111111;

    // Use lookup table to get the shuffle mask
    let (lookup, first_len, second_len) =
        *lookup::LOOKUP_DOUBLE_STEP1.get_unchecked(bitmask as usize);
    let shuf = *lookup::LOOKUP_DOUBLE_VEC.get_unchecked(lookup as usize);

    let comb = _mm_shuffle_epi8(b, shuf);

    let first_num;
    let second_num;

    // Only use "turbo" mode if PDEP/PEXT are not faster
    let should_turbo = cfg!(not(all(target_feature = "bmi2", very_fast_pdep)));
    if should_turbo {
        // const, so optimized out

        let x = if T::MAX_VARINT_BYTES <= 2 && U::MAX_VARINT_BYTES <= 2 {
            dual_u8_stage2(comb)
        } else if T::MAX_VARINT_BYTES <= 3 && U::MAX_VARINT_BYTES <= 3 {
            dual_u16_stage2(comb)
        } else {
            dual_u32_stage2(comb)
        };

        let x: [u32; 4] = core::mem::transmute(x);
        // _mm_extract_epi32 requires SSE4.1
        first_num = T::cast_u32(x[0]);
        second_num = U::cast_u32(x[2]);
    } else {
        first_num = T::vector_to_num(core::mem::transmute(comb));
        second_num = U::vector_to_num(core::mem::transmute(_mm_bsrli_si128(comb, 8)));
    }

    (first_num, second_num, first_len, second_len)
}

#[inline(always)]
unsafe fn dual_u8_stage2(comb: __m128i) -> __m128i {
    _mm_or_si128(
        _mm_and_si128(comb, _mm_set_epi64x(0x000000000000007f, 0x000000000000007f)),
        _mm_srli_epi64(
            _mm_and_si128(comb, _mm_set_epi64x(0x0000000000000100, 0x0000000000000100)),
            1,
        ),
    )
}

#[inline(always)]
unsafe fn dual_u16_stage2(comb: __m128i) -> __m128i {
    _mm_or_si128(
        _mm_or_si128(
            _mm_and_si128(comb, _mm_set_epi64x(0x000000000000007f, 0x000000000000007f)),
            _mm_srli_epi64(
                _mm_and_si128(comb, _mm_set_epi64x(0x0000000000030000, 0x0000000000030000)),
                2,
            ),
        ),
        _mm_srli_epi64(
            _mm_and_si128(comb, _mm_set_epi64x(0x0000000000007f00, 0x0000000000007f00)),
            1,
        ),
    )
}

#[inline(always)]
unsafe fn dual_u32_stage2(comb: __m128i) -> __m128i {
    _mm_or_si128(
        _mm_or_si128(
            _mm_and_si128(comb, _mm_set_epi64x(0x000000000000007f, 0x000000000000007f)),
            _mm_srli_epi64(
                _mm_and_si128(comb, _mm_set_epi64x(0x0000000f00000000, 0x0000000f00000000)),
                4,
            ),
        ),
        _mm_or_si128(
            _mm_or_si128(
                _mm_srli_epi64(
                    _mm_and_si128(comb, _mm_set_epi64x(0x000000007f000000, 0x000000007f000000)),
                    3,
                ),
                _mm_srli_epi64(
                    _mm_and_si128(comb, _mm_set_epi64x(0x00000000007f0000, 0x00000000007f0000)),
                    2,
                ),
            ),
            _mm_srli_epi64(
                _mm_and_si128(comb, _mm_set_epi64x(0x0000000000007f00, 0x0000000000007f00)),
                1,
            ),
        ),
    )
}

/// **Experimental. May have relatively poor performance.** Decode two adjacent varints
/// simultaneously from the input pointer. Requires AVX2. Allows for decoding a pair of `u64`
/// values. For smaller values, the non-wide variation of this function will probably be faster.
///
/// Returns a tuple containing the two decoded values and the two lengths of bytes read for each
/// value.
///
/// # Safety
/// There must be at least 32 bytes of allocated memory after the beginning of the pointer.
/// Otherwise, there may be undefined behavior. Calling code should ensure that AVX2 is supported
/// before referencing this function.
#[inline]
#[cfg(any(target_feature = "avx2", doc))]
#[cfg_attr(rustc_nightly, doc(cfg(target_feature = "avx2")))]
pub unsafe fn decode_two_wide_unsafe<T: VarIntTarget, U: VarIntTarget>(
    bytes: *const u8,
) -> (T, U, u8, u8) {
    let b = _mm256_loadu_si256(bytes as *const __m256i);

    // Get the most significant bits
    let bitmask = _mm256_movemask_epi8(b) as u32;

    // Find the number of bytes taken up by each varint
    let bm_not = !bitmask;
    let first_len = bm_not.trailing_zeros() + 1; // should compile to bsf or tzcnt
    let bm_not_2 = bm_not >> first_len;
    let second_len = bm_not_2.trailing_zeros() + 1;

    // Create and parse vector consisting solely of the first varint
    let ascend = _mm_setr_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
    let first_mask = _mm_cmplt_epi8(ascend, _mm_set1_epi8(first_len as i8));
    let first = _mm_and_si128(_mm256_extracti128_si256(b, 0), first_mask);

    // The second is much more tricky.
    let shuf_gen = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
        12, 13, 14, 15,
    );

    // Rearrange each 128-bit lane such that ORing them together results in the window of data we want)
    let shuf_add = _mm256_set_m128i(
        _mm_set1_epi8(-(16i8 - first_len as i8)),
        _mm_set1_epi8(first_len as i8),
    );
    let shuf_added = _mm256_add_epi8(shuf_gen, shuf_add);
    let shuf = _mm256_or_si256(
        shuf_added,
        _mm256_cmpgt_epi8(shuf_added, _mm256_set1_epi8(15)), // TODO: Is this really necessary?
    );
    let shuffled = _mm256_shuffle_epi8(b, shuf);

    // OR the halves together, and now we have a view of the second varint
    let second_shifted = _mm_or_si128(
        _mm256_extracti128_si256(shuffled, 0),
        _mm256_extracti128_si256(shuffled, 1),
    );
    let second_mask = _mm_cmplt_epi8(ascend, _mm_set1_epi8(second_len as i8));
    let second = _mm_and_si128(second_shifted, second_mask);

    let first_num;
    let second_num;

    // PEXT on the two halves is still slower, at least on Coffee Lake and Broadwell
    let should_turbo = true;
    if should_turbo {
        // Decode the two halves in parallel using SSE2
        let comb_lo = _mm_unpacklo_epi64(first, second);
        let x_lo = _mm_or_si128(
            _mm_or_si128(
                _mm_or_si128(
                    _mm_and_si128(comb_lo, _mm_set1_epi64x(0x000000000000007f)),
                    _mm_srli_epi64(
                        _mm_and_si128(comb_lo, _mm_set1_epi64x(0x7f00000000000000)),
                        7,
                    ),
                ),
                _mm_or_si128(
                    _mm_srli_epi64(
                        _mm_and_si128(comb_lo, _mm_set1_epi64x(0x007f000000000000)),
                        6,
                    ),
                    _mm_srli_epi64(
                        _mm_and_si128(comb_lo, _mm_set1_epi64x(0x00007f0000000000)),
                        5,
                    ),
                ),
            ),
            _mm_or_si128(
                _mm_or_si128(
                    _mm_srli_epi64(
                        _mm_and_si128(comb_lo, _mm_set1_epi64x(0x0000007f00000000)),
                        4,
                    ),
                    _mm_srli_epi64(
                        _mm_and_si128(comb_lo, _mm_set1_epi64x(0x000000007f000000)),
                        3,
                    ),
                ),
                _mm_or_si128(
                    _mm_srli_epi64(
                        _mm_and_si128(comb_lo, _mm_set1_epi64x(0x00000000007f0000)),
                        2,
                    ),
                    _mm_srli_epi64(
                        _mm_and_si128(comb_lo, _mm_set1_epi64x(0x0000000000007f00)),
                        1,
                    ),
                ),
            ),
        );

        let comb_hi = _mm_unpackhi_epi64(first, second);
        let x_hi = _mm_or_si128(
            _mm_slli_epi64(
                _mm_and_si128(comb_hi, _mm_set1_epi64x(0x0000000000000100)),
                55,
            ),
            _mm_slli_epi64(
                _mm_and_si128(comb_hi, _mm_set1_epi64x(0x000000000000007f)),
                56,
            ),
        );

        let x = _mm_or_si128(x_lo, x_hi);

        first_num = T::cast_u64(_mm_extract_epi64(x, 0) as u64);
        second_num = U::cast_u64(_mm_extract_epi64(x, 1) as u64);
    } else {
        first_num = T::vector_to_num(core::mem::transmute(first));
        second_num = U::vector_to_num(core::mem::transmute(second));
    }

    (first_num, second_num, first_len as u8, second_len as u8)
}

/// Decodes four adjacent varints simultaneously. Target types must fit within 16 bytes when varint
/// encoded. Requires SSSE3 support.
///
/// Returns a tuple containing the four encoded values, followed by the number of bytes read for
/// each encoded value, followed by a boolean indicator for whether the length values may be
/// incorrect due to overflow.
///
/// For best performance, ensure each target type is `u16` or smaller.
///
/// # Safety
/// There must be at least 16 bytes of allocated memory after the start of the pointer. Otherwise,
/// there may be undefined behavior. Any data after the four varints are ignored. Truncated values
/// will be returned if a varint exceeds the target type's limit.
#[inline]
#[cfg(any(target_feature = "ssse3", doc))]
#[cfg_attr(rustc_nightly, doc(cfg(target_feature = "ssse3")))]
pub unsafe fn decode_four_unsafe<
    T: VarIntTarget,
    U: VarIntTarget,
    V: VarIntTarget,
    W: VarIntTarget,
>(
    bytes: *const u8,
) -> (T, U, V, W, u8, u8, u8, u8, bool) {
    if T::MAX_VARINT_BYTES + U::MAX_VARINT_BYTES + V::MAX_VARINT_BYTES + W::MAX_VARINT_BYTES > 16 {
        // check will be eliminated at compile time
        panic!(
            "exceeded length limit: cannot decode {}, {}, {}, and {}, total length {} exceeds 16 bytes",
            core::any::type_name::<T>(),
            core::any::type_name::<U>(),
            core::any::type_name::<V>(),
            core::any::type_name::<W>(),
            T::MAX_VARINT_BYTES + U::MAX_VARINT_BYTES + V::MAX_VARINT_BYTES + W::MAX_VARINT_BYTES
        );
    }

    if T::MAX_VARINT_BYTES <= 3
        && U::MAX_VARINT_BYTES <= 3
        && V::MAX_VARINT_BYTES <= 3
        && W::MAX_VARINT_BYTES <= 3
    {
        return decode_four_u16_unsafe(bytes);
    }

    let b = _mm_loadu_si128(bytes as *const __m128i);

    // First find where the boundaries are
    let bitmask = _mm_movemask_epi8(b) as u32;

    // Find the number of bytes taken up by each varint
    let bm_not = !bitmask;
    let first_len = bm_not.trailing_zeros() + 1; // should compile to bsf or tzcnt
    let bm_not_2 = bm_not >> first_len;
    let second_len = bm_not_2.trailing_zeros() + 1;
    let bm_not_3 = bm_not_2 >> second_len;
    let third_len = bm_not_3.trailing_zeros() + 1;
    let bm_not_4 = bm_not_3 >> third_len;
    let fourth_len = bm_not_4.trailing_zeros() + 1;

    let ascend = _mm_setr_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);

    let first_len_vec = _mm_set1_epi8(first_len as i8);
    let first_mask = _mm_cmplt_epi8(ascend, first_len_vec);
    let first = _mm_and_si128(b, first_mask);

    let second_shuf = _mm_add_epi8(ascend, first_len_vec);
    let second_shuffled = _mm_shuffle_epi8(b, second_shuf);
    let second_len_vec = _mm_set1_epi8(second_len as i8);
    let second_mask = _mm_cmplt_epi8(ascend, second_len_vec);
    let second = _mm_and_si128(second_shuffled, second_mask);

    let third_shuf = _mm_add_epi8(ascend, second_len_vec);
    let third_shuffled = _mm_shuffle_epi8(second_shuffled, third_shuf);
    let third_len_vec = _mm_set1_epi8(third_len as i8);
    let third_mask = _mm_cmplt_epi8(ascend, third_len_vec);
    let third = _mm_and_si128(third_shuffled, third_mask);

    let fourth_shuf = _mm_add_epi8(ascend, third_len_vec);
    let fourth_shuffled = _mm_shuffle_epi8(third_shuffled, fourth_shuf);
    let fourth_len_vec = _mm_set1_epi8(fourth_len as i8);
    let fourth_mask = _mm_cmplt_epi8(ascend, fourth_len_vec);
    let fourth = _mm_and_si128(fourth_shuffled, fourth_mask);

    let first_num;
    let second_num;
    let third_num;
    let fourth_num;

    // Only use "turbo" mode if the numbers fit in 64-bit lanes
    let should_turbo = T::MAX_VARINT_BYTES <= 4
        && U::MAX_VARINT_BYTES <= 4
        && V::MAX_VARINT_BYTES <= 4
        && W::MAX_VARINT_BYTES <= 4
        // PDEP/PEXT are still a little faster here
        && cfg!(not(all(
            target_feature = "bmi2",
            very_fast_pdep
        )));
    if should_turbo {
        // const, so optimized out
        let comb = _mm_or_si128(
            _mm_or_si128(first, _mm_bslli_si128(second, 4)),
            _mm_or_si128(_mm_bslli_si128(third, 8), _mm_bslli_si128(fourth, 12)),
        );

        let x = if T::MAX_VARINT_BYTES <= 2
            && U::MAX_VARINT_BYTES <= 2
            && V::MAX_VARINT_BYTES <= 2
            && W::MAX_VARINT_BYTES <= 2
        {
            _mm_or_si128(
                _mm_and_si128(comb, _mm_set1_epi32(0x0000007f)),
                _mm_srli_epi32(_mm_and_si128(comb, _mm_set1_epi32(0x00000100)), 1),
            )
        } else {
            _mm_or_si128(
                _mm_or_si128(
                    _mm_and_si128(comb, _mm_set1_epi32(0x0000007f)),
                    _mm_srli_epi32(_mm_and_si128(comb, _mm_set1_epi32(0x00030000)), 2),
                ),
                _mm_srli_epi32(_mm_and_si128(comb, _mm_set1_epi32(0x00007f00)), 1),
            )
        };

        let x: [u32; 4] = core::mem::transmute(x);
        // _mm_extract_epi32 requires SSE4.1
        first_num = T::cast_u32(x[0]);
        second_num = U::cast_u32(x[1]);
        third_num = V::cast_u32(x[2]);
        fourth_num = W::cast_u32(x[3]);
    } else {
        first_num = T::vector_to_num(core::mem::transmute(first));
        second_num = U::vector_to_num(core::mem::transmute(second));
        third_num = V::vector_to_num(core::mem::transmute(third));
        fourth_num = W::vector_to_num(core::mem::transmute(fourth));
    }

    (
        first_num,
        second_num,
        third_num,
        fourth_num,
        first_len as u8,
        second_len as u8,
        third_len as u8,
        fourth_len as u8,
        false,
    )
}

#[inline]
#[cfg(any(target_feature = "ssse3", doc))]
#[cfg_attr(rustc_nightly, doc(cfg(target_feature = "ssse3")))]
unsafe fn decode_four_u16_unsafe<
    T: VarIntTarget,
    U: VarIntTarget,
    V: VarIntTarget,
    W: VarIntTarget,
>(
    bytes: *const u8,
) -> (T, U, V, W, u8, u8, u8, u8, bool) {
    let b = _mm_loadu_si128(bytes as *const __m128i);

    // First find where the boundaries are
    let bitmask = _mm_movemask_epi8(b) as u32;

    // Use the lookup table
    let lookup = *lookup::LOOKUP_QUAD_STEP1.get_unchecked((bitmask & 0b111111111111) as usize);

    // Fetch the shuffle mask
    let shuf = *lookup::LOOKUP_QUAD_VEC.get_unchecked((lookup & 0b11111111) as usize);

    // Extract the lengths while we're waiting
    let first_len = (lookup >> 8) & 0b1111;
    let second_len = (lookup >> 12) & 0b1111;
    let third_len = (lookup >> 16) & 0b1111;
    let fourth_len = (lookup >> 20) & 0b1111;

    let comb = _mm_shuffle_epi8(b, shuf);

    let invalid = lookup >> 31;

    let first_num;
    let second_num;
    let third_num;
    let fourth_num;

    // PDEP/PEXT may be still a little faster here
    let should_turbo = cfg!(not(all(target_feature = "bmi2", very_fast_pdep)));
    if should_turbo {
        // const, so optimized out

        let x = if T::MAX_VARINT_BYTES <= 2
            && U::MAX_VARINT_BYTES <= 2
            && V::MAX_VARINT_BYTES <= 2
            && W::MAX_VARINT_BYTES <= 2
        {
            _mm_or_si128(
                _mm_and_si128(comb, _mm_set1_epi32(0x0000007f)),
                _mm_srli_epi32(_mm_and_si128(comb, _mm_set1_epi32(0x00000100)), 1),
            )
        } else {
            _mm_or_si128(
                _mm_or_si128(
                    _mm_and_si128(comb, _mm_set1_epi32(0x0000007f)),
                    _mm_srli_epi32(_mm_and_si128(comb, _mm_set1_epi32(0x00030000)), 2),
                ),
                _mm_srli_epi32(_mm_and_si128(comb, _mm_set1_epi32(0x00007f00)), 1),
            )
        };

        let x: [u32; 4] = core::mem::transmute(x);
        // _mm_extract_epi32 requires SSE4.1
        first_num = T::cast_u32(x[0]);
        second_num = U::cast_u32(x[1]);
        third_num = V::cast_u32(x[2]);
        fourth_num = W::cast_u32(x[3]);
    } else {
        first_num = T::vector_to_num(core::mem::transmute(comb));
        second_num = U::vector_to_num(core::mem::transmute(_mm_bsrli_si128(comb, 4)));
        third_num = V::vector_to_num(core::mem::transmute(_mm_bsrli_si128(comb, 8)));
        fourth_num = W::vector_to_num(core::mem::transmute(_mm_bsrli_si128(comb, 12)));
    }

    (
        first_num,
        second_num,
        third_num,
        fourth_num,
        first_len as u8,
        second_len as u8,
        third_len as u8,
        fourth_len as u8,
        invalid != 0,
    )
}

/// Decodes four adjacent varints into u8's simultaneously. Requires SSSE3 support. **Does not
/// perform overflow checking and may produce incorrect output.**
///
/// Returns a tuple containing an array of decoded values, and the total number of bytes read.
///
/// # Safety
/// There must be at least 16 bytes of allocated memory after the start of the pointer. Otherwise,
/// there may be undefined behavior. Truncated values will be returned if the varint represents
/// a number larger than what a u8 can handle.
///
/// This function does not perform overflow checking. If a varint exceeds two bytes in encoded
/// length, it may be interpreted as multiple varints, and the reported length of data read will
/// be shorter than expected. Caution is encouraged when using this function.
#[inline]
#[cfg(any(target_feature = "ssse3", doc))]
#[cfg_attr(rustc_nightly, doc(cfg(target_feature = "ssse3")))]
pub unsafe fn decode_eight_u8_unsafe(bytes: *const u8) -> ([u8; 8], u8) {
    let b = _mm_loadu_si128(bytes as *const __m128i);

    let ones = _mm_set1_epi8(1);
    let mut lens = _mm_setzero_si128();
    let mut shift = _mm_and_si128(_mm_cmplt_epi8(b, _mm_setzero_si128()), ones);
    let ascend = _mm_setr_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
    let asc_one = _mm_setr_epi8(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
    let mut window_small = _mm_setr_epi8(1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);

    let broadcast_mask = _mm_setzero_si128();

    // if the first byte is zero, shift down by 1, if the first byte is one, shift down by 2
    // 0
    let first_byte = _mm_shuffle_epi8(shift, broadcast_mask);
    shift = _mm_shuffle_epi8(shift, _mm_add_epi8(asc_one, first_byte));
    lens = _mm_or_si128(lens, _mm_and_si128(first_byte, window_small));
    window_small = _mm_bslli_si128(window_small, 1);

    // 1
    let first_byte = _mm_shuffle_epi8(shift, broadcast_mask);
    shift = _mm_shuffle_epi8(shift, _mm_add_epi8(asc_one, first_byte));
    lens = _mm_or_si128(lens, _mm_and_si128(first_byte, window_small));
    window_small = _mm_bslli_si128(window_small, 1);

    // 2
    let first_byte = _mm_shuffle_epi8(shift, broadcast_mask);
    shift = _mm_shuffle_epi8(shift, _mm_add_epi8(asc_one, first_byte));
    lens = _mm_or_si128(lens, _mm_and_si128(first_byte, window_small));
    window_small = _mm_bslli_si128(window_small, 1);

    // 3
    let first_byte = _mm_shuffle_epi8(shift, broadcast_mask);
    shift = _mm_shuffle_epi8(shift, _mm_add_epi8(asc_one, first_byte));
    lens = _mm_or_si128(lens, _mm_and_si128(first_byte, window_small));
    window_small = _mm_bslli_si128(window_small, 1);

    // 4
    let first_byte = _mm_shuffle_epi8(shift, broadcast_mask);
    shift = _mm_shuffle_epi8(shift, _mm_add_epi8(asc_one, first_byte));
    lens = _mm_or_si128(lens, _mm_and_si128(first_byte, window_small));
    window_small = _mm_bslli_si128(window_small, 1);

    // 5
    let first_byte = _mm_shuffle_epi8(shift, broadcast_mask);
    shift = _mm_shuffle_epi8(shift, _mm_add_epi8(asc_one, first_byte));
    lens = _mm_or_si128(lens, _mm_and_si128(first_byte, window_small));
    window_small = _mm_bslli_si128(window_small, 1);

    // 6
    let first_byte = _mm_shuffle_epi8(shift, broadcast_mask);
    shift = _mm_shuffle_epi8(shift, _mm_add_epi8(asc_one, first_byte));
    lens = _mm_or_si128(lens, _mm_and_si128(first_byte, window_small));
    window_small = _mm_bslli_si128(window_small, 1);

    // 7
    let first_byte = _mm_shuffle_epi8(shift, broadcast_mask);
    // shift = _mm_shuffle_epi8(shift, _mm_add_epi8(asc_one, first_byte));
    lens = _mm_or_si128(lens, _mm_and_si128(first_byte, window_small));
    // window_small = _mm_bslli_si128(window_small, 1);

    // Construct the shuffle

    let lens_invert = _mm_sub_epi8(ones, lens);
    let mut cumul_lens = _mm_add_epi8(lens_invert, _mm_bslli_si128(lens_invert, 1));
    cumul_lens = _mm_add_epi8(cumul_lens, _mm_bslli_si128(cumul_lens, 2));
    cumul_lens = _mm_add_epi8(cumul_lens, _mm_bslli_si128(cumul_lens, 4));
    cumul_lens = _mm_add_epi8(cumul_lens, _mm_bslli_si128(cumul_lens, 8));

    let cumul_lens_2: [u8; 16] = core::mem::transmute(cumul_lens);
    let last_len = 8 - cumul_lens_2[7] + 8;

    // Set one-lengthed second bytes to negative
    let second = _mm_shuffle_epi8(
        _mm_add_epi8(lens, ones),
        _mm_setr_epi8(-1, 0, -1, 1, -1, 2, -1, 3, -1, 4, -1, 5, -1, 6, -1, 7),
    );

    let shuf_pt1 = _mm_or_si128(ascend, _mm_cmpeq_epi8(second, ones));

    // Subtract the cumulative sum of zero-lengths to adjust the indexes
    let x_shuf = _mm_shuffle_epi8(
        _mm_bslli_si128(cumul_lens, 1),
        _mm_setr_epi8(0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7),
    );

    let shuf = _mm_sub_epi8(shuf_pt1, x_shuf);
    let comb = _mm_shuffle_epi8(b, shuf);

    let x = _mm_or_si128(
        _mm_and_si128(comb, _mm_set1_epi16(0x0000007f)),
        _mm_srli_epi16(_mm_and_si128(comb, _mm_set1_epi16(0x00000100)), 1),
    );

    let shuf = _mm_shuffle_epi8(
        x,
        _mm_setr_epi8(0, 2, 4, 6, 8, 10, 12, 14, -1, -1, -1, -1, -1, -1, -1, -1),
    );
    let lower: [u64; 2] = core::mem::transmute(shuf);
    let nums = lower[0].to_ne_bytes();

    (nums, last_len)
}
