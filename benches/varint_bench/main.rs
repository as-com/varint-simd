use bytes::Buf;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use integer_encoding::VarInt;
use rand::distributions::{Distribution, Standard};
use rand::{thread_rng, Rng};
use varint_simd::{
    decode,
    decode_eight_u8_unsafe,
    decode_four_unsafe,
    decode_two_unsafe, //decode_two_wide_unsafe,
    decode_unsafe,
    encode,
    VarIntTarget,
};

mod leb128;
mod prost_varint;

#[inline(always)]
fn create_batched_encoded_generator<T: VarInt + Default, R: Rng, const C: usize>(
    rng: &mut R,
) -> impl FnMut() -> (Vec<u8>, Vec<T>) + '_
where
    Standard: Distribution<T>,
{
    move || {
        let mut encoded = Vec::new();
        let mut idx = 0;
        for _ in 0..C {
            if encoded.len() < idx + 16 {
                encoded.extend(std::iter::repeat(0).take(idx + 11 - encoded.len()))
            }
            let len = rng.gen::<T>().encode_var(&mut encoded[idx..]);
            idx += len;
        }
        (encoded, vec![Default::default(); C])
    }
}

#[inline(always)]
fn decode_batched_varint_simd_unsafe<T: VarIntTarget, const C: usize>(
    input: &mut (Vec<u8>, Vec<T>),
) {
    let data = &input.0;
    let out = &mut input.1;

    let mut slice = &data[..];
    for i in 0..C {
        // SAFETY: the input slice should have at least 16 bytes of allocated padding at the end
        let (num, len) = unsafe { decode_unsafe::<T>(slice.as_ptr()) };
        out[i] = num;
        slice = &slice[(len as usize)..];
    }
}

#[inline(always)]
fn decode_batched_varint_simd_2x_unsafe<T: VarIntTarget, const C: usize>(
    input: &mut (Vec<u8>, Vec<T>),
) {
    let data = &input.0;
    let out = &mut input.1;

    let mut slice = &data[..];
    for i in 0..(C / 2) {
        let (num1, num2, len1, len2) = unsafe { decode_two_unsafe::<T, T>(slice.as_ptr()) };
        out[i * 2] = num1;
        out[i * 2 + 1] = num2;
        slice = &slice[((len1 + len2) as usize)..];
    }
}

// #[inline(always)]
// fn decode_batched_varint_simd_2x_wide_unsafe<T: VarIntTarget, const C: usize>(
//     input: &mut (Vec<u8>, Vec<T>),
// ) {
//     let data = &input.0;
//     let out = &mut input.1;
//
//     let mut slice = &data[..];
//     for i in 0..(C / 2) {
//         let (num1, num2, len1, len2) = unsafe { decode_two_wide_unsafe::<T, T>(slice.as_ptr()) };
//         out[i * 2] = num1;
//         out[i * 2 + 1] = num2;
//         slice = &slice[((len1 + len2) as usize)..];
//     }
// }

#[inline(always)]
fn decode_batched_varint_simd_4x_unsafe<T: VarIntTarget, const C: usize>(
    input: &mut (Vec<u8>, Vec<T>),
) {
    let data = &input.0;
    let out = &mut input.1;

    let mut slice = &data[..];
    for i in 0..(C / 4) {
        let (num1, num2, num3, num4, len1, len2, len3, len4, _invalid) =
            unsafe { decode_four_unsafe::<T, T, T, T>(slice.as_ptr()) };
        out[i * 4] = num1;
        out[i * 4 + 1] = num2;
        out[i * 4 + 2] = num3;
        out[i * 4 + 3] = num4;
        slice = &slice[((len1 + len2 + len3 + len4) as usize)..];
    }
}

#[inline(always)]
fn decode_batched_varint_simd_8x_u8_unsafe<const C: usize>(input: &mut (Vec<u8>, Vec<u8>)) {
    let data = &input.0;
    let out = &mut input.1;

    let mut slice = &data[..];
    for i in 0..(C / 8) {
        let (nums, total_len) = unsafe { decode_eight_u8_unsafe(slice.as_ptr()) };
        out[(i * 8)..(i * 8 + 8)].copy_from_slice(&nums);
        slice = &slice[(total_len as usize)..];
    }
}

#[inline(always)]
fn decode_batched_varint_simd_safe<T: VarIntTarget, const C: usize>(input: &mut (Vec<u8>, Vec<T>)) {
    let data = &input.0;
    let out = &mut input.1;

    let mut slice = &data[..];
    for i in 0..C {
        let (num, len) = decode::<T>(slice).unwrap();
        out[i] = num;
        slice = &slice[(len as usize)..];
    }
}

