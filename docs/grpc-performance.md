# gRPC Performance (`roto-grpc` vs. Rust `tonic`, C++, Go, Java, Python)

## Baseline Metadata

- **Git Commit SHA**: `7440ae1` (roto-grpc benchmark implementation: `3c17ca46a64f383028316e2a6e76323d9dbd2ef2`, `roto`/`roto-grpc` runtime: `9852399`)
- **Upstream `grpc_bench` SHA**: `48b6b958832c995eee021361bdf3544d7c356fcc` (`LesnyRumcajs/grpc_bench`)
- **Date**: 2026-10-22
- **Host Hardware & OS**:
  - **CPU**: Intel(R) Core(TM) i9-10850K CPU @ 3.60GHz (`x86_64`, 10 cores / 20 hardware threads, 800–5200 MHz, 20 MiB L3 cache, 1 NUMA node, `powersave` governor, Intel Turbo Boost enabled)
  - **Memory**: 125 GiB DDR4
  - **Kernel / OS**: Linux `7.2.11-1-cachyos` (Arch Linux container, unrestricted cgroup CPU/memory)
- **Load Generator & Workload**:
  - **Client**: `ghz v0.120.0` pinned to 12 dedicated hardware threads (`taskset`), `--concurrency=1000`, `--connections=50` (20 concurrent streams per HTTP/2 connection), `--insecure`
  - **Scenario**: `complex_proto` (`helloworld.Greeter/SayHello` unary echo of nested `Hello` message with strings, floats, ints, `oneof`, and repeated `Pet` sub-messages)
  - **Protocol**: 5 s warm-up followed by 20 s measured duration, repeated for **3 independent runs** per `(implementation, server_cpus)` cell. Server pinned to cores `0..(N-1)` (`taskset -c 0-$((N-1))`).
