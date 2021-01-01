varint-simd
==
[![Crates.io](https://img.shields.io/crates/v/varint-simd)](https://crates.io/crates/varint-simd)
[![Docs.rs](https://docs.rs/varint-simd/badge.svg)](https://docs.rs/varint-simd)
[![Continuous integration](https://github.com/as-com/varint-simd/workflows/Continuous%20integration/badge.svg)](https://github.com/as-com/varint-simd/actions?query=workflow%3A%22Continuous+integration%22)

varint-simd is a fast SIMD-accelerated [variable-length integer](https://developers.google.com/protocol-buffers/docs/encoding) 
encoder and decoder written in Rust. It is intended for use in implementations of Protocol Buffers (protobuf), Apache
Avro, and similar serialization formats.

This library currently targets a minimum of x86_64 processors with support for SSSE3 (Intel Core/AMD Bulldozer or 
newer), with optional optimizations for processors supporting POPCNT, LZCNT, BMI2, and/or AVX2.

## Usage
**Important:** For optimal performance, ensure the Rust compiler has an appropriate `target-cpu` setting. An example is
provided in [`.cargo/config`](.cargo/config), but you may need to edit the file to specify the oldest CPUs your compiled
binaries will support.

The `native-optimizations` feature should be enabled if and only if `target-cpu` is set to `native`, such as in the 
example. This enables some extra optimizations if suitable for your specific CPU. 
[Read more below.](#about-the-native-optimizations-feature)

```rust
use varint_simd::{encode, decode, encode_zigzag, decode_zigzag};

fn main() {
  let num: u32 = 300;
  
  let encoded = encode::<u32>(num); // turbofish for demonstration purposes, usually not necessary
  // encoded now contains a tuple
  // (
  //    [0xAC, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // encoded in a 128-bit vector
  //    2 // the number of bytes encoded
  // )
  
  let decoded = decode::<u32>(&encoded.0).unwrap();
  // decoded now contains another tuple:
  // (
  //    300, // the decoded number
  //    2 // the number of bytes read from the slice
  // )
  assert_eq!(decoded.0, num);
  
  // Signed integers can be encoded/decoded with convenience functions encode_zigzag and decode_zigzag
  let num: i32 = -20;
  let encoded = encode_zigzag::<i32>(num);
  let decoded = decode_zigzag::<i32>(&encoded.0).unwrap();
  assert_eq!(decoded.0, num);
}
```

The type parameter passed into the encode/decode functions greatly affects performance - the code takes shorter paths
for shorter integers, and may exhibit comparatively poor performance if you're decoding a lot of tiny integers 
into u64's.

## Safety
This crate uses *a lot* of unsafe code. Please exercise caution, although I do not expect there to be major issues.

There is also an optional "unsafe" interface for bypassing overflow and bounds checks. This can be used when you know 
your input data won't cause undefined behavior and your calling code can tolerate truncated numbers.

## Benchmarks
The benchmarks below reflect the performance of decoding and encoding a sequence of random integers bounded by each 
integer size. All benchmarks are run with native optimizations. 
For more details, please see [the source code for these benchmarks](benches/varint_bench.rs).

### Intel Core i7-8850H "Coffee Lake" (2018 15" MacBook Pro)

![benchmark graph](images/benchmark.png)

#### Decode
|   | varint-simd unsafe | varint-simd safe | [rustc](https://github.com/nnethercote/rust/blob/0f6f2d681b39c5f95459cd09cb936b6ceb27cd82/compiler/rustc_serialize/src/leb128.rs) | [integer-encoding-rs](https://github.com/dermesser/integer-encoding-rs) | [prost](https://github.com/danburkert/prost) |
| -- | -- | -- | -- | -- | -- |
| `u8`  | **1.85 ns** | **2.80 ns** | 7.23 ns | 7.18 ns | 70.6 ns |
| `u16` | **1.95 ns** | **2.78 ns** | 5.54 ns | 7.17 ns | 71.5 ns |
| `u32` | **2.41 ns** | **3.27 ns** | 7.35 ns | 7.41 ns | 73.6 ns |
| `u64` | **3.65 ns** | **4.15 ns** | 11.0 ns | 15.2 ns | 71.9 ns |

#### Encode
|   | varint-simd | rustc | integer-encoding-rs | prost |
| -- | -- | -- | -- | -- |
| `u8`  | **2.50 ns** | 5.20 ns | 6.24 ns | 10.5 ns |
| `u16` | **2.65 ns** | 5.47 ns | 6.63 ns | 11.5 ns |
| `u32` | **2.96 ns** | 6.43 ns | 7.74 ns | 13.7 ns |
| `u64` | **3.85 ns** | 14.1 ns | 13.0 ns | 21.8 ns |

### AMD Ryzen 5 2600X @ 4.125 GHz "Zen+"
#### Decode
|   | varint-simd unsafe | varint-simd safe | rustc | integer-encoding-rs | prost |
| -- | -- | -- | -- | -- | -- |
| `u8`  | **2.62 ns** | **3.66 ns** | 7.57 ns | 8.27 ns | 37.6 ns |
| `u16` | **3.14 ns** | **3.98 ns** | 6.57 ns | 7.56 ns | 36.7 ns |
| `u32` | **4.36 ns** | **4.83 ns** | 6.57 ns | 7.98 ns | 36.2 ns |
| `u64` | **6.97 ns** | **7.12 ns** | 12.5 ns | 13.2 ns | 40.3 ns |

#### Encode
|   | varint-simd | rustc | integer-encoding-rs | prost |
| -- | -- | -- | -- | -- |
| `u8`  | **3.94 ns** | 4.64 ns | 7.65 ns | 10.4 ns |
| `u16` | **4.23 ns** | 6.03 ns | 7.51 ns | 10.6 ns |
| `u32` | **4.62 ns** | 9.33 ns | 8.94 ns | 12.9 ns |
| `u64` | **5.78 ns** | 19.3 ns | 14.1 ns | 21.5 ns |

## TODO
* Encoding multiple values at once
* Support for ARM NEON
* Fallback scalar implementation
* Further optimization (I'm pretty sure I left some performance on the table)

Contributions are welcome. 🙂

## About the `native-optimizations` feature

This feature flag enables a build script that detects the current CPU and enables PDEP/PEXT optimizations if the CPU
supports running these instructions efficiently. It should be enabled if and only if the `target-cpu` option is set to 
`native`.

This is necessary because AMD Zen, Zen+, and Zen 2 processors implement these instructions in microcode, which means
they run much, much slower than if they were implemented in hardware. Additionally, [Rust does not allow conditional
compilation based on the `target-cpu` option](https://github.com/rust-lang/rust/issues/44036), so it is necessary to 
specify this feature manually.

Library crates **should not** enable this feature by default. A separate feature flag should be provided to enable this
feature in this crate. 

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
