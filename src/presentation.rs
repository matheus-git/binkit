use std::fmt::Arguments;
use std::io::{self, IsTerminal, Write};

fn colors_enabled() -> bool {
    io::stdout().is_terminal()
        && std::env::var_os("NO_COLOR").is_none()
        && std::env::var_os("TERM").is_none_or(|term| term != "dumb")
}

fn paint(code: &str, value: impl std::fmt::Display) -> String {
    if colors_enabled() {
        format!("\x1b[{code}m{value}\x1b[0m")
    } else {
        value.to_string()
    }
}

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
    let brand = paint("36", "◆ binkit");
    let title = paint("1", title);
    if context.is_empty() {
        line(format_args!("{brand}  {title}"))?;
    } else {
        line(format_args!("{brand}  {title}  {}", paint("2", context)))?;
    }
    line(format_args!("{}", paint("2", "─".repeat(64))))
}

pub fn field(label: &str, value: impl std::fmt::Display) -> io::Result<()> {
    let label = paint("2", format_args!("{label:<18}"));
    line(format_args!("{label} {value}"))
}

pub fn accent_field(label: &str, value: impl std::fmt::Display) -> io::Result<()> {
    let label = paint("2", format_args!("{label:<18}"));
    line(format_args!("{label} {}", paint("36", value)))
}

pub fn write_field(
    output: &mut impl Write,
    label: &str,
    value: impl std::fmt::Display,
) -> io::Result<()> {
    let label = paint("2", format_args!("{label:<18}"));
    writeln!(output, "{label} {value}")
}

pub fn success(message: impl std::fmt::Display) -> io::Result<()> {
    line(format_args!("{} {message}", paint("32", "✓")))
}
