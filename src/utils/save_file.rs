use std::fs;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

fn temporary_path(destination: &Path) -> io::Result<PathBuf> {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let name = destination.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "destination has no file name")
    })?;
    let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    Ok(parent.join(format!(
        ".{}.binkit-tmp-{}-{id}",
        name.to_string_lossy(),
        std::process::id()
    )))
}

pub fn save_file(
    file: &str,
    buf: &[u8],
    overwrite: bool,
    permissions_from: Option<&str>,
) -> Result<(), io::Error> {
    let destination = Path::new(file);
    if !overwrite && destination.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "destination already exists; use --force to overwrite it",
        ));
    }

    let temporary = temporary_path(destination)?;
    let result = (|| {
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        output.write_all(buf)?;
        if let Some(source) = permissions_from {
            output.set_permissions(fs::metadata(source)?.permissions())?;
        }
        output.sync_all()?;

        if overwrite {
            fs::rename(&temporary, destination)
        } else {
            fs::hard_link(&temporary, destination)?;
            fs::remove_file(&temporary)
        }
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::save_file;

    #[test]
    fn reports_an_error_when_the_destination_is_a_directory() {
        assert!(save_file(".", b"data", true, None).is_err());
    }
}
