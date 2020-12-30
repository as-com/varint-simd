/*!
`varint_simd` is a fast SIMD-accelerated [variable-length integer](https://developers.google.com/protocol-buffers/docs/encoding)
encoder and decoder written in Rust.

**For more information, please see the [README](https://github.com/as-com/varint-simd#readme).**
*/

#![cfg_attr(rustc_nightly, feature(doc_cfg))]

#[cfg(target_arch = "x86")]
use std::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use std::fmt::Debug;

pub mod decode;
pub mod encode;
pub mod num;

pub use decode::*;
pub use encode::*;

// Functions to help with debugging
#[allow(dead_code)]
fn slice_m128i(n: __m128i) -> [u8; 16] {
    unsafe { std::mem::transmute(n) }
}

#[allow(dead_code)]
fn slice_m256i(n: __m256i) -> [i8; 32] {
    unsafe { std::mem::transmute(n) }
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

#[cfg(test)]
mod tests {
    use crate::{decode, decode_two_unsafe, encode, VarIntTarget};

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

    #[test]
    fn test_two() {
        // let result = unsafe { decode_two_unsafe::<u32, u32>([0x80, 0x80, 0x80, 0x80, 0x01, 0x80, 0x80, 0x80, 0x80, 0x01, 0, 0, 0, 0, 0, 0].as_ptr()) };
        let result = unsafe {
            decode_two_unsafe::<u8, u8>(
                [
                    0x80, 0x01, 0x70, 0x01, 0x01, 0x80, 0x80, 0x80, 0x80, 0x01, 0, 0, 0, 0, 0, 0,
                ]
                .as_ptr(),
            )
        };
        println!("{:?}", result);
    }
}
