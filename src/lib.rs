#[cfg(target_arch = "x86")]
use core::arch::x86 as arch;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::cmp::min;
use std::fmt::Debug;

// Functions to help with debugging
fn slice_m128i(n: __m128i) -> [i8; 16] {
    unsafe { std::mem::transmute(n) }
}

fn slice_m256i(n: __m256i) -> [i8; 32] {
    unsafe { std::mem::transmute(n) }
}

/// Represents a scalar value that can be encoded to and decoded from a varint.
pub trait VarIntTarget: Debug + Eq + PartialEq + Sized + Copy {
    /// The maximum length of varint that is necessary to represent this number
    const MAX_VARINT_BYTES: u8;

    /// Converts a 128-bit vector to this number
    fn vector_to_num(res: [u8; 16]) -> Self;

    /// Splits this number into 7-bit segments for encoding
    fn num_to_vector_stage1(self) -> [u8; 16];
}

impl VarIntTarget for u8 {
    const MAX_VARINT_BYTES: u8 = 2;

    #[inline(always)]
    fn vector_to_num(res: [u8; 16]) -> Self {
        (res[0] as u8) | ((res[1] as u8) << 7)
    }

    #[inline(always)]
    fn num_to_vector_stage1(self) -> [u8; 16] {
        let mut res = [0u8; 16];
        res[0] = self & 127;
        res[1] = (self >> 7) & 127;

        res
    }
}

impl VarIntTarget for u16 {
    const MAX_VARINT_BYTES: u8 = 3;

    #[inline(always)]
    fn vector_to_num(res: [u8; 16]) -> Self {
        (res[0] as u16) | ((res[1] as u16) << 7) | ((res[2] as u16) << 2 * 7)
    }

    #[inline(always)]
    fn num_to_vector_stage1(self) -> [u8; 16] {
        let mut res = [0u8; 16];
        res[0] = self as u8 & 127;
        res[1] = (self >> 7) as u8 & 127;
        res[2] = (self >> 2 * 7) as u8 & 127;

        res
    }
}

impl VarIntTarget for u32 {
    const MAX_VARINT_BYTES: u8 = 5;

    #[inline(always)]
    fn vector_to_num(res: [u8; 16]) -> Self {
        (res[0] as u32)
            | ((res[1] as u32) << 7)
            | ((res[2] as u32) << 2 * 7)
            | ((res[3] as u32) << 3 * 7)
            | ((res[4] as u32) << 4 * 7)
    }

    #[inline(always)]
    fn num_to_vector_stage1(self) -> [u8; 16] {
        let mut res = [0u8; 16];
        res[0] = self as u8 & 127;
        res[1] = (self >> 7) as u8 & 127;
        res[2] = (self >> 2 * 7) as u8 & 127;
        res[3] = (self >> 3 * 7) as u8 & 127;
        res[4] = (self >> 4 * 7) as u8 & 127;

        res
    }
}

impl VarIntTarget for u64 {
    const MAX_VARINT_BYTES: u8 = 10;

    #[inline(always)]
    fn vector_to_num(res: [u8; 16]) -> Self {
        // This line should be auto-vectorized when compiling for AVX2-capable processors
        // TODO: Find out a way to make these run faster on older processors
        (res[0] as u64)
            | ((res[1] as u64) << 7)
            | ((res[2] as u64) << 2 * 7)
            | ((res[3] as u64) << 3 * 7)
            | ((res[4] as u64) << 4 * 7)
            | ((res[5] as u64) << 5 * 7)
            | ((res[6] as u64) << 6 * 7)
            | ((res[7] as u64) << 7 * 7)
            | ((res[8] as u64) << 8 * 7)
            | ((res[9] as u64) << 9 * 7)
    }

    #[inline(always)]
    fn num_to_vector_stage1(self) -> [u8; 16] {
        let mut res = [0u8; 16];
        res[0] = self as u8 & 127;
        res[1] = (self >> 7) as u8 & 127;
        res[2] = (self >> 2 * 7) as u8 & 127;
        res[3] = (self >> 3 * 7) as u8 & 127;
        res[4] = (self >> 4 * 7) as u8 & 127;
        res[5] = (self >> 5 * 7) as u8 & 127;
        res[6] = (self >> 6 * 7) as u8 & 127;
        res[7] = (self >> 7 * 7) as u8 & 127;
        res[8] = (self >> 8 * 7) as u8 & 127;
        res[9] = (self >> 9 * 7) as u8 & 127;

        res
    }
}

#[derive(Debug)]
pub enum VarIntDecodeError {
    Overflow,
    NotEnoughBytes,
}

