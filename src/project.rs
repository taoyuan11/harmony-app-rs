use std::path::{Path, PathBuf};

use cargo_metadata::{Metadata, MetadataCommand, Package, Target};

use crate::errors::{HarmonyAppError, Result};

#[derive(Clone, Debug)]
pub struct ProjectInfo {
    pub manifest_path: PathBuf,
    pub project_dir: PathBuf,
    pub package_name: String,
    pub lib_name: String,
    pub target_dir: PathBuf,
    pub uses_winit_ohos: bool,
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
            .any(|dependency| dependency.name == "winit-ohos");

        Ok(Self {
            manifest_path: canonical_manifest,
            project_dir,
            package_name: package.name.to_string(),
            lib_name: library.name.replace('-', "_"),
            target_dir: metadata.target_directory.into_std_path_buf(),
            uses_winit_ohos,
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

#[cfg(test)]
mod tests {
    use std::fs;

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
        assert_eq!(info.lib_name, "demo_app");
        assert!(!info.uses_winit_ohos);
    }
}
