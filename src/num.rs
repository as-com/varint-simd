#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use std::fmt::Debug;

/// Represents an unsigned scalar value that can be encoded to and decoded from a varint.
pub trait VarIntTarget: Debug + Eq + PartialEq + Sized + Copy {
    /// The signed version of this type
    type Signed: SignedVarIntTarget;

    /// The maximum length of varint that is necessary to represent this number
    const MAX_VARINT_BYTES: u8;

    /// The maximum value of the last byte if the varint is MAX_VARINT_BYTES long such that the
    /// varint would not overflow the target
    const MAX_LAST_VARINT_BYTE: u8;

    /// Converts a 128-bit vector to this number
    ///
    /// Note: Despite operating on 128-bit SIMD vectors, these functions accept and return static
    /// arrays due to a lack of optimization capability by the compiler when passing or returning
    /// intrinsic vectors.
    fn vector_to_num(res: [u8; 16]) -> Self;

    /// Cast from u32 to self
    fn cast_u32(num: u32) -> Self;

    /// Cast from u64 to self
    fn cast_u64(num: u64) -> Self;

    /// Splits this number into 7-bit segments for encoding
    fn num_to_vector_stage1(self) -> [u8; 16];

    /// ZigZag encodes this value
    fn zigzag(from: Self::Signed) -> Self;

    /// ZigZag decodes this value
    fn unzigzag(self) -> Self::Signed;
}

impl VarIntTarget for u8 {
    type Signed = i8;
    const MAX_VARINT_BYTES: u8 = 2;
    const MAX_LAST_VARINT_BYTE: u8 = 0b00000001;

    #[inline(always)]
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2", fast_pdep))]
    fn vector_to_num(res: [u8; 16]) -> Self {
        let arr: [u64; 2] = unsafe { std::mem::transmute(res) };
        let x = arr[0];

        unsafe { _pext_u64(x, 0x000000000000017f) as u8 }
    }

    #[inline(always)]
    #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2", fast_pdep)))]
    fn vector_to_num(res: [u8; 16]) -> Self {
        let res: [u64; 2] = unsafe { std::mem::transmute(res) };
        let x = res[0];
        ((x & 0x000000000000007f) | ((x & 0x0000000000000100) >> 1)) as u8
    }

    #[inline(always)]
    fn cast_u32(num: u32) -> Self {
        num as u8
    }

    #[inline(always)]
    fn cast_u64(num: u64) -> Self {
        num as u8
    }

    #[inline(always)]
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2", fast_pdep))]
    fn num_to_vector_stage1(self) -> [u8; 16] {
        let mut res = [0u64; 2];
        let x = self as u64;

        res[0] = unsafe { _pdep_u64(x, 0x000000000000017f) };

        unsafe { std::mem::transmute(res) }
    }

    #[inline(always)]
    #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2", fast_pdep)))]
    fn num_to_vector_stage1(self) -> [u8; 16] {
        let mut res = [0u64; 2];
        let x = self as u64;

        res[0] = (x & 0x000000000000007f) | ((x & 0x0000000000000080) << 1);

        unsafe { std::mem::transmute(res) }
    }

    #[inline(always)]
    fn zigzag(from: Self::Signed) -> Self {
        ((from << 1) ^ (from >> 7)) as Self
    }

    #[inline(always)]
    fn unzigzag(self) -> Self::Signed {
        ((self >> 1) ^ (-((self & 1) as i8)) as u8) as i8
    }
}

impl VarIntTarget for u16 {
    type Signed = i16;
    const MAX_VARINT_BYTES: u8 = 3;
    const MAX_LAST_VARINT_BYTE: u8 = 0b00000011;

    #[inline(always)]
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2", fast_pdep))]
    fn vector_to_num(res: [u8; 16]) -> Self {
        let arr: [u64; 2] = unsafe { std::mem::transmute(res) };
        let x = arr[0];

        unsafe { _pext_u64(x, 0x0000000000037f7f) as u16 }
    }

