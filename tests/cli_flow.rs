use std::fs;
use std::path::{Path, PathBuf};

use ohos_app::{HarmonyAppError, OhosAppError, Result, run_with};
use tempfile::TempDir;

use ohos_app::runner::{CommandRunner, CommandSpec};

#[derive(Default)]
struct RecordingRunner {
    commands: Vec<CommandSpec>,
    fail_program_suffixes: Vec<String>,
}

impl CommandRunner for RecordingRunner {
    fn run(&mut self, spec: &CommandSpec) -> Result<()> {
        self.commands.push(spec.clone());
        for suffix in &self.fail_program_suffixes {
            if spec.program.to_string_lossy().ends_with(suffix) {
                return Err(HarmonyAppError::CommandFailed {
                    program: spec.program.display().to_string(),
                    cwd: spec.cwd.clone(),
                    code: Some(1),
                });
            }
        }
        Ok(())
    }
}

#[test]
fn init_generates_ohos_shell_structure() {
    let fixture = TestFixture::new();
    let mut runner = RecordingRunner::default();
    let mut stdout = Vec::new();

    run_with(
        fixture.common_args(["init"]),
        fixture.project_dir.path(),
        &mut runner,
        &mut stdout,
    )
    .unwrap();

    assert!(fixture.output_dir().join("AppScope/app.json5").exists());
    assert!(
        fixture
            .output_dir()
            .join("entry/src/main/module.json5")
            .exists()
    );
    assert!(fixture.output_dir().join("hvigorw.bat").exists());
}

#[test]
fn init_adds_staticlib_and_rlib_to_manifest() {
    let fixture = TestFixture::new_without_lib_section();
    let mut runner = RecordingRunner::default();
    let mut stdout = Vec::new();

    run_with(
        fixture.common_args(["init"]),
        fixture.project_dir.path(),
        &mut runner,
        &mut stdout,
    )
    .unwrap();

    let manifest = fs::read_to_string(fixture.project_dir.path().join("Cargo.toml")).unwrap();
    assert!(manifest.contains("[lib]"));
    assert!(manifest.contains(r#"crate-type = ["staticlib", "rlib"]"#));
}

#[test]
fn init_updates_manifest_when_manifest_path_is_relative() {
    let fixture = TestFixture::new_without_lib_section();
    let mut runner = RecordingRunner::default();
    let mut stdout = Vec::new();

    run_with(
        vec![
            "cargo-ohos-app".to_string(),
            "init".to_string(),
            "--manifest-path".to_string(),
            "Cargo.toml".to_string(),
            "--sdk-root".to_string(),
            fixture.sdk_root.display().to_string(),
            "--deveco-studio-dir".to_string(),
            fixture.deveco_dir.display().to_string(),
            "--ohpm-path".to_string(),
            fixture.ohpm_path.display().to_string(),
        ],
        fixture.project_dir.path(),
        &mut runner,
        &mut stdout,
    )
    .unwrap();

    let manifest = fs::read_to_string(fixture.project_dir.path().join("Cargo.toml")).unwrap();
    assert!(manifest.contains(r#"crate-type = ["staticlib", "rlib"]"#));
}

#[test]
fn build_dry_run_prints_cargo_and_copy_plan() {
    let fixture = TestFixture::new();
    let mut runner = RecordingRunner::default();
    let mut stdout = Vec::new();

    run_with(
        fixture.common_args(["build", "--dry-run"]),
        fixture.project_dir.path(),
        &mut runner,
        &mut stdout,
    )
    .unwrap();

    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("cargo rustc --lib"));
    assert!(output.contains("copy"));
    assert!(output.contains("libcounter_native.a"));
}

#[test]
fn package_dry_run_lists_full_sequence() {
    let fixture = TestFixture::new();
    let mut runner = RecordingRunner::default();
    let mut stdout = Vec::new();

    run_with(
        fixture.common_args(["package", "--dry-run"]),
        fixture.project_dir.path(),
        &mut runner,
        &mut stdout,
    )
    .unwrap();

    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("generate OHOS shell"));
    assert!(output.contains("cargo rustc --lib"));
    assert!(output.contains("ohpm.bat install"));
    assert!(output.contains("hvigorw.bat clean assembleHap --no-daemon"));
}

#[test]
fn package_app_dry_run_uses_app_task() {
    let fixture = TestFixture::new();
    let mut runner = RecordingRunner::default();
    let mut stdout = Vec::new();

    run_with(
        fixture.common_args(["package", "--artifact", "app", "--dry-run"]),
        fixture.project_dir.path(),
        &mut runner,
        &mut stdout,
    )
    .unwrap();

    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("hvigorw.bat clean assembleApp --no-daemon"));
}

#[test]
fn package_x86_64_dry_run_switches_target_and_output_dir() {
    let fixture = TestFixture::new();
    let mut runner = RecordingRunner::default();
    let mut stdout = Vec::new();

    run_with(
        fixture.common_args(["package", "--target", "x86_64", "--dry-run"]),
        fixture.project_dir.path(),
        &mut runner,
        &mut stdout,
    )
    .unwrap();

    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("--target x86_64-unknown-linux-ohos"));
    assert!(output.contains("cpp\\libs\\x86_64"));
}

