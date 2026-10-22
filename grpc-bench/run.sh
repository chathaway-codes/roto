#!/usr/bin/env bash
# Benchmark roto-grpc with LesnyRumcajs/grpc_bench (requires Docker).
#
# Usage: grpc-bench/run.sh [benchmark...]   (default: rust_roto_bench rust_tonic_mt_bench)
#
# GRPC_BENCH_RUNS=N         repeat every benchmark N times (default 1) for confidence intervals
# GRPC_BENCH_CPUS="1 2 4"   sweep server CPU counts (default: $GRPC_SERVER_CPUS or 1)
#
# grpc_bench's own GRPC_* variables (GRPC_REQUEST_SCENARIO, GRPC_CLIENT_CPUS,
# GRPC_BENCHMARK_DURATION, ...) pass through unchanged. Results and summary.txt land in
# $GRPC_BENCH_DIR/results/sweep-<timestamp>/cpus<N>/run<i>/.
set -euo pipefail

ROOT=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
DIR=${GRPC_BENCH_DIR:-$ROOT/target/grpc_bench}
REV=${GRPC_BENCH_REV:-48b6b958832c995eee021361bdf3544d7c356fcc}
BENCHMARKS=${*:-rust_roto_bench rust_tonic_mt_bench}
RUNS=${GRPC_BENCH_RUNS:-1}
CPUS=${GRPC_BENCH_CPUS:-${GRPC_SERVER_CPUS:-1}}

[ -d "$DIR/.git" ] || git clone -q https://github.com/LesnyRumcajs/grpc_bench "$DIR"
git -C "$DIR" checkout -q "$REV"

# Stage the Dockerfile plus a snapshot of this working tree (tracked + untracked, minus ignored).
STAGE=$DIR/rust_roto_bench
rm -rf "$STAGE"
mkdir -p "$STAGE/roto"
cp "$ROOT/grpc-bench/rust_roto_bench/Dockerfile" "$STAGE/"
git -C "$ROOT" ls-files -z -co --exclude-standard |
    tar -C "$ROOT" --null -T - --ignore-failed-read -cf - |
    tar -C "$STAGE/roto" -xf -

cd "$DIR"
./build.sh $BENCHMARKS

SWEEP=results/sweep-$(date '+%y%m%dT%H%M%S')
for cpus in $CPUS; do
    mkdir -p "$SWEEP/cpus$cpus"
    for run in $(seq "$RUNS"); do
        GRPC_SERVER_CPUS=$cpus ./bench.sh $BENCHMARKS
        # bench.sh writes each run to a fresh results/<timestamp>/ directory.
        mv "$(ls -td results/[0-9]*/ | head -1)" "$SWEEP/cpus$cpus/run$run"
    done
done
"$ROOT/grpc-bench/summarize.py" "$SWEEP" | tee "$SWEEP/summary.txt"
