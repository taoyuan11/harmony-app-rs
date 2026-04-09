use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    bin_name = "cargo ohos-app",
    author,
    version,
    about = "Package Rust GUI applications as OHOS apps"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init(InitCommand),
    Build(BuildCommand),
    Package(PackageCommand),
}

#[derive(Debug, Clone, Args, Default)]
pub struct CommonArgs {
    #[arg(long)]
    pub deveco_studio_dir: Option<PathBuf>,
    #[arg(long)]
    pub ohpm_path: Option<PathBuf>,
    #[arg(long)]
    pub sdk_root: Option<PathBuf>,
    #[arg(long)]
    pub sdk_version: Option<String>,
    #[arg(long)]
    pub version_name: Option<String>,
    #[arg(long)]
    pub version_code: Option<u32>,
    #[arg(long)]
    pub manifest_path: Option<PathBuf>,
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub out_dir: Option<PathBuf>,
    #[arg(long)]
    pub bundle_name: Option<String>,
    #[arg(long)]
    pub app_name: Option<String>,
    #[arg(long)]
    pub module_name: Option<String>,
    #[arg(long)]
    pub app_icon_path: Option<PathBuf>,
    #[arg(long)]
    pub start_icon_path: Option<PathBuf>,
    #[arg(long)]
    pub release: bool,
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Args)]
pub struct InitCommand {
    #[command(flatten)]
    pub common: CommonArgs,
}

#[derive(Debug, Clone, Args)]
pub struct BuildCommand {
    #[command(flatten)]
    pub common: CommonArgs,
}

#[derive(Debug, Clone, Args)]
pub struct PackageCommand {
    #[command(flatten)]
    pub common: CommonArgs,
    #[arg(long, value_enum, default_value_t = PackageArtifact::Hap)]
    pub artifact: PackageArtifact,
    #[arg(long)]
    pub skip_init: bool,
    #[arg(long)]
    pub skip_rust_build: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum PackageArtifact {
    Hap,
    App,
}
