use std::{
    io::{self, Write},
    process::ExitCode,
};

use mizar_driver::cli::{CliExitCode, CliInvocation};
use mizar_lsp::Server;

fn main() -> ExitCode {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() == 1 && args[0] == "lsp" {
        return match Server::default().serve(&mut io::stdin().lock(), &mut io::stdout().lock()) {
            Ok(code) => ExitCode::from(code as u8),
            Err(error) => {
                let _ = writeln!(io::stderr().lock(), "mizar: {error}");
                ExitCode::FAILURE
            }
        };
    }
    let (code, reason) = match args
        .into_iter()
        .map(|arg| arg.into_string())
        .collect::<Result<Vec<_>, _>>()
    {
        Err(_) => (CliExitCode::Usage, "non-Unicode argument".to_owned()),
        Ok(args) => match CliInvocation::parse(args) {
            Err(error) => (CliExitCode::Usage, error.reason),
            Ok(_) => (
                CliExitCode::UnavailableOwner,
                "filesystem build bootstrap is unavailable".to_owned(),
            ),
        },
    };
    let _ = writeln!(io::stderr().lock(), "mizar: {reason}");
    ExitCode::from(code.process_code() as u8)
}
