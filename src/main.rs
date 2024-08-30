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
use crate::erc4337::Execution;
use crate::execution::ExecutionHelper;
use accounts::safe::{Safe7579HelperData, SAFE_PROXY_FACTORY};
use alloy::network::EthereumWallet;
use alloy::primitives::{address, b256, bytes, Bytes, U256};
use alloy::rpc::types::SendUserOperation;
use alloy::signers::local::PrivateKeySigner;
use alloy::transports::http::reqwest::Url;
use alloy::{node_bindings::Anvil, providers::ProviderBuilder};
use alloy_provider::ext::Erc4337Api;
use clap::Parser;
use erc4337::{PackedUserOperation, ENTRYPOINT_ADDR};
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

    run(config).await
}

async fn run(config: Config) -> Result<(), Box<dyn StdError>> {
    let rpc_url = Url::parse(&config.general.rpc_node_url)?;

    let anvil = Anvil::new().fork(rpc_url.clone()).try_spawn()?;

    // Provider which would be used to predicted address computation and init data structuring
    let chain_provider = ProviderBuilder::new().on_http(rpc_url);
    let chain_provider_arc = Arc::new(chain_provider);

    let account = SmartAccount::new().with_provider(chain_provider_arc.clone());

    let (init_code, account_address) = <Safe7579HelperImpl as Safe7579Helper>::make_account(
        chain_provider_arc.clone(),
        config.general.account_salt,
        config.general.owners.clone(),
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

    println!("Encode execution");

    let userop = PackedUserOperation::new()
        .with_sender(account_address)
        .with_nonce(nonce)
        .with_init_code(init_code.clone())
        .with_calldata(execution);

    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);

    // we use anvil provider here for the creating smart account directly calling the entrypoint contract
    let account = account.with_url(anvil.endpoint_url(), &wallet);

    let userop1 = userop.clone();

    account.send_user_op(userop).await?;

    let Safe7579HelperData {
        proxy_call,
        factory_initializer: _,
    } = Safe7579HelperImpl::make_factory_data(
        chain_provider_arc.clone(),
        config.general.account_salt,
        config.general.owners,
        config.general.validator_modules.clone(),
    )
    .await?;

    // the provider for the erc4337 bundler in order to send the PackedUserOperation via
    let rpc_url = Url::parse("http://localhost:3000")?;
    let bundler_provider = ProviderBuilder::new().on_http(rpc_url);

    println!("Sender {}", userop1.sender);
    // preparing the PackedUserOperation
    let send_user_operation =
        SendUserOperation::EntryPointV07(alloy::rpc::types::PackedUserOperation {
            sender: userop1.sender,
            nonce: userop1.nonce,
            factory: SAFE_PROXY_FACTORY,
            factory_data: proxy_call,
            call_data: userop1.callData,
            call_gas_limit: U256::try_from(userop1.accountGasLimits).unwrap(),
            verification_gas_limit: U256::try_from(userop1.accountGasLimits).unwrap(),
            pre_verification_gas: userop1.preVerificationGas,
            max_fee_per_gas: U256::try_from(userop1.gasFees).unwrap(),
            max_priority_fee_per_gas: U256::try_from(userop1.gasFees).unwrap(),
            paymaster: ENTRYPOINT_ADDR,
            paymaster_verification_gas_limit: U256::try_from(userop1.accountGasLimits).unwrap(),
            paymaster_post_op_gas_limit: U256::try_from(userop1.accountGasLimits).unwrap(),
            paymaster_data: Bytes::default(),
            signature: userop1.signature,
        });

    let resp = bundler_provider
        .send_user_operation(send_user_operation, ENTRYPOINT_ADDR)
        .await?;

    println!("{:?}", resp);

    Ok(())
}
