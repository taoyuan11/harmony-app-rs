use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::errors::{HarmonyAppError, Result};

#[derive(Clone, Debug)]
pub struct SdkInfo {
    pub root: PathBuf,
    pub version: String,
    pub version_dir: PathBuf,
    pub display_version: String,
    pub native_dir: PathBuf,
    pub toolchains_dir: PathBuf,
}

#[derive(Clone, Debug)]
pub struct HvigorInfo {
    pub wrapper_bat: PathBuf,
    pub wrapper_js: PathBuf,
    pub hvigor_package_dir: PathBuf,
    pub hvigor_plugin_package_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct UniPackage {
    #[serde(rename = "apiVersion")]
    api_version: String,
    version: String,
}

pub fn discover_sdk(root: &Path, requested: Option<&str>) -> Result<SdkInfo> {
    if !root.exists() {
        return Err(HarmonyAppError::MissingSdkRoot {
            path: root.to_path_buf(),
        });
    }

    let version = match requested {
        Some(value) if !value.eq_ignore_ascii_case("auto") => value.to_string(),
        _ => discover_latest_sdk_version(root)?,
    };

    let version_dir = root.join(&version);
    if !version_dir.exists() {
        return Err(HarmonyAppError::MissingSdkVersion { path: version_dir });
    }

    let manifest_path = version_dir.join("ets").join("oh-uni-package.json");
    let manifest_text = fs::read_to_string(&manifest_path)
        .map_err(|source| HarmonyAppError::io(&manifest_path, source))?;
    let manifest: UniPackage = serde_json::from_str(&manifest_text).map_err(|error| {
        HarmonyAppError::message(format!(
            "failed to parse SDK manifest [{}]: {error}",
            manifest_path.display()
        ))
    })?;
    let display_parts = manifest.version.split('.').take(3).collect::<Vec<_>>();
    let display_version = format!("{}({})", display_parts.join("."), manifest.api_version);

    Ok(SdkInfo {
        root: root.to_path_buf(),
        version,
        version_dir: version_dir.clone(),
        display_version,
        native_dir: version_dir.join("native"),
        toolchains_dir: version_dir.join("toolchains"),
    })
}

pub fn discover_hvigor(deveco_studio_dir: &Path) -> Result<HvigorInfo> {
    let wrapper_bat = deveco_studio_dir
        .join("tools")
        .join("hvigor")
        .join("bin")
        .join("hvigorw.bat");
    let wrapper_js = deveco_studio_dir
        .join("tools")
        .join("hvigor")
        .join("bin")
        .join("hvigorw.js");
    let hvigor_package_dir = deveco_studio_dir
        .join("tools")
        .join("hvigor")
        .join("hvigor");
    let hvigor_plugin_package_dir = deveco_studio_dir
        .join("tools")
        .join("hvigor")
        .join("hvigor-ohos-plugin");

    for path in [
        &wrapper_bat,
        &wrapper_js,
        &hvigor_package_dir,
        &hvigor_plugin_package_dir,
    ] {
        if !path.exists() {
            return Err(HarmonyAppError::MissingFile { path: path.clone() });
        }
    }

    Ok(HvigorInfo {
        wrapper_bat,
        wrapper_js,
        hvigor_package_dir,
        hvigor_plugin_package_dir,
    })
}

pub fn target_to_abi(target: &str) -> Result<&'static str> {
    match target {
        "aarch64-unknown-linux-ohos" => Ok("arm64-v8a"),
        "armv7-unknown-linux-ohos" => Ok("armeabi-v7a"),
        "x86_64-unknown-linux-ohos" => Ok("x86_64"),
        "loongarch64-unknown-linux-ohos" => Ok("loongarch64"),
        _ => Err(HarmonyAppError::UnsupportedTarget {
            target: target.to_string(),
        }),
    }
}

pub fn abi_to_target(abi: &str) -> Result<&'static str> {
    match abi {
        "arm64-v8a" => Ok("aarch64-unknown-linux-ohos"),
        "armeabi-v7a" => Ok("armv7-unknown-linux-ohos"),
        "x86_64" => Ok("x86_64-unknown-linux-ohos"),
        "loongarch64" => Ok("loongarch64-unknown-linux-ohos"),
        _ => Err(HarmonyAppError::message(format!(
            "unsupported OHOS ABI [{abi}]"
        ))),
    }
}

fn discover_latest_sdk_version(root: &Path) -> Result<String> {
    let mut versions = fs::read_dir(root)
        .map_err(|source| HarmonyAppError::io(root, source))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_type()
                .ok()
                .map(|kind| kind.is_dir())
                .unwrap_or(false)
        })
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            name.parse::<u32>().ok().map(|_| name)
        })
        .collect::<Vec<_>>();
    versions.sort_by_key(|value| value.parse::<u32>().unwrap_or_default());
    versions
        .pop()
        .ok_or_else(|| HarmonyAppError::NoSdkVersionsFound {
            root: root.to_path_buf(),
        })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::{abi_to_target, discover_sdk, target_to_abi};

    #[test]
    fn auto_selects_largest_numeric_sdk_version() {
        let temp = TempDir::new().unwrap();
        for version in ["12", "20", "9"] {
            let sdk_dir = temp.path().join(version).join("ets");
            fs::create_dir_all(&sdk_dir).unwrap();
            fs::write(
                sdk_dir.join("oh-uni-package.json"),
                format!(r#"{{"apiVersion":"{version}","version":"6.0.0.47"}}"#),
            )
            .unwrap();
        }

        let sdk = discover_sdk(temp.path(), None).unwrap();
        assert_eq!(sdk.version, "20");
        assert_eq!(sdk.display_version, "6.0.0(20)");
    }

    #[test]
    fn maps_targets_to_abis() {
        assert_eq!(
            target_to_abi("aarch64-unknown-linux-ohos").unwrap(),
            "arm64-v8a"
        );
        assert_eq!(
            target_to_abi("armv7-unknown-linux-ohos").unwrap(),
            "armeabi-v7a"
        );
        assert_eq!(
            abi_to_target("x86_64").unwrap(),
            "x86_64-unknown-linux-ohos"
        );
    }
}
