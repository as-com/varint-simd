/*!
`varint_simd` is a fast SIMD-accelerated [variable-length integer](https://developers.google.com/protocol-buffers/docs/encoding)
encoder and decoder written in Rust.

**For more information, please see the [README](https://github.com/as-com/varint-simd#readme).**
*/

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(rustc_nightly, feature(doc_cfg))]

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use core::fmt::Debug;

pub mod decode;
pub mod encode;
pub mod num;

#[doc(inline)]
pub use decode::*;
#[doc(inline)]
pub use encode::*;
pub use num::*;

// Functions to help with debugging
#[allow(dead_code)]
fn slice_m128i(n: __m128i) -> [u8; 16] {
    unsafe { core::mem::transmute(n) }
}

#[allow(dead_code)]
fn slice_m256i(n: __m256i) -> [i8; 32] {
    unsafe { core::mem::transmute(n) }
}

#[derive(Debug)]
pub enum VarIntDecodeError {
    Overflow,
    NotEnoughBytes,
}

impl core::fmt::Display for VarIntDecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for VarIntDecodeError {}

#[cfg(test)]
mod tests {
    #[cfg(target_feature = "avx2")]
    use crate::decode_two_wide_unsafe;
    use crate::{
        decode, decode_len, decode_eight_u8_unsafe, decode_four_unsafe, decode_two_unsafe, encode,
        encode_to_slice, VarIntTarget
    };

    use lazy_static::lazy_static;

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

        let roundtrip: (T, usize) = decode(&expected).unwrap();
        assert_eq!(roundtrip.0, value);
        assert_eq!(roundtrip.1 as usize, encoded.len());

        let len = decode_len::<T>(&expected).unwrap();
        assert_eq!(len, encoded.len());
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

    #[test]
    fn overflow_u8() {
        let encoded = encode(u8::MAX as u16 + 1);
        decode::<u8>(&encoded.0).expect_err("should overflow");
    }

    #[test]
    fn overflow_u16() {
        let encoded = encode(u16::MAX as u32 + 1);
        decode::<u16>(&encoded.0).expect_err("should overflow");
    }

    #[test]
    fn overflow_u32() {
        let encoded = encode(u32::MAX as u64 + 1);
        decode::<u32>(&encoded.0).expect_err("should overflow");
    }

