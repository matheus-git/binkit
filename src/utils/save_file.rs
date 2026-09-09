use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;

pub fn save_file(file: &str, buf: &[u8]) -> Result<(), io::Error> {
    fs::write(file, buf)?;
    let mut perms = fs::metadata(file)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(file, perms)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::save_file;

    #[test]
    fn reports_an_error_when_the_destination_is_a_directory() {
        assert!(save_file(".", b"data").is_err());
    }
}
