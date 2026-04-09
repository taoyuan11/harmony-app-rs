use std::path::{Path, PathBuf};

use cargo_metadata::{Metadata, MetadataCommand, Package, Target};
use serde::Deserialize;

use crate::errors::{HarmonyAppError, Result};

#[derive(Clone, Debug)]
pub struct ProjectInfo {
    pub manifest_path: PathBuf,
    pub project_dir: PathBuf,
    pub package_name: String,
    pub package_version: String,
    pub lib_name: String,
    pub target_dir: PathBuf,
    pub uses_winit_ohos: bool,
    pub metadata_config: MetadataConfig,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct MetadataConfig {
    #[serde(default)]
    pub default: ProfileConfig,
    #[serde(default)]
    pub debug: ProfileConfig,
    #[serde(default)]
    pub release: ProfileConfig,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct ProfileConfig {
    pub deveco_studio_dir: Option<PathBuf>,
    pub ohpm_path: Option<PathBuf>,
    pub sdk_root: Option<PathBuf>,
    pub sdk_version: Option<String>,
    pub version_name: Option<String>,
    pub version_code: Option<u32>,
    pub app_name: Option<String>,
    pub app_icon_path: Option<PathBuf>,
    pub start_icon_path: Option<PathBuf>,
    pub bundle_name: Option<String>,
    pub module_name: Option<String>,
    pub target: Option<String>,
    pub profile: Option<String>,
    pub output_dir: Option<PathBuf>,
}

impl ProfileConfig {
    pub fn merged_with(&self, overlay: &ProfileConfig) -> ProfileConfig {
        ProfileConfig {
            deveco_studio_dir: overlay
                .deveco_studio_dir
                .clone()
                .or_else(|| self.deveco_studio_dir.clone()),
            ohpm_path: overlay.ohpm_path.clone().or_else(|| self.ohpm_path.clone()),
            sdk_root: overlay.sdk_root.clone().or_else(|| self.sdk_root.clone()),
            sdk_version: overlay
                .sdk_version
                .clone()
                .or_else(|| self.sdk_version.clone()),
            version_name: overlay
                .version_name
                .clone()
                .or_else(|| self.version_name.clone()),
            version_code: overlay.version_code.or(self.version_code),
            app_name: overlay.app_name.clone().or_else(|| self.app_name.clone()),
            app_icon_path: overlay
                .app_icon_path
                .clone()
                .or_else(|| self.app_icon_path.clone()),
            start_icon_path: overlay
                .start_icon_path
                .clone()
                .or_else(|| self.start_icon_path.clone()),
            bundle_name: overlay
                .bundle_name
                .clone()
                .or_else(|| self.bundle_name.clone()),
            module_name: overlay
                .module_name
                .clone()
                .or_else(|| self.module_name.clone()),
            target: overlay.target.clone().or_else(|| self.target.clone()),
            profile: overlay.profile.clone().or_else(|| self.profile.clone()),
            output_dir: overlay
                .output_dir
                .clone()
                .or_else(|| self.output_dir.clone()),
        }
    }
}

impl ProjectInfo {
    pub fn load(manifest_path: &Path) -> Result<Self> {
        let canonical_manifest = manifest_path
            .canonicalize()
            .map_err(|source| HarmonyAppError::io(manifest_path, source))?;
        let metadata = MetadataCommand::new()
            .manifest_path(&canonical_manifest)
            .exec()?;
        let package = find_package(&metadata, &canonical_manifest).ok_or_else(|| {
            HarmonyAppError::message(format!(
                "could not locate package metadata for manifest [{}]",
                canonical_manifest.display()
            ))
        })?;
        let library =
            find_library_target(package).ok_or_else(|| HarmonyAppError::MissingLibraryTarget {
                manifest_path: canonical_manifest.clone(),
            })?;

        let project_dir = canonical_manifest
            .parent()
            .ok_or_else(|| HarmonyAppError::message("manifest path has no parent directory"))?
            .to_path_buf();
        let uses_winit_ohos = package
            .dependencies
            .iter()
            .any(|dependency| dependency.name == "tgui-winit-ohos");
        let metadata_config = load_metadata_config(package, &canonical_manifest)?;

        Ok(Self {
            manifest_path: canonical_manifest,
            project_dir,
            package_name: package.name.to_string(),
            package_version: package.version.to_string(),
            lib_name: library.name.replace('-', "_"),
            target_dir: metadata.target_directory.into_std_path_buf(),
            uses_winit_ohos,
            metadata_config,
        })
    }

    pub fn static_artifact_path(&self, target: &str, profile_dir: &str) -> PathBuf {
        self.target_dir
            .join(target)
            .join(profile_dir)
            .join(format!("lib{}.a", self.lib_name))
    }
}

fn find_package<'a>(metadata: &'a Metadata, manifest_path: &Path) -> Option<&'a Package> {
    metadata.packages.iter().find(|package| {
        package
            .manifest_path
            .as_std_path()
            .canonicalize()
            .ok()
            .as_deref()
            == Some(manifest_path)
    })
}

