use std::io;
use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, OhosAppError>;

#[derive(Debug, Error)]
pub enum OhosAppError {
    #[error("{message}")]
    Message { message: String },
    #[error("failed to parse package metadata in [{manifest_path}]: {source}")]
    ConfigParse {
        manifest_path: PathBuf,
        source: serde_json::Error,
    },
    #[error(
        "missing required configuration [{field}]; provide it via {cli_flag}, env {env_names}, or [package.metadata.ohos-app.<profile>] in [{manifest_path}]"
    )]
    MissingRequiredConfig {
        field: &'static str,
        cli_flag: &'static str,
        env_names: &'static str,
        manifest_path: PathBuf,
    },
    #[error("failed to read file [{path}]: {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("failed to read Cargo metadata: {0}")]
    CargoMetadata(#[from] cargo_metadata::Error),
    #[error("Rust project [{manifest_path}] must define a library target")]
    MissingLibraryTarget { manifest_path: PathBuf },
    #[error("unsupported OHOS target triple [{target}]")]
    UnsupportedTarget { target: String },
    #[error("OpenHarmony SDK root does not exist: [{path}]")]
    MissingSdkRoot { path: PathBuf },
    #[error("OpenHarmony SDK version directory does not exist: [{path}]")]
    MissingSdkVersion { path: PathBuf },
    #[error("failed to discover an SDK version under [{root}]")]
    NoSdkVersionsFound { root: PathBuf },
    #[error("required file is missing: [{path}]")]
    MissingFile { path: PathBuf },
    #[error("failed to spawn command [{program}] in [{cwd}]: {source}")]
    CommandSpawn {
        program: String,
        cwd: PathBuf,
        source: io::Error,
    },
    #[error("command failed [{program}] in [{cwd}] with exit code {code:?}")]
    CommandFailed {
        program: String,
        cwd: PathBuf,
        code: Option<i32>,
    },
    #[error("no .app artifact was found under [{search_root}] after packaging")]
    PackageArtifactNotFound { search_root: PathBuf },
}

pub type HarmonyAppError = OhosAppError;

impl OhosAppError {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message {
            message: message.into(),
        }
    }

    pub fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}

impl From<io::Error> for OhosAppError {
    fn from(source: io::Error) -> Self {
        Self::Message {
            message: format!("I/O error: {source}"),
        }
    }
}