impl std::fmt::Display for VarIntDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for VarIntDecodeError {}

/// Decodes a single varint from the input slice. Requires SSSE3 support. For best performance,
/// provide a slice at least 16 bytes in length, or use the unsafe version directly.
#[inline]
pub fn decode<T: VarIntTarget>(bytes: &[u8]) -> Result<(T, u8), VarIntDecodeError> {
    let result = if bytes.len() >= 16 {
        unsafe { decode_unsafe(bytes) }
    } else if bytes.len() >= 1 {
        let mut data = [0u8;16];
        let len = min(10, bytes.len());
        data[..len].copy_from_slice(&bytes[..len]);
        unsafe { decode_unsafe(&data) }
    } else {
        return Err(VarIntDecodeError::NotEnoughBytes);
    };

    if result.1 > T::MAX_VARINT_BYTES {
        Err(VarIntDecodeError::Overflow)
    } else {
        Ok(result)
    }
}

/// Decodes a single varint from the input slice. Requires SSSE3 support.
///
/// There must be at least 16 bytes of allocated memory after the beginning of the pointer.
/// Otherwise, there may be undefined behavior. Any data after the end of the varint is ignored.
/// Behavior is undefined if the varint represents a number too large for the target type.
///
/// You may prefer to use this unsafe interface if you know what you are doing and need a little
/// extra performance.
#[inline]
pub unsafe fn decode_unsafe<T: VarIntTarget>(bytes: &[u8]) -> (T, u8) {
    // It looks like you're trying to understand what this code does. You should probably read
    // this first: https://developers.google.com/protocol-buffers/docs/encoding#varints

    let b = _mm_loadu_si128(bytes.as_ptr() as *const __m128i);

    // Get the most significant bits of each byte
    let bitmask: i32 = _mm_movemask_epi8(b);

    // A zero most significant bit indicates the end of a varint
    // Find the end, and mask out everything afterwards
    let bm_not = !bitmask;
    let cleaned = (bm_not - 1) ^ bm_not; // blsmsk equivalent

    // Expand the bitmask into full bytes to mask out the input
    // Goal: for each bit that is 1 in the bitmask, the corresponding byte should be 0xFF
    // Place the bitmask into a vector
    let bc = _mm_set_epi32(0, 0, 0, cleaned as i32);

    // Use a shuffle operation to distribute the upper half of the bitmask to the upper parts of the
    // vector, and the lower half to the lower parts of the vector
    let shuffle = _mm_set_epi64x(0x0101010101010101, 0x0000000000000000);
    let shuffled = _mm_shuffle_epi8(bc, shuffle);

    // Mask out the irrelevant bits in each byte, such that the only bit that should remain on
    // in each byte is the bit from the bitmask that corresponds to the byte
    let mask = _mm_set_epi8(128u8 as i8, 64, 32, 16, 8, 4, 2, 1, 128u8 as i8, 64, 32, 16, 8, 4, 2, 1);
    let t = _mm_and_si128(shuffled, mask);

    // Expand out the set bits into full 0xFF values
    let fat_mask = _mm_cmpeq_epi8(mask, t);

    // Mask out the irrelevant bytes in the input
    let varint_part = _mm_and_si128(fat_mask, b);

    // Turn off the most significant bits
    let msb_masked = _mm_and_si128(
        varint_part,
        _mm_set_epi8(0, 0, 0, 0, 0, 0, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127),
    );

    // Turn the vector into a scalar value by concatenating the 7-bit values
    let res: [u8; 16] = std::mem::transmute(msb_masked);
    let num = T::vector_to_num(res); // specialized functions for different number sizes

    // Count the number of bytes we actually read
    let bytes_read = cleaned.count_ones() as u8; // popcnt on supported CPUs

    (num, bytes_read)
}

