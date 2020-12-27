use bytes::{Buf, BufMut};
use std::cmp::min;

/// Encodes an integer value into LEB128 variable length format, and writes it to the buffer.
/// The buffer must have enough remaining space (maximum 10 bytes).
#[inline]
pub fn encode_varint<B>(mut value: u64, buf: &mut B)
    where
        B: BufMut,
{
    // Safety notes:
    //
    // - ptr::write is an unsafe raw pointer write. The use here is safe since the length of the
    //   uninit slice is checked.
    // - advance_mut is unsafe because it could cause uninitialized memory to be advanced over. The
    //   use here is safe since each byte which is advanced over has been written to in the
    //   previous loop iteration.
    unsafe {
        let mut i;
        'outer: loop {
            i = 0;

            let uninit_slice = buf.chunk_mut();
            for offset in 0..uninit_slice.len() {
                i += 1;
                let ptr = uninit_slice.as_mut_ptr().add(offset);
                if value < 0x80 {
                    ptr.write(value as u8);
                    break 'outer;
                } else {
                    ptr.write(((value & 0x7F) | 0x80) as u8);
                    value >>= 7;
                }
            }

            buf.advance_mut(i);
            debug_assert!(buf.has_remaining_mut());
        }

        buf.advance_mut(i);
    }
}

/// Decodes a LEB128-encoded variable length integer from the buffer.
pub fn decode_varint<B>(buf: &mut B) -> Result<u64, ()>
where
    B: Buf,
{
    let bytes = buf.chunk();
    let len = bytes.len();
    if len == 0 {
        return Err(());
    }

    let byte = unsafe { *bytes.get_unchecked(0) };
    if byte < 0x80 {
        buf.advance(1);
        Ok(u64::from(byte))
    } else if len > 10 || bytes[len - 1] < 0x80 {
        let (value, advance) = unsafe { decode_varint_slice(bytes) }?;
        buf.advance(advance);
        Ok(value)
    } else {
        decode_varint_slow(buf)
    }
}

/// Decodes a LEB128-encoded variable length integer from the slice, returning the value and the
/// number of bytes read.
///
/// Based loosely on [`ReadVarint64FromArray`][1].
///
/// ## Safety
///
/// The caller must ensure that `bytes` is non-empty and either `bytes.len() >= 10` or the last
/// element in bytes is < `0x80`.
///
/// [1]: https://github.com/google/protobuf/blob/3.3.x/src/google/protobuf/io/coded_stream.cc#L365-L406
#[inline]
unsafe fn decode_varint_slice(bytes: &[u8]) -> Result<(u64, usize), ()> {
    // Fully unrolled varint decoding loop. Splitting into 32-bit pieces gives better performance.

    let mut b: u8;
    let mut part0: u32;
    b = *bytes.get_unchecked(0);
    part0 = u32::from(b);
    if b < 0x80 {
        return Ok((u64::from(part0), 1));
    };
    part0 -= 0x80;
    b = *bytes.get_unchecked(1);
    part0 += u32::from(b) << 7;
    if b < 0x80 {
        return Ok((u64::from(part0), 2));
    };
    part0 -= 0x80 << 7;
    b = *bytes.get_unchecked(2);
    part0 += u32::from(b) << 14;
    if b < 0x80 {
        return Ok((u64::from(part0), 3));
    };
    part0 -= 0x80 << 14;
    b = *bytes.get_unchecked(3);
    part0 += u32::from(b) << 21;
    if b < 0x80 {
        return Ok((u64::from(part0), 4));
    };
    part0 -= 0x80 << 21;
    let value = u64::from(part0);

    let mut part1: u32;
    b = *bytes.get_unchecked(4);
    part1 = u32::from(b);
    if b < 0x80 {
        return Ok((value + (u64::from(part1) << 28), 5));
    };
    part1 -= 0x80;
    b = *bytes.get_unchecked(5);
    part1 += u32::from(b) << 7;
    if b < 0x80 {
        return Ok((value + (u64::from(part1) << 28), 6));
    };
    part1 -= 0x80 << 7;
    b = *bytes.get_unchecked(6);
    part1 += u32::from(b) << 14;
    if b < 0x80 {
        return Ok((value + (u64::from(part1) << 28), 7));
    };
    part1 -= 0x80 << 14;
    b = *bytes.get_unchecked(7);
    part1 += u32::from(b) << 21;
    if b < 0x80 {
        return Ok((value + (u64::from(part1) << 28), 8));
    };
    part1 -= 0x80 << 21;
    let value = value + ((u64::from(part1)) << 28);

    let mut part2: u32;
    b = *bytes.get_unchecked(8);
    part2 = u32::from(b);
    if b < 0x80 {
        return Ok((value + (u64::from(part2) << 56), 9));
    };
    part2 -= 0x80;
    b = *bytes.get_unchecked(9);
    part2 += u32::from(b) << 7;
    if b < 0x80 {
        return Ok((value + (u64::from(part2) << 56), 10));
    };

    // We have overrun the maximum size of a varint (10 bytes). Assume the data is corrupt.
    Err(())
}

/// Decodes a LEB128-encoded variable length integer from the buffer, advancing the buffer as
/// necessary.
#[inline(never)]
fn decode_varint_slow<B>(buf: &mut B) -> Result<u64, ()>
where
    B: Buf,
{
    let mut value = 0;
    for count in 0..min(10, buf.remaining()) {
        let byte = buf.get_u8();
        value |= u64::from(byte & 0x7F) << (count * 7);
        if byte <= 0x7F {
            return Ok(value);
        }
    }

    Err(())
}
