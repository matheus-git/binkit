# Binkit

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)

Binkit is a Rust command-line toolbox for inspecting, disassembling, and modifying ELF64
binaries. It can display ELF metadata, disassemble x86-64 code, calculate an injection plan,
append a payload, and update the executable entry point.

> Use Binkit only on binaries you own or are authorized to modify. You are responsible for complying with applicable laws, licenses, and policies.

## Features

- Display ELF64, program-header, and section-header information for the supported format subset.
- Disassemble ELF sections or raw x86-64 machine code using Intel syntax.
- Calculate aligned payload addresses and signed `rel32` return offsets.
- Append a payload by repurposing a compatible section and program header.
- Update the ELF entry point.
- Protect existing output files unless `--force` is explicitly provided.
- Write files atomically while preserving the source permissions.
- Reject malformed or unsupported ELF structures with descriptive errors.

See [ELF compatibility](docs/elf-compatibility.md) for the exact supported formats and current
limitations.

## Performance

Binkit's iterative Capstone decoder outperformed GNU `objdump` on all three `.text` fixtures in
the reference benchmark:

- 26 KiB ELF: **2.00 ms** versus `objdump` at 2.42 ms (**1.21x faster**).
- 139 KiB ELF: **8.57 ms** versus `objdump` at 13.37 ms (**1.56x faster**).
- 7.65 MiB ELF: **261.48 ms** versus `objdump` at 464.37 ms (**1.78x faster**).

Section listing remains faster in GNU `readelf`: memory-mapped Binkit took 0.83–1.13 ms,
compared with 0.46–0.51 ms for `readelf`. The large disassembly's peak memory use fell from
about 1.9 GiB in the original implementation to about 10 MiB with iterative decoding and `mmap`.

These are warm-cache medians from 15 measured runs after 3 warm-ups on Linux x86-64 with GNU
binutils 2.42. Lower is better. See the [full methodology and reproducible benchmark](docs/benchmarks.md).

## Installation

### Cargo

```sh
cargo install --locked binkit
```

### Build from source

```sh
git clone https://github.com/matheus-git/binkit.git
cd binkit
cargo build --release --locked
./target/release/binkit --help
```

Prebuilt binaries may also be available on the
[GitHub Releases](https://github.com/matheus-git/binkit/releases) page.

## Commands

Run `binkit --help` or `binkit <COMMAND> --help` for the complete CLI reference.

### Inspect ELF metadata

The `info` command displays the complete ELF header and its program or section tables. Options
can be combined:

```sh
binkit info ./program --header
binkit info ./program --programs
binkit info ./program --sections
binkit info ./program --header --programs --sections
```

The header output includes identification fields, byte order, ABI, file type, architecture,
entry point, flags, table offsets, entry sizes, counts, and the section-name table index.

### Disassemble code

Disassemble `.text` or another section from a little-endian x86-64 ELF file:

```sh
binkit disasm ./program
binkit disasm ./program --section .init
binkit disasm ./program --address 0x401000 --count 20
binkit disasm ./program --section .text --offset 64 --bytes 256
```

Disassemble a file containing raw x86-64 machine code:

```sh
binkit disasm ./payload.bin --bin
binkit disasm ./payload.bin --bin --offset 16 --count 10
```

`--offset` starts at a byte offset relative to the selected section or raw input, while
`--address` starts at a hexadecimal virtual address inside an ELF section. The two options are
mutually exclusive. `--bytes` limits the input window and `--count` limits decoded instructions;
when combined, disassembly stops at whichever limit is reached first.

Disassembly is streamed in aligned address, raw-byte, and assembly columns in a terminal. Pipes
and redirected output use compact separators to avoid emitting unnecessary padding:

```text
Address            │ Bytes                                        │ Assembly
───────────────────┼──────────────────────────────────────────────┼─────────────────────────────────
0x0000000000401000 │ 55                                           │ push rbp
0x0000000000401001 │ 48 89 E5                                     │ mov rbp, rsp
```

### Check an injection plan

Calculate the aligned injection address and the signed `rel32` displacement back to the current
entry point:

```sh
binkit check-inject ./program
binkit check-inject ./program --return-address 0x401000
```

This command only reports values; it does not modify or create a file.

### Inject a payload

Append a payload and write the modified ELF to a new file:

```sh
binkit inject ./program \
  --inject ./payload.bin \
  --output ./program.injected
```

By default, Binkit repurposes `.note.gnu.property`, renames it to `.injected`, and assigns an
aligned virtual address. Select another compatible section or address when needed:

```sh
binkit inject ./program \
  --inject ./payload.bin \
  --section .custom-note \
  --address 0x405000 \
  --return-address 0x401000 \
  --output ./program.injected
```

Injection requires the selected section to have a matching program header and enough space in
the section-name string table for `.injected`. The command fails without creating the output if
these requirements are not met.

Existing destinations are protected. Use `--force` only when replacement is intentional:

```sh
binkit inject ./program -i ./payload.bin -o ./program.injected --force
```

The payload is appended as provided. Binkit reports the injection address and return displacement
but does not generate a jump, trampoline, or architecture-specific payload instructions.

### Update the entry point

Write a new ELF entry point to a separate output file:

```sh
binkit update ./program --entry 0x405000 --output ./program.updated
```

Omitting `--output` updates the input path atomically. Replacing a different existing destination
requires `--force`.

## Library usage

The ELF64 parser and core calculations are available as a Rust library:

```rust
use binkit::elf64::{Elf64Binary, calculate_rel32};

let bytes = std::fs::read("program")?;
let binary = Elf64Binary::new(&bytes)?;

println!("Entry point: 0x{:X}", binary.entry());
println!("rel32: {}", calculate_rel32(0x405000, binary.entry())?);

# Ok::<(), anyhow::Error>(())
```

## Development

The project toolchain is pinned in `rust-toolchain.toml` and is installed automatically by
`rustup`. Run the same quality checks used by CI:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release --locked
```

The integration tests also use GNU binutils, GCC, Clang, and lld. They compile several ELF64
variants, execute an injected x86-64 payload, and verify that another payload can restore the
initial process state and return to the original entry point. On Debian or Ubuntu:

```sh
sudo apt-get update
sudo apt-get install binutils build-essential clang lld
```

The fuzz target requires nightly Rust and `cargo-fuzz`:

```sh
cargo install cargo-fuzz --locked
cargo +nightly fuzz run elf64-roundtrip
```

Generated fuzz corpora, artifacts, and build output are ignored by Git.

## Contributing

Contributions should include tests for behavior changes and pass the formatting, lint, test, and
release-build commands above. Useful areas include additional ELF validation, executable
injection fixtures, architecture support, structured output, and other binary formats.

## License

Licensed under the [MIT License](LICENSE).
