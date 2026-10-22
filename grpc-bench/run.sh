#!/usr/bin/env bash
# Benchmark roto-grpc with LesnyRumcajs/grpc_bench (requires Docker).
#
# Usage: grpc-bench/run.sh [benchmark...]   (default: rust_roto_bench rust_tonic_mt_bench)
# grpc_bench's GRPC_* environment variables (GRPC_REQUEST_SCENARIO, GRPC_SERVER_CPUS,
# GRPC_BENCHMARK_DURATION, ...) pass through unchanged.
set -euo pipefail

ROOT=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
DIR=${GRPC_BENCH_DIR:-$ROOT/target/grpc_bench}
REV=${GRPC_BENCH_REV:-48b6b958832c995eee021361bdf3544d7c356fcc}
BENCHMARKS=${*:-rust_roto_bench rust_tonic_mt_bench}

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
./bench.sh $BENCHMARKS
