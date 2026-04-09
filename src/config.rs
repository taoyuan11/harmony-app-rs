use std::env;
use std::path::{Path, PathBuf};

use crate::cli::CommonArgs;
use crate::errors::{HarmonyAppError, Result};
use crate::project::{ProfileConfig, ProjectInfo};
use crate::sdk::{
    HvigorInfo, SdkInfo, discover_hvigor, discover_sdk, target_to_abi_dir, target_to_rust_triple,
};

const DEFAULT_TARGET: &str = "arm64-v8a";

#[derive(Clone, Debug)]
pub struct AppContext {
    pub project: ProjectInfo,
    pub config: ResolvedConfig,
    pub sdk: SdkInfo,
    pub hvigor: HvigorInfo,
}

#[derive(Clone, Debug)]
pub struct ResolvedConfig {
    pub deveco_studio_dir: PathBuf,
    pub ohpm_path: PathBuf,
    pub sdk_root: PathBuf,
    pub sdk_version: Option<String>,
    pub version_name: String,
    pub version_code: u32,
    pub app_name: String,
    pub app_icon_path: Option<PathBuf>,
    pub start_icon_path: Option<PathBuf>,
    pub target: String,
    pub abi: String,
    pub profile_dir: String,
    pub output_dir: PathBuf,
    pub bundle_name: String,
    pub module_name: String,
}

impl AppContext {
    pub fn load(common: &CommonArgs, cwd: &Path) -> Result<Self> {
        let manifest_path = resolve_manifest_path(common, cwd)?;
        let project = ProjectInfo::load(&manifest_path)?;
        let profile_config = resolve_profile_config(&project, common.release);

        let target = if let Some(target) = common.target.clone() {
            target
        } else if let Some(target) = env_var_any(&["OHOS_APP_TARGET", "HARMONY_APP_TARGET"]) {
            target
        } else if let Some(target) = profile_config.target.clone() {
            target
        } else {
            DEFAULT_TARGET.to_string()
        };
        let abi = target_to_abi_dir(&target)?.to_string();
        let rust_target = target_to_rust_triple(&target)?.to_string();

        let profile_dir = env_var_any(&["OHOS_APP_PROFILE", "HARMONY_APP_PROFILE"])
            .or_else(|| profile_config.profile.clone())
            .unwrap_or_else(|| {
                if common.release {
                    "release".to_string()
                } else {
                    "debug".to_string()
                }
            });

        let output_dir = resolve_output_dir(
            common.out_dir.as_ref(),
            env_path_any(&["OHOS_APP_OUTPUT_DIR", "HARMONY_APP_OUTPUT_DIR"]).as_ref(),
            profile_config.output_dir.as_ref(),
            &project.project_dir,
        );

        let default_bundle_name = default_bundle_name(&project.package_name);
        let version_name = common
            .version_name
            .clone()
            .or_else(|| env_var_any(&["OHOS_APP_VERSION_NAME", "HARMONY_APP_VERSION_NAME"]))
            .or_else(|| profile_config.version_name.clone())
            .unwrap_or_else(|| project.package_version.clone());
        let version_code = common
            .version_code
            .or_else(|| {
                env_var_any(&["OHOS_APP_VERSION_CODE", "HARMONY_APP_VERSION_CODE"])
                    .and_then(|value| value.parse::<u32>().ok())
            })
            .or(profile_config.version_code)
            .unwrap_or(1_000_000);
        let app_name = common
            .app_name
            .clone()
            .or_else(|| env_var_any(&["OHOS_APP_NAME", "HARMONY_APP_NAME"]))
            .or_else(|| profile_config.app_name.clone())
            .unwrap_or_else(|| project.package_name.clone());
        let module_name = common
            .module_name
            .clone()
            .or_else(|| env_var_any(&["OHOS_APP_MODULE_NAME", "HARMONY_APP_MODULE_NAME"]))
            .or_else(|| profile_config.module_name.clone())
            .unwrap_or_else(|| "entry".to_string());
        let bundle_name = common
            .bundle_name
            .clone()
            .or_else(|| env_var_any(&["OHOS_APP_BUNDLE_NAME", "HARMONY_APP_BUNDLE_NAME"]))
            .or_else(|| profile_config.bundle_name.clone())
            .unwrap_or(default_bundle_name);
        let app_icon_path = common
            .app_icon_path
            .clone()
            .or_else(|| env_path_any(&["OHOS_APP_APP_ICON_PATH", "HARMONY_APP_APP_ICON_PATH"]))
            .or_else(|| profile_config.app_icon_path.clone())
            .map(|path| resolve_project_path(path, &project.project_dir));
        let start_icon_path = common
            .start_icon_path
            .clone()
            .or_else(|| env_path_any(&["OHOS_APP_START_ICON_PATH", "HARMONY_APP_START_ICON_PATH"]))
            .or_else(|| profile_config.start_icon_path.clone())
            .map(|path| resolve_project_path(path, &project.project_dir));

        let deveco_studio_dir = require_path_config(
            "deveco_studio_dir",
            "--deveco-studio-dir",
            "OHOS_APP_DEVECOSTUDIO_DIR / HARMONY_APP_DEVECOSTUDIO_DIR",
            common
                .deveco_studio_dir
                .clone()
                .or_else(|| {
                    env_path_any(&["OHOS_APP_DEVECOSTUDIO_DIR", "HARMONY_APP_DEVECOSTUDIO_DIR"])
                })
                .or_else(|| profile_config.deveco_studio_dir.clone()),
            &project,
        )?;
        let ohpm_path = require_path_config(
            "ohpm_path",
            "--ohpm-path",
            "OHOS_APP_OHPM_PATH / HARMONY_APP_OHPM_PATH",
            common
                .ohpm_path
                .clone()
                .or_else(|| env_path_any(&["OHOS_APP_OHPM_PATH", "HARMONY_APP_OHPM_PATH"]))
                .or_else(|| profile_config.ohpm_path.clone()),
            &project,
        )?;
        let sdk_root = require_path_config(
            "sdk_root",
            "--sdk-root",
            "OHOS_APP_SDK_ROOT / HARMONY_APP_SDK_ROOT",
            common
                .sdk_root
                .clone()
                .or_else(|| env_path_any(&["OHOS_APP_SDK_ROOT", "HARMONY_APP_SDK_ROOT"]))
                .or_else(|| profile_config.sdk_root.clone()),
            &project,
        )?;
        let sdk_version = common
            .sdk_version
            .clone()
            .or_else(|| env_var_any(&["OHOS_APP_SDK_VERSION", "HARMONY_APP_SDK_VERSION"]))
            .or(profile_config.sdk_version.clone());

        let sdk = discover_sdk(&sdk_root, sdk_version.as_deref())?;
        let hvigor = discover_hvigor(&deveco_studio_dir)?;

        Ok(Self {
            project,
            config: ResolvedConfig {
                deveco_studio_dir,
                ohpm_path,
                sdk_root,
                sdk_version,
                version_name,
                version_code,
                app_name,
                app_icon_path,
                start_icon_path,
                target: rust_target,
                abi,
                profile_dir,
                output_dir,
                bundle_name,
                module_name,
            },
            sdk,
            hvigor,
        })
    }
}

