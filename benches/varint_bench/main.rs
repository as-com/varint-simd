use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use integer_encoding::VarInt;
use rand::distributions::{Distribution, Standard};
use rand::{thread_rng, Rng};
use varint_simd::{
    decode, decode_eight_u8_unsafe, decode_four_unsafe, decode_two_unsafe, decode_two_wide_unsafe,
    decode_unsafe, encode,
};

mod leb128;
mod prost_varint;

fn create_encoded_generator<T: VarInt, R: Rng>(rng: &mut R) -> impl FnMut() -> [u8; 16] + '_
where
    Standard: Distribution<T>,
{
    move || {
        let mut encoded = [0; 16];
        rng.gen::<T>().encode_var(&mut encoded);
        encoded
    }
}

fn create_double_encoded_generator<T: VarInt, U: VarInt, R: Rng>(
    rng: &mut R,
) -> impl FnMut() -> [u8; 16] + '_
where
    Standard: Distribution<T>,
    Standard: Distribution<U>,
{
    move || {
        let mut encoded = [0; 16];
        let first_len = rng.gen::<T>().encode_var(&mut encoded);
        rng.gen::<U>().encode_var(&mut encoded[first_len..]);
        encoded
    }
}

fn create_double_encoded_generator_wide<T: VarInt, U: VarInt, R: Rng>(
    rng: &mut R,
) -> impl FnMut() -> [u8; 32] + '_
where
    Standard: Distribution<T>,
    Standard: Distribution<U>,
{
    move || {
        let mut encoded = [0; 32];
        let first_len = rng.gen::<T>().encode_var(&mut encoded);
        rng.gen::<U>().encode_var(&mut encoded[first_len..]);
        encoded
    }
}

fn create_quad_encoded_generator<T: VarInt, U: VarInt, V: VarInt, W: VarInt, R: Rng>(
    rng: &mut R,
) -> impl FnMut() -> [u8; 16] + '_
where
    Standard: Distribution<T>,
    Standard: Distribution<U>,
    Standard: Distribution<V>,
    Standard: Distribution<W>,
{
    move || {
        let mut encoded = [0; 16];
        let first_len = rng.gen::<T>().encode_var(&mut encoded);
        let second_len = rng.gen::<U>().encode_var(&mut encoded[first_len..]);
        let third_len = rng
            .gen::<V>()
            .encode_var(&mut encoded[first_len + second_len..]);
        rng.gen::<W>()
            .encode_var(&mut encoded[first_len + second_len + third_len..]);
        encoded
    }
}

fn create_octuple_encoded_generator<R: Rng>(rng: &mut R) -> impl FnMut() -> [u8; 16] + '_ {
    move || {
        let mut encoded = [0; 16];
        let first_len = rng.gen::<u8>().encode_var(&mut encoded);
        let second_len = rng.gen::<u8>().encode_var(&mut encoded[first_len..]);
        let third_len = rng
            .gen::<u8>()
            .encode_var(&mut encoded[first_len + second_len..]);
        let fourth_len = rng
            .gen::<u8>()
            .encode_var(&mut encoded[first_len + second_len + third_len..]);
        let fifth_len = rng
            .gen::<u8>()
            .encode_var(&mut encoded[first_len + second_len + third_len + fourth_len..]);
        let sixth_len = rng.gen::<u8>().encode_var(
            &mut encoded[first_len + second_len + third_len + fourth_len + fifth_len..],
        );
        let seventh_len = rng.gen::<u8>().encode_var(
            &mut encoded[first_len + second_len + third_len + fourth_len + fifth_len + sixth_len..],
        );
        rng.gen::<u8>().encode_var(
            &mut encoded[first_len
                + second_len
                + third_len
                + fourth_len
                + fifth_len
                + sixth_len
                + seventh_len..],
        );

        encoded
    }
}

