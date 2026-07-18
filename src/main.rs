use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use zenith::{
    APEX_EXEC_M3_REVISION, Diagnostic, Phase, ProcessVerifier, SourceId, SourceMap,
    compile_project, lex, parse, record_verification, render_artifacts, render_ast,
    render_diagnostics, render_tokens, write_artifacts,
};

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

fn main() -> ExitCode {
    let arguments: Vec<_> = env::args_os().skip(1).collect();

    match arguments.as_slice() {
        [] => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        [argument]
            if matches!(
                argument.to_str(),
                Some("help") | Some("-h") | Some("--help")
            ) =>
        {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        [argument]
            if matches!(
                argument.to_str(),
                Some("version") | Some("-V") | Some("--version")
            ) =>
        {
            println!("zenith {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        [command, path] if command == OsStr::new("tokens") => tokens(Path::new(path)),
        [command, path] if command == OsStr::new("ast") => ast(Path::new(path)),
        [command] if command == OsStr::new("check") => {
            project_command("check", Path::new("."), None)
        }
        [command, path] if command == OsStr::new("check") => {
            project_command("check", Path::new(path), None)
        }
        [command] if command == OsStr::new("build") => {
            project_command("build", Path::new("."), None)
        }
        [command, path] if command == OsStr::new("build") => {
            project_command("build", Path::new(path), None)
        }
        [command, flag, executable]
            if command == OsStr::new("build") && flag == OsStr::new("--verify-apex-exec") =>
        {
            project_command("build", Path::new("."), Some(Path::new(executable)))
        }
        [command, path, flag, executable]
            if command == OsStr::new("build") && flag == OsStr::new("--verify-apex-exec") =>
        {
            project_command("build", Path::new(path), Some(Path::new(executable)))
        }
        [command] if command == OsStr::new("emit") => project_command("emit", Path::new("."), None),
        [command, path] if command == OsStr::new("emit") => {
            project_command("emit", Path::new(path), None)
        }
        [command, ..] if command == OsStr::new("tokens") => {
            eprintln!("error: usage: zenith tokens <file.zen>");
            ExitCode::from(2)
        }
        [command, ..] if command == OsStr::new("ast") => {
            eprintln!("error: usage: zenith ast <file.zen>");
            ExitCode::from(2)
        }
        [command, ..]
            if matches!(
                command.to_str(),
                Some("check") | Some("build") | Some("emit")
            ) =>
        {
            eprintln!(
                "error: usage: zenith {} [project]",
                command.to_string_lossy()
            );
            ExitCode::from(2)
        }
        [command, ..] => {
            eprintln!(
                "error: unknown or unavailable command `{}`",
                command.to_string_lossy()
            );
            eprintln!("Run `zenith --help` for the available command surface.");
            ExitCode::from(2)
        }
    }
}

fn project_command(command: &str, path: &Path, verifier: Option<&Path>) -> ExitCode {
    let mut compilation = compile_project(path);
    if compilation.has_errors() {
        eprint!(
            "{}",
            render_diagnostics(&compilation.sources, &compilation.diagnostics)
        );
        return ExitCode::from(1);
    }

    match command {
        "check" => {
            println!("Checked {} classes.", compilation.class_count());
            ExitCode::SUCCESS
        }
        "emit" => {
            print!("{}", render_artifacts(&compilation.artifacts));
            ExitCode::SUCCESS
        }
        "build" => {
            let config = compilation
                .config
                .as_ref()
                .expect("successful project compilation has configuration");
            let output_root = path.join(&config.output_root);
            if let Err(diagnostic) = write_artifacts(&output_root, &compilation.artifacts) {
                eprint!(
                    "{}",
                    render_diagnostics(&compilation.sources, &[*diagnostic])
                );
                return ExitCode::from(1);
            }
            if let Some(executable) = verifier {
                let result = ProcessVerifier::apex_exec(executable, APEX_EXEC_M3_REVISION)
                    .verify(&output_root);
                record_verification(&mut compilation.artifacts, &result);
                if let Err(diagnostic) = write_artifacts(&output_root, &compilation.artifacts) {
                    eprint!(
                        "{}",
                        render_diagnostics(&compilation.sources, &[*diagnostic])
                    );
                    return ExitCode::from(1);
                }
                println!(
                    "Apex verification: {} ({}, revision {}, profile {}).",
                    result.outcome.as_str(),
                    result.backend,
                    result.revision,
                    result.capability_profile
                );
                if !result.stdout.is_empty() {
                    print!("{}", result.stdout);
                    if !result.stdout.ends_with('\n') {
                        println!();
                    }
                }
                if !result.stderr.is_empty() {
                    eprint!("{}", result.stderr);
                    if !result.stderr.ends_with('\n') {
                        eprintln!();
                    }
                }
            }
            println!(
                "Built {} classes to {}.",
                compilation.class_count(),
                output_root.to_string_lossy()
            );
            ExitCode::SUCCESS
        }
        _ => unreachable!("project command was selected by argument parser"),
    }
}

fn tokens(path: &Path) -> ExitCode {
    let (sources, source) = match load_source(path) {
        Ok(loaded) => loaded,
        Err(exit) => return exit,
    };
    let file = sources.get(source).expect("source was just inserted");
    let result = lex(file);

    if result.has_errors() {
        eprint!("{}", render_diagnostics(&sources, &result.diagnostics));
        ExitCode::from(1)
    } else {
        print!("{}", render_tokens(file, &result.tokens));
        ExitCode::SUCCESS
    }
}

fn ast(path: &Path) -> ExitCode {
    let (sources, source) = match load_source(path) {
        Ok(loaded) => loaded,
        Err(exit) => return exit,
    };
    let file = sources.get(source).expect("source was just inserted");
    let lexical = lex(file);
    if lexical.has_errors() {
        eprint!("{}", render_diagnostics(&sources, &lexical.diagnostics));
        return ExitCode::from(1);
    }

    let parsed = parse(file, &lexical.tokens);
    if parsed.has_errors() {
        eprint!("{}", render_diagnostics(&sources, &parsed.diagnostics));
        ExitCode::from(1)
    } else {
        let unit = parsed
            .unit
            .as_ref()
            .expect("successful parsing produces a compilation unit");
        print!("{}", render_ast(file, unit));
        ExitCode::SUCCESS
    }
}

fn load_source(path: &Path) -> Result<(SourceMap, SourceId), ExitCode> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            let diagnostic = Diagnostic::coded_error(
                Phase::Source,
                "source.read-failed",
                format!("failed to read `{}`", path.to_string_lossy()),
                None,
            )
            .with_note(error.to_string());
            eprint!("{}", render_diagnostics(&SourceMap::new(), &[diagnostic]));
            return Err(ExitCode::from(1));
        }
    };

    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => {
            let valid_up_to = error.utf8_error().valid_up_to();
            let diagnostic = Diagnostic::coded_error(
                Phase::Source,
                "source.invalid-utf8",
                format!("`{}` is not valid UTF-8", path.to_string_lossy()),
                None,
            )
            .with_note(format!("invalid UTF-8 begins at byte {valid_up_to}"))
            .with_help("save Zenith source as UTF-8");
            eprint!("{}", render_diagnostics(&SourceMap::new(), &[diagnostic]));
            return Err(ExitCode::from(1));
        }
    };

    let mut sources = SourceMap::new();
    let source = sources.add(path, text);
    Ok((sources, source))
}
