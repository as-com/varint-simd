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
    fn vector_to_num(res: [u8; 16]) -> Self;

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

    #[inline(always)]
    fn zigzag(from: Self::Signed) -> Self {
        ((from << 1) ^ (from >> 63)) as Self
    }

    #[inline(always)]
    fn unzigzag(self) -> Self::Signed {
        ((self >> 1) ^ (-((self & 1) as i64)) as u64) as i64
    }
}

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

impl SignedVarIntTarget for i8 { type Unsigned = u8; }
impl SignedVarIntTarget for i16 { type Unsigned = u16; }
impl SignedVarIntTarget for i32 { type Unsigned = u32; }
impl SignedVarIntTarget for i64 { type Unsigned = u64; }
