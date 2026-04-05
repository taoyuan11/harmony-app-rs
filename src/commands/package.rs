use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::cli::{InitCommand, PackageArtifact, PackageCommand};
use crate::commands::build::build_plan;
use crate::commands::command_env;
use crate::config::AppContext;
use crate::errors::{HarmonyAppError, Result};
use crate::runner::{CommandRunner, CommandSpec};
use crate::template::write_shell_project;

pub fn run<R: CommandRunner, W: Write>(
    command: &PackageCommand,
    cwd: &Path,
    runner: &mut R,
    stdout: &mut W,
) -> Result<()> {
    let app = AppContext::load(&command.common, cwd)?;
    let build_plan = build_plan(&app);
    let install_command = CommandSpec {
        program: app.config.ohpm_path.clone(),
        args: vec!["install".to_string()],
        cwd: app.config.output_dir.clone(),
        env: command_env(&app),
    };
    let hvigor_command = CommandSpec {
        program: app.hvigor.wrapper_bat.clone(),
        args: vec![
            "clean".to_string(),
            hvigor_task(command.artifact).to_string(),
            "--no-daemon".to_string(),
        ],
        cwd: app.config.output_dir.clone(),
        env: command_env(&app),
    };

    if command.common.dry_run {
        if !command.skip_init {
            writeln!(
                stdout,
                "[dry-run] {}",
                describe_init(
                    &InitCommand {
                        common: command.common.clone()
                    },
                    &app
                )
            )?;
        }
        if !command.skip_rust_build {
            writeln!(stdout, "[dry-run] {}", build_plan.command.display())?;
            writeln!(
                stdout,
                "[dry-run] copy {} -> {}",
                build_plan.source.display(),
                build_plan.destination.display()
            )?;
        }
        writeln!(stdout, "[dry-run] {}", install_command.display())?;
        writeln!(stdout, "[dry-run] {}", hvigor_command.display())?;
        return Ok(());
    }

    if !command.skip_init {
        write_shell_project(&app)?;
    }
    if !command.skip_rust_build {
        runner.run(&build_plan.command)?;
        if !build_plan.source.exists() {
            return Err(HarmonyAppError::MissingFile {
                path: build_plan.source.clone(),
            });
        }
        if build_plan.legacy_shared_destination.exists() {
            fs::remove_file(&build_plan.legacy_shared_destination).map_err(|source| {
                HarmonyAppError::io(&build_plan.legacy_shared_destination, source)
            })?;
        }
        if let Some(parent) = build_plan.destination.parent() {
            fs::create_dir_all(parent).map_err(|source| HarmonyAppError::io(parent, source))?;
        }
        fs::copy(&build_plan.source, &build_plan.destination)
            .map_err(|source| HarmonyAppError::io(&build_plan.destination, source))?;
    }

    runner.run(&install_command).or_else(|_error| {
        let fallback = CommandSpec {
            program: PathBuf::from("npm.cmd"),
            args: vec!["install".to_string()],
            cwd: app.config.output_dir.clone(),
            env: command_env(&app),
        };
        runner.run(&fallback)
    })?;
    runner.run(&hvigor_command)?;

    let artifact = find_artifact(&app.config.output_dir, command.artifact).ok_or_else(|| {
        HarmonyAppError::PackageArtifactNotFound {
            search_root: app.config.output_dir.clone(),
        }
    })?;

    writeln!(
        stdout,
        "Packaged OHOS {} at {}",
        artifact_label(command.artifact),
        artifact.display()
    )?;
    Ok(())
}

fn describe_init(_command: &InitCommand, app: &AppContext) -> String {
    format!("generate OHOS shell at {}", app.config.output_dir.display())
}

fn find_artifact(root: &Path, artifact: PackageArtifact) -> Option<PathBuf> {
    let extension = artifact_extension(artifact);
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
                return Some(path);
            }
        }
    }
    None
}

fn hvigor_task(artifact: PackageArtifact) -> &'static str {
    match artifact {
        PackageArtifact::App => "assembleApp",
        PackageArtifact::Hap => "assembleHap",
    }
}

fn artifact_extension(artifact: PackageArtifact) -> &'static str {
    match artifact {
        PackageArtifact::App => "app",
        PackageArtifact::Hap => "hap",
    }
}

fn artifact_label(artifact: PackageArtifact) -> &'static str {
    match artifact {
        PackageArtifact::App => ".app",
        PackageArtifact::Hap => ".hap",
    }
}
