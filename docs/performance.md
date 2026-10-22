# Protobuf Serialization & Deserialization Performance (`roto` vs. `upb` vs. `prost`)

## Baseline Metadata

- **Git Commit SHA**: `e8c471586ba6da75ba126edad3826a1c1cefa75c` (`e8c4715`)
- **Date**: 2026-10-22
- **CPU**: Intel(R) Core(TM) i9-10850K CPU @ 3.60GHz (`x86_64`, 20 hardware threads)
- **Toolchain**:
  - `rustc 1.99.1 (58cf8f9c3 2026-10-13)` (`--release`)
  - `gcc (GCC) 16.2.1 20260810` (`-O3`)
  - `libprotoc 36.1`
- **Compared Implementations**:
  - **`roto`**: `#![no_std]`, zero-allocation lazy `*View<'a>` and single-pass backward `*Builder<'b>` (`roto` / `roto-runtime`).
  - **Native C `upb` (`36.1.0`)**: Official Google C `upb` runtime (`libupb.a` + `protoc-gen-upb` + `protoc-gen-upb_minitable`), evaluated under two arena allocation modes:
    - `upb (stack arena)`: `upb_Arena_Init` backed by a pre-allocated stack/reused buffer.
    - `upb (heap arena)`: `upb_Arena_New` using `upb`'s default heap block allocator.
  - **`prost` (`0.14.4`)**: Standard heap-allocated Rust Protobuf implementation (`prost` + `prost-build`).

## Reproducing the Benchmarks

Run the benchmark suite from the repository root:

```bash
make bench
# or directly:
cargo run --release --manifest-path proto-benchmarks/Cargo.toml
```

Before timing any workload, [`proto-benchmarks/src/main.rs`](../proto-benchmarks/src/main.rs) verifies exact field-value and wire-roundtrip equivalence across `roto`, `upb`, and `prost`.

---

## 1. End-to-End Dataset Benchmarks (`google_message1` & `perftest_data`)

*Multipliers in parentheses are relative to `roto` (`>1.00x` means slower than `roto`, `<1.00x` means faster than `roto`).*

| Dataset / Workload | Operation | `roto` | `upb` (stack arena) | `upb` (heap arena) | `prost` |
| :--- | :--- | ---: | ---: | ---: | ---: |
| **`google_message1_proto2`** (228 B) | **Partial Read (5 flds)** | **131.2 ns** | 307.3 ns (`2.34x`) | 330.5 ns (`2.52x`) | 225.9 ns (`1.72x`) |
| **`google_message1_proto2`** (228 B) | **Full Deserialize (61 flds)** | **216.3 ns** | 296.0 ns (`1.37x`) | 336.3 ns (`1.56x`) | 228.1 ns (`1.05x`) |
| **`google_message1_proto2`** (228 B) | **Serialize (Build+Enc)** | **63.9 ns** | 401.7 ns (`6.28x`) | 431.0 ns (`6.74x`) | 122.1 ns (`1.91x`) |
| **`google_message1_proto2`** (228 B) | **Serialize (Pre-built)** | **63.9 ns** | 239.3 ns (`3.74x`) | 248.6 ns (`3.89x`) | 122.1 ns (`1.91x`) |
| **`google_message1_proto3`** (222 B) | **Partial Read (5 flds)** | **121.9 ns** | 302.3 ns (`2.48x`) | 329.2 ns (`2.70x`) | 201.4 ns (`1.65x`) |
| **`google_message1_proto3`** (222 B) | **Full Deserialize (61 flds)** | 215.2 ns | 310.4 ns (`1.44x`) | 338.5 ns (`1.57x`) | **206.7 ns (`0.96x`)** |
| **`google_message1_proto3`** (222 B) | **Serialize (Build+Enc)** | **62.0 ns** | 271.6 ns (`4.38x`) | 304.6 ns (`4.91x`) | 107.7 ns (`1.74x`) |
| **`google_message1_proto3`** (222 B) | **Serialize (Pre-built)** | **62.0 ns** | 210.3 ns (`3.39x`) | 220.6 ns (`3.56x`) | 107.7 ns (`1.74x`) |
| **`perftest_data`** (104 KB) | **Partial Read (`test1` + `s1`)** | **397.5 ns** | 2801.8 ns (`7.05x`) | 3143.1 ns (`7.91x`) | 6236.4 ns (`15.69x`) |
| **`perftest_data`** (104 KB) | **Full Deserialize** | **1716.8 ns** | 2912.4 ns (`1.70x`) | 3265.9 ns (`1.90x`) | 6345.0 ns (`3.70x`) |
| **`perftest_data`** (104 KB) | **Full Serialize** | **2115.8 ns** | 2684.3 ns (`1.27x`) | 2705.9 ns (`1.28x`) | 2792.9 ns (`1.32x`) |

---

## 2. `perftest_data` Sub-Category Micro-Benchmarks

