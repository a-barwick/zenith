use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use zenith::{
    Diagnostic, Phase, SourceId, SourceMap, lex, parse, render_ast, render_diagnostics,
    render_tokens,
};

const HELP: &str = "\
Zenith

A safe, bulk-first language that compiles to Salesforce Apex.

Usage:
  zenith [--help]
  zenith --version
  zenith tokens <file.zen>
  zenith ast <file.zen>

Commands:
  tokens    Print the stable lexical token stream for one source file.
  ast       Print the stable parsed syntax tree for one source file.
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
        [command, ..] if command == OsStr::new("tokens") => {
            eprintln!("error: usage: zenith tokens <file.zen>");
            ExitCode::from(2)
        }
        [command, ..] if command == OsStr::new("ast") => {
            eprintln!("error: usage: zenith ast <file.zen>");
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
