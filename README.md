varint-simd
==

Very fast varint encoder and decoder written in Rust.

## Benchmarks
[Source code for benchmarks](benches/varint_bench.rs)

### Intel Core i7-8850H "Coffee Lake" (2018 15" MacBook Pro)

![benchmark graph](benchmark.png)

#### Decode
|   | varint-simd unsafe | varint-simd safe | integer-encoding-rs | prost |
| -- | -- | -- | -- | -- |
| `u8`  | **2.27 ns** | **3.19 ns** | 8.68 ns | 73.3 ns |
| `u16` | **3.04 ns** | **3.88 ns** | 7.63 ns | 71.0 ns |
| `u32` | **4.30 ns** | **5.24 ns** | 8.26 ns | 69.7 ns |
| `u64` | **7.50 ns** | **8.68 ns** | 13.3 ns | 74.1 ns |

#### Encode
|   | varint-simd | integer-encoding-rs | prost |
| -- | -- | -- | -- |
| `u8`  | **2.79 ns** | 7.58 ns | 68.6 ns |
| `u16` | **3.39 ns** | 7.22 ns | 69.3 ns |
| `u32` | **4.37 ns** | 8.62 ns | 73.1 ns |
| `u64` | **5.88 ns** | 14.5 ns | 84.5 ns |

### AMD Ryzen 5 2600X @ 4.125 GHz "Zen+"
#### Decode
|   | varint-simd unsafe | varint-simd safe | integer-encoding-rs | prost |
| -- | -- | -- | -- | -- |
| `u8`  | **2.75 ns** | **3.48 ns** | 8.00 ns | 38.2 ns |
| `u16` | **3.34 ns** | **3.95 ns** | 7.54 ns | 35.6 ns |
| `u32` | **4.82 ns** | **5.10 ns** | 7.88 ns | 34.9 ns |
| `u64` | **6.94 ns** | **7.91 ns** | 13.4 ns | 40.0 ns |

#### Encode
|   | varint-simd | integer-encoding-rs | prost |
| -- | -- | -- | -- |
| `u8`  | **3.90 ns** | 7.53 ns | 63.4 ns |
| `u16` | **4.14 ns** | 7.26 ns | 64.3 ns |
| `u32` | **4.89 ns** | 8.50 ns | 64.0 ns |
| `u64` | **6.26 ns** | 14.1 ns | 76.4 ns |

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