fn resolve_profile_config(project: &ProjectInfo, release: bool) -> ProfileConfig {
    let overlay = if release {
        &project.metadata_config.release
    } else {
        &project.metadata_config.debug
    };
    project.metadata_config.default.merged_with(overlay)
}

fn require_path_config(
    field: &'static str,
    cli_flag: &'static str,
    env_names: &'static str,
    value: Option<PathBuf>,
    project: &ProjectInfo,
) -> Result<PathBuf> {
    value.ok_or_else(|| HarmonyAppError::MissingRequiredConfig {
        field,
        cli_flag,
        env_names,
        manifest_path: project.manifest_path.clone(),
    })
}

fn resolve_manifest_path(common: &CommonArgs, cwd: &Path) -> Result<PathBuf> {
    let path = common
        .manifest_path
        .clone()
        .or_else(|| env_path_any(&["OHOS_APP_MANIFEST_PATH", "HARMONY_APP_MANIFEST_PATH"]))
        .unwrap_or_else(|| cwd.join("Cargo.toml"));
    if path.exists() {
        Ok(path)
    } else {
        Err(HarmonyAppError::MissingFile { path })
    }
}

fn resolve_output_dir(
    cli: Option<&PathBuf>,
    env: Option<&PathBuf>,
    file: Option<&PathBuf>,
    project_dir: &Path,
) -> PathBuf {
    let candidate = cli
        .cloned()
        .or_else(|| env.cloned())
        .or_else(|| file.cloned())
        .unwrap_or_else(|| PathBuf::from("ohos-app"));
    if candidate.is_absolute() {
        candidate
    } else {
        project_dir.join(candidate)
    }
}

fn resolve_project_path(path: PathBuf, project_dir: &Path) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        project_dir.join(path)
    }
}

fn env_var_any(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| env::var(name).ok())
}

fn env_path_any(names: &[&str]) -> Option<PathBuf> {
    names
        .iter()
        .find_map(|name| env::var_os(name).map(PathBuf::from))
}

