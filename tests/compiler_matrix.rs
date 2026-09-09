use binkit::elf64::Elf64Binary;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("binkit-compiler-test-{}-{id}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn tool_exists(tool: &str) -> bool {
    Command::new(tool).arg("--version").output().is_ok()
}

fn compile_and_verify(compiler: &str, name: &str, flags: &[&str]) {
    if !tool_exists(compiler) {
        eprintln!("skipping {name}: {compiler} is not installed");
        return;
    }

    let dir = TestDir::new();
    let source = dir.0.join("fixture.c");
    let binary = dir.0.join(name);
    fs::write(
        &source,
        "int answer(void) { return 42; }\nint main(void) { return answer() != 42; }\n",
    )
    .unwrap();

    let status = Command::new(compiler)
        .arg(&source)
        .args(flags)
        .arg("-o")
        .arg(&binary)
        .status()
        .unwrap();
    assert!(status.success(), "failed to compile {name} with {compiler}");

    let raw = fs::read(&binary).unwrap();
    Elf64Binary::new(&raw).expect("compiler output should parse as ELF64");

    let readelf = Command::new("readelf")
        .args(["-h", binary.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(readelf.status.success());

    let info = Command::new(env!("CARGO_BIN_EXE_binkit"))
        .args(["info", binary.to_str().unwrap(), "--header", "--sections"])
        .output()
        .unwrap();
    assert!(
        info.status.success(),
        "binkit info failed for {name}: {}",
        String::from_utf8_lossy(&info.stderr)
    );
}

#[test]
fn parses_gcc_pie_output() {
    compile_and_verify("gcc", "gcc-pie", &[]);
}

#[test]
fn parses_gcc_non_pie_output() {
    compile_and_verify("gcc", "gcc-non-pie", &["-no-pie"]);
}

#[test]
fn parses_gcc_static_output() {
    compile_and_verify("gcc", "gcc-static", &["-static"]);
}

#[test]
fn parses_gcc_shared_object() {
    compile_and_verify("gcc", "libfixture.so", &["-shared", "-fPIC"]);
}

#[test]
fn parses_clang_lld_output() {
    compile_and_verify("clang", "clang-lld", &["-fuse-ld=lld"]);
}

#[test]
fn parses_stripped_output() {
    if !tool_exists("gcc") || !tool_exists("strip") {
        eprintln!("skipping stripped fixture: required tools are not installed");
        return;
    }
    let dir = TestDir::new();
    let source = dir.0.join("fixture.c");
    let binary = dir.0.join("stripped");
    fs::write(&source, "int main(void) { return 0; }\n").unwrap();
    assert!(
        Command::new("gcc")
            .args([source.to_str().unwrap(), "-o", binary.to_str().unwrap()])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("strip")
            .arg(&binary)
            .status()
            .unwrap()
            .success()
    );
    let raw = fs::read(binary).unwrap();
    Elf64Binary::new(&raw).expect("stripped compiler output should parse as ELF64");
}