#[inline(always)]
fn decode_batched_integer_encoding<T: VarInt, const C: usize>(input: &mut (Vec<u8>, Vec<T>)) {
    let data = &input.0;
    let out = &mut input.1;

    let mut slice = &data[..];
    for i in 0..C {
        let (num, len) = T::decode_var(slice).unwrap();
        out[i] = num;
        slice = &slice[len..];
    }
}

#[inline(always)]
fn decode_batched_rustc_u8<const C: usize>(input: &mut (Vec<u8>, Vec<u8>)) {
    let data = &input.0;
    let out = &mut input.1;

    let mut slice = &data[..];
    for i in 0..C {
        let (num, len) = leb128::read_u16_leb128(slice);
        out[i] = num as u8;
        slice = &slice[len..];
    }
}

#[inline(always)]
fn decode_batched_rustc_u16<const C: usize>(input: &mut (Vec<u8>, Vec<u16>)) {
    let data = &input.0;
    let out = &mut input.1;

    let mut slice = &data[..];
    for i in 0..C {
        let (num, len) = leb128::read_u16_leb128(slice);
        out[i] = num;
        slice = &slice[len..];
    }
}

#[inline(always)]
fn decode_batched_rustc_u32<const C: usize>(input: &mut (Vec<u8>, Vec<u32>)) {
    let data = &input.0;
    let out = &mut input.1;

    let mut slice = &data[..];
    for i in 0..C {
        let (num, len) = leb128::read_u32_leb128(slice);
        out[i] = num;
        slice = &slice[len..];
    }
}

#[inline(always)]
fn decode_batched_rustc_u64<const C: usize>(input: &mut (Vec<u8>, Vec<u64>)) {
    let data = &input.0;
    let out = &mut input.1;

    let mut slice = &data[..];
    for i in 0..C {
        let (num, len) = leb128::read_u64_leb128(slice);
        out[i] = num;
        slice = &slice[len..];
    }
}

