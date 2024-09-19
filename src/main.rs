mod accounts;
mod erc4337;
mod erc7579;
use accounts::{AccountType, SmartAccountBuilder};
use alloy_provider::{ext::Erc4337Api, ProviderBuilder, RootProvider};
use url::Url;

use alloy::primitives::address;

use alloy::providers::Provider;
use alloy::transports::http::{Client, Http};

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
        .connect(Some(account_address), AccountType::Nexus, Box::new(bundler), Box::new(provider.clone()))
        .await?;
    println!("{:?}", account);

    // let builder = account_type.create_builder::<RootProvider<Http<Client>>>()?;
    //
    //
    // let user_op =prepare_user_operation(account_address, validator_address).await?;

    Ok(())
}
