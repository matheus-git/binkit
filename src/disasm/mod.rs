use crate::presentation::heading;
use crate::utils::bytes_to_hex::bytes_to_hex;
use anyhow::{Result, anyhow};
use capstone_sys::{
    cs_arch, cs_close, cs_disasm_iter, cs_err, cs_errno, cs_free, cs_insn, cs_malloc, cs_mode,
    cs_open, cs_opt_type, cs_opt_value, cs_option, cs_strerror, csh,
};
use std::ffi::CStr;
use std::io::{self, Write};
use std::ptr;

struct IterativeDisassembler {
    handle: csh,
    instruction: *mut cs_insn,
}

impl IterativeDisassembler {
    fn x86_64() -> Result<Self> {
        let mut handle = 0;
        // SAFETY: the output pointer is valid and this architecture/mode pair is supported.
        let error = unsafe { cs_open(cs_arch::CS_ARCH_X86, cs_mode::CS_MODE_64, &mut handle) };
        if error != cs_err::CS_ERR_OK {
            return Err(capstone_error(error));
        }

        // SAFETY: `handle` is live and Intel is a valid x86 syntax.
        let error = unsafe {
            cs_option(
                handle,
                cs_opt_type::CS_OPT_SYNTAX,
                cs_opt_value::CS_OPT_SYNTAX_INTEL as usize,
            )
        };
        if error != cs_err::CS_ERR_OK {
            // SAFETY: this is the sole owner of the live handle.
            unsafe { cs_close(&mut handle) };
            return Err(capstone_error(error));
        }

        // SAFETY: `handle` is live. The allocation is released in Drop.
        let instruction = unsafe { cs_malloc(handle) };
        if instruction.is_null() {
            // SAFETY: this is the sole owner of the live handle.
            unsafe { cs_close(&mut handle) };
            return Err(anyhow!("Capstone failed to allocate an instruction"));
        }
        Ok(Self {
            handle,
            instruction,
        })
    }

    fn write_all(
        &mut self,
        output: &mut impl Write,
        buf: &[u8],
        addr: u64,
        max_instructions: Option<usize>,
    ) -> Result<usize> {
        let mut code = buf.as_ptr();
        let mut remaining = buf.len();
        let mut address = addr;
        let mut count = 0;

        while remaining != 0 && max_instructions.is_none_or(|maximum| count < maximum) {
            // SAFETY: `code` and `remaining` describe the unread suffix of `buf`; the remaining
            // pointers refer to writable state owned by this method and decoder.
            let decoded = unsafe {
                cs_disasm_iter(
                    self.handle,
                    &mut code,
                    &mut remaining,
                    &mut address,
                    self.instruction,
                )
            };
            if !decoded {
                // SAFETY: the handle remains live for this method's duration.
                let error = unsafe { cs_errno(self.handle) };
                if error != cs_err::CS_ERR_OK {
                    return Err(capstone_error(error));
                }
                break;
            }

            // SAFETY: successful decoding initialized the one-entry instruction cache.
            let instruction = unsafe { &*self.instruction };
            let size = usize::from(instruction.size);
            let bytes = instruction
                .bytes
                .get(..size)
                .ok_or_else(|| anyhow!("Capstone returned an invalid instruction size"))?;
            // SAFETY: Capstone guarantees NUL-terminated strings in these fixed arrays.
            let mnemonic = unsafe { CStr::from_ptr(instruction.mnemonic.as_ptr()) }
                .to_str()
                .unwrap_or("<unknown>");
            // SAFETY: same guarantee as for `mnemonic`.
            let operands = unsafe { CStr::from_ptr(instruction.op_str.as_ptr()) }
                .to_str()
                .unwrap_or("");
            let separator = if operands.is_empty() { "" } else { " " };
            let bytes = bytes_to_hex(bytes);
            let assembly_width = mnemonic.len() + separator.len() + operands.len();
            let padding = 48_usize.saturating_sub(assembly_width);
            write!(
                output,
                "0x{:016X} │ {mnemonic}{separator}{operands}",
                instruction.address,
            )?;
            writeln!(output, "{:<padding$} │ {bytes}", "")?;
            count += 1;
        }
        Ok(count)
    }
}

