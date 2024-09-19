use alloy::{
    hex::{self, FromHex},
    primitives::{Bytes, U256},
};
use clap::Parser;
use cli_tools::{
    cli::{Cli, Command},
    setup_config::{self, Config},
};
use erc4337::Execution;
use error::AppError;
use smart_account::SmartAccount;

mod account_builder;
mod accounts;
mod cli_tools;
mod erc4337;
mod error;
mod execution;
mod module_ops;
mod smart_account;

multiplexertokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Command::Config(cmd) => setup_config::run_config_command(&cli, cmd)?,
        Command::CreateAccount {
            account_type,
            data_path,
        } => {
            let config = Config::new(&cli)?;

            let smart_account = SmartAccount::new(config)?;

            let builder = account_type.create_account_builder(data_path)?;

            smart_account.create_account(builder).await?;
        }
        Command::ExecuteOperation(e) => {
            let config = Config::new(&cli)?;
            let smart_account = SmartAccount::new(config)?;

            let executions = e
                .target
                .iter()
                .zip(e.value.iter())
                .zip(e.calldata.iter())
                .map(|((&target, &value), call_data)| {
                    (target, value, call_data.clone().into_inner())
                })
                .collect();

            smart_account
                .execute_operation(e.sender, e.validator_module, executions)
                .await?;
        }
        Command::ModuleOperation {
            sender,
            module_action,
        } => {
            let config = Config::new(&cli)?;

            let smart_account = SmartAccount::new(config)?;

            smart_account
                .execute_module_operation(sender, module_action)
                .await?;
        }
    };


    Ok(())
}
