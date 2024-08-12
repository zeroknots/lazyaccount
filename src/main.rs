mod account;
mod accounts;
mod config;
mod erc4337;
mod execution;
mod types;
use crate::account::{BaseAccount, SmartAccount};
use crate::accounts::safe::{Safe7579Helper, Safe7579HelperImpl};
// use crate::accounts::umsa::{AccountEnvironment, AccountEnvironmentHelper};
use crate::config::{parse_config, Config};
use crate::erc4337::{Execution, PackedUserOperation};
use crate::execution::ExecutionHelper;
use alloy::network::EthereumWallet;
use alloy::primitives::{address, b256, Bytes, bytes, U256};
use alloy::signers::local::PrivateKeySigner;
use alloy::transports::http::reqwest::Url;
use alloy::{node_bindings::Anvil, providers::ProviderBuilder};
use clap::Parser;
use std::error::Error as StdError;
use std::path::PathBuf;
use std::sync::Arc;
use std::{fs, str::FromStr};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the JSON input file
    #[arg(short, long)]
    config: PathBuf,
    #[arg(short, long)]
    private_key: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let args = Args::parse();
    let config = parse_config(args.config).unwrap();

    run(config, args.private_key).await
}

async fn run(config: Config, priv_key: String) -> Result<(), Box<dyn StdError>> {
    // let signer = PrivateKeySigner::from_str(&priv_key)?;
    // let wallet = EthereumWallet::from(signer);
    // let url = Url::parse("http://localhost:8545");

    let anvil = Anvil::new().fork("https://sepolia.drpc.org").try_spawn()?;

    // Create a provider.
    let rpc_url = anvil.endpoint().parse()?;
    let provider = ProviderBuilder::new().on_http(rpc_url);
    let provider_arc = Arc::new(provider);

    println!("Hello LazyAccount");

    let account = SmartAccount::new().with_provider(provider_arc.clone());


    let (init_code, account_address) = <Safe7579HelperImpl as Safe7579Helper>::make_account(
        provider_arc.clone(),
        config.general.account_salt,
        config.general.owners,
        config.general.validator_modules.clone(),
    )
    .await?;

    if let Some(_address) = config.general.account_address {
        if _address != account_address {
            panic!("Address mismatch");
        }
    }


    let default_validator = config.general.validator_modules[0];
    let nonce = account.get_nonce(default_validator).await?;

    let execution = account.encode_execution(vec![Execution {
        target: account_address,
        value: U256::from(100),
        callData: Bytes::from(""),
    }]);

    let userop = PackedUserOperation::new()
        .with_sender(account_address)
        .with_nonce(nonce)
        .with_init_code(init_code)
        .with_calldata(execution);
    account.send_user_op(userop).await?;

    Ok(())
}
