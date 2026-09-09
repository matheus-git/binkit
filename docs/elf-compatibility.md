# ELF compatibility

Binkit intentionally supports a focused subset of ELF instead of silently guessing how
to handle variants it cannot process safely.

## File parsing

- ELF64 files with the current ELF version are supported.
- Little-endian and big-endian encodings are supported for parsing, inspection, entry-point
  updates, and serialization.
- ELF32, invalid byte-order identifiers, and non-current ELF versions are rejected.
- The complete 64-byte ELF64 header must be present. Standard 56-byte program-header entries
  and 64-byte section-header entries are required when their corresponding tables are present.
- Program-header and section-header tables must be contained within the file, must not overlap
  the ELF header, and must not overlap each other.
- Extended program-header counts, section-header counts, and section-name table indexes are
  rejected. Files with no section table remain valid when both the section offset and count
  are zero.

## Architecture-specific operations

Disassembly and payload injection support only little-endian x86-64 ELF files. Binkit rejects
other machine types and big-endian files before changing or decoding data. Raw binary
disassembly (`disasm --bin`) is interpreted as x86-64 machine code.

Injection additionally requires the selected section (by default `.note.gnu.property`) and a
program header with the same original file offset. If either structure is absent, injection
fails without creating the output file. Renaming the selected section to `.injected` must fit
in its existing string-table slot without overlapping another section name.

## Compatibility boundaries

Binkit does not currently interpret architecture-specific relocation records, dynamic-linker
state, compressed sections, or extended ELF numbering. Successful parsing means the structural
ELF64 subset is understood; it does not guarantee that every platform-specific ELF feature can
be rewritten safely.