fn create_encoded_vec_generator<T: VarInt, R: Rng>(rng: &mut R) -> impl FnMut() -> Vec<u8> + '_
where
    Standard: Distribution<T>,
{
    move || {
        let mut encoded = [0; 16];
        rng.gen::<T>().encode_var(&mut encoded);
        encoded.to_vec()
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = thread_rng();

    let mut group = c.benchmark_group("varint-u8/decode");
    group.throughput(Throughput::Elements(1));
    group.bench_function("integer-encoding", |b| {
        b.iter_batched_ref(
            create_encoded_generator::<u8, _>(&mut rng),
            |encoded| u8::decode_var(encoded).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("rustc", |b| {
        b.iter_batched_ref(
            create_encoded_generator::<u8, _>(&mut rng),
            |encoded| leb128::read_u16_leb128(encoded),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched_ref(
            create_encoded_vec_generator::<u8, _>(&mut rng),
            |encoded| prost_varint::decode_varint(&mut encoded.as_slice()).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/unsafe", |b| {
        b.iter_batched_ref(
            create_encoded_generator::<u8, _>(&mut rng),
            |encoded| unsafe { decode_unsafe::<u8>(encoded.as_ptr()) },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/safe", |b| {
        b.iter_batched_ref(
            create_encoded_generator::<u8, _>(&mut rng),
            |encoded| decode::<u8>(encoded).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.throughput(Throughput::Elements(2));
    group.bench_function("varint-simd/2x/unsafe", |b| {
        b.iter_batched_ref(
            create_double_encoded_generator::<u8, u8, _>(&mut rng),
            |encoded| unsafe { decode_two_unsafe::<u8, u8>(encoded.as_ptr()) },
            BatchSize::SmallInput,
        )
    });

    group.throughput(Throughput::Elements(4));
    group.bench_function("varint-simd/4x/unsafe", |b| {
        b.iter_batched_ref(
            create_quad_encoded_generator::<u8, u8, u8, u8, _>(&mut rng),
            |encoded| unsafe { decode_four_unsafe::<u8, u8, u8, u8>(encoded.as_ptr()) },
            BatchSize::SmallInput,
        )
    });

    group.throughput(Throughput::Elements(8));
    group.bench_function("varint-simd/8x/unsafe", |b| {
        b.iter_batched_ref(
            create_octuple_encoded_generator(&mut rng),
            |encoded| unsafe { decode_eight_u8_unsafe(encoded.as_ptr()) },
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
    group.throughput(Throughput::Elements(1));
    group.bench_function("integer-encoding", |b| {
        b.iter_batched_ref(
            create_encoded_generator::<u16, _>(&mut rng),
            |encoded| u16::decode_var(encoded).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("rustc", |b| {
        b.iter_batched_ref(
            create_encoded_generator::<u16, _>(&mut rng),
            |encoded| leb128::read_u16_leb128(encoded),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched_ref(
            create_encoded_vec_generator::<u16, _>(&mut rng),
            |encoded| prost_varint::decode_varint(&mut encoded.as_slice()).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/unsafe", |b| {
        b.iter_batched_ref(
            create_encoded_generator::<u16, _>(&mut rng),
            |encoded| unsafe { decode_unsafe::<u16>(encoded.as_ptr()) },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/safe", |b| {
        b.iter_batched_ref(
            create_encoded_generator::<u16, _>(&mut rng),
            |encoded| decode::<u16>(encoded).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.throughput(Throughput::Elements(2));
    group.bench_function("varint-simd/2x/unsafe", |b| {
        b.iter_batched_ref(
            create_double_encoded_generator::<u16, u16, _>(&mut rng),
            |encoded| unsafe { decode_two_unsafe::<u16, u16>(encoded.as_ptr()) },
            BatchSize::SmallInput,
        )
    });

    group.throughput(Throughput::Elements(4));
    group.bench_function("varint-simd/4x/unsafe", |b| {
        b.iter_batched_ref(
            create_quad_encoded_generator::<u16, u16, u16, u16, _>(&mut rng),
            |encoded| unsafe { decode_four_unsafe::<u16, u16, u16, u16>(encoded.as_ptr()) },
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
    group.throughput(Throughput::Elements(1));
    group.bench_function("integer-encoding", |b| {
        b.iter_batched_ref(
            create_encoded_generator::<u32, _>(&mut rng),
            |encoded| u32::decode_var(encoded).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("rustc", |b| {
        b.iter_batched_ref(
            create_encoded_generator::<u32, _>(&mut rng),
            |encoded| leb128::read_u32_leb128(encoded),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched_ref(
            create_encoded_vec_generator::<u32, _>(&mut rng),
            |encoded| prost_varint::decode_varint(&mut encoded.as_slice()).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/unsafe", |b| {
        b.iter_batched_ref(
            create_encoded_generator::<u32, _>(&mut rng),
            |encoded| unsafe { decode_unsafe::<u32>(encoded.as_ptr()) },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/safe", |b| {
        b.iter_batched_ref(
            create_encoded_generator::<u32, _>(&mut rng),
            |encoded| decode::<u32>(encoded).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.throughput(Throughput::Elements(2));
    group.bench_function("varint-simd/2x/unsafe", |b| {
        b.iter_batched_ref(
            create_double_encoded_generator::<u32, u32, _>(&mut rng),
            |encoded| unsafe { decode_two_unsafe::<u32, u32>(encoded.as_ptr()) },
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
    group.throughput(Throughput::Elements(1));
    group.bench_function("integer-encoding", |b| {
        b.iter_batched_ref(
            create_encoded_generator::<u64, _>(&mut rng),
            |encoded| u64::decode_var(encoded).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("rustc", |b| {
        b.iter_batched_ref(
            create_encoded_generator::<u64, _>(&mut rng),
            |encoded| leb128::read_u64_leb128(encoded),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched_ref(
            create_encoded_vec_generator::<u64, _>(&mut rng),
            |encoded| prost_varint::decode_varint(&mut encoded.as_slice()).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/unsafe", |b| {
        b.iter_batched_ref(
            create_encoded_generator::<u64, _>(&mut rng),
            |encoded| unsafe { decode_unsafe::<u64>(encoded.as_ptr()) },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd/safe", |b| {
        b.iter_batched_ref(
            create_encoded_generator::<u64, _>(&mut rng),
            |encoded| decode::<u64>(encoded).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.throughput(Throughput::Elements(2));
    group.bench_function("varint-simd/2x_wide/unsafe", |b| {
        b.iter_batched_ref(
            create_double_encoded_generator_wide::<u64, u64, _>(&mut rng),
            |encoded| unsafe { decode_two_wide_unsafe::<u64, u64>(encoded.as_ptr()) },
            BatchSize::SmallInput,
        )
    });

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
