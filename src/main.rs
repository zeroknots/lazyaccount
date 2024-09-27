mod accounts;
mod cli;
mod erc4337;
mod erc7579;
mod types;

use std::fs;

use accounts::SmartAccount;
use alloy::{
    primitives::{aliases::U192, Address, Bytes},
    sol,
    sol_types::SolEvent,
};
use clap::Parser;
use cli::{Cli, ExecuteCmd, ModuleCli, ModuleCmd, ModuleSubCmd};
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

sol! {
    #[derive(Debug, PartialEq, Eq)]
    contract Test {
        event ModuleInitialized(address indexed account, address owner);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    println!(
        "ModuleInitialized event: {:?}",
        Test::ModuleInitialized::SIGNATURE_HASH
    );
    match cli {
        Cli::Execute(ExecuteCmd { input, base }) => {
            let account = SmartAccount::from_base_args(base).await?;

            let input = fs::read_to_string(input)?;
            let config: Executions = serde_json::from_str(&input)?;

            let executions = config.executions;

            account.execute(executions, 0).await?;
        }
        Cli::Module { module: inner_cmd } => {
            match inner_cmd {
                ModuleCli::Install(ModuleSubCmd {
                    module,
                    module_type_id,
                    data,
                    base,
                }) => {
                    let account = SmartAccount::from_base_args(base).await?;

                    let data = hex::decode(data)?;
                    account
                        .install_module(module_type_id, module, Bytes::from(data))
                        .await?;
                }
                ModuleCli::Uninstall(cmd) => {
                    // account
                    //     .uninstall_module(cmd.module_type_id, cmd.module_address, cmd.deinit_data)
                    //     .await?;
                    unimplemented!()
                }
                ModuleCli::IsInstalled(ModuleSubCmd {
                    module_type_id,
                    module,
                    data: additional_context,
                    base,
                }) => {
                    let account = SmartAccount::from_base_args(base).await?;

                    let additional_context = hex::decode(additional_context)?;
                    let is_installed = account
                        .is_module_installed(
                            module_type_id,
                            module,
                            Bytes::from(additional_context),
                        )
                        .await?;
                    println!("Is module installed: {}", is_installed);
                }
            }
        }
    }
    Ok(())
}