#[test]
fn init_fails_fast_when_required_paths_are_missing() {
    let fixture = TestFixture::new();
    let mut runner = RecordingRunner::default();
    let mut stdout = Vec::new();

    let error = run_with(
        vec!["cargo-ohos-app".to_string(), "init".to_string()],
        fixture.project_dir.path(),
        &mut runner,
        &mut stdout,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        OhosAppError::MissingRequiredConfig {
            field: "deveco_studio_dir",
            ..
        }
    ));
}

#[test]
fn build_dry_run_uses_release_metadata_overrides() {
    let fixture = TestFixture::new();
    fixture.write_release_profile_metadata();
    let mut runner = RecordingRunner::default();
    let mut stdout = Vec::new();

    run_with(
        fixture.common_args(["build", "--dry-run", "--release"]),
        fixture.project_dir.path(),
        &mut runner,
        &mut stdout,
    )
    .unwrap();

    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("--release"));
    assert!(output.contains("ohos-app-release"));
}

#[test]
fn package_surfaces_ohpm_failure() {
    let fixture = TestFixture::new();
    fixture.seed_built_library();

    let mut runner = RecordingRunner {
        commands: Vec::new(),
        fail_program_suffixes: vec!["ohpm.bat".to_string(), "npm.cmd".to_string()],
    };
    let mut stdout = Vec::new();

    let error = run_with(
        fixture.common_args(["package"]),
        fixture.project_dir.path(),
        &mut runner,
        &mut stdout,
    )
    .unwrap_err();

    assert!(matches!(error, OhosAppError::CommandFailed { .. }));
}

struct TestFixture {
    _temp: TempDir,
    project_dir: TempDir,
    sdk_root: PathBuf,
    deveco_dir: PathBuf,
    ohpm_path: PathBuf,
}

impl TestFixture {
    fn new() -> Self {
        let temp = TempDir::new().unwrap();
        let project_dir = TempDir::new_in(temp.path()).unwrap();
        let sdk_root = temp.path().join("sdk");
        let deveco_dir = temp.path().join("DevEco Studio");
        let ohpm_path = deveco_dir.join("tools/ohpm/bin/ohpm.bat");

        create_project(project_dir.path());
        create_sdk(&sdk_root);
        create_deveco(&deveco_dir);

        Self {
            _temp: temp,
            project_dir,
            sdk_root,
            deveco_dir,
            ohpm_path,
        }
    }

    fn new_without_lib_section() -> Self {
        let fixture = Self::new();
        fs::write(
            fixture.project_dir.path().join("Cargo.toml"),
            r#"[package]
name = "counter-native"
version = "0.1.0"
edition = "2024"

[package.metadata.ohos-app.default]
bundle_name = "com.example.counternative"
module_name = "entry"
target = "arm64-v8a"
output_dir = "ohos-app"
"#,
        )
        .unwrap();
        fixture
    }