fn default_bundle_name(package_name: &str) -> String {
    let normalized = package_name.replace(['-', '_'], "").to_ascii_lowercase();
    format!("com.example.{normalized}")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;

    use crate::cli::CommonArgs;
    use crate::config::AppContext;
    use crate::errors::OhosAppError;

    #[test]
    fn loads_default_metadata_configuration() {
        let temp = TempDir::new().unwrap();
        let sdk_root = temp.path().join("sdk");
        let deveco_dir = temp.path().join("DevEco Studio");
        create_sdk(&sdk_root);
        create_deveco(&deveco_dir);
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            &format!(
                r#"[package]
name = "demo-app"
version = "0.1.0"
edition = "2024"

[package.metadata.ohos-app.default]
deveco_studio_dir = "{deveco}"
ohpm_path = "{ohpm}"
sdk_root = "{sdk}"
sdk_version = "20"
version_name = "1.2.3"
version_code = 42
app_name = "Demo App"
bundle_name = "com.example.demo"

[lib]
crate-type = ["staticlib"]
"#,
                deveco = escape_toml_path(&deveco_dir),
                ohpm = escape_toml_path(&deveco_dir.join("tools/ohpm/bin/ohpm.bat")),
                sdk = escape_toml_path(&sdk_root),
            ),
        )
        .unwrap();
        fs::write(temp.path().join("src/lib.rs"), "pub fn marker() {}").unwrap();

        let app = AppContext::load(&CommonArgs::default(), temp.path()).unwrap();
        assert_eq!(app.config.sdk_version.as_deref(), Some("20"));
        assert_eq!(app.config.version_name, "1.2.3");
        assert_eq!(app.config.version_code, 42);
        assert_eq!(app.config.app_name, "Demo App");
        assert_eq!(app.config.bundle_name, "com.example.demo");
        assert_eq!(app.config.sdk_root, sdk_root);
        assert_eq!(app.config.target, "aarch64-unknown-linux-ohos");
    }

    #[test]
    fn release_profile_overrides_default_values() {
        let temp = TempDir::new().unwrap();
        let sdk_root = temp.path().join("sdk");
        let deveco_dir = temp.path().join("DevEco Studio");
        create_sdk(&sdk_root);
        create_deveco(&deveco_dir);
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            &format!(
                r#"[package]
name = "demo-app"
version = "0.1.0"
edition = "2024"

[package.metadata.ohos-app.default]
deveco_studio_dir = "{deveco}"
ohpm_path = "{ohpm}"
sdk_root = "{sdk}"
output_dir = "ohos-app"
profile = "debug"

[package.metadata.ohos-app.release]
output_dir = "ohos-app-release"
profile = "release-lto"

[lib]
crate-type = ["staticlib"]
"#,
                deveco = escape_toml_path(&deveco_dir),
                ohpm = escape_toml_path(&deveco_dir.join("tools/ohpm/bin/ohpm.bat")),
                sdk = escape_toml_path(&sdk_root),
            ),
        )
        .unwrap();
        fs::write(temp.path().join("src/lib.rs"), "pub fn marker() {}").unwrap();

        let app = AppContext::load(
            &CommonArgs {
                release: true,
                ..CommonArgs::default()
            },
            temp.path(),
        )
        .unwrap();
        assert_eq!(app.config.profile_dir, "release-lto");
        assert!(app.config.output_dir.ends_with("ohos-app-release"));
    }

    #[test]
    fn missing_required_paths_fail_fast() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"[package]
name = "demo-app"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["staticlib"]
"#,
        )
        .unwrap();
        fs::write(temp.path().join("src/lib.rs"), "pub fn marker() {}").unwrap();

        let error = AppContext::load(&CommonArgs::default(), temp.path()).unwrap_err();
        assert!(matches!(
            error,
            OhosAppError::MissingRequiredConfig {
                field: "deveco_studio_dir",
                ..
            }
        ));
    }

    #[test]
    fn resolves_relative_icon_paths_from_project_directory() {
        let temp = TempDir::new().unwrap();
        let sdk_root = temp.path().join("sdk");
        let deveco_dir = temp.path().join("DevEco Studio");
        create_sdk(&sdk_root);
        create_deveco(&deveco_dir);
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("app-icon.png"), [1_u8, 2, 3, 4]).unwrap();
        fs::write(temp.path().join("start-icon.png"), [5_u8, 6, 7, 8]).unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            &format!(
                r#"[package]
name = "demo-app"
version = "0.1.0"
edition = "2024"

[package.metadata.ohos-app.default]
deveco_studio_dir = "{deveco}"
ohpm_path = "{ohpm}"
sdk_root = "{sdk}"
app_icon_path = "app-icon.png"
start_icon_path = "start-icon.png"

[lib]
crate-type = ["staticlib"]
"#,
                deveco = escape_toml_path(&deveco_dir),
                ohpm = escape_toml_path(&deveco_dir.join("tools/ohpm/bin/ohpm.bat")),
                sdk = escape_toml_path(&sdk_root),
            ),
        )
        .unwrap();
        fs::write(temp.path().join("src/lib.rs"), "pub fn marker() {}").unwrap();

        let app = AppContext::load(&CommonArgs::default(), temp.path()).unwrap();
        assert!(
            app.config
                .app_icon_path
                .as_deref()
                .is_some_and(|path| path.ends_with("app-icon.png"))
        );
        assert!(
            app.config
                .start_icon_path
                .as_deref()
                .is_some_and(|path| path.ends_with("start-icon.png"))
        );
    }

    fn escape_toml_path(path: &std::path::Path) -> String {
        path.display().to_string().replace('\\', "\\\\")
    }

    fn create_sdk(root: &std::path::Path) {
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

    fn create_deveco(root: &std::path::Path) {
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
}
