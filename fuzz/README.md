# Fuzzing

Install `cargo-fuzz` and run the ELF64 parser and serializer target:

```sh
cargo install cargo-fuzz --locked
cargo fuzz run elf64-roundtrip
```

Use a bounded run for routine local verification:

```sh
cargo fuzz run elf64-roundtrip -- -max_total_time=60
```

Crashes and minimized inputs are written under `fuzz/artifacts/` and must not be committed unless they are intentionally converted into regression fixtures.
