#![no_main]
use libfuzzer_sys::fuzz_target;

use integer_encoding::VarInt;

fuzz_target!(|data: u64| {
    let mut reference_out = [0u8; 16];
    let reference_size = u64::encode_var(data, &mut reference_out);

    let (simd_out, simd_size) = unsafe { varint_simd::encode_unsafe(data) };

    assert_eq!(reference_size, simd_size as usize);
    assert_eq!(reference_out[0..reference_size], simd_out[0..reference_size]);
});