- **Compared Implementations & Toolchains**:
  - **`rust_roto` (`roto-grpc`)**: `#![no_std]` Sans-I/O HTTP/2 + HPACK + gRPC state machine driven over `tokio 1.52.1` (`rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `--release`, `lto = true`, `codegen-units = 1`, `MAX_STREAMS = 32`, `REASM_CAP = 16 KiB`). Zero-copy request view forwarded directly via `BackwardWriter::copy_unset_fields`.
  - **`rust_tonic_mt`**: `tonic 0.14` + `prost 0.14` + `tokio 1.52.1` + `tikv-jemallocator 0.6` (`rustc 1.99.1`, `--release`, `lto = true`, `codegen-units = 1`).
  - **`cpp_grpc_mt`**: gRPC C++ `1.51.1` + `libprotobuf 3.21.12` + `jemalloc 5.4.0` (`x86_64-conda-linux-gnu-g++ 16.2.0`, `-std=c++20 -O3 -flto`).
  - **`java_hotspot_grpc_g1gc`**: `grpc-java 1.58.0` + `protobuf-java 3.21.7` on Eclipse Temurin JDK `23.0.2+7` (`-XX:+UseG1GC -XX:MinRAMPercentage=70 -XX:MaxRAMPercentage=70 -XX:MaxRAM=512m -XX:ActiveProcessorCount=$GRPC_SERVER_CPUS`).
  - **`go_grpc`**: `grpc-go v1.79.3` + `protobuf-go v1.33.0` on Go `1.26.2` (`GOMAXPROCS=$GRPC_SERVER_CPUS`).
  - **`python_grpc` & `python_async_grpc`**: `grpcio 1.84.1` + `protobuf 7.36.2` on CPython `3.13.17`.

## Reproducing the Benchmarks

On a host with Docker installed, run the benchmark harness from the repository root:

```bash
# Default: rust_roto_bench vs. rust_tonic_mt_bench (1 run, 1 server CPU)
make grpc-bench

# Multi-language matrix with 3 repeated runs and server CPU sweep (1, 2, 4 CPUs):
GRPC_BENCH_RUNS=3 GRPC_BENCH_CPUS="1 2 4" GRPC_CLIENT_CPUS=12 \
make grpc-bench BENCHMARKS="rust_roto_bench rust_tonic_mt_bench cpp_grpc_mt_bench go_grpc_bench \
  java_hotspot_grpc_g1gc_bench python_async_grpc_bench python_grpc_bench"
```

Results and `summary.txt` are written to `target/grpc_bench/results/sweep-<timestamp>/` via [`grpc-bench/summarize.py`](../grpc-bench/summarize.py).

> [!NOTE]
> Because this development container does not have a Docker daemon, the baseline numbers below were collected by compiling and running the exact `grpc_bench` server implementations natively on the host kernel using `taskset` core pinning in place of `docker run --cpus` (tracked in `#052`).

---

## 1. Multi-Language Unary gRPC Throughput & Latency (`complex_proto`)

*`req/s` is `OK responses / total duration` averaged across 3×20 s runs. `±95%` is the two-sided 95% Student's-t confidence half-width (`df = 2`). `req/s/core` normalizes throughput by measured server CPU utilization (`req/s / (cpu% / 100)`).*

### 1 Server CPU (`GRPC_SERVER_CPUS=1`)

| Implementation | `req/s` | `±95%` | `±%` | `p50` (ms) | `p99` (ms) | Server CPU % | `req/s/core` | RSS (MiB) | Relative to `roto` |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| **`rust_roto` (`roto-grpc`)** | **222,034** | ±37,255 | 16.8% | 4.11 | **9.20** | 91% | **245,010** | 84 | **1.00x** |
| **`rust_tonic_mt`** | 101,632 | ±21,556 | 21.2% | **3.01** | 108.16 | 87% | 116,848 | **21** | `0.46x` (`2.18x` slower) |
| **`cpp_grpc_mt`** | 66,157 | ±16,435 | 24.8% | 15.21 | 17.86 | 86% | 76,531 | 24 | `0.30x` (`3.36x` slower) |
| **`java_hotspot_grpc_g1gc`** | 57,521 | ±678 | 1.2% | 14.54 | 46.55 | 98% | 58,989 | 194 | `0.26x` (`3.86x` slower) |
| **`go_grpc`** | 55,469 | ±11,382 | 20.5% | 17.50 | 38.18 | 93% | 59,659 | 32 | `0.25x` (`4.00x` slower) |
| **`python_grpc`** | 5,983 | ±69 | 1.1% | 166.54 | 170.91 | 97% | 6,164 | 67 | `0.027x` (`37.1x` slower) |
| **`python_async_grpc`** | 4,724 | ±72 | 1.5% | 210.32 | 214.04 | 98% | 4,833 | 61 | `0.021x` (`47.0x` slower) |

### 2 Server CPUs (`GRPC_SERVER_CPUS=2`)

| Implementation | `req/s` | `±95%` | `±%` | `p50` (ms) | `p99` (ms) | Server CPU % | `req/s/core` | RSS (MiB) | Relative (`req/s/core`) |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| **`rust_roto` (`roto-grpc`)** \* | **262,107** | ±10,011 | 3.8% | **1.97** | **9.03** | 139% | **188,025** | 87 | **1.00x** |
| **`rust_tonic_mt`** | 165,239 | ±2,967 | 1.8% | 4.15 | 25.18 | 184% | 89,869 | **29** | `0.48x` (`2.09x` slower) |
| **`cpp_grpc_mt`** | 137,858 | ±8,481 | 6.2% | 7.20 | 10.64 | 187% | 73,660 | 65 | `0.39x` (`2.55x` slower) |
| **`java_hotspot_grpc_g1gc`** | 116,466 | ±1,951 | 1.7% | 8.24 | 20.65 | 194% | 60,041 | 205 | `0.32x` (`3.13x` slower) |
| **`go_grpc`** | 99,881 | ±3,651 | 3.7% | 9.51 | 18.96 | 193% | 51,627 | 35 | `0.27x` (`3.64x` slower) |
| **`python_grpc`** | 8,001 | ±190 | 2.4% | 124.34 | 133.28 | 141% | 5,692 | 69 | `0.030x` (`33.0x` slower) |
| **`python_async_grpc`** | 7,665 | ±283 | 3.7% | 129.88 | 144.27 | 131% | 5,851 | 62 | `0.031x` (`32.1x` slower) |

### 4 Server CPUs (`GRPC_SERVER_CPUS=4`)

| Implementation | `req/s` | `±95%` | `±%` | `p50` (ms) | `p99` (ms) | Server CPU % | `req/s/core` | RSS (MiB) | Relative (`req/s/core`) |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| **`rust_roto` (`roto-grpc`)** \* | **253,857** | ±12,597 | 5.0% | **2.03** | **8.22** | 161% | **157,349** | 93 | **1.00x** |
| **`rust_tonic_mt`** | 213,036 | ±7,983 | 3.7% | 3.36 | 10.67 | 330% | 64,591 | **35** | `0.41x` (`2.44x` slower) |
| **`cpp_grpc_mt`** | 197,841 | ±6,363 | 3.2% | 4.38 | 10.81 | 351% | 56,336 | 95 | `0.36x` (`2.79x` slower) |
| **`java_hotspot_grpc_g1gc`** | 175,696 | ±10,458 | 6.0% | 5.19 | 13.65 | 361% | 48,735 | 233 | `0.31x` (`3.23x` slower) |
| **`go_grpc`** | 147,003 | ±5,602 | 3.8% | 5.98 | 15.09 | 371% | 39,578 | 35 | `0.25x` (`3.98x` slower) |
| **`python_grpc`** | 9,619 | ±811 | 8.4% | 103.58 | 114.93 | 166% | 5,793 | 68 | `0.037x` (`27.2x` slower) |
| **`python_async_grpc`** | 7,966 | ±393 | 4.9% | 125.92 | 136.06 | 134% | 5,964 | 62 | `0.038x` (`26.4x` slower) |

*\* At 2 and 4 server CPUs, `rust_roto` is bottlenecked by the `ghz` load generator (see §2 below).*

---

## 2. Key Findings & Bottleneck Analysis

1. **Single-Core Efficiency (`1 CPU`)**:
   - When constrained to 1 hardware thread (where the server is the bottleneck and `ghz` has headroom), `roto-grpc` achieves **222,034 req/s** (**245,010 req/s per active core**) with a **9.20 ms `p99` latency**.
   - This is **2.18x faster than `tonic`** (101,632 req/s, 108.16 ms `p99`), **3.36x faster than C++ gRPC** (66,157 req/s), **3.86x faster than Java `grpc-java`** (57,521 req/s), and **4.00x faster than `grpc-go`** (55,469 req/s).
2. **Load Generator Saturation at $\ge 2$ Server CPUs (`#051`, `#053`)**:
   - At 2 and 4 server CPUs, `roto-grpc` plateaus around **254k–262k req/s** while consuming only **139%–161% CPU** (~1.4–1.6 cores out of 2–4 allocated).
   - Direct `/proc/<pid>/stat` profiling confirmed that the **`ghz` client saturates all 12 of its allocated cores (`~1,141%` CPU)** at ~255k req/s; expanding `ghz` to 16 cores (`~1,502%` CPU) increased throughput by only +2% (to 257.8k req/s).
   - Measured by CPU efficiency (`req/s/core`), `roto-grpc` delivers **157k–188k req/s/core** at 2–4 CPUs — still **2.1x–2.4x more efficient per core than `tonic`** (65k–90k req/s/core) and **2.6x–4.0x more efficient than C++, Java, and Go**.
3. **Memory Footprint & Known Issues Discovered**:
   - `roto-grpc` uses ~84–93 MiB RSS across 50 connections because each connection pre-allocates a `GrpcServer<'_, 32, 16384>` (~1.09 MiB) and a 512 KiB `tx_buf` on the heap (`#050`).
   - Under the `string_10kB` scenario at 1,000 concurrency, ~1% of connections abort with `outbound control frame queue overflow` because `H2Connection`'s fixed 512-byte `TX_QUEUE_CAP` fills up with `WINDOW_UPDATE` frames during large multi-frame `recv()` bursts (`#049`).

---

## 3. Architectural Shortcuts, Tradeoffs & Benchmark Caveats

The numbers above reflect real wire-to-wire throughput, but `roto-grpc` performs significantly less work per RPC than general-purpose gRPC stacks (`tonic`, `grpc-c++`, `grpc-go`, `grpc-java`). The speedup comes from deliberate architectural constraints and shortcuts across four layers:

### 1. Shallow Echo Handler (No Deep Deserialization or Re-Encoding)

In [`grpc-bench/rust_roto_bench/src/main.rs`](../grpc-bench/rust_roto_bench/src/main.rs), the `SayHello` echo handler forwards the inner `Hello` message via `b.copy_unset_fields(&hello)`:

```rust
let hello = req
    .request()
    .map_err(|_| GrpcStatus::invalid_argument("corrupt request"))?;
if let Some(hello) = hello {
    resp.write_response(|b| {
        b.copy_unset_fields(&hello);
    });
}
```

Whereas `tonic` (`prost`), C++, Go, and Java deeply deserialize `HelloRequest` (allocating heap strings and a `Vec<Pet>`, decoding every nested `Pet` sub-message, and validating UTF-8 on every `string` field) and then re-serialize that in-memory object tree back to wire bytes:
- **`HelloView::from_bytes` performs only a shallow top-level scan**: It decodes top-level scalars (`d`, `f`, `b`, `n`, `l`, `c2`) and records byte-span `FieldMarker`s for `name`, `c1`, and `repeated Pet pets`. It never parses the nested `Pet` sub-messages and never validates UTF-8 on `name`, `c1`, or `Pet.name` (UTF-8 validation in `roto` is deferred until string accessors are called).
- **`copy_unset_fields` copies raw wire spans**: Backed by [`BackwardWriter::copy_filtered_fields`](../roto-runtime/src/writer.rs), it scans top-level wire tags in `hello.as_bytes()` and `memcpy`s each field's raw tag+payload span into the response buffer without re-encoding scalars or descending into `Pet` sub-messages.

### 2. Protocol & Feature Omissions in `roto-grpc`

`roto-grpc` implements the minimal subset of HTTP/2 and gRPC required for wire framing while omitting per-RPC protocol features:
- **Stateless HPACK (`SETTINGS_HEADER_TABLE_SIZE = 0`) & No Outbound Huffman**: [`H2Settings::new`](../roto-grpc/src/h2.rs) advertises `header_table_size = 0` and [`accept`](../roto-grpc/src/server.rs) decodes headers with [`HpackDecoder::<0>::stateless()`](../roto-grpc/src/hpack.rs), avoiding dynamic table insertions and evictions. [`HpackEncoder::new`](../roto-grpc/src/hpack.rs) defaults to `use_huffman = false`, emitting static-table indices and raw unencoded literals.
- **Custom Metadata is Silently Ignored**: [`GrpcRequestHeaders::on_header`](../roto-grpc/src/hpack.rs) matches 10 hardcoded header names (`:method`, `:scheme`, `:authority`, `:path`, `content-type`, `te`, `grpc-encoding`, `grpc-accept-encoding`, `grpc-timeout`, `user-agent`) and discards all user metadata (`*-bin` base64 and custom ASCII headers) without allocating a `HeaderMap`.
- **No `grpc-timeout` / Deadline Enforcement**: `grpc-timeout` is parsed into `GrpcRequestHeaders` and discarded in [`accept`](../roto-grpc/src/server.rs); no timer or cancellation state is created per RPC.
- **No Payload Compression**: Any frame with the 5-byte LPM compression flag set to `1` is rejected with `GrpcCode::Internal` ([`validate_call`](../roto-grpc/src/server.rs)).
- **No TLS / ALPN**: Operates strictly on cleartext byte streams (`h2c`).
- **No HTTP/2 Priority Scheduling**: `PRIORITY` frames and `HEADERS` priority fields are validated for wire length and ignored ([`H2Connection::process_input_frame`](../roto-grpc/src/h2.rs)).

### 3. Execution, Concurrency & Memory Tradeoffs

General-purpose gRPC stacks spawn an independent async task per HTTP/2 stream, allocate reference-counted buffers (`Bytes`), and synchronize access to the connection write queue across worker threads. `roto-grpc` avoids that overhead by imposing strict structural limits:
- **Synchronous, Single-Task Execution Per Connection**: [`GrpcServer::recv`](../roto-grpc/src/server.rs) dispatches RPC handlers synchronously inline while draining the socket read buffer, reusing a single shared `msg_buf` per connection with zero task spawning, `Waker` registration, or cross-thread locking. Even in [`AsyncGrpcServer::recv`](../roto-grpc/src/server.rs), `.await`ing an async handler suspends the entire connection's `recv()` loop—streams multiplexed on the same HTTP/2 connection do not execute concurrently.
- **Batch-Buffered Streaming RPCs**:
  - *Client-streaming* ([`CallMode::ClientStream`](../roto-grpc/src/server.rs)) accumulates all inbound LPM frames into `PendingCall::req_buf: [u8; REASM_CAP]` and invokes the handler only after `END_STREAM` arrives.
  - *Server-streaming* encodes all outbound messages into `msg_buf` in a single call. Because [`BackwardWriter`](../roto-runtime/src/writer.rs) writes back-to-front, [`WriterSendStream::send`](../roto-grpc/src/streaming.rs) calls [`BackwardWriter::rotate_written_to_end`](../roto-runtime/src/writer.rs) (`slice::rotate_left`) after each message to preserve forward wire order, making multi-message streaming $O(N^2)$ in total bytes written.
- **Large Fixed Per-Connection Memory Footprint (~1.6 MiB per connection)**: Because `H2Connection` and `GrpcServer` embed `[StreamSlot<REASM_CAP>; MAX_STREAMS]` and `[PendingCall<REASM_CAP>; MAX_STREAMS]` inline for `#![no_std]` zero-allocation operation, `GrpcServer<'_, 32, 16384>` is **~1.09 MiB** by value (plus a 512 KiB `tx_buf`). Across 50 connections, `roto-grpc` consumes **84–93 MiB RSS** vs. `tonic`'s **21–35 MiB**, and requires a **16 MiB worker thread stack** in [`main.rs`](../grpc-bench/rust_roto_bench/src/main.rs) to construct `GrpcServer::new` before boxing (`#050`).
- **Linear Scans Instead of Hash Tables**: Stream slot lookups ([`H2Connection::find_stream_idx`](../roto-grpc/src/h2.rs)), pending call lookups ([`take`](../roto-grpc/src/server.rs)), and route matching ([`dispatch`](../roto-grpc/src/server.rs)) use $O(N)$ linear array scans, which are cache-friendly at `MAX_STREAMS = 32` and 1 route but do not scale to thousands of concurrent streams or routes.
- **Fixed 512-Byte Control Frame Queue (`TX_QUEUE_CAP = 512`)**: [`H2Connection`](../roto-grpc/src/h2.rs) buffers outbound control frames (`SETTINGS`, `PING`, `RST_STREAM`, `GOAWAY`, `WINDOW_UPDATE`) in a 512-byte inline array (~39 `WINDOW_UPDATE` frames). Large multi-frame `recv()` bursts (`string_10kB` at 1,000 concurrency) can overflow this queue before `poll_transmit()` drains it, aborting ~1% of connections (`#049`).

### 4. Benchmark Environment Caveats

- **Native `taskset` Pinning vs. Docker CFS Quotas (`#052`)**: Because this environment lacks a Docker daemon, the baseline was collected by running the server binaries natively under `taskset` hardware thread pinning rather than `docker run --cpus` (Linux CFS bandwidth throttling and Docker bridge networking).
- **Load Generator Ceiling at $\ge 2$ CPUs (`#051`, `#053`)**: At 2 and 4 server CPUs, `roto-grpc` raw throughput plateaus at ~254k–262k req/s (`139%–161%` server CPU) because `ghz` saturates all 12 client cores. The 2-CPU and 4-CPU `Relative` columns therefore compare CPU-normalized efficiency (`req/s/core`) rather than raw wall-clock throughput.
