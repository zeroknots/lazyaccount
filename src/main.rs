mod accounts;
mod cli;
mod config;
mod erc4337;
mod erc7579;
mod utils;

use std::fs;

use accounts::SmartAccount;
use clap::Parser;
use cli::{Cli, ExecuteCmd, ModuleCli, ModuleCmd};
use config::Config;

pub type RootProviderType =
    alloy::providers::RootProvider<alloy::transports::http::Http<alloy::transports::http::Client>>;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // let rpc_url = Url::parse("http://localhost:8545")?;
    // let bundler_url = Url::parse("http://localhost:4337")?;

    // let account_address = address!("c2b17e73603dccc195118a36f3203134fd7985f5");
    // let validator_address = address!("503b54Ed1E62365F0c9e4caF1479623b08acbe77");

    let cli = Cli::parse();

    match cli {
        Cli::Execute(ExecuteCmd { input, base }) => {
            let account = SmartAccount::from_base_args(base).await?;

            let input = fs::read_to_string(input)?;
            let config: Config = serde_json::from_str(&input)?;

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
