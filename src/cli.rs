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

#[derive(Debug, Clone, Args)]
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
    pub manifest_path: Option<PathBuf>,
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long, value_enum)]
    pub abi: Option<Abi>,
    #[arg(long)]
    pub out_dir: Option<PathBuf>,
    #[arg(long)]
    pub bundle_name: Option<String>,
    #[arg(long)]
    pub module_name: Option<String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Abi {
    #[value(name = "arm64-v8a")]
    Arm64V8a,
    #[value(name = "armeabi-v7a")]
    ArmeabiV7a,
    #[value(name = "x86_64")]
    X86_64,
    #[value(name = "loongarch64")]
    Loongarch64,
}

impl Abi {
    pub fn as_str(self) -> &'static str {
        match self {
            Abi::Arm64V8a => "arm64-v8a",
            Abi::ArmeabiV7a => "armeabi-v7a",
            Abi::X86_64 => "x86_64",
            Abi::Loongarch64 => "loongarch64",
        }
    }
}