fn find_library_target(package: &Package) -> Option<&Target> {
    package.targets.iter().find(|target| {
        target
            .kind
            .iter()
            .any(|kind| matches!(kind.to_string().as_str(), "lib" | "cdylib" | "staticlib"))
            || target
                .crate_types
                .iter()
                .any(|kind| matches!(kind.to_string().as_str(), "cdylib" | "staticlib"))
    })
}

fn load_metadata_config(package: &Package, manifest_path: &Path) -> Result<MetadataConfig> {
    if package.metadata.is_null() {
        return Ok(MetadataConfig::default());
    }

    let metadata = package
        .metadata
        .get("ohos-app")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    if metadata.is_null() {
        return Ok(MetadataConfig::default());
    }

    serde_json::from_value(metadata).map_err(|source| HarmonyAppError::ConfigParse {
        manifest_path: manifest_path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use super::ProjectInfo;

    #[test]
    fn loads_library_project_info() {
        let temp = TempDir::new().unwrap();
        let manifest = temp.path().join("Cargo.toml");
        fs::write(
            &manifest,
            r#"[package]
name = "demo-app"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["staticlib"]
"#,
        )
        .unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/lib.rs"), "pub fn marker() {}").unwrap();

        let info = ProjectInfo::load(&manifest).unwrap();
        assert_eq!(info.package_name, "demo-app");
        assert_eq!(info.package_version, "0.1.0");
        assert_eq!(info.lib_name, "demo_app");
        assert!(!info.uses_winit_ohos);
    }

    #[test]
    fn detects_tgui_winit_ohos_dependency() {
        let temp = TempDir::new().unwrap();
        let manifest = temp.path().join("Cargo.toml");
        fs::write(
            &manifest,
            r#"[package]
name = "demo-app"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["staticlib"]

[dependencies]
tgui-winit-ohos = "0.0.1"
"#,
        )
        .unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/lib.rs"), "pub fn marker() {}").unwrap();

        let info = ProjectInfo::load(&manifest).unwrap();
        assert!(info.uses_winit_ohos);
    }

    #[test]
    fn reads_package_metadata_configuration() {
        let temp = TempDir::new().unwrap();
        let manifest = temp.path().join("Cargo.toml");
        fs::write(
            &manifest,
            r#"[package]
name = "demo-app"
version = "0.1.0"
edition = "2024"

[package.metadata.ohos-app.default]
bundle_name = "com.example.demo"
output_dir = "ohos-app"

[package.metadata.ohos-app.release]
output_dir = "ohos-app-release"

[lib]
crate-type = ["staticlib"]
"#,
        )
        .unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/lib.rs"), "pub fn marker() {}").unwrap();

        let info = ProjectInfo::load(&manifest).unwrap();
        assert_eq!(
            info.metadata_config.default.bundle_name.as_deref(),
            Some("com.example.demo")
        );
        assert_eq!(
            info.metadata_config.release.output_dir.as_deref(),
            Some(Path::new("ohos-app-release"))
        );
    }
}
