# Benchmarks

This benchmark compares Binkit's overlapping read-only operations with the established GNU
binutils tools available on Linux:

- `binkit info --sections` versus `readelf -W -S`
- `binkit disasm --section .text` versus `objdump -d -j .text -M intel`

These pairs answer the same practical request, but their text formats are not byte-for-byte
identical. Injection and entry-point modification are excluded because GNU binutils has no
single directly equivalent command; feature breadth should be evaluated separately from speed.

## Reference result

Measured on Linux 7.0 x86-64 with GNU binutils 2.42, using 3 warm-ups and 15 measured runs
per command (September 12, 2026):

| Input | Size | Operation | Binkit median | Competitor median | Binkit/competitor |
|---|---:|---|---:|---:|---:|
| `true` | 26.3 KiB | sections (`readelf`) | 1.59 ms | 0.76 ms | 2.09x |
| `true` | 26.3 KiB | disassembly (`objdump`) | 2.36 ms | 2.57 ms | 0.92x |
| `ls` | 139.0 KiB | sections (`readelf`) | 1.00 ms | 0.52 ms | 1.93x |
| `ls` | 139.0 KiB | disassembly (`objdump`) | 11.39 ms | 13.31 ms | 0.86x |
| `python3.12` | 7.65 MiB | sections (`readelf`) | 0.91 ms | 0.51 ms | 1.78x |
| `python3.12` | 7.65 MiB | disassembly (`objdump`) | 369.96 ms | 463.85 ms | 0.80x |

Lower is better. On this run GNU remained faster at listing sections, while Binkit's iterative
disassembly was faster on all three fixtures.
Relative to Binkit's original table-building implementation, the large-fixture median fell from
1,342.59 ms to 369.96 ms (3.6x faster).
Peak RSS in a separate `/usr/bin/time` run fell from about 1.9 GiB to about 10 MiB. The decoder uses
Capstone's `cs_disasm_iter` API and reuses one instruction allocation for the whole section;
the `info` and `disasm` commands map their input instead of copying the entire file.

The release executables on this host were 9.11 MiB for Binkit, 771 KiB for `readelf`, and
382 KiB for `objdump`. This is an installed-file comparison, not a like-for-like accounting:
Binkit combines multiple operations in one Rust executable, while the GNU programs dynamically
share libraries and belong to a larger tool suite.

## Run it

Build and run the benchmark on an otherwise idle machine:

```sh
python3 benches/benchmark.py --runs 15 --warmup 3 --json target/benchmark.json
```

Pass `--fixture PATH` repeatedly to use a controlled corpus. By default the script tests
`/bin/true`, `/bin/ls`, and the resolved `python3` executable. Standard output is discarded so
terminal rendering is not measured. Each command is warmed up, measured with a monotonic clock,
and run in deterministic shuffled order to reduce ordering bias. The report uses the median as
the primary result and p95 as a simple variability indicator.

Results are specific to the machine, OS cache state, compiler, fixture binaries, and tool
versions. Keep the generated JSON with any published result; it contains raw samples and tool
metadata. For stronger claims, reboot or drop caches between cold-start trials, pin execution to
one CPU, disable frequency scaling, and repeat the experiment on multiple machines.

## What this does not measure

The benchmark does not score output completeness, correctness, malformed-input handling, memory
use, or modification features. It is a warm-cache CLI latency comparison, not a claim that the
tools expose identical information or provide interchangeable security guarantees.
