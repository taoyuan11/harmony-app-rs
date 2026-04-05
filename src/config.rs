use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::cli::CommonArgs;
use crate::errors::{HarmonyAppError, Result};
use crate::project::ProjectInfo;
use crate::sdk::{
    HvigorInfo, SdkInfo, abi_to_target, discover_hvigor, discover_sdk, target_to_abi,
};

const DEFAULT_DEV_ECO_STUDIO_DIR: &str = r"D:\Apps\code\DevEco Studio";
const DEFAULT_OHPM_PATH: &str = r"D:\Apps\code\DevEco Studio\tools\ohpm\bin\ohpm.bat";
const DEFAULT_SDK_ROOT: &str = r"C:\Users\25422\AppData\Local\OpenHarmony\Sdk";
const DEFAULT_TARGET: &str = "aarch64-unknown-linux-ohos";

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
    pub target: String,
    pub abi: String,
    pub profile_dir: String,
    pub output_dir: PathBuf,
    pub bundle_name: String,
    pub module_name: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct FileConfig {
    deveco_studio_dir: Option<PathBuf>,
    ohpm_path: Option<PathBuf>,
    sdk_root: Option<PathBuf>,
    sdk_version: Option<String>,
    bundle_name: Option<String>,
    module_name: Option<String>,
    target: Option<String>,
    abi: Option<String>,
    profile: Option<String>,
    output_dir: Option<PathBuf>,
}

impl AppContext {
    pub fn load(common: &CommonArgs, cwd: &Path) -> Result<Self> {
        let manifest_path = resolve_manifest_path(common, cwd)?;
        let project = ProjectInfo::load(&manifest_path)?;
        let file_config = load_file_config(&project.project_dir)?;

        let target = if let Some(target) = common.target.clone() {
            target
        } else if let Some(abi) = common.abi {
            abi_to_target(abi.as_str())?.to_string()
        } else if let Some(target) = env_var_any(&["OHOS_APP_TARGET", "HARMONY_APP_TARGET"]) {
            target
        } else if let Some(abi) = env_var_any(&["OHOS_APP_ABI", "HARMONY_APP_ABI"]) {
            abi_to_target(&abi)?.to_string()
        } else if let Some(target) = file_config.target.clone() {
            target
        } else if let Some(abi) = file_config.abi.clone() {
            abi_to_target(&abi)?.to_string()
        } else {
            DEFAULT_TARGET.to_string()
        };
        let abi = target_to_abi(&target)?.to_string();

        let profile_dir = if common.release {
            "release".to_string()
        } else {
            file_config
                .profile
                .clone()
                .or_else(|| env_var_any(&["OHOS_APP_PROFILE", "HARMONY_APP_PROFILE"]))
                .unwrap_or_else(|| "debug".to_string())
        };

        let output_dir = resolve_output_dir(
            common.out_dir.as_ref(),
            env_path_any(&["OHOS_APP_OUTPUT_DIR", "HARMONY_APP_OUTPUT_DIR"]).as_ref(),
            file_config.output_dir.as_ref(),
            &project.project_dir,
        );

        let default_bundle_name = default_bundle_name(&project.package_name);
        let module_name = common
            .module_name
            .clone()
            .or_else(|| env_var_any(&["OHOS_APP_MODULE_NAME", "HARMONY_APP_MODULE_NAME"]))
            .or_else(|| file_config.module_name.clone())
            .unwrap_or_else(|| "entry".to_string());
        let bundle_name = common
            .bundle_name
            .clone()
            .or_else(|| env_var_any(&["OHOS_APP_BUNDLE_NAME", "HARMONY_APP_BUNDLE_NAME"]))
            .or_else(|| file_config.bundle_name.clone())
            .unwrap_or(default_bundle_name);

        let deveco_studio_dir = common
            .deveco_studio_dir
            .clone()
            .or_else(|| {
                env_path_any(&["OHOS_APP_DEVECOSTUDIO_DIR", "HARMONY_APP_DEVECOSTUDIO_DIR"])
            })
            .or(file_config.deveco_studio_dir.clone())
            .unwrap_or_else(|| PathBuf::from(DEFAULT_DEV_ECO_STUDIO_DIR));
        let ohpm_path = common
            .ohpm_path
            .clone()
            .or_else(|| env_path_any(&["OHOS_APP_OHPM_PATH", "HARMONY_APP_OHPM_PATH"]))
            .or(file_config.ohpm_path.clone())
            .unwrap_or_else(|| PathBuf::from(DEFAULT_OHPM_PATH));
        let sdk_root = common
            .sdk_root
            .clone()
            .or_else(|| env_path_any(&["OHOS_APP_SDK_ROOT", "HARMONY_APP_SDK_ROOT"]))
            .or(file_config.sdk_root.clone())
            .unwrap_or_else(|| PathBuf::from(DEFAULT_SDK_ROOT));
        let sdk_version = common
            .sdk_version
            .clone()
            .or_else(|| env_var_any(&["OHOS_APP_SDK_VERSION", "HARMONY_APP_SDK_VERSION"]))
            .or(file_config.sdk_version.clone());

        let sdk = discover_sdk(&sdk_root, sdk_version.as_deref())?;
        let hvigor = discover_hvigor(&deveco_studio_dir)?;

        Ok(Self {
            project,
            config: ResolvedConfig {
                deveco_studio_dir,
                ohpm_path,
                sdk_root,
                sdk_version,
                target,
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

fn load_file_config(project_dir: &Path) -> Result<FileConfig> {
    let mut path = project_dir.join("ohos-app.toml");
    if !path.exists() {
        path = project_dir.join("harmony-app.toml");
        if !path.exists() {
            return Ok(FileConfig::default());
        }
    }
    let text = fs::read_to_string(&path).map_err(|source| HarmonyAppError::ConfigRead {
        path: path.clone(),
        source,
    })?;
    toml::from_str(&text).map_err(|source| HarmonyAppError::ConfigParse { path, source })
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

    use super::load_file_config;

    #[test]
    fn reads_flat_configuration_file() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("ohos-app.toml"),
            r#"
deveco_studio_dir = "D:\\Apps\\code\\DevEco Studio"
sdk_root = "C:\\Users\\25422\\AppData\\Local\\OpenHarmony\\Sdk"
sdk_version = "20"
bundle_name = "com.example.demo"
module_name = "entry"
"#,
        )
        .unwrap();
        let file_config = load_file_config(temp.path()).unwrap();
        assert_eq!(file_config.sdk_version.as_deref(), Some("20"));
        assert_eq!(file_config.bundle_name.as_deref(), Some("com.example.demo"));
    }

    #[test]
    fn falls_back_to_legacy_configuration_file_name() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("harmony-app.toml"),
            r#"sdk_version = "20""#,
        )
        .unwrap();

        let file_config = load_file_config(temp.path()).unwrap();
        assert_eq!(file_config.sdk_version.as_deref(), Some("20"));
    }
}
