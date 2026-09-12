use binkit::elf64::{Elf64Binary, calculate_rel32};
use std::fs;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("binkit-test-{}-{id}", std::process::id()));
        fs::create_dir(&path).expect("test directory should be created");
        Self(path)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn elf64_fixture() -> Vec<u8> {
    const PROGRAM_OFFSET: usize = 64;
    const NOTE_OFFSET: usize = 120;
    const STRING_TABLE_OFFSET: usize = 124;
    const SECTION_TABLE_OFFSET: usize = 160;
    const SECTION_SIZE: usize = 64;

    let string_table = b"\0.note.gnu.property\0.shstrtab\0";
    let mut bytes = vec![0_u8; SECTION_TABLE_OFFSET + 3 * SECTION_SIZE];

    bytes[0..4].copy_from_slice(b"\x7fELF");
    bytes[4] = 2;
    bytes[5] = 1;
    bytes[6] = 1;
    write_u16(&mut bytes, 16, 2);
    write_u16(&mut bytes, 18, 62);
    write_u32(&mut bytes, 20, 1);
    write_u64(&mut bytes, 24, 0x401000);
    write_u64(&mut bytes, 32, PROGRAM_OFFSET as u64);
    write_u64(&mut bytes, 40, SECTION_TABLE_OFFSET as u64);
    write_u16(&mut bytes, 52, 64);
    write_u16(&mut bytes, 54, 56);
    write_u16(&mut bytes, 56, 1);
    write_u16(&mut bytes, 58, SECTION_SIZE as u16);
    write_u16(&mut bytes, 60, 3);
    write_u16(&mut bytes, 62, 2);

    write_u32(&mut bytes, PROGRAM_OFFSET, 4);
    write_u32(&mut bytes, PROGRAM_OFFSET + 4, 4);
    write_u64(&mut bytes, PROGRAM_OFFSET + 8, NOTE_OFFSET as u64);
    write_u64(&mut bytes, PROGRAM_OFFSET + 16, 0x400078);
    write_u64(&mut bytes, PROGRAM_OFFSET + 24, 0x400078);
    write_u64(&mut bytes, PROGRAM_OFFSET + 32, 4);
    write_u64(&mut bytes, PROGRAM_OFFSET + 40, 4);
    write_u64(&mut bytes, PROGRAM_OFFSET + 48, 4);

    bytes[NOTE_OFFSET..NOTE_OFFSET + 4].copy_from_slice(&[4, 0, 0, 0]);
    bytes[STRING_TABLE_OFFSET..STRING_TABLE_OFFSET + string_table.len()]
        .copy_from_slice(string_table);

    let note = SECTION_TABLE_OFFSET + SECTION_SIZE;
    write_u32(&mut bytes, note, 1);
    write_u32(&mut bytes, note + 4, 7);
    write_u64(&mut bytes, note + 8, 2);
    write_u64(&mut bytes, note + 16, 0x400078);
    write_u64(&mut bytes, note + 24, NOTE_OFFSET as u64);
    write_u64(&mut bytes, note + 32, 4);
    write_u64(&mut bytes, note + 48, 4);

    let strings = SECTION_TABLE_OFFSET + 2 * SECTION_SIZE;
    write_u32(&mut bytes, strings, 20);
    write_u32(&mut bytes, strings + 4, 3);
    write_u64(&mut bytes, strings + 24, STRING_TABLE_OFFSET as u64);
    write_u64(&mut bytes, strings + 32, string_table.len() as u64);
    write_u64(&mut bytes, strings + 48, 1);

    bytes
}