/// **Experimental.** Decodes three adjacent varints from the given pointer simultaneously.
/// This currently runs much slower than a scalar or hybrid implementation. Requires AVX2 support.
///
/// There must be at least 32 bytes of memory allocated after the beginning of the pointer.
/// Otherwise, there may be undefined behavior.
#[inline]
#[cfg(target_feature = "avx2")]
pub unsafe fn decode_three_unsafe<T: VarIntTarget, U: VarIntTarget, V: VarIntTarget>(bytes: &[u8]) -> (T, u8, U, u8, V, u8) {
    let b = _mm256_loadu_si256(bytes.as_ptr() as *const __m256i);

    // Get the most significant bits
    let bitmask = _mm256_movemask_epi8(b) as u32;

    // Find the number of bytes taken up by each varint
    let bm_not = !bitmask;
    let first_len = bm_not.trailing_zeros() + 1; // should compile to bsf or tzcnt (?), verify
    let bm_not_2 = bm_not >> first_len;
    let second_len = bm_not_2.trailing_zeros() + 1;
    let bm_not_3 = bm_not_2 >> second_len;
    let third_len = bm_not_3.trailing_zeros() + 1;

    // println!("{} {} {}", first_len, second_len, third_len);

    // Create and parse vector consisting solely of the first varint
    let ascend = _mm_setr_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
    let first_mask = _mm_cmplt_epi8(ascend, _mm_set1_epi8(first_len as i8));
    let first = _mm_and_si128(_mm256_extracti128_si256(b, 0), first_mask);
    // println!("{:?}", slice_m128i(first));

    let msb_mask = _mm_set_epi8(0, 0, 0, 0, 0, 0, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127);
    let first_msb = _mm_and_si128(msb_mask, first);
    let first_result = T::vector_to_num(std::mem::transmute(first_msb));

    // The second and third are much more tricky.
    let shuf_gen = _mm256_setr_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);

    // Rearrange each 128-bit lane such that ORing them together results in the window of data we want)
    let shuf_add = _mm256_set_m128i(_mm_set1_epi8(-(16i8 - first_len as i8)), _mm_set1_epi8(first_len as i8));
    let shuf_added = _mm256_add_epi8(shuf_gen, shuf_add);
    let shuf = _mm256_or_si256(shuf_added, _mm256_cmpgt_epi8(shuf_added, _mm256_set1_epi8(15)));
    let shuffled = _mm256_shuffle_epi8(b, shuf);

    // OR the halves together, and now we have a view of the second varint
    let second_shifted = _mm_or_si128(_mm256_extracti128_si256(shuffled, 0), _mm256_extracti128_si256(shuffled, 1));
    let second_mask = _mm_cmplt_epi8(ascend, _mm_set1_epi8(second_len as i8));
    let second = _mm_and_si128(second_shifted, second_mask);
    // println!("second {:?}", slice_m128i(second));

    // Mask out the MSB, and we're done
    let second_msb = _mm_and_si128(msb_mask, second);
    let second_result = U::vector_to_num(std::mem::transmute(second_msb));

    // The third is done similarly
    let shuf_add = _mm256_set_m128i(_mm_set1_epi8(-(16i8 - (first_len + second_len) as i8)), _mm_set1_epi8((first_len + second_len) as i8));
    let shuf_added = _mm256_add_epi8(shuf_gen, shuf_add);
    let shuf = _mm256_or_si256(shuf_added, _mm256_cmpgt_epi8(shuf_added, _mm256_set1_epi8(15)));
    let shuffled = _mm256_shuffle_epi8(b, shuf);

    let third_shifted = _mm_or_si128(_mm256_extracti128_si256(shuffled, 0), _mm256_extracti128_si256(shuffled, 1));
    let third_mask = _mm_cmplt_epi8(ascend, _mm_set1_epi8(third_len as i8));
    let third = _mm_and_si128(third_mask, third_shifted);
    // println!("third {:?}", slice_m128i(third));

    let third_msb = _mm_and_si128(msb_mask, third);
    let third_result = V::vector_to_num(std::mem::transmute(third_msb));

    (first_result, first_len as u8,
     second_result, second_len as u8,
     third_result, third_len as u8)
}

/// Encodes a single number to a varint. Produces a tuple, with the encoded data followed by the
/// number of bytes used to encode the varint.
#[inline]
pub fn encode<T: VarIntTarget>(num: T) -> ([u8; 16], u8) {
    unsafe { encode_unsafe(num) }
}

/// Encodes a single number to a varint, and writes the resulting data to the slice. Returns the
/// number of bytes written.
///
/// **Panics:** if the slice is too small to contain the varint.
#[inline]
pub fn encode_to_slice<T: VarIntTarget>(num: T, slice: &mut [u8]) -> u8 {
    let (data, size) = encode(num);
    slice[..size as usize].copy_from_slice(&data[..size as usize]);

    size
}

