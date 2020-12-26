#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

fn slice_m128i(n: __m128i) -> [u8; 16] {
    unsafe { std::mem::transmute(n) }
}

pub trait VarIntTarget {
    #[inline(always)]
    fn vector_to_num(res: [u8; 16]) -> Self;
}

impl VarIntTarget for u8 {
    #[inline(always)]
    fn vector_to_num(res: [u8; 16]) -> Self {
        (res[0] as u8) | ((res[1] as u8) << 7)
    }
}

impl VarIntTarget for u16 {
    #[inline(always)]
    fn vector_to_num(res: [u8; 16]) -> Self {
        (res[0] as u16) | ((res[1] as u16) << 7) | ((res[2] as u16) << 2 * 7)
    }
}

impl VarIntTarget for u32 {
    #[inline(always)]
    fn vector_to_num(res: [u8; 16]) -> Self {
        (res[0] as u32)
            | ((res[1] as u32) << 7)
            | ((res[2] as u32) << 2 * 7)
            | ((res[3] as u32) << 3 * 7)
            | ((res[4] as u32) << 4 * 7)
    }
}

impl VarIntTarget for u64 {
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
}

/// There must be at least 16 bytes of allocated memory after the beginning of the pointer
#[inline]
pub unsafe fn decode_unsafe<T: VarIntTarget>(bytes: &[u8]) -> (T, i32) {
    // It looks like you're trying to understand what this code does. You should probably read
    // this first: https://developers.google.com/protocol-buffers/docs/encoding#varints

    assert!(bytes.len() >= 1);
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
    let mask = _mm_set_epi8(128u8 as i8, 64, 32, 16, 8, 4, 2, 1, 128u8 as i8, 64, 32, 16, 8, 4, 2, 1,);
    let t = _mm_and_si128(shuffled, mask);

    // Expand out the set bits into full 0xFF values
    let fat_mask = _mm_cmpeq_epi8(mask, t);

    // Mask out the irrelevant bytes in the input
    let varint_part = _mm_and_si128(fat_mask, b);

    // Turn off the most significant bits
    let msb_masked = _mm_and_si128(
        varint_part,
        _mm_set_epi8(0, 0, 0, 0, 0, 0, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127,),
    );

    // Turn the vector into a scalar value by concatenating the 7-bit values
    let res: [u8; 16] = std::mem::transmute(msb_masked);
    let num = T::vector_to_num(res); // specialized functions for different number sizes

    // Count the number of bytes we actually read
    let bytes_read = _popcnt32(cleaned);

    (num, bytes_read)
}

#[cfg(test)]
mod tests {
    use crate::decode_unsafe;

    #[test]
    fn it_works() {
        println!("{:?}", unsafe {
            decode_unsafe::<u64>(&vec![
                0xff, 0xff, 0xff, 0xff, 0xff, 0x0f, 0b10101100, 0b10101100, 0, 0, 0, 0b10101100, 0,
                0, 0, 0, 1,
            ])
        });
        assert_eq!(2 + 2, 4);
    }
}
