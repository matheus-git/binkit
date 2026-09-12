use std::fs;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

fn parent_or_current(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

#[cfg(unix)]
fn sync_parent(path: &Path) -> io::Result<()> {
    fs::File::open(parent_or_current(path))?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent(_path: &Path) -> io::Result<()> {
    Ok(())
}

fn temporary_path(destination: &Path) -> io::Result<PathBuf> {
    let parent = parent_or_current(destination);
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
            fs::rename(&temporary, destination)?;
        } else {
            fs::hard_link(&temporary, destination)?;
            fs::remove_file(&temporary)?;
        }
        sync_parent(destination)
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{parent_or_current, save_file};
    use std::path::Path;

    #[test]
    fn reports_an_error_when_the_destination_is_a_directory() {
        assert!(save_file(".", b"data", true, None).is_err());
    }

    #[test]
    fn treats_a_bare_filename_as_being_in_the_current_directory() {
        assert_eq!(parent_or_current(Path::new("output.bin")), Path::new("."));
    }
}