/// Encodes a single number to a varint. Produces a tuple, with the encoded data followed by the
/// number of bytes used to encode the varint.
#[inline]
pub unsafe fn encode_unsafe<T: VarIntTarget>(num: T) -> ([u8; 16], u8) {
    // Break the number into 7-bit parts and spread them out into a vector
    let stage1: __m128i = std::mem::transmute(num.num_to_vector_stage1());

    // Create a mask for where there exist values
    // This signed comparison works because all MSBs should be cleared at this point
    // Also handle the special case when num == 0
    let minimum = _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xffu8 as i8);
    let exists = _mm_or_si128(_mm_cmpgt_epi8(stage1, _mm_setzero_si128()), minimum);
    let bits = _mm_movemask_epi8(exists);

    // Count the number of bytes used
    let bytes = 32 - bits.leading_zeros() as u8; // lzcnt on supported CPUs

    // Fill that many bytes into a vector
    let ascend = _mm_setr_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
    let mask = _mm_cmplt_epi8(ascend, _mm_set1_epi8(bytes as i8));

    // Shift it down 1 byte so the last MSB is the only one set, and make sure only the MSB is set
    let shift = _mm_bsrli_si128(mask, 1);
    let msbmask = _mm_and_si128(shift, _mm_set1_epi8(128u8 as i8));

    // Merge the MSB bits into the vector
    let merged = _mm_or_si128(stage1, msbmask);

    (std::mem::transmute(merged), bytes)
}

#[cfg(test)]
mod tests {
    use crate::{VarIntTarget, encode, decode};

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    fn check<T: VarIntTarget>(value: T, encoded: &[u8]) {
        let mut expected = [0u8; 16];
        expected[..encoded.len()].copy_from_slice(encoded);

        let a = encode(value);
        assert_eq!(a.0, expected);
        assert_eq!(a.1 as usize, encoded.len());

        let roundtrip: (T, u8) = decode(&expected).unwrap();
        assert_eq!(roundtrip.0, value);
        assert_eq!(roundtrip.1 as usize, encoded.len());
    }

    // Test cases borrowed from prost

    #[test]
    fn roundtrip_u8() {
        check(2u8.pow(0) - 1, &[0x00]);
        check(2u8.pow(0), &[0x01]);

        check(2u8.pow(7) - 1, &[0x7F]);
        check(2u8.pow(7), &[0x80, 0x01]);
    }

    #[test]
    fn roundtrip_u16() {
        check(2u16.pow(0) - 1, &[0x00]);
        check(2u16.pow(0), &[0x01]);

        check(2u16.pow(7) - 1, &[0x7F]);
        check(2u16.pow(7), &[0x80, 0x01]);
        check(300u16, &[0xAC, 0x02]);

        check(2u16.pow(14) - 1, &[0xFF, 0x7F]);
        check(2u16.pow(14), &[0x80, 0x80, 0x01]);
    }

    #[test]
    fn roundtrip_u32() {
        check(2u32.pow(0) - 1, &[0x00]);
        check(2u32.pow(0), &[0x01]);

        check(2u32.pow(7) - 1, &[0x7F]);
        check(2u32.pow(7), &[0x80, 0x01]);
        check(300u32, &[0xAC, 0x02]);

        check(2u32.pow(14) - 1, &[0xFF, 0x7F]);
        check(2u32.pow(14), &[0x80, 0x80, 0x01]);

        check(2u32.pow(21) - 1, &[0xFF, 0xFF, 0x7F]);
        check(2u32.pow(21), &[0x80, 0x80, 0x80, 0x01]);

        check(2u32.pow(28) - 1, &[0xFF, 0xFF, 0xFF, 0x7F]);
        check(2u32.pow(28), &[0x80, 0x80, 0x80, 0x80, 0x01]);
    }


    #[test]
    fn roundtrip_u64() {
        check(2u64.pow(0) - 1, &[0x00]);
        check(2u64.pow(0), &[0x01]);

        check(2u64.pow(7) - 1, &[0x7F]);
        check(2u64.pow(7), &[0x80, 0x01]);
        check(300u64, &[0xAC, 0x02]);

        check(2u64.pow(14) - 1, &[0xFF, 0x7F]);
        check(2u64.pow(14), &[0x80, 0x80, 0x01]);

        check(2u64.pow(21) - 1, &[0xFF, 0xFF, 0x7F]);
        check(2u64.pow(21), &[0x80, 0x80, 0x80, 0x01]);

        check(2u64.pow(28) - 1, &[0xFF, 0xFF, 0xFF, 0x7F]);
        check(2u64.pow(28), &[0x80, 0x80, 0x80, 0x80, 0x01]);

        check(2u64.pow(35) - 1, &[0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);
        check(2u64.pow(35), &[0x80, 0x80, 0x80, 0x80, 0x80, 0x01]);

        check(2u64.pow(42) - 1, &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);
        check(2u64.pow(42), &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01]);

        check(
            2u64.pow(49) - 1,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        );
        check(
            2u64.pow(49),
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );

        check(
            2u64.pow(56) - 1,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        );
        check(
            2u64.pow(56),
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );

        check(
            2u64.pow(63) - 1,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        );
        check(
            2u64.pow(63),
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );

        check(
            u64::MAX,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
        );
    }
}
