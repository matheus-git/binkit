#!/usr/bin/env python3
"""Reproducible CLI benchmark for Binkit and comparable GNU binutils tools."""

from __future__ import annotations

import argparse
import json
import platform
import random
import shutil
import statistics
import subprocess
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def percentile(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int(len(ordered) * fraction))]


def version(command: str) -> str:
    result = subprocess.run(
        [command, "--version"], text=True, capture_output=True, check=False
    )
    return (result.stdout or result.stderr).splitlines()[0]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runs", type=int, default=15, help="measured runs per command")
    parser.add_argument("--warmup", type=int, default=3, help="warm-up runs per command")
    parser.add_argument(
        "--fixture",
        action="append",
        dest="fixtures",
        help="ELF file to test (repeatable; defaults to true, ls, and python3)",
    )
    parser.add_argument("--json", type=Path, help="also write the raw result as JSON")
    args = parser.parse_args()
    if args.runs < 1 or args.warmup < 0:
        parser.error("--runs must be positive and --warmup cannot be negative")

    binkit = ROOT / "target/release/binkit"
    if not binkit.exists():
        subprocess.run(["cargo", "build", "--release", "--locked"], cwd=ROOT, check=True)

    readelf = shutil.which("readelf")
    objdump = shutil.which("objdump")
    if not readelf or not objdump:
        raise SystemExit("GNU readelf and objdump must be installed")

    fixture_names = args.fixtures or ["/bin/true", "/bin/ls", shutil.which("python3")]
    fixtures = [Path(name).resolve() for name in fixture_names if name]
    cases = {
        "sections": (
            "readelf",
            {
                "binkit": lambda path: [str(binkit), "info", str(path), "--sections"],
                "readelf": lambda path: [readelf, "-W", "-S", str(path)],
            },
        ),
        "disassembly": (
            "objdump",
            {
                "binkit": lambda path: [
                    str(binkit),
                    "disasm",
                    str(path),
                    "--section",
                    ".text",
                ],
                "objdump": lambda path: [
                    objdump,
                    "-d",
                    "-j",
                    ".text",
                    "-M",
                    "intel",
                    str(path),
                ],
            },
        ),
    }

    rows: list[dict[str, object]] = []
    rng = random.Random(0)
    for fixture in fixtures:
        if not fixture.is_file():
            raise SystemExit(f"fixture does not exist: {fixture}")
        for operation, (baseline, competitors) in cases.items():
            commands = {name: make(fixture) for name, make in competitors.items()}
            for command in commands.values():
                for _ in range(args.warmup):
                    subprocess.run(command, stdout=subprocess.DEVNULL, check=True)

            timings = {name: [] for name in commands}
            order = list(commands) * args.runs
            rng.shuffle(order)
            for name in order:
                started = time.perf_counter_ns()
                subprocess.run(commands[name], stdout=subprocess.DEVNULL, check=True)
                timings[name].append((time.perf_counter_ns() - started) / 1_000_000)

            medians = {name: statistics.median(values) for name, values in timings.items()}
            for name, values in timings.items():
                rows.append(
                    {
                        "fixture": str(fixture),
                        "bytes": fixture.stat().st_size,
                        "operation": operation,
                        "tool": name,
                        "baseline": baseline,
                        "median_ms": medians[name],
                        "p95_ms": percentile(values, 0.95),
                        "relative_to_baseline": medians[name] / medians[baseline],
                        "samples_ms": values,
                    }
                )

    metadata = {
        "platform": platform.platform(),
        "cpu": platform.processor(),
        "python": platform.python_version(),
        "binkit_bytes": binkit.stat().st_size,
        "readelf": version(readelf),
        "readelf_bytes": Path(readelf).resolve().stat().st_size,
        "objdump": version(objdump),
        "objdump_bytes": Path(objdump).resolve().stat().st_size,
        "runs": args.runs,
        "warmup": args.warmup,
    }
    print("| Input | Size | Operation | Tool | Median | p95 | vs competitor |")
    print("|---|---:|---|---|---:|---:|---:|")
    for row in rows:
        print(
            f"| {Path(str(row['fixture'])).name} | {row['bytes'] / 1024:.1f} KiB "
            f"| {row['operation']} | {row['tool']} | {row['median_ms']:.2f} ms "
            f"| {row['p95_ms']:.2f} ms | {row['relative_to_baseline']:.2f}x |"
        )

    if args.json:
        args.json.parent.mkdir(parents=True, exist_ok=True)
        args.json.write_text(json.dumps({"metadata": metadata, "results": rows}, indent=2) + "\n")


if __name__ == "__main__":
    main()
