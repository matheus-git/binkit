use std::fmt::Arguments;
use std::io::{self, Write};

pub fn line(arguments: Arguments<'_>) -> io::Result<()> {
    let result = writeln!(io::stdout().lock(), "{arguments}");
    match result {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        other => other,
    }
}

pub fn blank() -> io::Result<()> {
    line(format_args!(""))
}

pub fn heading(title: &str, context: &str) -> io::Result<()> {
    if context.is_empty() {
        line(format_args!("{title}"))?;
    } else {
        line(format_args!("{title}  {context}"))?;
    }
    line(format_args!("{}", "─".repeat(64)))
}

pub fn field(label: &str, value: impl std::fmt::Display) -> io::Result<()> {
    line(format_args!("{label:<18} {value}"))
}

pub fn success(message: impl std::fmt::Display) -> io::Result<()> {
    line(format_args!("✓ {message}"))
}
