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

#[tokio::main]
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
        Command::ExecuteOperation {
            // TODO add support for BATCH_EXECUTION mode
            sender,
            validator_module,
            target,
            value,
            calldata,
        } => {
            let config = Config::new(&cli)?;

            let smart_account = SmartAccount::new(config)?;

            let value = U256::try_from_be_slice(&hex::decode(value)?)
                .ok_or(AppError::ExecutionValueOverflow)?;
            let calldata = Bytes::from_hex(calldata)?;

            let execution = Execution {
                target: target.clone(),
                value,
                callData: calldata,
            };

            smart_account
                .execute_operation(sender, validator_module, vec![execution])
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