    #[test]
    fn overflow_u64() {
        decode::<u8>(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x02])
            .expect_err("should overflow");
    }

    fn check_decode_2x<T: VarIntTarget, U: VarIntTarget>(a: &[T], b: &[U]) {
        for i in a {
            for j in b {
                let mut enc = [0u8; 16];

                let first_len = encode_to_slice(*i, &mut enc);
                let second_len = encode_to_slice(*j, &mut enc[first_len as usize..]);

                let decoded = unsafe { decode_two_unsafe::<T, U>(enc.as_ptr()) };
                assert_eq!(decoded.0, *i);
                assert_eq!(decoded.1, *j);
                assert_eq!(decoded.2, first_len);
                assert_eq!(decoded.3, second_len);
            }
        }
    }

    #[cfg(target_feature = "avx2")]
    fn check_decode_wide_2x<T: VarIntTarget, U: VarIntTarget>(a: &[T], b: &[U]) {
        for i in a {
            for j in b {
                let mut enc = [0u8; 32];

                let first_len = encode_to_slice(*i, &mut enc);
                let second_len = encode_to_slice(*j, &mut enc[first_len as usize..]);

                let decoded = unsafe { decode_two_wide_unsafe::<T, U>(enc.as_ptr()) };
                assert_eq!(decoded.0, *i);
                assert_eq!(decoded.1, *j);
                assert_eq!(decoded.2, first_len);
                assert_eq!(decoded.3, second_len);
            }
        }
    }

    fn check_decode_4x<T: VarIntTarget, U: VarIntTarget, V: VarIntTarget, W: VarIntTarget>(
        a: &[T],
        b: &[U],
        c: &[V],
        d: &[W],
    ) {
        for i in a {
            for j in b {
                for k in c {
                    for l in d {
                        let mut enc = [0u8; 16];

                        let first_len = encode_to_slice(*i, &mut enc);
                        let second_len = encode_to_slice(*j, &mut enc[first_len as usize..]);
                        let third_len =
                            encode_to_slice(*k, &mut enc[(first_len + second_len) as usize..]);
                        let fourth_len = encode_to_slice(
                            *l,
                            &mut enc[(first_len + second_len + third_len) as usize..],
                        );

                        let decoded = unsafe { decode_four_unsafe::<T, U, V, W>(enc.as_ptr()) };

                        assert_eq!(decoded.0, *i);
                        assert_eq!(decoded.1, *j);
                        assert_eq!(decoded.2, *k);
                        assert_eq!(decoded.3, *l);
                        assert_eq!(decoded.4, first_len);
                        assert_eq!(decoded.5, second_len);
                        assert_eq!(decoded.6, third_len);
                        assert_eq!(decoded.7, fourth_len);
                        assert!(!decoded.8);
                    }
                }
            }
        }
    }

    lazy_static! {
        static ref NUMS_U8: [u8; 5] = [
            2u8.pow(0) - 1,
            2u8.pow(0),
            2u8.pow(7) - 1,
            2u8.pow(7),
            u8::MAX
        ];
        static ref NUMS_U16: [u16; 8] = [
            2u16.pow(0) - 1,
            2u16.pow(0),
            2u16.pow(7) - 1,
            2u16.pow(7),
            300,
            2u16.pow(14) - 1,
            2u16.pow(14),
            u16::MAX
        ];
        static ref NUMS_U32: [u32; 12] = [
            2u32.pow(0) - 1,
            2u32.pow(0),
            2u32.pow(7) - 1,
            2u32.pow(7),
            300,
            2u32.pow(14) - 1,
            2u32.pow(14),
            2u32.pow(21) - 1,
            2u32.pow(21),
            2u32.pow(28) - 1,
            2u32.pow(28),
            u32::MAX
        ];
        static ref NUMS_U64: [u64; 22] = [
            2u64.pow(0) - 1,
            2u64.pow(0),
            2u64.pow(7) - 1,
            2u64.pow(7),
            300,
            2u64.pow(14) - 1,
            2u64.pow(14),
            2u64.pow(21) - 1,
            2u64.pow(21),
            2u64.pow(28) - 1,
            2u64.pow(28),
            2u64.pow(35) - 1,
            2u64.pow(35),
            2u64.pow(42) - 1,
            2u64.pow(42),
            2u64.pow(49) - 1,
            2u64.pow(49),
            2u64.pow(56) - 1,
            2u64.pow(56),
            2u64.pow(63) - 1,
            2u64.pow(63),
            u64::MAX
        ];
    }

    #[test]
    fn test_decode_2x_u8_x() {
        check_decode_2x::<u8, u8>(&NUMS_U8[..], &NUMS_U8[..]);
        check_decode_2x::<u8, u16>(&NUMS_U8[..], &NUMS_U16[..]);
        check_decode_2x::<u8, u32>(&NUMS_U8[..], &NUMS_U32[..]);
        check_decode_2x::<u8, u64>(&NUMS_U8[..], &NUMS_U64[..]);
    }

    #[test]
    #[cfg(target_feature = "avx2")]
    fn test_decode_2x_wide_u8_x() {
        check_decode_wide_2x::<u8, u8>(&NUMS_U8[..], &NUMS_U8[..]);
        check_decode_wide_2x::<u8, u16>(&NUMS_U8[..], &NUMS_U16[..]);
        check_decode_wide_2x::<u8, u32>(&NUMS_U8[..], &NUMS_U32[..]);
        check_decode_wide_2x::<u8, u64>(&NUMS_U8[..], &NUMS_U64[..]);
    }

    #[test]
    fn test_decode_2x_u16_x() {
        check_decode_2x::<u16, u8>(&NUMS_U16[..], &NUMS_U8[..]);
        check_decode_2x::<u16, u16>(&NUMS_U16[..], &NUMS_U16[..]);
        check_decode_2x::<u16, u32>(&NUMS_U16[..], &NUMS_U32[..]);
        check_decode_2x::<u16, u64>(&NUMS_U16[..], &NUMS_U64[..]);
    }

    #[test]
    #[cfg(target_feature = "avx2")]
    fn test_decode_2x_wide_u16_x() {
        check_decode_wide_2x::<u16, u8>(&NUMS_U16[..], &NUMS_U8[..]);
        check_decode_wide_2x::<u16, u16>(&NUMS_U16[..], &NUMS_U16[..]);
        check_decode_wide_2x::<u16, u32>(&NUMS_U16[..], &NUMS_U32[..]);
        check_decode_wide_2x::<u16, u64>(&NUMS_U16[..], &NUMS_U64[..]);
    }

    #[test]
    fn test_decode_2x_u32_x() {
        check_decode_2x::<u32, u8>(&NUMS_U32[..], &NUMS_U8[..]);
        check_decode_2x::<u32, u16>(&NUMS_U32[..], &NUMS_U16[..]);
        check_decode_2x::<u32, u32>(&NUMS_U32[..], &NUMS_U32[..]);
        check_decode_2x::<u32, u64>(&NUMS_U32[..], &NUMS_U64[..]);
    }

    #[test]
    #[cfg(target_feature = "avx2")]
    fn test_decode_2x_wide_u32_x() {
        check_decode_wide_2x::<u32, u8>(&NUMS_U32[..], &NUMS_U8[..]);
        check_decode_wide_2x::<u32, u16>(&NUMS_U32[..], &NUMS_U16[..]);
        check_decode_wide_2x::<u32, u32>(&NUMS_U32[..], &NUMS_U32[..]);
        check_decode_wide_2x::<u32, u64>(&NUMS_U32[..], &NUMS_U64[..]);
    }

    #[test]
    fn test_decode_2x_u64_x() {
        check_decode_2x::<u64, u8>(&NUMS_U64[..], &NUMS_U8[..]);
        check_decode_2x::<u64, u16>(&NUMS_U64[..], &NUMS_U16[..]);
        check_decode_2x::<u64, u32>(&NUMS_U64[..], &NUMS_U32[..]);
        // check_decode_2x::<u64, u64>(&NUMS_U64[..], &NUMS_U64[..]);
    }

    #[test]
    #[cfg(target_feature = "avx2")]
    fn test_decode_2x_wide_u64_x() {
        check_decode_wide_2x::<u64, u8>(&NUMS_U64[..], &NUMS_U8[..]);
        check_decode_wide_2x::<u64, u16>(&NUMS_U64[..], &NUMS_U16[..]);
        check_decode_wide_2x::<u64, u32>(&NUMS_U64[..], &NUMS_U32[..]);
        check_decode_wide_2x::<u64, u64>(&NUMS_U64[..], &NUMS_U64[..]);
    }

    #[test]
    fn test_decode_4x_u8_u8_x_x() {
        check_decode_4x::<u8, u8, u8, u8>(&NUMS_U8[..], &NUMS_U8[..], &NUMS_U8[..], &NUMS_U8[..]);
        check_decode_4x::<u8, u8, u8, u16>(&NUMS_U8[..], &NUMS_U8[..], &NUMS_U8[..], &NUMS_U16[..]);
        check_decode_4x::<u8, u8, u8, u32>(&NUMS_U8[..], &NUMS_U8[..], &NUMS_U8[..], &NUMS_U32[..]);
        check_decode_4x::<u8, u8, u8, u64>(&NUMS_U8[..], &NUMS_U8[..], &NUMS_U8[..], &NUMS_U64[..]);

        check_decode_4x::<u8, u8, u16, u8>(&NUMS_U8[..], &NUMS_U8[..], &NUMS_U16[..], &NUMS_U8[..]);
        check_decode_4x::<u8, u8, u16, u16>(
            &NUMS_U8[..],
            &NUMS_U8[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
        );
        check_decode_4x::<u8, u8, u16, u32>(
            &NUMS_U8[..],
            &NUMS_U8[..],
            &NUMS_U16[..],
            &NUMS_U32[..],
        );
    }

    #[test]
    fn test_decode_4x_u8_u16_x_x() {
        check_decode_4x::<u8, u16, u8, u8>(&NUMS_U8[..], &NUMS_U16[..], &NUMS_U8[..], &NUMS_U8[..]);
        check_decode_4x::<u8, u16, u8, u16>(
            &NUMS_U8[..],
            &NUMS_U16[..],
            &NUMS_U8[..],
            &NUMS_U16[..],
        );
        check_decode_4x::<u8, u16, u8, u32>(
            &NUMS_U8[..],
            &NUMS_U16[..],
            &NUMS_U8[..],
            &NUMS_U32[..],
        );

        check_decode_4x::<u8, u16, u16, u8>(
            &NUMS_U8[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
            &NUMS_U8[..],
        );
        check_decode_4x::<u8, u16, u16, u16>(
            &NUMS_U8[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
        );
        check_decode_4x::<u8, u16, u16, u32>(
            &NUMS_U8[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
            &NUMS_U32[..],
        );
    }

    #[test]
    fn test_decode_4x_u8_u32_x_x() {
        check_decode_4x::<u8, u32, u8, u8>(&NUMS_U8[..], &NUMS_U32[..], &NUMS_U8[..], &NUMS_U8[..]);
        check_decode_4x::<u8, u32, u8, u16>(
            &NUMS_U8[..],
            &NUMS_U32[..],
            &NUMS_U8[..],
            &NUMS_U16[..],
        );
        check_decode_4x::<u8, u32, u8, u32>(
            &NUMS_U8[..],
            &NUMS_U32[..],
            &NUMS_U8[..],
            &NUMS_U32[..],
        );

        check_decode_4x::<u8, u32, u16, u8>(
            &NUMS_U8[..],
            &NUMS_U32[..],
            &NUMS_U16[..],
            &NUMS_U8[..],
        );
        check_decode_4x::<u8, u32, u16, u16>(
            &NUMS_U8[..],
            &NUMS_U32[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
        );
        check_decode_4x::<u8, u32, u16, u32>(
            &NUMS_U8[..],
            &NUMS_U32[..],
            &NUMS_U16[..],
            &NUMS_U32[..],
        );
    }

    #[test]
    fn test_decode_4x_u8_u64_x_x() {
        check_decode_4x::<u8, u64, u8, u8>(&NUMS_U8[..], &NUMS_U64[..], &NUMS_U8[..], &NUMS_U8[..]);
    }

    #[test]
    fn test_decode_4x_u16_u8_x_x() {
        check_decode_4x::<u16, u8, u8, u8>(&NUMS_U16[..], &NUMS_U8[..], &NUMS_U8[..], &NUMS_U8[..]);
        check_decode_4x::<u16, u8, u8, u16>(
            &NUMS_U16[..],
            &NUMS_U8[..],
            &NUMS_U8[..],
            &NUMS_U16[..],
        );
        check_decode_4x::<u16, u8, u8, u32>(
            &NUMS_U16[..],
            &NUMS_U8[..],
            &NUMS_U8[..],
            &NUMS_U32[..],
        );

        check_decode_4x::<u16, u8, u16, u8>(
            &NUMS_U16[..],
            &NUMS_U8[..],
            &NUMS_U16[..],
            &NUMS_U8[..],
        );
        check_decode_4x::<u16, u8, u16, u16>(
            &NUMS_U16[..],
            &NUMS_U8[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
        );
        check_decode_4x::<u16, u8, u16, u32>(
            &NUMS_U16[..],
            &NUMS_U8[..],
            &NUMS_U16[..],
            &NUMS_U32[..],
        );
    }

    #[test]
    fn test_decode_4x_u16_u16_x_x() {
        check_decode_4x::<u16, u16, u8, u8>(
            &NUMS_U16[..],
            &NUMS_U16[..],
            &NUMS_U8[..],
            &NUMS_U8[..],
        );
        check_decode_4x::<u16, u16, u8, u16>(
            &NUMS_U16[..],
            &NUMS_U16[..],
            &NUMS_U8[..],
            &NUMS_U16[..],
        );
        check_decode_4x::<u16, u16, u8, u32>(
            &NUMS_U16[..],
            &NUMS_U16[..],
            &NUMS_U8[..],
            &NUMS_U32[..],
        );

        check_decode_4x::<u16, u16, u16, u8>(
            &NUMS_U16[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
            &NUMS_U8[..],
        );
        check_decode_4x::<u16, u16, u16, u16>(
            &NUMS_U16[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
        );
        check_decode_4x::<u16, u16, u16, u32>(
            &NUMS_U16[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
            &NUMS_U32[..],
        );
    }

    #[test]
    fn test_decode_4x_u16_u32_x_x() {
        check_decode_4x::<u16, u32, u8, u8>(
            &NUMS_U16[..],
            &NUMS_U32[..],
            &NUMS_U8[..],
            &NUMS_U8[..],
        );
        check_decode_4x::<u16, u32, u8, u16>(
            &NUMS_U16[..],
            &NUMS_U32[..],
            &NUMS_U8[..],
            &NUMS_U16[..],
        );
        check_decode_4x::<u16, u32, u8, u32>(
            &NUMS_U16[..],
            &NUMS_U32[..],
            &NUMS_U8[..],
            &NUMS_U32[..],
        );

        check_decode_4x::<u16, u32, u16, u8>(
            &NUMS_U16[..],
            &NUMS_U32[..],
            &NUMS_U16[..],
            &NUMS_U8[..],
        );
        check_decode_4x::<u16, u32, u16, u16>(
            &NUMS_U16[..],
            &NUMS_U32[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
        );
        check_decode_4x::<u16, u32, u16, u32>(
            &NUMS_U16[..],
            &NUMS_U32[..],
            &NUMS_U16[..],
            &NUMS_U32[..],
        );
    }

    #[test]
    fn test_decode_4x_u32_u8_x_x() {
        check_decode_4x::<u32, u8, u8, u8>(&NUMS_U32[..], &NUMS_U8[..], &NUMS_U8[..], &NUMS_U8[..]);
        check_decode_4x::<u32, u8, u8, u16>(
            &NUMS_U32[..],
            &NUMS_U8[..],
            &NUMS_U8[..],
            &NUMS_U16[..],
        );
        check_decode_4x::<u32, u8, u8, u32>(
            &NUMS_U32[..],
            &NUMS_U8[..],
            &NUMS_U8[..],
            &NUMS_U32[..],
        );

        check_decode_4x::<u32, u8, u16, u8>(
            &NUMS_U32[..],
            &NUMS_U8[..],
            &NUMS_U16[..],
            &NUMS_U8[..],
        );
        check_decode_4x::<u32, u8, u16, u16>(
            &NUMS_U32[..],
            &NUMS_U8[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
        );
        check_decode_4x::<u32, u8, u16, u32>(
            &NUMS_U32[..],
            &NUMS_U8[..],
            &NUMS_U16[..],
            &NUMS_U32[..],
        );
    }

    #[test]
    fn test_decode_4x_u32_u16_x_x() {
        check_decode_4x::<u32, u16, u8, u8>(
            &NUMS_U32[..],
            &NUMS_U16[..],
            &NUMS_U8[..],
            &NUMS_U8[..],
        );
        check_decode_4x::<u32, u16, u8, u16>(
            &NUMS_U32[..],
            &NUMS_U16[..],
            &NUMS_U8[..],
            &NUMS_U16[..],
        );
        check_decode_4x::<u32, u16, u8, u32>(
            &NUMS_U32[..],
            &NUMS_U16[..],
            &NUMS_U8[..],
            &NUMS_U32[..],
        );

        check_decode_4x::<u32, u16, u16, u8>(
            &NUMS_U32[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
            &NUMS_U8[..],
        );
        check_decode_4x::<u32, u16, u16, u16>(
            &NUMS_U32[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
        );
        check_decode_4x::<u32, u16, u16, u32>(
            &NUMS_U32[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
            &NUMS_U32[..],
        );
    }

    #[test]
    fn test_decode_4x_u32_u32_x_x() {
        check_decode_4x::<u32, u32, u8, u8>(
            &NUMS_U32[..],
            &NUMS_U32[..],
            &NUMS_U8[..],
            &NUMS_U8[..],
        );
        check_decode_4x::<u32, u32, u8, u16>(
            &NUMS_U32[..],
            &NUMS_U32[..],
            &NUMS_U8[..],
            &NUMS_U16[..],
        );

        check_decode_4x::<u32, u32, u16, u8>(
            &NUMS_U32[..],
            &NUMS_U32[..],
            &NUMS_U16[..],
            &NUMS_U8[..],
        );
        check_decode_4x::<u32, u32, u16, u16>(
            &NUMS_U32[..],
            &NUMS_U32[..],
            &NUMS_U16[..],
            &NUMS_U16[..],
        );
    }

    #[test]
    fn test_decode_4x_u64_u8_x_x() {
        check_decode_4x::<u64, u8, u8, u8>(&NUMS_U64[..], &NUMS_U8[..], &NUMS_U8[..], &NUMS_U8[..]);
    }

    fn check_decode_8x_u8(a: &[u8]) {
        for i in a {
            for j in a {
                for k in a {
                    for l in a {
                        for m in a {
                            for n in a {
                                for o in a {
                                    for p in a {
                                        let mut enc = [0u8; 16];

                                        let first_len = encode_to_slice(*i, &mut enc);
                                        let second_len =
                                            encode_to_slice(*j, &mut enc[first_len as usize..]);
                                        let third_len = encode_to_slice(
                                            *k,
                                            &mut enc[(first_len + second_len) as usize..],
                                        );
                                        let fourth_len = encode_to_slice(
                                            *l,
                                            &mut enc
                                                [(first_len + second_len + third_len) as usize..],
                                        );
                                        let fifth_len = encode_to_slice(
                                            *m,
                                            &mut enc[(first_len
                                                + second_len
                                                + third_len
                                                + fourth_len)
                                                as usize..],
                                        );
                                        let sixth_len = encode_to_slice(
                                            *n,
                                            &mut enc[(first_len
                                                + second_len
                                                + third_len
                                                + fourth_len
                                                + fifth_len)
                                                as usize..],
                                        );
                                        let seventh_len = encode_to_slice(
                                            *o,
                                            &mut enc[(first_len
                                                + second_len
                                                + third_len
                                                + fourth_len
                                                + fifth_len
                                                + sixth_len)
                                                as usize..],
                                        );
                                        let eighth_len = encode_to_slice(
                                            *p,
                                            &mut enc[(first_len
                                                + second_len
                                                + third_len
                                                + fourth_len
                                                + fifth_len
                                                + sixth_len
                                                + seventh_len)
                                                as usize..],
                                        );

                                        let decoded =
                                            unsafe { decode_eight_u8_unsafe(enc.as_ptr()) };

                                        assert_eq!(decoded.0, [*i, *j, *k, *l, *m, *n, *o, *p]);
                                        assert_eq!(
                                            decoded.1,
                                            first_len
                                                + second_len
                                                + third_len
                                                + fourth_len
                                                + fifth_len
                                                + sixth_len
                                                + seventh_len
                                                + eighth_len
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_decode_8x_u8() {
        check_decode_8x_u8(&NUMS_U8[..]);
    }

    // #[test]
    // fn test_two() {
    //     // let result = unsafe { decode_two_unsafe::<u32, u32>([0x80, 0x80, 0x80, 0x80, 0x01, 0x80, 0x80, 0x80, 0x80, 0x01, 0, 0, 0, 0, 0, 0].as_ptr()) };
    //     let result = unsafe {
    //         decode_two_wide_unsafe::<u8, u8>(
    //             [
    //                 0x80, 0x01, 0x70, 0x01, 0x01, 0x80, 0x80, 0x80, 0x80, 0x01, 0, 0, 0, 0, 0, 0,
    //             ]
    //             .as_ptr(),
    //         )
    //     };
    //     println!("{:?}", result);
    // }
    //
    // #[test]
    // fn test_four() {
    //     let result = unsafe {
    //         decode_four_unsafe::<u16, u16, u16, u16>(
    //             [
    //                 0x01, 0x82, 0x01, 0x83, 0x80, 0x01, 0x84, 0x80, 0x01, 0, 0, 0, 0, 0, 0,
    //             ]
    //             .as_ptr(),
    //         )
    //     };
    //
    //     println!("{:?}", result);
    // }
    //
    // #[test]
    // fn test_eight() {
    //     let result = unsafe {
    //         decode_eight_u8_unsafe(
    //             [
    //                 0x80, 0x01, 0x80, 0x01, 0x01, 0x90, 0x01, 0x01, 0x01, 0x02, 0x90, 0x01, 0, 0,
    //                 0, 0, 0, 0, 0, 0,
    //             ]
    //             .as_ptr(),
    //         )
    //     };
    //
    //     println!("{:?}", result);
    // }
}
