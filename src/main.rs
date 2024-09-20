mod accounts;
mod erc4337;
mod erc7579;
use accounts::{AccountType, SmartAccountBuilder};
use alloy_provider::{ext::Erc4337Api, ProviderBuilder, RootProvider};
use erc4337::EntryPointApi;
use erc7579::Execution;
use url::Url;

use alloy::{
    hex::FromHex,
    primitives::{address, aliases::U192, Bytes, U256},
};

pub type RootProviderType =
    alloy::providers::RootProvider<alloy::transports::http::Http<alloy::transports::http::Client>>;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let rpc_url = Url::parse("http://localhost:8545")?;
    let bundler_url = Url::parse("http://localhost:4337")?;

    let account_address = address!("c2b17e73603dccc195118a36f3203134fd7985f5");
    let validator_address = address!("503b54Ed1E62365F0c9e4caF1479623b08acbe77");

    let provider = ProviderBuilder::new().on_http(rpc_url);
    let bundler = ProviderBuilder::new().on_http(bundler_url);

    let account = provider
        .connect(
            Some(account_address),
            AccountType::Safe7579,
            Box::new(bundler.clone()),
            Box::new(provider.clone()),
        )
        .await?;

    let mut key = [0u8; 24];
    key[4..].copy_from_slice(validator_address.as_slice());

    let real_nonce = provider
        .get_nonce(account_address, U192::from_be_bytes(key))
        .await?;

    println!("real nonce: {:?}", real_nonce.to_be_bytes_vec());

    println!("{:?}", account);

    let executions: Vec<Execution> = vec![Execution {
        target: account_address,
        value: U256::from(100000),
        // simple transfer
        callData: Bytes::default(),
    }];

    account
        .execute(bundler, validator_address, real_nonce, executions)
        .await?;
    Ok(())
}
