
mod account;
mod erc4337;
mod config;
use std::error::Error as StdError;
use std::{fs, str::FromStr};
use crate::config::{parse_config, Config};
use crate::account::{SmartAccount, BaseAccount};
use alloy::primitives::address;
use alloy::transports::http::reqwest::Url;
use alloy::network::EthereumWallet;
use alloy::signers::local::PrivateKeySigner;
use clap::Parser;
use std::path::PathBuf;

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

async fn run(config: Config, priv_key:String) -> Result<(), Box<dyn StdError>> {
    println!("Hello LazyAccount");
    let rpc_url = "http://localhost:8545";
    let signer = PrivateKeySigner::from_str(&priv_key)?;
    let wallet = EthereumWallet::from(signer);

    let account = SmartAccount::new(Url::parse(rpc_url)?, &wallet);

    let validator = address!("fB43116489394D843B2B29a7F6aa3eC0d590d795");

    account.unwrap().get_nonce(validator).await?;

    let userop = erc4337::PackedUserOperation::new();
    account.send_user_op(userop).await?;


    Ok(())
}
