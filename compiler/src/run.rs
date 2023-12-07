use miette::IntoDiagnostic;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn run(output: &str) -> miette::Result<()> {
    Command::new("chmod")
        .arg("+x")
        .arg(output)
        .output()
        .into_diagnostic()?;

    Command::new(format!("./{output}"))
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .into_diagnostic()?;

    fs::remove_file(Path::new(output)).into_diagnostic()
}
