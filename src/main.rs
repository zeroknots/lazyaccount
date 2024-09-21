mod accounts;
mod cli;
mod erc4337;
mod erc7579;
mod types;

use std::fs;

use accounts::SmartAccount;
use alloy::primitives::{aliases::U192, Address};
use clap::Parser;
use cli::{Cli, ExecuteCmd, ModuleCli, ModuleCmd};
use types::Executions;

pub type RootProviderType =
    alloy::providers::RootProvider<alloy::transports::http::Http<alloy::transports::http::Client>>;

pub type Result<T> = eyre::Result<T>;

/// Convert address to key
/// TODO: move to utils once there are more utility functions
pub(crate) fn address_to_key(address: &Address) -> U192 {
    let mut key_bytes = [0u8; 24];
    key_bytes[..20].copy_from_slice(&address.as_slice());
    U192::from_be_bytes(key_bytes)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Execute(ExecuteCmd { input, base }) => {
            let account = SmartAccount::from_base_args(base).await?;

            let input = fs::read_to_string(input)?;
            let config: Executions = serde_json::from_str(&input)?;

            let executions = config.executions;

            account.execute(executions, 0).await?;
        }
        Cli::Module(ModuleCmd {
            module: inner_cmd,
            base,
        }) => {
            let account = SmartAccount::from_base_args(base).await?;

            match inner_cmd {
                ModuleCli::Install(cmd) => {
                    // account
                    // .install_module(cmd.module_type_id, cmd.module_address, cmd.init_data)
                    // .await?;
                    unimplemented!()
                }
                ModuleCli::Uninstall(cmd) => {
                    // account
                    //     .uninstall_module(cmd.module_type_id, cmd.module_address, cmd.deinit_data)
                    //     .await?;
                    unimplemented!()
                }
                ModuleCli::IsInstalled(cmd) => {
                    // account
                    //     .is_module_installed(cmd.module_type_id, cmd.module_address, cmd.additional_context)
                    //     .await?;
                    unimplemented!()
                }
            }
        }
    }
    Ok(())
}
