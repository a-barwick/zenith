use std::process::Command;

#[test]
fn prints_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_zenith"))
        .arg("--version")
        .output()
        .expect("run zenith binary");

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "zenith 0.1.0\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn reports_unavailable_commands_explicitly() {
    let output = Command::new(env!("CARGO_BIN_EXE_zenith"))
        .arg("check")
        .output()
        .expect("run zenith binary");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("unknown or unavailable command `check`")
    );
}
