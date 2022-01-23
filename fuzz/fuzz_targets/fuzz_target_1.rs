#![no_main]
use libfuzzer_sys::fuzz_target;

use integer_encoding::VarInt;

fuzz_target!(|data: [u8; 16]| {
    let reference = u64::decode_var(&data);

    let simd = unsafe { varint_simd::decode_unsafe(data.as_ptr()) };

    if let Some(reference) = reference {
        assert_eq!(reference.0, simd.0);
        assert_eq!(reference.1, simd.1);
    }
});