impl Drop for IterativeDisassembler {
    fn drop(&mut self) {
        // SAFETY: both resources are uniquely owned and released exactly once here.
        unsafe {
            if !self.instruction.is_null() {
                cs_free(self.instruction, 1);
                self.instruction = ptr::null_mut();
            }
            if self.handle != 0 {
                cs_close(&mut self.handle);
            }
        }
    }
}

fn capstone_error(error: cs_err::Type) -> anyhow::Error {
    // SAFETY: Capstone returns a static NUL-terminated string for each error code.
    let message = unsafe { CStr::from_ptr(cs_strerror(error)) }.to_string_lossy();
    anyhow!("Capstone error: {message}")
}

pub fn disass(addr: u64, buf: &[u8]) -> Result<()> {
    disass_with_count(addr, buf, None)
}

pub fn disass_with_count(addr: u64, buf: &[u8], max_instructions: Option<usize>) -> Result<()> {
    let mut decoder = IterativeDisassembler::x86_64()?;
    heading("Disassembly", "x86-64 · Intel syntax")?;

    let stdout = io::stdout();
    let mut output = io::BufWriter::new(stdout.lock());
    let result = (|| -> Result<()> {
        writeln!(output, "{:<18} │ {:<48} │ Bytes", "Address", "Assembly")?;
        writeln!(
            output,
            "{}─┼─{}─┼─{}",
            "─".repeat(18),
            "─".repeat(48),
            "─".repeat(44)
        )?;
        let instruction_count = decoder.write_all(&mut output, buf, addr, max_instructions)?;
        writeln!(output)?;
        writeln!(output, "{:<18} {instruction_count}", "Instructions")?;
        writeln!(output, "{:<18} {}", "Decoded bytes", buf.len())?;
        output.flush()?;
        Ok(())
    })();

    match result {
        Err(error)
            if error
                .downcast_ref::<io::Error>()
                .is_some_and(|error| error.kind() == io::ErrorKind::BrokenPipe) =>
        {
            Ok(())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::IterativeDisassembler;
    use std::io::{self, Write};

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn iterator_stops_at_instruction_limit() {
        let mut decoder = IterativeDisassembler::x86_64().unwrap();
        let mut output = Vec::new();
        let count = decoder
            .write_all(&mut output, &[0x90, 0x90, 0xc3], 0x1000, Some(2))
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(count, 2);
        assert_eq!(output.matches("nop").count(), 2);
        assert!(!output.contains("ret"));
    }

    #[test]
    fn iterator_handles_incomplete_instruction_without_error() {
        let mut decoder = IterativeDisassembler::x86_64().unwrap();
        let mut output = Vec::new();
        let count = decoder.write_all(&mut output, &[0x0f], 0, None).unwrap();
        assert_eq!(count, 0);
        assert!(output.is_empty());
    }

    #[test]
    fn zero_instruction_limit_decodes_nothing() {
        let mut decoder = IterativeDisassembler::x86_64().unwrap();
        let mut output = Vec::new();
        let count = decoder.write_all(&mut output, &[0x90], 0, Some(0)).unwrap();
        assert_eq!(count, 0);
        assert!(output.is_empty());
    }

    #[test]
    fn iterator_propagates_writer_errors() {
        let mut decoder = IterativeDisassembler::x86_64().unwrap();
        let error = decoder
            .write_all(&mut FailingWriter, &[0x90], 0, None)
            .unwrap_err();
        assert_eq!(
            error.downcast_ref::<io::Error>().unwrap().kind(),
            io::ErrorKind::BrokenPipe
        );
    }
}
