use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize};
use integer_encoding::VarInt;
use varint_simd::decode_unsafe;
use rand::{thread_rng, Rng};

pub fn criterion_benchmark(c: &mut Criterion) {
    let my_u64 = 4294967295;

    let mut encoded = [0;16];
    black_box(my_u64).encode_var(&mut encoded);

    assert_eq!(my_u64, u32::decode_var(&encoded).unwrap().0);
    assert_eq!(my_u64, unsafe { decode_unsafe::<u32>(&encoded).0 });

    let mut rng = thread_rng();

    let mut group = c.benchmark_group("varint-u8");
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(|| {
            let mut encoded = [0;16];
            rng.gen::<u8>().encode_var(&mut encoded);
            encoded
        }, |encoded| {
            u64::decode_var(black_box(&encoded)).unwrap().0
        }, BatchSize::SmallInput)
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(|| {
            let mut encoded = [0;16];
            rng.gen::<u8>().encode_var(&mut encoded);
            encoded
        }, |encoded| {
            unsafe { decode_unsafe::<u8>(black_box(&encoded)).0 }
        }, BatchSize::SmallInput)
    });
    group.finish();

    let mut group = c.benchmark_group("varint-u16");
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(|| {
            let mut encoded = [0;16];
            rng.gen::<u16>().encode_var(&mut encoded);
            encoded
        }, |encoded| {
            u64::decode_var(black_box(&encoded)).unwrap().0
        }, BatchSize::SmallInput)
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(|| {
            let mut encoded = [0;16];
            rng.gen::<u16>().encode_var(&mut encoded);
            encoded
        }, |encoded| {
            unsafe { decode_unsafe::<u16>(black_box(&encoded)).0 }
        }, BatchSize::SmallInput)
    });
    group.finish();

    let mut group = c.benchmark_group("varint-u32");
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(|| {
            let mut encoded = [0;16];
            rng.gen::<u32>().encode_var(&mut encoded);
            encoded
        }, |encoded| {
            u64::decode_var(black_box(&encoded)).unwrap().0
        }, BatchSize::SmallInput)
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(|| {
            let mut encoded = [0;16];
            rng.gen::<u32>().encode_var(&mut encoded);
            encoded
        }, |encoded| {
            unsafe { decode_unsafe::<u32>(black_box(&encoded)).0 }
        }, BatchSize::SmallInput)
    });
    group.finish();

    let mut group = c.benchmark_group("varint-u64");
    group.bench_function("integer-encoding", |b| {
        b.iter_batched(|| {
            let mut encoded = [0;16];
            rng.gen::<u64>().encode_var(&mut encoded);
            encoded
        }, |encoded| {
            u64::decode_var(black_box(&encoded)).unwrap().0
        }, BatchSize::SmallInput)
    });

    group.bench_function("varint-simd", |b| {
        b.iter_batched(|| {
            let mut encoded = [0;16];
            rng.gen::<u64>().encode_var(&mut encoded);
            encoded
        }, |encoded| {
            unsafe { decode_unsafe::<u64>(black_box(&encoded)).0 }
        }, BatchSize::SmallInput)
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);