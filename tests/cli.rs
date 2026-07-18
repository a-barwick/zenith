use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const HELP: &str = "\
Zenith

A safe, bulk-first language that compiles to Salesforce Apex.

Usage:
  zenith [--help]
  zenith --version
  zenith tokens <file.zen>
  zenith ast <file.zen>
  zenith check [project]
  zenith build [project]
  zenith build [project] --verify-apex-exec <executable>
  zenith emit [project]

Commands:
  tokens    Print the stable lexical token stream for one source file.
  ast       Print the stable parsed syntax tree for one source file.
  check     Check a Zenith project without writing generated files.
  build     Check and write deterministic SFDX-compatible Apex.
  emit      Check and print generated artifacts without writing them.
";

fn zenith() -> Command {
    Command::new(env!("CARGO_BIN_EXE_zenith"))
}

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

struct TempSource(PathBuf);

impl TempSource {
    fn new(bytes: &[u8]) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("zenith-cli-{}-{id}.zen", std::process::id()));
        fs::write(&path, bytes).expect("write temporary Zenith source");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempSource {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

struct TempProject(PathBuf);

impl TempProject {
    fn new(source: &str) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("zenith-cli-project-{}-{id}", std::process::id()));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("zenith.toml"),
            "salesforce-api-version = \"65.0\"\n",
        )
        .unwrap();
        fs::write(root.join("src/Main.zen"), source).unwrap();
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
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
    let output = zenith().arg("verify").output().expect("run zenith binary");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: unknown or unavailable command `verify`\n\
Run `zenith --help` for the available command surface.\n"
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

#[test]
fn tokenizes_hello_fixture_with_stable_golden_output() {
    let output = zenith()
        .args(["tokens"])
        .arg(fixture("examples/hello.zen"))
        .output()
        .expect("run zenith tokens");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        include_str!("golden/hello.tokens")
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn tokenizes_broad_lexical_fixture_with_stable_golden_output() {
    let output = zenith()
        .args(["tokens"])
        .arg(fixture("examples/lexical-baseline.zen"))
        .output()
        .expect("run zenith tokens");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        include_str!("golden/lexical-baseline.tokens")
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn parses_hello_fixture_with_stable_golden_output() {
    let output = zenith()
        .args(["ast"])
        .arg(fixture("examples/hello.zen"))
        .output()
        .expect("run zenith ast");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        include_str!("golden/hello.ast")
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn parses_broad_m2_fixture_with_stable_golden_output() {
    let output = zenith()
        .args(["ast"])
        .arg(fixture("examples/lexical-baseline.zen"))
        .output()
        .expect("run zenith ast");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        include_str!("golden/lexical-baseline.ast")
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn parses_complete_m2_surface_with_stable_golden_output() {
    let output = zenith()
        .args(["ast"])
        .arg(fixture("examples/parsed-baseline.zen"))
        .output()
        .expect("run zenith ast");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        include_str!("golden/parsed-baseline.ast")
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn ast_disambiguates_conditional_expressions_from_nullable_declarations() {
    let source = TempSource::new(
        b"class Conditional {
            void choose() {
                condition ? whenTrue : whenFalse;
            }
        }",
    );
    let output = zenith()
        .arg("ast")
        .arg(source.path())
        .output()
        .expect("run zenith ast");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("conditional @3:17..3:49\n"));
    assert!(stdout.contains("name condition @3:17..3:26\n"));
    assert!(stdout.contains("name whenTrue @3:29..3:37\n"));
    assert!(stdout.contains("name whenFalse @3:40..3:49\n"));
}

#[test]
fn reports_tokens_usage_errors_with_status_two() {
    for arguments in [vec!["tokens"], vec!["tokens", "one.zen", "two.zen"]] {
        let output = zenith().args(arguments).output().expect("run zenith");

        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "error: usage: zenith tokens <file.zen>\n"
        );
    }
}

#[test]
fn reports_ast_usage_errors_with_status_two() {
    for arguments in [vec!["ast"], vec!["ast", "one.zen", "two.zen"]] {
        let output = zenith().args(arguments).output().expect("run zenith");

        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "error: usage: zenith ast <file.zen>\n"
        );
    }
}

#[test]
fn renders_ordered_lexical_diagnostics_and_suppresses_tokens() {
    let source = TempSource::new(b"_bad $ \"double\"\n'ok\\q' /* open");
    let output = zenith()
        .arg("tokens")
        .arg(source.path())
        .output()
        .expect("run zenith tokens");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    let invalid_identifier = stderr.find("lex.invalid-identifier").unwrap();
    let invalid_character = stderr.find("lex.invalid-character").unwrap();
    let double_quote = stderr.find("lex.double-quoted-string").unwrap();
    let invalid_escape = stderr.find("lex.invalid-escape").unwrap();
    let unterminated_comment = stderr.find("lex.unterminated-comment").unwrap();
    assert!(invalid_identifier < invalid_character);
    assert!(invalid_character < double_quote);
    assert!(double_quote < invalid_escape);
    assert!(invalid_escape < unterminated_comment);
    assert!(stderr.contains(":1:1\n"));
    assert!(stderr.contains(":2:4\n"));
    assert!(stderr.ends_with("  = help: add `*/` before the end of the file\n"));
}

#[test]
fn reports_invalid_utf8_as_a_source_diagnostic() {
    let source = TempSource::new(b"valid\xff");
    let output = zenith()
        .arg("tokens")
        .arg(source.path())
        .output()
        .expect("run zenith tokens");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("error[source.invalid-utf8]: "));
    assert!(stderr.contains("  = note: invalid UTF-8 begins at byte 5\n"));
    assert!(stderr.ends_with("  = help: save Zenith source as UTF-8\n"));
}

