use crate::{account_builder::AccountType, module_ops::ModuleAction};
use alloy::hex::FromHexError;
use alloy::primitives::hex::FromHex;
use alloy::primitives::ruint::aliases::U256;
use alloy::primitives::Address;
use alloy::primitives::Bytes;
use std::path::PathBuf;
use std::str::FromStr;

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

    ExecuteOperation(ExecuteOperationParams),

    ModuleOperation {
        #[arg(value_enum, short, long)]
        sender: Address,
        #[arg(value_enum, short, long)]
        module_action: ModuleAction,
    },

    #[command(subcommand)]
    Config(ConfigSubcommand),
}

#[derive(Debug, Clone)]
pub struct HexEncodedBytes(Bytes);

impl HexEncodedBytes {
    pub fn into_inner(self) -> Bytes {
        self.0
    }
}

impl FromStr for HexEncodedBytes {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Bytes::from_hex(s).map(HexEncodedBytes)
    }
}

#[derive(clap::Args, Clone, Debug)]
pub struct ExecuteOperationParams {
    #[arg(long)]
    pub sender: Address,
    #[arg(long)]
    pub validator_module: Address,
    #[arg(long, required(true))]
    pub target: Vec<Address>,
    #[arg(long, required(true))]
    pub value: Vec<U256>,
    #[arg(long, required(true))]
    pub calldata: Vec<HexEncodedBytes>,
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
