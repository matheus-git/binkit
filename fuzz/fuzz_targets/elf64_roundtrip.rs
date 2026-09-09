#![no_main]

use binkit::elf64::Elf64Binary;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(binary) = Elf64Binary::new(data) else {
        return;
    };

    let _ = binary.strtab();
    if let Ok(serialized) = Vec::<u8>::try_from(&binary) {
        let _ = Elf64Binary::new(&serialized);
    }
});