#[inline(always)]
fn decode_batched_prost<T: VarIntTarget, const C: usize>(input: &mut (Vec<u8>, Vec<T>)) {
    let data = &input.0;
    let out = &mut input.1;

    let mut slice = &data[..];
    for i in 0..C {
        let num = prost_varint::decode_varint(&mut slice).unwrap();
        out[i] = T::cast_u64(num);
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = thread_rng();

    // Must be a multiple of 8
    const SEQUENCE_LEN: usize = 256;

    let mut group = c.benchmark_group("varint-u8/decode");
    group.throughput(Throughput::Elements(SEQUENCE_LEN as u64));
    group.bench_function("integer-encoding", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u8, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_integer_encoding::<u8, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("rustc", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u8, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_rustc_u8::<SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u8, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_prost::<u8, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/unsafe", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u8, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_varint_simd_unsafe::<u8, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/safe", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u8, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_varint_simd_safe::<u8, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/2x/unsafe", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u8, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_varint_simd_2x_unsafe::<u8, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/4x/unsafe", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u8, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_varint_simd_4x_unsafe::<u8, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/8x/unsafe", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u8, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_varint_simd_8x_u8_unsafe::<SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });
    group.finish();

    let mut group = c.benchmark_group("varint-u8/encode");
    group.throughput(Throughput::Elements(1));
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(
            || rng.gen::<u8>(),
            |num| {
                let mut target = [0u8; 16];
                u8::encode_var(num, &mut target)
            },
            BatchSize::SmallInput,
        )
    });

    let mut target = Vec::with_capacity(16);
    group.bench_function("rustc", |b| {
        b.iter_batched(
            || rng.gen::<u8>(),
            |num| {
                target.clear();
                leb128::write_u16_leb128(&mut target, num as u16);
            },
            BatchSize::SmallInput,
        )
    });

    let mut target = Vec::with_capacity(16);
    group.bench_function("prost-varint", |b| {
        b.iter_batched(
            || rng.gen::<u8>(),
            |num| {
                target.clear();
                prost_varint::encode_varint(num as u64, &mut target)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(|| rng.gen::<u8>(), |num| encode(num), BatchSize::SmallInput)
    });
    group.finish();

    let mut group = c.benchmark_group("varint-u16/decode");
    group.throughput(Throughput::Elements(SEQUENCE_LEN as u64));
    group.bench_function("integer-encoding", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u16, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_integer_encoding::<u16, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("rustc", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u16, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_rustc_u16::<SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u16, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_prost::<u16, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/unsafe", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u16, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_varint_simd_unsafe::<u16, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/safe", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u16, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_varint_simd_safe::<u16, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/2x/unsafe", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u16, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_varint_simd_2x_unsafe::<u16, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/4x/unsafe", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u16, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_varint_simd_4x_unsafe::<u16, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.finish();

    let mut group = c.benchmark_group("varint-u16/encode");
    group.throughput(Throughput::Elements(1));
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(
            || rng.gen::<u16>(),
            |num| {
                let mut target = [0u8; 16];
                u16::encode_var(num, &mut target)
            },
            BatchSize::SmallInput,
        )
    });

    let mut target = Vec::with_capacity(16);
    group.bench_function("rustc", |b| {
        b.iter_batched(
            || rng.gen::<u16>(),
            |num| {
                target.clear();
                leb128::write_u16_leb128(&mut target, num);
            },
            BatchSize::SmallInput,
        )
    });

    let mut target = Vec::with_capacity(16);
    group.bench_function("prost-varint", |b| {
        b.iter_batched(
            || rng.gen::<u16>(),
            |num| {
                target.clear();
                prost_varint::encode_varint(num as u64, &mut target)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(
            || rng.gen::<u16>(),
            |num| encode(num),
            BatchSize::SmallInput,
        )
    });
    group.finish();

    let mut group = c.benchmark_group("varint-u32/decode");
    group.throughput(Throughput::Elements(SEQUENCE_LEN as u64));
    group.bench_function("integer-encoding", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u32, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_integer_encoding::<u32, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("rustc", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u32, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_rustc_u32::<SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u32, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_prost::<u32, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/unsafe", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u32, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_varint_simd_unsafe::<u32, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/safe", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u32, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_varint_simd_safe::<u32, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/2x/unsafe", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u32, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_varint_simd_2x_unsafe::<u32, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });
    group.finish();

    let mut group = c.benchmark_group("varint-u32/encode");
    group.throughput(Throughput::Elements(1));
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(
            || rng.gen::<u32>(),
            |num| {
                let mut target = [0u8; 16];
                u32::encode_var(num, &mut target)
            },
            BatchSize::SmallInput,
        )
    });

    let mut target = Vec::with_capacity(16);
    group.bench_function("rustc", |b| {
        b.iter_batched(
            || rng.gen::<u32>(),
            |num| {
                target.clear();
                leb128::write_u32_leb128(&mut target, num);
            },
            BatchSize::SmallInput,
        )
    });

    let mut target = Vec::with_capacity(16);
    group.bench_function("prost-varint", |b| {
        b.iter_batched(
            || rng.gen::<u32>(),
            |num| {
                target.clear();
                prost_varint::encode_varint(num as u64, &mut target)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(
            || rng.gen::<u32>(),
            |num| encode(num),
            BatchSize::SmallInput,
        )
    });
    group.finish();

    let mut group = c.benchmark_group("varint-u64/decode");
    group.throughput(Throughput::Elements(SEQUENCE_LEN as u64));
    group.bench_function("integer-encoding", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u64, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_integer_encoding::<u64, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("rustc", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u64, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_rustc_u64::<SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u64, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_prost::<u64, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/unsafe", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u64, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_varint_simd_unsafe::<u64, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/safe", |b| {
        b.iter_batched_ref(
            create_batched_encoded_generator::<u64, _, SEQUENCE_LEN>(&mut rng),
            decode_batched_varint_simd_safe::<u64, SEQUENCE_LEN>,
            BatchSize::SmallInput,
        )
    });

    // group.bench_function("varint-simd/2x_wide/unsafe", |b| {
    //     b.iter_batched_ref(
    //         create_batched_encoded_generator::<u64, _, SEQUENCE_LEN>(&mut rng),
    //         decode_batched_varint_simd_2x_wide_unsafe::<u64, SEQUENCE_LEN>,
    //         BatchSize::SmallInput,
    //     )
    // });

    group.finish();

    let mut group = c.benchmark_group("varint-u64/encode");
    group.throughput(Throughput::Elements(1));
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(
            || rng.gen::<u64>(),
            |num| {
                let mut target = [0u8; 16];
                u64::encode_var(num, &mut target)
            },
            BatchSize::SmallInput,
        )
    });

    let mut target = Vec::with_capacity(16);
    group.bench_function("rustc", |b| {
        b.iter_batched(
            || rng.gen::<u64>(),
            |num| {
                target.clear();
                leb128::write_u64_leb128(&mut target, num);
            },
            BatchSize::SmallInput,
        )
    });

    let mut target = Vec::with_capacity(16);
    group.bench_function("prost-varint", |b| {
        b.iter_batched(
            || rng.gen::<u64>(),
            |num| {
                target.clear();
                prost_varint::encode_varint(num as u64, &mut target)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(
            || rng.gen::<u64>(),
            |num| encode(num),
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
