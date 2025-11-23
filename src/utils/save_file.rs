use std::os::unix::fs::PermissionsExt;
use std::fs;
use clap::Error;

pub fn save_file(file: &str, buf: &[u8]) -> Result<(), Error>{
    let _ = fs::write(file, buf);
    let mut perms = fs::metadata(&file)?.permissions();
    perms.set_mode(0o755); 
    fs::set_permissions(&file, perms)?;
    Ok(())
}