| Perftest Sub-Category | Operation | `roto` | `upb` (stack arena) | `upb` (heap arena) | `prost` |
| :--- | :--- | ---: | ---: | ---: | ---: |
| **`perftest::test1` (varints)** | **Deserialize** | 28.6 ns | 82.2 ns (`2.87x`) | 91.8 ns (`3.21x`) | **19.0 ns (`0.66x`)** |
| **`perftest::test1` (varints)** | **Serialize** | **11.7 ns** | 99.3 ns (`8.50x`) | 109.9 ns (`9.41x`) | 14.0 ns (`1.20x`) |
| **`perftest::packed_int32`** | **Deserialize** | **87.0 ns** | 168.3 ns (`1.93x`) | 178.0 ns (`2.05x`) | 175.3 ns (`2.01x`) |
| **`perftest::packed_int32`** | **Serialize** | **45.1 ns** | 179.0 ns (`3.97x`) | 191.7 ns (`4.25x`) | 62.7 ns (`1.39x`) |
| **`perftest::test_strings`** | **Deserialize** | **90.1 ns** | 119.0 ns (`1.32x`) | 146.7 ns (`1.63x`) | 155.1 ns (`1.72x`) |
| **`perftest::test_strings`** | **Serialize** | **30.9 ns** | 138.6 ns (`4.49x`) | 169.0 ns (`5.48x`) | 42.2 ns (`1.37x`) |
| **`perftest::optional_msgs`** | **Deserialize** | **279.1 ns** | 348.7 ns (`1.25x`) | 375.3 ns (`1.34x`) | 310.5 ns (`1.11x`) |
| **`perftest::optional_msgs`** | **Serialize** | **97.6 ns** | 439.0 ns (`4.50x`) | 479.4 ns (`4.91x`) | 267.8 ns (`2.75x`) |
| **`perftest::repeated_msgs`** | **Deserialize** | 615.9 ns | **386.9 ns (`0.63x`)** | 444.4 ns (`0.72x`) | 400.2 ns (`0.65x`) |
| **`perftest::repeated_msgs`** | **Serialize** | **131.1 ns** | 579.7 ns (`4.42x`) | 676.2 ns (`5.16x`) | 212.4 ns (`1.62x`) |
| **`perftest::small_bytes(1K)`** | **Deserialize** | **11.8 ns** | 44.4 ns (`3.75x`) | 93.7 ns (`7.93x`) | 49.8 ns (`4.21x`) |
| **`perftest::small_bytes(1K)`** | **Serialize** | **15.3 ns** | 50.5 ns (`3.30x`) | 113.1 ns (`7.40x`) | 19.5 ns (`1.28x`) |
| **`perftest::large_bytes(100K)`** | **Deserialize** | **12.6 ns** | 2095.3 ns (`166.36x`) | 2520.9 ns (`200.14x`) | 4600.7 ns (`365.27x`) |
| **`perftest::large_bytes(100K)`** | **Serialize** | 1886.7 ns | 2035.3 ns (`1.08x`) | **1752.8 ns (`0.93x`)** | 1939.8 ns (`1.03x`) |

---

## 3. Impact of `#044` and `#045` Optimizations

Comparing `roto` before and after:
- **Issue `#044`**: Eagerly decoding scalar fields (`varint`, `fixed32`, `fixed64`, `bool`, `enum`) during `from_bytes` and storing native values + compact bitmasks in `*View<'a>` structs instead of 8-byte `FieldMarker` spans.
- **Issue `#045`**: Bounding repeated field iterators by recording the `[first_raw_start..last_raw_end]` byte span in `*View<'a>` structs during `from_bytes`.

| Workload | Operation | `roto` (Pre-`#044`/`#045`) | `roto` (`e8c4715`) | Speedup |
| :--- | :--- | ---: | ---: | ---: |
| **`google_message1_proto2`** | **Full Deserialize** | 351.5 ns | **216.3 ns** | **1.63x faster (-38%)** |
| **`google_message1_proto3`** | **Full Deserialize** | 354.3 ns | **215.2 ns** | **1.65x faster (-39%)** |
| **`google_message1_proto3`** | **Partial Read** | 141.5 ns | **121.9 ns** | **1.16x faster (-14%)** |
| **`perftest_data` (104 KB)** | **Partial Read** | 601.7 ns | **397.5 ns** | **1.51x faster (-34%)** |
| **`perftest_data` (104 KB)** | **Full Deserialize** | 2862.8 ns | **1716.8 ns** | **1.67x faster (-40%)** |
| **`perftest::packed_int32`** | **Deserialize** | 109.7 ns | **87.0 ns** | **1.26x faster (-21%)** |
| **`perftest::repeated_msgs`** | **Deserialize** | 713.3 ns | **615.9 ns** | **1.16x faster (-14%)** |

---

## 4. Architectural Summary

1. **Single-Pass Backward Serialization (`BackwardWriter`)**:
   - `roto` is **1.7x–2.7x faster than `prost`** and **3.4x–6.7x faster than `upb`** across `google_message1` (`proto2` & `proto3`) and nested message trees (`optional_msgs`, `repeated_msgs`).
   - By writing fields in reverse from the end of a slice, `roto` computes and prefixes sub-message and packed-field lengths in a single pass, avoiding `prost`'s two-pass `encoded_len()` + `encode()` tree walk and `upb`'s mini-table reflection overhead.
2. **Selective / Partial Deserialization**:
   - On `google_message1`, reading 5 fields takes **121.9–131.2 ns** in `roto` (**1.65x–1.72x faster than `prost`** and **2.34x–2.70x faster than `upb`**).
   - On `perftest_data` (104 KB), selective field access takes **397.5 ns** in `roto` (**7.05x faster than `upb`** and **15.69x faster than `prost`**) because unaccessed sub-messages and byte arrays are skipped without allocation or recursive parsing.
3. **Full Deserialization**:
   - With eager scalar decoding (`#044`) and bounded repeated spans (`#045`), `roto` outperforms `upb` by **1.37x–1.44x** and matches or beats `prost` on `google_message1` (**216.3 ns vs. 228.1 ns** on `proto2`, **215.2 ns vs. 206.7 ns** on `proto3`) while performing zero heap allocations.
   - On the 104 KB `perftest_data` workload, `roto` (**1,716.8 ns**) is **1.70x faster than `upb`** (2,912.4 ns) and **3.70x faster than `prost`** (6,345.0 ns).
