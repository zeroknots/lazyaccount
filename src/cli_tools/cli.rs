use crate::{account_builder::AccountType, module_ops::ModuleAction};
use alloy::primitives::Address;
use std::path::PathBuf;

#[derive(clap::Parser, Clone, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Configuration profile name (could be any string, default is "default")
    #[arg(short, long, default_value = "default")]
    pub profile: String,

    /// URL of the RPC node
    #[arg(short, long)]
    pub rpc_node_url: Option<String>,

    /// URL of the bundler RPC
    #[arg(long)]
    pub rpc_bundler_url: Option<String>,

    /// Address of the entrypoint contract
    #[arg(short, long)]
    pub entry_point_addr: Option<Address>,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum Command {
    CreateAccount {
        #[arg(value_enum, short, long)]
        account_type: AccountType,
        #[arg(short, long)]
        data_path: PathBuf,
    },

    ExecuteOperation {
        #[arg(value_enum, short, long)]
        sender: Address,
        #[arg(value_enum, short, long)]
        validator_module: Address,
        #[arg(value_enum, short, long)]
        target: Address,
        #[arg(short, long)]
        value: String,
        #[arg(short, long)]
        calldata: String,
    },

    ModuleOperation {
        #[arg(value_enum, short, long)]
        sender: Address,
        #[arg(value_enum, short, long)]
        module_action: ModuleAction,
    },

    #[command(subcommand)]
    Config(ConfigSubcommand),
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum InstallModuleSub {
    ModuleType { module_type: String },
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum ConfigSubcommand {
    /// Print configuration value for selected profile
    Get {
        /// Configuration item to print (print all if omitted)
        #[arg(value_enum)]
        key: Option<ConfigItem>,
    },

    /// Update configuration value for selected profile
    Set {
        /// Configuration item to update
        #[arg(value_enum)]
        key: ConfigItem,

        /// New value to be set
        value: String,
    },

    /// Remove configuration value from selected profile
    Unset {
        /// Configuration item to removed
        #[arg(value_enum)]
        key: ConfigItem,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, clap::ValueEnum, Debug)]
pub enum ConfigItem {
    RpcNodeUrl,
    EntryPointAddr,
    RpcBundlerUrl,
}