    #[inline(always)]
    #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2", fast_pdep)))]
    fn vector_to_num(res: [u8; 16]) -> Self {
        let arr: [u64; 2] = unsafe { std::mem::transmute(res) };
        let x = arr[0];

        ((x & 0x000000000000007f)
            | ((x & 0x0000000000030000) >> 2)
            | ((x & 0x0000000000007f00) >> 1)) as u16
    }

    #[inline(always)]
    fn cast_u32(num: u32) -> Self {
        num as u16
    }

    #[inline(always)]
    fn cast_u64(num: u64) -> Self {
        num as u16
    }

    #[inline(always)]
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2", fast_pdep))]
    fn num_to_vector_stage1(self) -> [u8; 16] {
        let mut res = [0u64; 2];
        let x = self as u64;

        res[0] = unsafe { _pdep_u64(x, 0x0000000000037f7f) };

        unsafe { std::mem::transmute(res) }
    }

    #[inline(always)]
    #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2", fast_pdep)))]
    fn num_to_vector_stage1(self) -> [u8; 16] {
        let mut res = [0u64; 2];
        let x = self as u64;
        res[0] = (x & 0x000000000000007f)
            | ((x & 0x0000000000003f80) << 1)
            | ((x & 0x000000000000c000) << 2);

        unsafe { std::mem::transmute(res) }
    }

    #[inline(always)]
    fn zigzag(from: Self::Signed) -> Self {
        ((from << 1) ^ (from >> 15)) as Self
    }

    #[inline(always)]
    fn unzigzag(self) -> Self::Signed {
        ((self >> 1) ^ (-((self & 1) as i16)) as u16) as i16
    }
}

impl VarIntTarget for u32 {
    type Signed = i32;
    const MAX_VARINT_BYTES: u8 = 5;
    const MAX_LAST_VARINT_BYTE: u8 = 0b00001111;

    #[inline(always)]
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2", fast_pdep))]
    fn vector_to_num(res: [u8; 16]) -> Self {
        let arr: [u64; 2] = unsafe { std::mem::transmute(res) };
        let x = arr[0];

        unsafe { _pext_u64(x, 0x0000000f7f7f7f7f) as u32 }
    }

    #[inline(always)]
    #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2", fast_pdep)))]
    fn vector_to_num(res: [u8; 16]) -> Self {
        let arr: [u64; 2] = unsafe { std::mem::transmute(res) };
        let x = arr[0];

        ((x & 0x000000000000007f)
            | ((x & 0x0000000f00000000) >> 4)
            | ((x & 0x000000007f000000) >> 3)
            | ((x & 0x00000000007f0000) >> 2)
            | ((x & 0x0000000000007f00) >> 1)) as u32
    }

    #[inline(always)]
    fn cast_u32(num: u32) -> Self {
        num
    }

    #[inline(always)]
    fn cast_u64(num: u64) -> Self {
        num as u32
    }

    #[inline(always)]
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2", fast_pdep))]
    fn num_to_vector_stage1(self) -> [u8; 16] {
        let mut res = [0u64; 2];
        let x = self as u64;

        res[0] = unsafe { _pdep_u64(x, 0x0000000f7f7f7f7f) };

        unsafe { std::mem::transmute(res) }
    }

    #[inline(always)]
    #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2", fast_pdep)))]
    fn num_to_vector_stage1(self) -> [u8; 16] {
        let mut res = [0u64; 2];
        let x = self as u64;
        res[0] = (x & 0x000000000000007f)
            | ((x & 0x0000000000003f80) << 1)
            | ((x & 0x00000000001fc000) << 2)
            | ((x & 0x000000000fe00000) << 3)
            | ((x & 0x00000000f0000000) << 4);

        unsafe { std::mem::transmute(res) }
    }

    #[inline(always)]
    fn zigzag(from: Self::Signed) -> Self {
        ((from << 1) ^ (from >> 31)) as Self
    }

    #[inline(always)]
    fn unzigzag(self) -> Self::Signed {
        ((self >> 1) ^ (-((self & 1) as i32)) as u32) as i32
    }
}