fn binkit(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_binkit"))
        .args(args)
        .output()
        .expect("binkit should start")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_fixture(path: &Path) {
    fs::write(path, elf64_fixture()).expect("ELF fixture should be written");
}

#[test]
fn info_prints_all_requested_header_groups() {
    let dir = TestDir::new();
    let input = dir.path("fixture.elf");
    write_fixture(&input);

    let output = binkit(&[
        "info",
        input.to_str().unwrap(),
        "--header",
        "--programs",
        "--sections",
    ]);

    assert_success(&output);
    let output = stdout(&output);
    assert!(output.contains("◆ binkit  ELF64"));
    assert!(output.contains("Program headers  1 total"));
    assert!(output.contains("Section headers  3 total"));
    assert!(output.contains("Entry point"));
    assert!(output.contains(".note.gnu.property"));
    assert!(output.contains('│'));
}

#[test]
fn disasm_accepts_raw_machine_code() {
    let dir = TestDir::new();
    let input = dir.path("code.bin");
    fs::write(&input, [0x90, 0xc3]).unwrap();

    let output = binkit(&["disasm", input.to_str().unwrap(), "--bin"]);

    assert_success(&output);
    let output = stdout(&output);
    assert!(output.contains("Disassembly  x86-64 · Intel syntax"));
    assert!(output.contains("Address"));
    assert!(output.contains("nop"));
    assert!(output.contains("ret"));
    assert!(output.contains("Instructions       2"));
}

#[test]
fn disasm_limits_raw_input_by_offset_bytes_and_count() {
    let dir = TestDir::new();
    let input = dir.path("code.bin");
    fs::write(&input, [0xcc, 0x90, 0x90, 0xc3]).unwrap();

    let output = binkit(&[
        "disasm",
        input.to_str().unwrap(),
        "--bin",
        "--offset",
        "1",
        "--bytes",
        "2",
        "--count",
        "1",
    ]);

    assert_success(&output);
    let output = stdout(&output);
    assert!(output.contains("0x0000000000000001 │ 90"));
    assert!(output.contains("│ nop"));
    assert!(output.contains("Instructions       1"));
    assert!(!output.contains("ret"));
}

#[test]
fn disasm_starts_at_elf_virtual_address() {
    let dir = TestDir::new();
    let input = dir.path("fixture.elf");
    let mut fixture = elf64_fixture();
    fixture[120..124].copy_from_slice(&[0x90, 0xc3, 0x90, 0x90]);
    fs::write(&input, fixture).unwrap();

    let output = binkit(&[
        "disasm",
        input.to_str().unwrap(),
        "--section",
        ".note.gnu.property",
        "--address",
        "0x400079",
        "--count",
        "1",
    ]);

    assert_success(&output);
    let output = stdout(&output);
    assert!(output.contains("0x0000000000400079 │ C3"));
    assert!(output.contains("│ ret"));
    assert!(output.contains("Instructions       1"));
}

#[test]
fn disasm_rejects_invalid_ranges_and_conflicting_starts() {
    let dir = TestDir::new();
    let input = dir.path("code.bin");
    fs::write(&input, [0x90]).unwrap();

    let outside = binkit(&["disasm", input.to_str().unwrap(), "--bin", "--offset", "2"]);
    assert!(!outside.status.success());
    assert!(String::from_utf8_lossy(&outside.stderr).contains("outside the input"));

    let conflict = binkit(&[
        "disasm",
        input.to_str().unwrap(),
        "--offset",
        "0",
        "--address",
        "0x0",
    ]);
    assert!(!conflict.status.success());
    assert!(String::from_utf8_lossy(&conflict.stderr).contains("cannot be used with"));
}

#[test]
fn disasm_exits_cleanly_when_a_pipe_closes_early() {
    let dir = TestDir::new();
    let input = dir.path("large-code.bin");
    fs::write(&input, vec![0x90; 64 * 1024]).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_binkit"))
        .args(["disasm", input.to_str().unwrap(), "--bin"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("binkit should start");
    let mut stdout = child.stdout.take().unwrap();
    let mut prefix = [0_u8; 1024];
    stdout.read_exact(&mut prefix).unwrap();
    drop(stdout);

    let output = child.wait_with_output().unwrap();
    assert_success(&output);
    assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
}

#[test]
fn architecture_specific_commands_reject_non_x86_64_elf() {
    let dir = TestDir::new();
    let input = dir.path("aarch64.elf");
    let payload = dir.path("payload.bin");
    let output_path = dir.path("injected.elf");
    let mut fixture = elf64_fixture();
    write_u16(&mut fixture, 18, 183);
    fs::write(&input, fixture).unwrap();
    fs::write(&payload, [0x90]).unwrap();

    let disasm = binkit(&[
        "disasm",
        input.to_str().unwrap(),
        "--section",
        ".note.gnu.property",
    ]);
    assert!(!disasm.status.success());
    assert!(
        String::from_utf8_lossy(&disasm.stderr)
            .contains("supports only little-endian x86-64 ELF files")
    );

    let inject = binkit(&[
        "inject",
        input.to_str().unwrap(),
        "--inject",
        payload.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert!(!inject.status.success());
    assert!(
        String::from_utf8_lossy(&inject.stderr)
            .contains("supports only little-endian x86-64 ELF files")
    );
    assert!(!output_path.exists());
}

#[test]
fn check_inject_reports_the_selected_addresses() {
    let dir = TestDir::new();
    let input = dir.path("fixture.elf");
    write_fixture(&input);

    let output = binkit(&[
        "check-inject",
        input.to_str().unwrap(),
        "--return-address",
        "0x401000",
    ]);

    assert_success(&output);
    let output = stdout(&output);
    assert!(output.contains("Injection plan"));
    assert!(output.contains("Virtual address"));
    assert!(output.contains("Return address     0x0000000000401000"));
    assert!(output.contains("Return rel32"));
}

#[test]
fn update_changes_the_entry_point_and_produces_valid_elf() {
    let dir = TestDir::new();
    let input = dir.path("fixture.elf");
    let output_path = dir.path("updated.elf");
    write_fixture(&input);

    let output = binkit(&[
        "update",
        input.to_str().unwrap(),
        "--entry",
        "0x402000",
        "--output",
        output_path.to_str().unwrap(),
    ]);

    assert_success(&output);
    let command_output = stdout(&output);
    assert!(command_output.contains("Update complete"));
    assert!(command_output.contains("Entry point"));
    assert!(command_output.contains("0x0000000000402000"));
    assert!(command_output.contains("Output written atomically · permissions preserved"));
    let raw = fs::read(&output_path).unwrap();
    assert_eq!(Elf64Binary::new(&raw).unwrap().entry(), 0x402000);
    let readelf = Command::new("readelf")
        .args(["-h", output_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&readelf);
    assert!(stdout(&readelf).contains("0x402000"));
}

#[test]
fn inject_adds_payload_and_produces_valid_elf_structure() {
    let dir = TestDir::new();
    let input = dir.path("fixture.elf");
    let payload = dir.path("payload.bin");
    let output_path = dir.path("injected.elf");
    write_fixture(&input);
    fs::write(&payload, [0x90, 0xc3]).unwrap();

    let output = binkit(&[
        "inject",
        input.to_str().unwrap(),
        "--inject",
        payload.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);

    assert_success(&output);
    let command_output = stdout(&output);
    assert!(command_output.contains("Injection complete"));
    assert!(command_output.contains("Section            .injected"));
    assert!(command_output.contains("Output written atomically · permissions preserved"));
    let raw = fs::read(&output_path).unwrap();
    assert!(raw.ends_with(&[0x90, 0xc3]));
    Elf64Binary::new(&raw).expect("injected output should remain parseable");
    let readelf = Command::new("readelf")
        .args(["-S", output_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&readelf);
    assert!(stdout(&readelf).contains(".injected"));
    let objdump = Command::new("objdump")
        .args(["-h", output_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&objdump);
    assert!(stdout(&objdump).contains(".injected"));
}

#[test]
fn injected_payload_executes_from_the_updated_entry_point() {
    let dir = TestDir::new();
    let source = dir.path("fixture.c");
    let input = dir.path("fixture");
    let payload = dir.path("exit-42.bin");
    let output_path = dir.path("fixture.injected");
    fs::write(&source, "int main(void) { return 0; }\n").unwrap();

    let compile = Command::new("gcc")
        .args([
            "-no-pie",
            "-fcf-protection=full",
            source.to_str().unwrap(),
            "-o",
            input.to_str().unwrap(),
        ])
        .output()
        .expect("gcc should start");
    assert_success(&compile);

    let raw = fs::read(&input).unwrap();
    let injection_address = Elf64Binary::new(&raw)
        .unwrap()
        .get_address_to_inject()
        .unwrap();

    // mov edi, 42; mov eax, 60; syscall
    fs::write(
        &payload,
        [
            0xbf, 0x2a, 0x00, 0x00, 0x00, 0xb8, 0x3c, 0x00, 0x00, 0x00, 0x0f, 0x05,
        ],
    )
    .unwrap();

    let inject = binkit(&[
        "inject",
        input.to_str().unwrap(),
        "--inject",
        payload.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert_success(&inject);

    let entry = format!("0x{injection_address:X}");
    let update = binkit(&["update", output_path.to_str().unwrap(), "--entry", &entry]);
    assert_success(&update);

    let execution = Command::new(&output_path)
        .output()
        .expect("injected executable should start");
    assert_eq!(execution.status.code(), Some(42));
    assert!(execution.stdout.is_empty());
    assert!(execution.stderr.is_empty());
}

#[test]
fn injected_payload_returns_to_the_original_entry_point() {
    let dir = TestDir::new();
    let source = dir.path("return-fixture.c");
    let input = dir.path("return-fixture");
    let payload = dir.path("return-payload.bin");
    let output_path = dir.path("return-fixture.injected");
    fs::write(
        &source,
        "#include <stdio.h>\nint main(void) { puts(\"original\"); return 7; }\n",
    )
    .unwrap();

    let compile = Command::new("gcc")
        .args([
            "-no-pie",
            "-fcf-protection=full",
            source.to_str().unwrap(),
            "-o",
            input.to_str().unwrap(),
        ])
        .output()
        .expect("gcc should start");
    assert_success(&compile);

    let raw = fs::read(&input).unwrap();
    let binary = Elf64Binary::new(&raw).unwrap();
    let original_entry = binary.entry();
    let injection_address = binary.get_address_to_inject().unwrap();

    // Preserve initial state, write "injected\n", restore state, then jump to the original entry.
    let mut payload_bytes = vec![
        0x9c, 0x50, 0x57, 0x56, 0x52, 0x51, 0x41, 0x53, 0xb8, 0x01, 0x00, 0x00, 0x00, 0xbf, 0x01,
        0x00, 0x00, 0x00, 0x48, 0x8d, 0x35, 0x14, 0x00, 0x00, 0x00, 0xba, 0x09, 0x00, 0x00, 0x00,
        0x0f, 0x05, 0x41, 0x5b, 0x59, 0x5a, 0x5e, 0x5f, 0x58, 0x9d, 0xe9, 0x00, 0x00, 0x00, 0x00,
        b'i', b'n', b'j', b'e', b'c', b't', b'e', b'd', b'\n',
    ];
    let address_after_jump = injection_address.checked_add(45).unwrap();
    let jump = calculate_rel32(address_after_jump, original_entry)
        .unwrap()
        .to_le_bytes();
    payload_bytes[41..45].copy_from_slice(&jump);
    fs::write(&payload, payload_bytes).unwrap();

    let inject = binkit(&[
        "inject",
        input.to_str().unwrap(),
        "--inject",
        payload.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert_success(&inject);

    let entry = format!("0x{injection_address:X}");
    let update = binkit(&["update", output_path.to_str().unwrap(), "--entry", &entry]);
    assert_success(&update);

    let execution = Command::new(&output_path)
        .output()
        .expect("injected executable should return to the original entry point");
    assert_eq!(execution.status.code(), Some(7));
    assert_eq!(execution.stdout, b"injected\noriginal\n");
    assert!(execution.stderr.is_empty());
}

#[test]
fn inject_rejects_a_section_name_slot_that_is_too_short() {
    let dir = TestDir::new();
    let input = dir.path("short-name.elf");
    let payload = dir.path("payload.bin");
    let output_path = dir.path("injected.elf");
    let mut fixture = elf64_fixture();
    fixture[125..128].copy_from_slice(b".x\0");
    fs::write(&input, fixture).unwrap();
    fs::write(&payload, [0x90]).unwrap();

    let output = binkit(&[
        "inject",
        input.to_str().unwrap(),
        "--inject",
        payload.to_str().unwrap(),
        "--section",
        ".x",
        "--output",
        output_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    assert!(!output_path.exists());
    assert!(String::from_utf8_lossy(&output.stderr).contains("too short"));
}

#[test]
fn commands_report_common_input_errors() {
    let dir = TestDir::new();
    let missing = dir.path("missing.elf");
    let invalid = dir.path("invalid.elf");
    let fixture = dir.path("fixture.elf");
    let payload = dir.path("payload.bin");
    fs::write(&invalid, b"not an ELF").unwrap();
    write_fixture(&fixture);
    fs::write(&payload, [0x90]).unwrap();

    assert!(
        !binkit(&["info", missing.to_str().unwrap(), "--header"])
            .status
            .success()
    );
    assert!(
        !binkit(&["info", invalid.to_str().unwrap(), "--header"])
            .status
            .success()
    );
    assert!(
        !binkit(&["update", fixture.to_str().unwrap(), "--entry", "invalid"])
            .status
            .success()
    );
    assert!(
        !binkit(&[
            "inject",
            fixture.to_str().unwrap(),
            "--inject",
            payload.to_str().unwrap(),
            "--section",
            ".missing",
            "--output",
            dir.path("output.elf").to_str().unwrap(),
        ])
        .status
        .success()
    );
}

#[test]
fn malformed_section_name_returns_an_error_without_panicking() {
    let dir = TestDir::new();
    let input = dir.path("malformed.elf");
    let mut fixture = elf64_fixture();
    write_u32(&mut fixture, 160 + 64, u32::MAX);
    fs::write(&input, fixture).unwrap();

    let output = binkit(&["info", input.to_str().unwrap(), "--sections"]);

    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
}

#[test]
fn update_reports_a_destination_write_failure() {
    let dir = TestDir::new();
    let input = dir.path("fixture.elf");
    write_fixture(&input);

    let output = binkit(&[
        "update",
        input.to_str().unwrap(),
        "--entry",
        "0x402000",
        "--output",
        dir.0.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
}

#[test]
fn update_requires_force_to_overwrite_an_existing_output() {
    let dir = TestDir::new();
    let input = dir.path("fixture.elf");
    let output_path = dir.path("existing.elf");
    write_fixture(&input);
    fs::write(&output_path, b"keep me").unwrap();

    let args = [
        "update",
        input.to_str().unwrap(),
        "--entry",
        "0x402000",
        "--output",
        output_path.to_str().unwrap(),
    ];
    let rejected = binkit(&args);
    assert!(!rejected.status.success());
    assert_eq!(fs::read(&output_path).unwrap(), b"keep me");

    let mut forced_args = args.to_vec();
    forced_args.push("--force");
    assert_success(&binkit(&forced_args));
    assert_ne!(fs::read(&output_path).unwrap(), b"keep me");
    assert!(fs::read_dir(&dir.0).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("binkit-tmp")
    }));
}

#[test]
fn update_preserves_source_permissions() {
    let dir = TestDir::new();
    let input = dir.path("fixture.elf");
    let output_path = dir.path("updated.elf");
    write_fixture(&input);
    fs::set_permissions(&input, fs::Permissions::from_mode(0o750)).unwrap();

    assert_success(&binkit(&[
        "update",
        input.to_str().unwrap(),
        "--entry",
        "0x402000",
        "--output",
        output_path.to_str().unwrap(),
    ]));

    let mode = fs::metadata(output_path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o750);
}
