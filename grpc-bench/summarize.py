#!/usr/bin/env python3
"""Summarize repeated grpc_bench runs per implementation and server CPU count.

Usage: summarize.py <sweep_dir>
Layout: <sweep_dir>/cpus<N>/run<i>/<name>.report (ghz summary) and optional <name>.stats
(`docker stats` CPU%/memory samples), both as written by grpc_bench's bench.sh.

req/s, cpu and mem follow grpc_bench's analyzer: OK responses / total time; mean of stats
samples excluding the last one and any below 1% CPU. ±95% is the confidence half-width of the
mean req/s across runs (Student's t; needs >= 2 runs). req/s/core = req/s / (cpu% / 100): when
the load generator saturates first, req/s stops scaling but req/s/core still compares servers.
"""

import math
import re
import statistics
import sys
from collections import defaultdict
from pathlib import Path

TO_MS = {"ns": 1e-6, "µs": 1e-3, "us": 1e-3, "ms": 1.0, "s": 1e3}
TO_MIB = {"KiB": 1 / 1024, "MiB": 1.0, "GiB": 1024.0}
T95 = [12.71, 4.30, 3.18, 2.78, 2.57, 2.45, 2.36, 2.31, 2.26, 2.23]  # two-sided, df 1..10


def t95(df):
    return T95[df - 1] if df <= len(T95) else 1.96 + 2.4 / df


def duration_ms(text, pattern):
    m = re.search(pattern + r"\s*([\d.]+)\s*(ns|µs|us|ms|s)\b", text)
    return float(m.group(1)) * TO_MS[m.group(2)] if m else math.nan


def stats(path):
    samples = [
        (float(cpu), float(mem) * TO_MIB.get(unit, math.nan))
        for cpu, mem, unit in re.findall(r"([\d.]+)%\s+([\d.]+)(\w+)", path.read_text())[:-1]
        if float(cpu) >= 1
    ]
    if not samples:
        return math.nan, math.nan
    return statistics.mean(s[0] for s in samples), statistics.mean(s[1] for s in samples)


def parse(report):
    text = report.read_text()
    count = re.search(r"Count:\s*(\d+)", text)
    ok = re.search(r"\[OK\]\s*(\d+)", text)
    ok = int(ok.group(1)) if ok else 0
    stats_file = report.with_suffix(".stats")
    cpu, mem = stats(stats_file) if stats_file.exists() else (math.nan, math.nan)
    return {
        "rps": ok / (duration_ms(text, r"Total:") / 1e3),
        "p50": duration_ms(text, r"50 % in"),
        "p99": duration_ms(text, r"99 % in"),
        "errors": (int(count.group(1)) if count else 0) - ok,
        "cpu": cpu,
        "mem": mem,
    }


def main(sweep):
    runs = defaultdict(list)
    for report in sorted(Path(sweep).glob("cpus*/run*/*.report")):
        cpus = int(report.parent.parent.name.removeprefix("cpus"))
        runs[(cpus, report.stem.removesuffix("_bench"))].append(parse(report))
    if not runs:
        sys.exit(f"no reports under {sweep}/cpus*/run*/")

    row = "{:>4}  {:<24} {:>4} {:>8} {:>7} {:>5} {:>7} {:>7} {:>6} {:>6} {:>10} {:>7}"
    print(row.format("cpus", "name", "runs", "req/s", "±95%", "±%", "p50 ms", "p99 ms",
                     "non-OK", "cpu %", "req/s/core", "mem MiB"))
    for cpus in sorted({c for c, _ in runs}):
        group = [(n, r) for (c, n), r in runs.items() if c == cpus]
        for name, rs in sorted(group, key=lambda g: -statistics.mean(r["rps"] for r in g[1])):
            rps = [r["rps"] for r in rs]
            mean = statistics.mean(rps)
            margin = t95(len(rps) - 1) * statistics.stdev(rps) / math.sqrt(len(rps)) if len(rps) > 1 else math.nan
            cpu = statistics.mean(r["cpu"] for r in rs)
            print(row.format(
                cpus, name, len(rs), f"{mean:.0f}", f"{margin:.0f}", f"{100 * margin / mean:.1f}",
                f"{statistics.mean(r['p50'] for r in rs):.2f}",
                f"{statistics.mean(r['p99'] for r in rs):.2f}",
                sum(r["errors"] for r in rs),
                f"{cpu:.0f}", f"{100 * mean / cpu:.0f}",
                f"{statistics.mean(r['mem'] for r in rs):.0f}",
            ))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
