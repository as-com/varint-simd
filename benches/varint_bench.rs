use criterion::{black_box, criterion_group, criterion_main, Criterion};
use integer_encoding::VarInt;
use varint_simd::decode_unsafe;

pub fn criterion_benchmark(c: &mut Criterion) {
    let my_u64 = 94949291991190 as u64;

    let mut encoded = [0;16];
    black_box(my_u64).encode_var(&mut encoded);

    assert_eq!(my_u64, u64::decode_var(&encoded).unwrap().0);
    assert_eq!(my_u64, unsafe { decode_unsafe(&encoded).0 });

    c.bench_function("integer-encoding", |b| b.iter(|| u64::decode_var(black_box(&encoded)).unwrap().0));
    c.bench_function("varint-simd", |b| b.iter(|| unsafe { decode_unsafe(black_box(&encoded)).0 }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);