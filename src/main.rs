use std::env;
use std::process::ExitCode;

const HELP: &str = "\
Zenith

A safe, bulk-first language that compiles to Salesforce Apex.

Usage:
  zenith [--help]
  zenith --version

Compiler commands begin with roadmap milestone M1.
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
        [command, ..] => {
            eprintln!(
                "error: unknown or unavailable command `{}`",
                command.to_string_lossy()
            );
            eprintln!("Run `zenith --help` for the bootstrap command surface.");
            ExitCode::from(2)
        }
    }
}
