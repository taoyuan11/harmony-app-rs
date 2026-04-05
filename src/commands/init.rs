use std::io::Write;
use std::path::Path;

use crate::cli::InitCommand;
use crate::config::AppContext;
use crate::errors::Result;
use crate::runner::CommandRunner;
use crate::template::{template_context, write_shell_project};

pub fn run<R: CommandRunner, W: Write>(
    command: &InitCommand,
    cwd: &Path,
    _runner: &mut R,
    stdout: &mut W,
) -> Result<()> {
    let app = AppContext::load(&command.common, cwd)?;
    let context = template_context(&app);

    if command.common.dry_run {
        writeln!(
            stdout,
            "[dry-run] would generate OHOS shell at {} for bundle {}",
            app.config.output_dir.display(),
            context.bundle_name
        )?;
        return Ok(());
    }

    write_shell_project(&app)?;
    writeln!(
        stdout,
        "Generated OHOS shell at {} using SDK {} ({})",
        app.config.output_dir.display(),
        app.config.sdk_root.display(),
        app.config
            .sdk_version
            .as_deref()
            .unwrap_or(app.sdk.version.as_str())
    )?;
    Ok(())
}
