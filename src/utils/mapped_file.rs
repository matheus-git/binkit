use anyhow::{Context, Result, anyhow};
use std::fs::File;
use std::os::fd::AsRawFd;
use std::path::Path;
use std::ptr::NonNull;

pub struct MappedFile {
    pointer: NonNull<u8>,
    len: usize,
}

impl MappedFile {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file =
            File::open(path).with_context(|| format!("Failed to open '{}'", path.display()))?;
        let len = usize::try_from(file.metadata()?.len()).context("File is too large to map")?;
        if len == 0 {
            return Ok(Self {
                pointer: NonNull::dangling(),
                len: 0,
            });
        }

        // SAFETY: the descriptor is open for reading, `len` is the current file length, and the
        // returned mapping is kept alive independently of the descriptor until Drop calls munmap.
        let pointer = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                file.as_raw_fd(),
                0,
            )
        };
        if pointer == libc::MAP_FAILED {
            return Err(anyhow!(std::io::Error::last_os_error()))
                .with_context(|| format!("Failed to map '{}'", path.display()));
        }

        Ok(Self {
            pointer: NonNull::new(pointer.cast()).context("mmap returned a null pointer")?,
            len,
        })
    }
}

impl AsRef<[u8]> for MappedFile {
    fn as_ref(&self) -> &[u8] {
        // SAFETY: the mapping covers `len` readable bytes and remains live for `self`'s lifetime.
        unsafe { std::slice::from_raw_parts(self.pointer.as_ptr(), self.len) }
    }
}

impl Drop for MappedFile {
    fn drop(&mut self) {
        if self.len != 0 {
            // SAFETY: this exact pointer/length pair came from mmap and is unmapped once here.
            unsafe { libc::munmap(self.pointer.as_ptr().cast(), self.len) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MappedFile;
    use std::fs;

    #[test]
    fn maps_file_contents() {
        let path = std::env::temp_dir().join(format!("binkit-mmap-{}", std::process::id()));
        fs::write(&path, b"mapped bytes").unwrap();
        let mapped = MappedFile::open(&path).unwrap();
        assert_eq!(mapped.as_ref(), b"mapped bytes");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn maps_an_empty_file() {
        let path = std::env::temp_dir().join(format!("binkit-mmap-empty-{}", std::process::id()));
        fs::write(&path, []).unwrap();
        let mapped = MappedFile::open(&path).unwrap();
        assert!(mapped.as_ref().is_empty());
        fs::remove_file(path).unwrap();
    }
}
