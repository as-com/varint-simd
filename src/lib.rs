#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

fn slice_m128i(n: __m128i) -> [u8;16] {
    unsafe { std::mem::transmute(n) }
}

pub trait VarIntTarget {
    #[inline(always)]
    fn vector_to_num(res: [u8;16]) -> Self;
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
        (res[0] as u32) | ((res[1] as u32) << 7) | ((res[2] as u32) << 2 * 7) | ((res[3] as u32) << 3 * 7) | ((res[4] as u32) << 4 * 7)
    }
}

impl VarIntTarget for u64 {
    #[inline(always)]
    fn vector_to_num(res: [u8; 16]) -> Self {
        (res[0] as u64) | ((res[1] as u64) << 7) | ((res[2] as u64) << 2*7) | ((res[3] as u64) << 3*7) | ((res[4] as u64) << 4*7) | ((res[5] as u64) << 5*7) | ((res[6] as u64) << 6*7) | ((res[7] as u64) << 7*7) | ((res[8] as u64) << 8*7)
    }
}

/// There must be at least 16 bytes of allocated memory after the beginning of the pointer
#[inline]
pub unsafe fn decode_unsafe<T: VarIntTarget>(bytes: &[u8]) -> (T, i32) {
    assert!(bytes.len() >= 1);
    let b = _mm_loadu_si128(bytes.as_ptr() as *const __m128i);

    // println!("{:?}", slice_m128i(b));

    let bitmask: i32 = _mm_movemask_epi8(b);
    let bm_not = !bitmask;
    let cleaned = (bm_not - 1) ^ bm_not; // blsmsk (Haswell+)

    // println!("{:#018b}", cleaned);

    let bc = _mm_set_epi32(0, 0, 0, cleaned as i32);
    // println!("bc {:?}", slice_m128i(bc));
    let shuffle = _mm_set_epi64x(0x0101010101010101, 0x0000000000000000);
    // println!("shuffle {:?}", slice_m128i(shuffle));
    let shuffled = _mm_shuffle_epi8(bc, shuffle);
    // println!("shuffled {:?}", slice_m128i(shuffled));
    let mask = _mm_set_epi8(128u8 as i8, 64, 32, 16, 8, 4, 2, 1, 128u8 as i8, 64, 32, 16, 8, 4, 2, 1);
    // println!("mask {:?}", slice_m128i(mask));
    let t = _mm_and_si128(shuffled, mask);
    // println!("t {:?}", slice_m128i(t));
    let fat_mask = _mm_cmpeq_epi8(mask, t);
    // println!("fat mask {:?}", slice_m128i(fat_mask));
    
    let varint_part = _mm_and_si128(fat_mask, b);
    let msb_masked = _mm_and_si128(varint_part, _mm_set_epi8(0, 0, 0, 0, 0, 0, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127));
    // println!("{:?}", slice_m128i(msb_masked));

    let res: [u8;16] = std::mem::transmute(msb_masked);

    // This line gets auto-vectorized with AVX2, but I don't think it's possible on older CPUs
    let num = T::vector_to_num(res);
    let bytes_read = _popcnt32(cleaned);

    (num, bytes_read)
}

#[cfg(test)]
mod tests {
    use crate::decode_unsafe;

    #[test]
    fn it_works() {
        println!("{:?}", unsafe { decode_unsafe::<u64>(&vec![0x0f, 0xff, 0xff, 0xff, 0x0F, 0b10101100, 0b10101100, 0, 0, 0, 0b10101100, 0, 0, 0, 0, 1]) });
        assert_eq!(2 + 2, 4);
    }
}
