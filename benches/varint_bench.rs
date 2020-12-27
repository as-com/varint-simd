use bytes::Buf;
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use integer_encoding::VarInt;
use rand::{thread_rng, Rng};
use varint_simd::{decode_unsafe, encode_unsafe, decode_three_unsafe};
use rand::prelude::ThreadRng;

mod prost_varint;

pub fn criterion_benchmark(c: &mut Criterion) {
    let my_u32 = 4294967295;

    let mut encoded = [0; 16];
    black_box(my_u32).encode_var(&mut encoded);

    assert_eq!(my_u32, u32::decode_var(&encoded).unwrap().0);
    assert_eq!(my_u32, unsafe { decode_unsafe::<u32>(&encoded).0 });
    assert_eq!(
        my_u32 as u64,
        prost_varint::decode_varint(&mut encoded.to_vec().as_slice()).unwrap()
    );

    let mut rng = thread_rng();

    let mut group = c.benchmark_group("varint-u8/decode");
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(
            || {
                let mut encoded = [0; 16];
                rng.gen::<u8>().encode_var(&mut encoded);
                encoded
            },
            |encoded| u8::decode_var(black_box(&encoded)).unwrap().0,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched(
            || {
                let mut encoded = [0; 16];
                rng.gen::<u8>().encode_var(&mut encoded);
                encoded.to_vec()
            },
            |mut encoded| prost_varint::decode_varint(black_box(&mut encoded.as_slice())).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(
            || {
                let mut encoded = [0; 16];
                rng.gen::<u8>().encode_var(&mut encoded);
                encoded
            },
            |encoded| unsafe { decode_unsafe::<u8>(black_box(&encoded)).0 },
            BatchSize::SmallInput,
        )
    });
    group.finish();

    let mut group = c.benchmark_group("varint-u8/encode");
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(
            || {
                rng.gen::<u8>()
            },
            |num| {
                let mut target = [0u8;16];
                u8::encode_var(num,&mut target)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched(
            || {
                rng.gen::<u8>()
            },
            |num| {
                let mut target = Vec::with_capacity(16);
                prost_varint::encode_varint(num as u64, &mut target)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(
            || {
                rng.gen::<u8>()
            },
            |num| {
                unsafe { encode_unsafe(num) }
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();

    let mut group = c.benchmark_group("varint-u16/decode");
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(
            || {
                let mut encoded = [0; 16];
                rng.gen::<u16>().encode_var(&mut encoded);
                encoded
            },
            |encoded| u16::decode_var(black_box(&encoded)).unwrap().0,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched(
            || {
                let mut encoded = [0; 16];
                rng.gen::<u16>().encode_var(&mut encoded);
                encoded.to_vec()
            },
            |mut encoded| prost_varint::decode_varint(black_box(&mut encoded.as_slice())).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(
            || {
                let mut encoded = [0; 16];
                rng.gen::<u16>().encode_var(&mut encoded);
                encoded
            },
            |encoded| unsafe { decode_unsafe::<u16>(black_box(&encoded)).0 },
            BatchSize::SmallInput,
        )
    });
    group.finish();

    let mut group = c.benchmark_group("varint-u16/encode");
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(
            || {
                rng.gen::<u16>()
            },
            |num| {
                let mut target = [0u8;16];
                u16::encode_var(num,&mut target)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched(
            || {
                rng.gen::<u16>()
            },
            |num| {
                let mut target = Vec::with_capacity(16);
                prost_varint::encode_varint(num as u64, &mut target)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(
            || {
                rng.gen::<u16>()
            },
            |num| {
                unsafe { encode_unsafe(num) }
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();

    let mut group = c.benchmark_group("varint-u32/decode");
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(
            || {
                let mut encoded = [0; 16];
                rng.gen::<u32>().encode_var(&mut encoded);
                encoded
            },
            |encoded| u64::decode_var(black_box(&encoded)).unwrap().0,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched(
            || {
                let mut encoded = [0; 16];
                rng.gen::<u32>().encode_var(&mut encoded);
                encoded.to_vec()
            },
            |mut encoded| prost_varint::decode_varint(black_box(&mut encoded.as_slice())).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(
            || {
                let mut encoded = [0; 16];
                rng.gen::<u32>().encode_var(&mut encoded);
                encoded
            },
            |encoded| unsafe { decode_unsafe::<u32>(black_box(&encoded)).0 },
            BatchSize::SmallInput,
        )
    });
    group.finish();

    let mut group = c.benchmark_group("varint-u32/encode");
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(
            || {
                rng.gen::<u32>()
            },
            |num| {
                let mut target = [0u8;16];
                u32::encode_var(num,&mut target)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched(
            || {
                rng.gen::<u32>()
            },
            |num| {
                let mut target = Vec::with_capacity(16);
                prost_varint::encode_varint(num as u64, &mut target)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(
            || {
                rng.gen::<u32>()
            },
            |num| {
                unsafe { encode_unsafe(num) }
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();

    let mut group = c.benchmark_group("varint-u64");
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(
            || {
                let mut encoded = [0; 16];
                rng.gen::<u64>().encode_var(&mut encoded);
                encoded
            },
            |encoded| u64::decode_var(black_box(&encoded)).unwrap().0,
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched(
            || {
                let mut encoded = [0; 16];
                rng.gen::<u64>().encode_var(&mut encoded);
                encoded.to_vec()
            },
            |mut encoded| prost_varint::decode_varint(black_box(&mut encoded.as_slice())).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(
            || {
                let mut encoded = [0; 16];
                rng.gen::<u64>().encode_var(&mut encoded);
                encoded
            },
            |encoded| unsafe { decode_unsafe::<u64>(black_box(&encoded)).0 },
            BatchSize::SmallInput,
        )
    });
    group.finish();

    let mut group = c.benchmark_group("varint-u64/encode");
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(
            || {
                rng.gen::<u64>()
            },
            |num| {
                let mut target = [0u8;16];
                u64::encode_var(num,&mut target)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched(
            || {
                rng.gen::<u64>()
            },
            |num| {
                let mut target = Vec::with_capacity(16);
                prost_varint::encode_varint(num as u64, &mut target)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(
            || {
                rng.gen::<u64>()
            },
            |num| {
                unsafe { encode_unsafe(num) }
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();

    #[inline]
    fn generate_triple_u64_var(rng: &mut ThreadRng) -> [u8; 36] {
        let mut encoded = [0; 36];
        let first_len = rng.gen::<u64>().encode_var(&mut encoded);
        let second_len = rng.gen::<u64>().encode_var(&mut encoded[first_len..]);
        rng.gen::<u64>().encode_var(&mut encoded[first_len+second_len..]);
        encoded
    }

    let mut group = c.benchmark_group("varint-u64/triple");
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(
            || {
                generate_triple_u64_var(&mut rng)
            },
            |encoded| {
                let first = u64::decode_var(&encoded).unwrap();
                let second = u64::decode_var(&encoded[first.1..]).unwrap();
                let third = u64::decode_var(&encoded[first.1+second.1..]).unwrap();

                (first.0, second.0, third.0)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("prost-varint", |b| {
        b.iter_batched(
            || {
                generate_triple_u64_var(&mut rng).to_vec()
            },
            |mut encoded| {
                let mut slice = encoded.as_slice();
                let first = prost_varint::decode_varint(black_box(&mut slice)).unwrap();
                let second = prost_varint::decode_varint(black_box(&mut slice)).unwrap();
                let third = prost_varint::decode_varint(black_box(&mut slice)).unwrap();

                (first, second, third)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(
            || {
                generate_triple_u64_var(&mut rng).to_vec()
            },
            |encoded| {
                let decoded = unsafe { decode_three_unsafe::<u64, u64, u64>(&encoded) };
                (decoded.0, decoded.2, decoded.4)
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
