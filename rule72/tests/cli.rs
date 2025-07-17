use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn test_width_arg() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rule72")?;
    let mut child = cmd
        .arg("--width")
        .arg("80")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let child_stdin = child.stdin.as_mut().unwrap();
    child_stdin.write_all(b"This is a test of the width argument.")?;

    let output = child.wait_with_output()?;
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("This is a test of the width argument."));

    Ok(())
}

#[test]
fn test_division_by_zero_cli() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rule72")?;
    let mut child = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;

    let child_stdin = child.stdin.as_mut().unwrap();
    child_stdin.write_all(b"Subject\n\n \n")?;

    let output = child.wait_with_output()?;
    assert!(output.status.success());

    Ok(())
}


#[test]
fn test_headline_width_arg() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rule72")?;
    let mut child = cmd
        .arg("--headline-width")
        .arg("60")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let child_stdin = child.stdin.as_mut().unwrap();
    child_stdin.write_all(b"This is a test of the headline-width argument.")?;

    let output = child.wait_with_output()?;
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("This is a test of the headline-width argument."));

    Ok(())
}

#[test]
fn test_simple_reflow() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rule72")?;
    let mut child = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;

    let child_stdin = child.stdin.as_mut().unwrap();
    child_stdin.write_all(b"This is a long line that should be wrapped by the tool.")?;

    let output = child.wait_with_output()?;
    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("This is a long line that should be wrapped by the tool.")
    );

    Ok(())
}
