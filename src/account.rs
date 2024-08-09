use alloy::contract::SolCallBuilder;
use alloy::network::{Ethereum, EthereumWallet};
use alloy::primitives::{address, Address, U128, U256};
use alloy::providers::fillers::{
    ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller,
};
use alloy::providers::{ProviderBuilder, RootProvider};
use alloy::sol;
use alloy::transports::http::reqwest::Url;
use async_trait::async_trait;
use serde::Deserialize;
use std::error::Error as StdError;
use std::sync::Arc;

use crate::erc4337::{ERC7579Account, EntryPoint, PackedUserOperation};

pub type HttpProvider<'a> = alloy::providers::fillers::FillProvider<
    alloy::providers::fillers::JoinFill<
        alloy::providers::Identity,
        alloy::providers::fillers::WalletFiller<&'a EthereumWallet>,
    >,
    alloy::providers::RootProvider<alloy::transports::http::Http<alloy::transports::http::Client>>,
    alloy::transports::http::Http<alloy::transports::http::Client>,
    alloy::network::Ethereum,
>;

// pub type FillProvider<'a> = alloy::providers::fillers::FillProvider<
// alloy::providers::fillers::JoinFill<alloy::providers::Identity,
//   alloy::providers::fillers::WalletFiller<&'a EthereumWallet>>,
//     alloy::providers::RootProvider<alloy::transports::http::Http<alloy::transports::http::Client>>,
//     alloy::transports::http::Http<alloy::transports::http::Client>,
//     alloy::network::Ethereum,>;
//
pub type Foo<'a> = FillProvider<
    JoinFill<
        JoinFill<
            JoinFill<JoinFill<alloy::providers::Identity, GasFiller>, NonceFiller>,
            ChainIdFiller,
        >,
        WalletFiller<&'a EthereumWallet>,
    >,
    alloy::providers::RootProvider<alloy::transports::http::Http<alloy::transports::http::Client>>,
    alloy::transports::http::Http<alloy::transports::http::Client>,
    alloy::network::Ethereum,
>;

pub type RootProviderType<'a> =
    alloy::providers::RootProvider<alloy::transports::http::Http<alloy::transports::http::Client>>;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum AccountType {
    Unknown,
    Safe7579,
    Kernel,
}

#[derive(Debug)]
pub struct SmartAccount<'a> {
    pub account_type: AccountType,
    pub address: Option<Address>,
    pub provider: Option<Arc<RootProviderType<'a>>>,
    pub url_provider: Option<Arc<Foo<'a>>>,
}

impl<'a> SmartAccount<'a> {
    pub fn new() -> SmartAccount<'a> {
        let account = SmartAccount {
            account_type: AccountType::Safe7579,
            address: None,
            url_provider: None,
            provider: None,
        };
        account
    }
    pub fn with_url(mut self, url: Url, wallet: &'a EthereumWallet) -> Self {
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(url);
        println!("{:?}", provider);

        self.url_provider = Some(Arc::new(provider));
        self
    }
    pub fn with_provider(mut self, provider: Arc<RootProviderType<'a>>) -> Self {
        self.provider = Some(provider);
        self
    }
}

#[async_trait]
pub trait BaseAccount {
    async fn get_nonce(&self, validator_module: Address) -> Result<U256, Box<dyn StdError>>;
    async fn send_user_op(&self, userop: PackedUserOperation) -> Result<(), Box<dyn StdError>>;
}

#[async_trait]
impl<'a> BaseAccount for SmartAccount<'a> {
    async fn get_nonce(&self, validator_module: Address) -> Result<U256, Box<dyn StdError>> {
        let mut key_bytes = [0u8; 32];
        key_bytes[12..32].copy_from_slice(&validator_module.as_slice());
        let key = U256::from_be_bytes(key_bytes);
        // Truncate to 192 bits (24 bytes)
        let key = key & (U256::MAX >> 64); // Equivalent to uint192 in Solidity
        let ep: Address = address!("0000000071727De22E5E9d8BAf0edAc6f37da032");
        let contract = EntryPoint::new(ep, self.provider.as_ref().unwrap());
        let EntryPoint::getNonceReturn { nonce } =
            contract.getNonce(validator_module, key).call().await?;
        println!("Nonce: {:?}", nonce);
        let nonce = U256::from(0);
        Ok(nonce)
    }

    async fn send_user_op(&self, userop: PackedUserOperation) -> Result<(), Box<dyn StdError>> {
        let ep: Address = address!("0000000071727De22E5E9d8BAf0edAc6f37da032");
        let contract = EntryPoint::new(ep, self.provider.as_ref().unwrap());

        let tx_hash = contract
            .handleOps(vec![userop], ep)
            .gas(100000)
            .max_fee_per_gas(200000000000)
            .max_priority_fee_per_gas(1500000000)
            .send()
            .await?
            .watch()
            .await?;

        println!("{:?}", tx_hash);

        Ok(())
    }

}