#[test]
fn ast_reports_lexical_errors_before_parsing_and_suppresses_output() {
    let source = TempSource::new(b"class Broken { void run() { $; } }");
    let output = zenith()
        .arg("ast")
        .arg(source.path())
        .output()
        .expect("run zenith ast");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("error[lex.invalid-character]:"));
    assert!(!stderr.contains("parse."));
}

#[test]
fn ast_renders_ordered_recovered_parse_diagnostics_and_suppresses_tree() {
    let source = TempSource::new(
        b"class Broken {
            Integer = 1;
            void run() {
                Integer missing = ;
                return missing
                missing = 2;
            }
            String recovered;
        }",
    );
    let output = zenith()
        .arg("ast")
        .arg(source.path())
        .output()
        .expect("run zenith ast");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    let identifier = stderr.find("parse.expected-identifier").unwrap();
    let expression = stderr.find("parse.expected-expression").unwrap();
    let token = stderr.find("parse.expected-token").unwrap();
    assert!(identifier < expression);
    assert!(expression < token);
    assert!(stderr.contains("error[parse.expected-identifier]:"));
    assert!(stderr.contains("error[parse.expected-expression]:"));
    assert!(stderr.contains("error[parse.expected-token]:"));
    assert!(!stderr.contains("lex."));
}

#[test]
fn reports_missing_files_as_source_diagnostics() {
    let path = std::env::temp_dir().join(format!("zenith-missing-{}.zen", std::process::id()));
    let output = zenith()
        .arg("tokens")
        .arg(&path)
        .output()
        .expect("run zenith tokens");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("error[source.read-failed]: failed to read `"));
    assert!(stderr.contains("  = note: "));
}

#[test]
fn checks_and_emits_the_m3_acceptance_project() {
    let project = fixture("examples/m3-service");
    let checked = zenith()
        .arg("check")
        .arg(&project)
        .output()
        .expect("check M3 project");
    assert!(checked.status.success());
    assert_eq!(
        String::from_utf8(checked.stdout).unwrap(),
        "Checked 2 classes.\n"
    );
    assert!(checked.stderr.is_empty());

    let emitted = zenith()
        .arg("emit")
        .arg(&project)
        .output()
        .expect("emit M3 project");
    assert!(emitted.status.success());
    assert_eq!(
        String::from_utf8(emitted.stdout).unwrap(),
        fs::read_to_string(fixture("tests/golden/m3-service.emit")).unwrap()
    );
    assert!(emitted.stderr.is_empty());
}

#[cfg(unix)]
#[test]
fn build_writes_artifacts_and_records_optional_verification() {
    let project =
        TempProject::new("public class Main { public static String value() { return 'ok'; } }");
    let output = zenith()
        .arg("build")
        .arg(project.path())
        .arg("--verify-apex-exec")
        .arg("/usr/bin/true")
        .output()
        .expect("build and verify M3 project");
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with(
        "Apex verification: passed (apex-exec, revision \
         1e4f1ca1938abfc996651ae447f227e0db680b6a, profile zenith-m3-apex-baseline).\n"
    ));
    assert!(stdout.contains("Built 1 classes to "));
    assert!(output.stderr.is_empty());

    let manifest = fs::read_to_string(project.path().join(".zenith/build.json")).unwrap();
    assert!(manifest.contains("\"outcome\": \"passed\""));
    assert!(manifest.contains("\"exitStatus\": 0"));
    assert!(
        project
            .path()
            .join(".zenith/generated/main/default/classes/Main.cls")
            .is_file()
    );
    assert!(project.path().join(".zenith/sfdx-project.json").is_file());
}

#[test]
fn unavailable_verifier_is_evidence_not_a_build_failure() {
    let project = TempProject::new("public class Main {}");
    let output = zenith()
        .arg("build")
        .arg(project.path())
        .arg("--verify-apex-exec")
        .arg("definitely-not-a-real-zenith-verifier")
        .output()
        .expect("build with unavailable verifier");
    assert!(output.status.success(), "{output:?}");
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("Apex verification: unsupported")
    );
    assert!(output.stderr.is_empty());
    assert!(
        fs::read_to_string(project.path().join(".zenith/build.json"))
            .unwrap()
            .contains("\"outcome\": \"unsupported\"")
    );
}

#[test]
fn project_compilation_failures_use_status_one_and_write_nothing() {
    let project =
        TempProject::new("public class Main { public static String value() { return Missing; } }");
    let output = zenith()
        .arg("build")
        .arg(project.path())
        .output()
        .expect("build invalid M3 project");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("error[resolve.unknown-name]")
    );
    assert!(!project.path().join(".zenith").exists());
}

#[test]
fn project_commands_report_usage_errors_with_status_two() {
    for command in ["check", "emit"] {
        let output = zenith()
            .arg(command)
            .arg("one")
            .arg("two")
            .output()
            .expect("run invalid project command");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("error: usage: zenith {command} [project]\n")
        );
    }

    let output = zenith()
        .arg("build")
        .arg("one")
        .arg("two")
        .output()
        .expect("run invalid build command");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: usage: zenith build [project]\n"
    );
}