    fn common_args<const N: usize>(&self, tail: [&str; N]) -> Vec<String> {
        let mut args = vec!["cargo-ohos-app".to_string()];
        args.extend(tail.into_iter().map(ToString::to_string));
        args.push("--manifest-path".to_string());
        args.push(
            self.project_dir
                .path()
                .join("Cargo.toml")
                .display()
                .to_string(),
        );
        args.push("--sdk-root".to_string());
        args.push(self.sdk_root.display().to_string());
        args.push("--deveco-studio-dir".to_string());
        args.push(self.deveco_dir.display().to_string());
        args.push("--ohpm-path".to_string());
        args.push(self.ohpm_path.display().to_string());
        args
    }

    fn output_dir(&self) -> PathBuf {
        self.project_dir.path().join("ohos-app")
    }

    fn seed_built_library(&self) {
        let artifact = self
            .project_dir
            .path()
            .join("target")
            .join("aarch64-unknown-linux-ohos")
            .join("debug")
            .join("libcounter_native.a");
        if let Some(parent) = artifact.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(artifact, b"fake so").unwrap();
    }

    fn write_release_profile_metadata(&self) {
        fs::write(
            self.project_dir.path().join("Cargo.toml"),
            r#"[package]
name = "counter-native"
version = "0.1.0"
edition = "2024"

[package.metadata.ohos-app.default]
bundle_name = "com.example.counternative"
module_name = "entry"
target = "arm64-v8a"
output_dir = "ohos-app"

[package.metadata.ohos-app.release]
output_dir = "ohos-app-release"

[lib]
crate-type = ["cdylib", "staticlib"]
"#,
        )
        .unwrap();
    }
}

fn create_project(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "counter-native"
version = "0.1.0"
edition = "2024"

[package.metadata.ohos-app.default]
bundle_name = "com.example.counternative"
module_name = "entry"
target = "arm64-v8a"
output_dir = "ohos-app"

[lib]
crate-type = ["cdylib", "staticlib"]
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/lib.rs"),
        r#"use std::ffi::c_char;

static MESSAGE: &[u8] = b"Hello!\0";

#[unsafe(no_mangle)]
        pub extern "C" fn ohos_app_get_message() -> *const c_char {
    MESSAGE.as_ptr().cast()
}

#[unsafe(no_mangle)]
        pub extern "C" fn ohos_app_increment_counter() -> u32 {
    1
}
"#,
    )
    .unwrap();
}

fn create_sdk(root: &Path) {
    let ets_dir = root.join("20/ets");
    let toolchains_dir = root.join("20/toolchains");
    let native_dir = root.join("20/native");
    fs::create_dir_all(&ets_dir).unwrap();
    fs::create_dir_all(&toolchains_dir).unwrap();
    fs::create_dir_all(&native_dir).unwrap();
    fs::write(
        ets_dir.join("oh-uni-package.json"),
        r#"{"apiVersion":"20","version":"6.0.0.47"}"#,
    )
    .unwrap();
}

fn create_deveco(root: &Path) {
    let wrapper_dir = root.join("tools/hvigor/bin");
    let hvigor_dir = root.join("tools/hvigor/hvigor");
    let plugin_dir = root.join("tools/hvigor/hvigor-ohos-plugin");
    let ohpm_dir = root.join("tools/ohpm/bin");
    fs::create_dir_all(&wrapper_dir).unwrap();
    fs::create_dir_all(&hvigor_dir).unwrap();
    fs::create_dir_all(&plugin_dir).unwrap();
    fs::create_dir_all(&ohpm_dir).unwrap();
    fs::write(wrapper_dir.join("hvigorw.bat"), "@echo off\r\n").unwrap();
    fs::write(wrapper_dir.join("hvigorw.js"), "console.log('hvigor');\n").unwrap();
    fs::write(
        hvigor_dir.join("package.json"),
        r#"{"name":"@ohos/hvigor"}"#,
    )
    .unwrap();
    fs::write(
        plugin_dir.join("package.json"),
        r#"{"name":"@ohos/hvigor-ohos-plugin"}"#,
    )
    .unwrap();
    fs::write(ohpm_dir.join("ohpm.bat"), "@echo off\r\n").unwrap();
}