impl VarIntTarget for u64 {
    type Signed = i64;
    const MAX_VARINT_BYTES: u8 = 10;
    const MAX_LAST_VARINT_BYTE: u8 = 0b00000001;

    #[inline(always)]
    #[cfg(all(target_feature = "bmi2", fast_pdep))]
    fn vector_to_num(res: [u8; 16]) -> Self {
        let arr: [u64; 2] = unsafe { std::mem::transmute(res) };

        let x = arr[0];
        let y = arr[1];

        let res = unsafe { _pext_u64(x, 0x7f7f7f7f7f7f7f7f) }
            | (unsafe { _pext_u64(y, 0x000000000000017f) } << 56);

        res
    }

    #[inline(always)]
    #[cfg(all(target_feature = "bmi2", fast_pdep))]
    fn num_to_vector_stage1(self) -> [u8; 16] {
        let mut res = [0u64; 2];
        let x = self;

        res[0] = unsafe { _pdep_u64(x, 0x7f7f7f7f7f7f7f7f) };
        res[1] = unsafe { _pdep_u64(x >> 56, 0x000000000000017f) };

        unsafe { std::mem::transmute(res) }
    }

    #[inline(always)]
    #[cfg(all(target_feature = "avx2", not(all(target_feature = "bmi2", fast_pdep))))]
    fn vector_to_num(res: [u8; 16]) -> Self {
        let pt1 = unsafe {
            let b = std::mem::transmute(res);

            let c = _mm_broadcastq_epi64(b);
            let d = _mm_or_si128(
                _mm_or_si128(
                    _mm_srlv_epi64(
                        _mm_and_si128(c, _mm_set_epi64x(0x000000000000007f, 0x7f00000000000000)),
                        _mm_set_epi64x(0, 7),
                    ),
                    _mm_srlv_epi64(
                        _mm_and_si128(c, _mm_set_epi64x(0x007f000000000000, 0x00007f0000000000)),
                        _mm_set_epi64x(6, 5),
                    ),
                ),
                _mm_or_si128(
                    _mm_srlv_epi64(
                        _mm_and_si128(c, _mm_set_epi64x(0x0000007f00000000, 0x000000007f000000)),
                        _mm_set_epi64x(4, 3),
                    ),
                    _mm_srlv_epi64(
                        _mm_and_si128(c, _mm_set_epi64x(0x00000000007f0000, 0x0000000000007f00)),
                        _mm_set_epi64x(2, 1),
                    ),
                ),
            );

            let e = _mm_or_si128(d, _mm_bsrli_si128(d, 8));
            _mm_extract_epi64(e, 0) as u64
        };

        let arr: [u64; 2] = unsafe { std::mem::transmute(res) };

        let y = arr[1];

        // This incantation was generated with calcperm
        pt1
            // don't forget about bytes spilling to the other word
            | ((y & 0x0000000000000100) << 55)
            | ((y & 0x000000000000007f) << 56)
    }

    #[inline(always)]
    #[cfg(all(target_feature = "avx2", not(all(target_feature = "bmi2", fast_pdep))))]
    fn num_to_vector_stage1(self) -> [u8; 16] {
        let mut res = [0u64; 2];
        let x = self;

        let b = unsafe { _mm_set1_epi64x(self as i64) };
        let c = unsafe {
            _mm_or_si128(
                _mm_or_si128(
                    _mm_sllv_epi64(
                        _mm_and_si128(b, _mm_set_epi64x(0x00000007f0000000, 0x000003f800000000)),
                        _mm_set_epi64x(4, 5),
                    ),
                    _mm_sllv_epi64(
                        _mm_and_si128(b, _mm_set_epi64x(0x0001fc0000000000, 0x00fe000000000000)),
                        _mm_set_epi64x(6, 7),
                    ),
                ),
                _mm_or_si128(
                    _mm_sllv_epi64(
                        _mm_and_si128(b, _mm_set_epi64x(0x000000000000007f, 0x0000000000003f80)),
                        _mm_set_epi64x(0, 1),
                    ),
                    _mm_sllv_epi64(
                        _mm_and_si128(b, _mm_set_epi64x(0x00000000001fc000, 0x000000000fe00000)),
                        _mm_set_epi64x(2, 3),
                    ),
                ),
            )
        };
        let d = unsafe { _mm_or_si128(c, _mm_bsrli_si128(c, 8)) };

        res[0] = unsafe { _mm_extract_epi64(d, 0) as u64 };
        res[1] = ((x & 0x7f00000000000000) >> 56) | ((x & 0x8000000000000000) >> 55);

        unsafe { std::mem::transmute(res) }
    }

