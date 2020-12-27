#[cfg(target_arch = "x86")]
use core::arch::x86 as arch;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// Functions to help with debugging
fn slice_m128i(n: __m128i) -> [i8; 16] {
    unsafe { std::mem::transmute(n) }
}

fn slice_m256i(n: __m256i) -> [i8; 32] {
    unsafe { std::mem::transmute(n) }
}

/// Represents a scalar value that can be encoded to and decoded from a varint.
pub trait VarIntTarget {
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
    const MAX_VARINT_BYTES: u8 = 9;

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

        res
    }
}

/// Decodes a single varint from the input slice. Requires SSSE3 support.
///
/// There must be at least 16 bytes of allocated memory after the beginning of the pointer.
/// Otherwise, there may be undefined behavior. Any data after the end of the varint is ignored.
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

/// Encodes a single varint. Produces a tuple, with the encoded data followed by the number of
/// bytes used to encode the varint.
#[inline]
pub unsafe fn encode_unsafe<T: VarIntTarget>(num: T) -> ([u8; 16], u8) {
    // Break the number into 7-bit parts and spread them out into a vector
    let stage1: __m128i = std::mem::transmute(num.num_to_vector_stage1());

    // Create a mask for where there exist values
    let exists = _mm_cmpgt_epi8(stage1, _mm_setzero_si128());

    // Count the number of bytes set to 0xFF
    let set = _mm_movemask_epi8(exists);
    let bytes = set.count_ones() as u8; // popcnt on supported CPUs

    // Shift it down 1 byte so the last MSB is the only one set, and make sure only the MSB is set
    let shift = _mm_bsrli_si128(exists, 1);
    let msbmask = _mm_and_si128(shift, _mm_set1_epi8(128u8 as i8));

    // Merge the MSB bits into the vector
    let merged = _mm_or_si128(stage1, msbmask);

    (std::mem::transmute(merged), bytes)
}

#[cfg(test)]
mod tests {
    use crate::{decode_unsafe, encode_unsafe, decode_three_unsafe};

    #[test]
    fn it_works() {
        println!("{:?}", unsafe {
            decode_unsafe::<u64>(&vec![
                0xff, 0xff, 0xff, 0xff, 0xff, 0x0f, 0b10101100, 0b10101100, 0, 0, 0, 0b10101100, 0,
                0, 0, 0, 1,
            ])
        });

        println!("{:?}", unsafe {
            encode_unsafe::<u64>(549755813887)
        });

        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn decode_three() {
        println!("{:?}", unsafe {
            decode_three_unsafe::<u64, u64, u64>(&vec![
                248, 215, 255, 140, 238, 171, 187, 135, 64,
                248, 215, 141, 140, 238, 171, 255, 135, 64,
                248, 215, 141, 255, 238, 255, 187, 135, 64,
                0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee,
                0xee, 0xee, 0xee, 0xee
            ])
        })
    }
}
