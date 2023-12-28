#![no_main]
use libfuzzer_sys::fuzz_target;

use integer_encoding::VarInt;

fuzz_target!(|data: [u8; 16]| {
    let reference = u64::decode_var(&data);

    let len = unsafe { varint_simd::decode_len_unsafe::<u64>(data.as_ptr()) };

    if let Some(reference) = reference {
        assert_eq!(reference.1, len);
    }
});
