mod cli;
mod commands;
mod config;
mod errors;
mod project;
pub mod runner;
mod sdk;
mod template;

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::Path;

use clap::Parser;
use cli::{Cli, Commands};
pub use errors::{HarmonyAppError, OhosAppError, Result};
use runner::RealCommandRunner;

pub fn main_entry() -> Result<()> {
    let args = std::env::args_os().collect::<Vec<_>>();
    let cwd = std::env::current_dir().map_err(HarmonyAppError::from)?;
    let mut runner = RealCommandRunner;
    let mut stdout = io::stdout().lock();
    run_with(args, &cwd, &mut runner, &mut stdout)
}

pub fn run_with<I, S, R, W>(args: I, cwd: &Path, runner: &mut R, stdout: &mut W) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
    R: runner::CommandRunner,
    W: Write,
{
    let cli = Cli::parse_from(normalize_args(args));
    match &cli.command {
        Commands::Init(command) => commands::init::run(command, cwd, runner, stdout),
        Commands::Build(command) => commands::build::run(command, cwd, runner, stdout),
        Commands::Package(command) => commands::package::run(command, cwd, runner, stdout),
    }
}

fn normalize_args<I, S>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    if matches!(
        args.get(1).and_then(|value| value.to_str()),
        Some("ohos-app" | "harmony-app")
    ) {
        args.remove(1);
    }
    args
}

#[cfg(test)]
mod tests {
    use super::normalize_args;
    use std::ffi::OsString;

    #[test]
    fn strips_cargo_forwarded_subcommand_name() {
        for subcommand in ["ohos-app", "harmony-app"] {
            let args = normalize_args([
                OsString::from("cargo-ohos-app"),
                OsString::from(subcommand),
                OsString::from("init"),
            ]);
            let rendered = args
                .iter()
                .map(|value| value.to_string_lossy().to_string())
                .collect::<Vec<_>>();
            assert_eq!(rendered, vec!["cargo-ohos-app", "init"]);
        }
    }
}