    #[inline(always)]
    #[cfg(not(any(target_feature = "avx2", all(target_feature = "bmi2", fast_pdep))))]
    fn vector_to_num(res: [u8; 16]) -> Self {
        let arr: [u64; 2] = unsafe { std::mem::transmute(res) };

        let x = arr[0];
        let y = arr[1];

        // This incantation was generated with calcperm
        (x & 0x000000000000007f)
            | ((x & 0x7f00000000000000) >> 7)
            | ((x & 0x007f000000000000) >> 6)
            | ((x & 0x00007f0000000000) >> 5)
            | ((x & 0x0000007f00000000) >> 4)
            | ((x & 0x000000007f000000) >> 3)
            | ((x & 0x00000000007f0000) >> 2)
            | ((x & 0x0000000000007f00) >> 1)
            // don't forget about bytes spilling to the other word
            | ((y & 0x0000000000000100) << 55)
            | ((y & 0x000000000000007f) << 56)
    }

    #[inline(always)]
    #[cfg(not(any(target_feature = "avx2", all(target_feature = "bmi2", fast_pdep))))]
    fn num_to_vector_stage1(self) -> [u8; 16] {
        let mut res = [0u64; 2];
        let x = self;

        res[0] = (x & 0x000000000000007f)
            | ((x & 0x0000000000003f80) << 1)
            | ((x & 0x00000000001fc000) << 2)
            | ((x & 0x000000000fe00000) << 3)
            | ((x & 0x00000007f0000000) << 4)
            | ((x & 0x000003f800000000) << 5)
            | ((x & 0x0001fc0000000000) << 6)
            | ((x & 0x00fe000000000000) << 7);
        res[1] = ((x & 0x7f00000000000000) >> 56) | ((x & 0x8000000000000000) >> 55);

        unsafe { std::mem::transmute(res) }
    }

    #[inline(always)]
    fn cast_u32(num: u32) -> Self {
        num as u64
    }

    #[inline(always)]
    fn cast_u64(num: u64) -> Self {
        num
    }

    #[inline(always)]
    fn zigzag(from: Self::Signed) -> Self {
        ((from << 1) ^ (from >> 63)) as Self
    }

    #[inline(always)]
    fn unzigzag(self) -> Self::Signed {
        ((self >> 1) ^ (-((self & 1) as i64)) as u64) as i64
    }
}

/// Represents a signed scalar value that can be encoded to and decoded from a varint in ZigZag
/// format.
pub trait SignedVarIntTarget: Debug + Eq + PartialEq + Sized + Copy {
    type Unsigned: VarIntTarget<Signed = Self>;

    /// ZigZag encodes this value
    #[inline(always)]
    fn zigzag(from: Self) -> Self::Unsigned {
        Self::Unsigned::zigzag(from)
    }

    /// ZigZag decodes this value
    #[inline(always)]
    fn unzigzag(from: Self::Unsigned) -> Self {
        Self::Unsigned::unzigzag(from)
    }
}

impl SignedVarIntTarget for i8 {
    type Unsigned = u8;
}

impl SignedVarIntTarget for i16 {
    type Unsigned = u16;
}

impl SignedVarIntTarget for i32 {
    type Unsigned = u32;
}

impl SignedVarIntTarget for i64 {
    type Unsigned = u64;
}
