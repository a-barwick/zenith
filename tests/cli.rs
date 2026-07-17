use std::process::Command;

const HELP: &str = "\
Zenith

A safe, bulk-first language that compiles to Salesforce Apex.

Usage:
  zenith [--help]
  zenith --version

Compiler commands begin with roadmap milestone M1.
";

fn zenith() -> Command {
    Command::new(env!("CARGO_BIN_EXE_zenith"))
}

#[test]
fn prints_help_without_arguments() {
    let output = zenith().output().expect("run zenith binary");

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), HELP);
    assert!(output.stderr.is_empty());
}

#[test]
fn prints_help_for_every_supported_alias() {
    for argument in ["help", "-h", "--help"] {
        let output = zenith().arg(argument).output().expect("run zenith binary");

        assert!(output.status.success(), "argument {argument}");
        assert_eq!(String::from_utf8(output.stdout).unwrap(), HELP);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn prints_version_for_every_supported_alias() {
    for argument in ["version", "-V", "--version"] {
        let output = zenith().arg(argument).output().expect("run zenith binary");

        assert!(output.status.success(), "argument {argument}");
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "zenith 0.1.0\n");
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn reports_unavailable_commands_explicitly() {
    let output = zenith().arg("check").output().expect("run zenith binary");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: unknown or unavailable command `check`\n\
Run `zenith --help` for the bootstrap command surface.\n"
    );
}

#[cfg(unix)]
#[test]
fn handles_non_utf8_arguments_without_panicking() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let output = zenith()
        .arg(OsStr::from_bytes(b"\xff"))
        .output()
        .expect("run zenith binary");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .starts_with("error: unknown or unavailable command `")
    );
}
