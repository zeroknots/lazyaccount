//! CLI definition

use alloy::primitives::{Address, Bytes, U256};
use url::Url;

/// Main CLI
#[derive(Debug, clap::Parser)]
pub enum Cli {
    Execute(ExecuteCmd),
    Module(ModuleCmd),
}

/// Base args for all commands
#[derive(Debug, clap::Parser)]
pub struct BaseArgs {
    /// Client URL
    #[arg(short, long, default_value = "http://localhost:8545")]
    pub client: Url,
    /// Bundler URL
    #[arg(short, long, default_value = "http://localhost:4337")]
    pub bundler: Url,
    /// Account address
    #[arg(short, long)]
    pub account: Address,
    /// Validator address
    #[arg(short, long)]
    pub validator: Address,
}

/// Execute command
#[derive(Debug, clap::Parser)]
pub struct ExecuteCmd {
    #[arg(short, long, default_value = "execs.toml")]
    pub input: String,
    #[command(flatten)]
    pub base: BaseArgs,
}

#[derive(Debug, clap::Parser)]
pub struct ModuleCmd {
    #[command(subcommand)]
    pub module: ModuleCli,
    #[command(flatten)]
    pub base: BaseArgs,
}

/// Commands related to modules
#[derive(Debug, clap::Parser)]
pub enum ModuleCli {
    Install(ModuleSubCmd),
    Uninstall(ModuleSubCmd),
    IsInstalled(ModuleSubCmd),
}

/// Common args for module commands
#[derive(Debug, clap::Parser)]
pub struct ModuleSubCmd {
    /// Module type id
    #[arg(short, long)]
    pub module_type_id: U256,
    /// Module address
    #[arg(short, long)]
    pub module: Address,
    /// Module init/deinit data or additional context for `isModuleInstalled`
    #[arg(short, long)]
    pub data: Bytes,
}
