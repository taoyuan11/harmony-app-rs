use std::fs;
use std::io::Write;
use std::path::Path;

use crate::cli::{CommonArgs, InitCommand};
use crate::config::AppContext;
use crate::errors::{HarmonyAppError, Result};
use crate::runner::CommandRunner;
use crate::template::{template_context, write_shell_project};

pub fn run<R: CommandRunner, W: Write>(
    command: &InitCommand,
    cwd: &Path,
    _runner: &mut R,
    stdout: &mut W,
) -> Result<()> {
    ensure_library_manifest(&command.common, cwd)?;
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

pub(crate) fn ensure_library_manifest(common: &CommonArgs, cwd: &Path) -> Result<()> {
    let manifest_path = resolve_manifest_path(common, cwd)?;
    let manifest = fs::read_to_string(&manifest_path)
        .map_err(|source| HarmonyAppError::io(&manifest_path, source))?;
    let updated = ensure_crate_type_section(&manifest);
    if updated != manifest {
        fs::write(&manifest_path, updated).map_err(|source| HarmonyAppError::io(&manifest_path, source))?;
    }
    Ok(())
}

fn resolve_manifest_path(common: &CommonArgs, cwd: &Path) -> Result<std::path::PathBuf> {
    let path = common
        .manifest_path
        .clone()
        .or_else(|| std::env::var_os("OHOS_APP_MANIFEST_PATH").map(Into::into))
        .or_else(|| std::env::var_os("HARMONY_APP_MANIFEST_PATH").map(Into::into))
        .unwrap_or_else(|| cwd.join("Cargo.toml"));
    let path = if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    };
    if path.exists() {
        path.canonicalize()
            .map_err(|source| HarmonyAppError::io(&path, source))
    } else {
        Err(HarmonyAppError::MissingFile { path })
    }
}

fn ensure_crate_type_section(manifest: &str) -> String {
    let newline = if manifest.contains("\r\n") { "\r\n" } else { "\n" };
    let desired = r#"crate-type = ["staticlib", "rlib"]"#;
    let lines = lines_with_offsets(manifest);

    let lib_header = lines
        .iter()
        .position(|line| line.text.trim() == "[lib]");

    if let Some(lib_header_index) = lib_header {
        let section_end = lines
            .iter()
            .enumerate()
            .skip(lib_header_index + 1)
            .find(|(_, line)| is_table_header(line.text))
            .map(|(_, line)| line.start)
            .unwrap_or(manifest.len());

        if let Some(crate_type_line) = lines
            .iter()
            .enumerate()
            .skip(lib_header_index + 1)
            .take_while(|(_, line)| line.start < section_end)
            .find(|(_, line)| line.text.trim_start().starts_with("crate-type"))
            .map(|(_, line)| line)
        {
            let mut updated = String::with_capacity(manifest.len() + desired.len());
            updated.push_str(&manifest[..crate_type_line.start]);
            updated.push_str(desired);
            if crate_type_line.text.ends_with('\n') {
                updated.push_str(newline);
            }
            updated.push_str(&manifest[crate_type_line.end..]);
            return updated;
        }

        let insert_at = lines[lib_header_index].end;
        let mut updated = String::with_capacity(manifest.len() + desired.len() + newline.len());
        updated.push_str(&manifest[..insert_at]);
        if !lines[lib_header_index].text.ends_with('\n') {
            updated.push_str(newline);
        }
        updated.push_str(desired);
        updated.push_str(newline);
        updated.push_str(&manifest[insert_at..]);
        return updated;
    }

    let mut updated = manifest.to_string();
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push_str(newline);
    }
    if !updated.is_empty() {
        updated.push_str(newline);
    }
    updated.push_str("[lib]");
    updated.push_str(newline);
    updated.push_str(desired);
    updated.push_str(newline);
    updated
}

fn is_table_header(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed.len() >= 2
}

fn lines_with_offsets(text: &str) -> Vec<LineInfo<'_>> {
    let mut lines = Vec::new();
    let mut offset = 0;

    for line in text.split_inclusive('\n') {
        let end = offset + line.len();
        lines.push(LineInfo {
            start: offset,
            end,
            text: line,
        });
        offset = end;
    }

    if offset < text.len() {
        lines.push(LineInfo {
            start: offset,
            end: text.len(),
            text: &text[offset..],
        });
    }

    if lines.is_empty() {
        lines.push(LineInfo {
            start: 0,
            end: 0,
            text: "",
        });
    }

    lines
}

struct LineInfo<'a> {
    start: usize,
    end: usize,
    text: &'a str,
}

#[cfg(test)]
mod tests {
    use super::ensure_crate_type_section;

    #[test]
    fn appends_lib_section_when_missing() {
        let manifest = r#"[package]
name = "demo"
version = "0.1.0"
"#;

        let updated = ensure_crate_type_section(manifest);

        assert!(updated.contains("[lib]\ncrate-type = [\"staticlib\", \"rlib\"]\n"));
    }

    #[test]
    fn inserts_crate_type_into_existing_lib_section() {
        let manifest = r#"[package]
name = "demo"

[lib]
name = "demo"
"#;

        let updated = ensure_crate_type_section(manifest);

        assert!(updated.contains("[lib]\ncrate-type = [\"staticlib\", \"rlib\"]\nname = \"demo\"\n"));
    }

    #[test]
    fn replaces_existing_crate_type_line() {
        let manifest = r#"[package]
name = "demo"

[lib]
crate-type = ["cdylib", "staticlib"]
name = "demo"
"#;

        let updated = ensure_crate_type_section(manifest);

        assert!(updated.contains("crate-type = [\"staticlib\", \"rlib\"]"));
        assert!(!updated.contains("cdylib"));
    }
}
