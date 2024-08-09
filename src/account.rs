use alloy::primitives::{address, Address, U256};
use alloy::network::{Ethereum,EthereumWallet};
use alloy::providers::ProviderBuilder;
use alloy::sol;
use alloy::transports::http::reqwest::Url;
use alloy::contract::SolCallBuilder;
use serde::Deserialize;
use std::error::Error as StdError;
use std::sync::Arc;
use async_trait::async_trait;

use crate::erc4337::{ERC7579Account, EntryPoint, PackedUserOperation};



type HttpProvider<'a> = alloy::providers::fillers::FillProvider<alloy::providers::fillers::JoinFill<alloy::providers::Identity, alloy::providers::fillers::WalletFiller<&'a EthereumWallet>>, alloy::providers::RootProvider<alloy::transports::http::Http<alloy::transports::http::Client>>, alloy::transports::http::Http<alloy::transports::http::Client>, alloy::network::Ethereum>;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum AccountType {
    Unknown,
    Safe7579,
    Kernel,
}

#[derive(Debug  )]
pub struct SmartAccount<'a> {
    pub account_type: AccountType,
    pub address: Option<Address>,
    pub execution_cache: Option<ERC7579Account::ERC7579AccountCalls>,
    pub validators: Option<Vec<Address>>,
    pub provider: Option<Arc<HttpProvider<'a>>>
}

impl<'a> SmartAccount<'a> {
    pub fn new(url: Url, wallet: &'a EthereumWallet) -> Result<Self, Box<dyn std::error::Error>> {
        let provider: HttpProvider = ProviderBuilder::new().wallet(wallet).on_http(url);
        Ok(SmartAccount {
            account_type: AccountType::Safe7579,
            address: None,
            execution_cache: None,
            validators: None,
            provider: Some(Arc::new(provider)),
        })
    }
}


#[async_trait]
pub trait BaseAccount {
    async fn get_nonce(
        &self,
        validator_module: Address,
    ) -> Result<U256, Box<dyn StdError>>;
}

#[async_trait]
impl<'a>BaseAccount for SmartAccount<'a> {
    async fn get_nonce(
        &self,
        validator_module: Address,
    ) -> Result<U256, Box<dyn StdError>> {
        let mut key_bytes = [0u8; 32];
        key_bytes[12..32].copy_from_slice(&validator_module.as_slice());
        let key = U256::from_be_bytes(key_bytes);
        // Truncate to 192 bits (24 bytes)
        let key = key & (U256::MAX >> 64); // Equivalent to uint192 in Solidity
        let ep: Address = address!("0000000071727De22E5E9d8BAf0edAc6f37da032");
        let contract = EntryPoint::new(ep, self.provider.as_ref().unwrap());
        let EntryPoint::getNonceReturn { nonce } = contract
            .getNonce(validator_module, key)
            .call()
            .await?;
        println!("Nonce: {:?}", nonce);
        let nonce = U256::from(0);
        Ok(nonce)
    }
}


#[async_trait]
pub trait Bundler {
    async fn send_user_op(&self,userop:PackedUserOperation) -> Result<(), Box<dyn StdError>>;
}

#[async_trait]
impl<'a> Bundler for SmartAccount<'a> {
    async fn send_user_op(&self,userop:PackedUserOperation) -> Result<(), Box<dyn StdError>> { 
        let ep: Address = address!("0000000071727De22E5E9d8BAf0edAc6f37da032");

        let contract = EntryPoint::new(ep, self.provider.as_ref().unwrap());
        let tx_hash = contract.handleOps(vec![userop], ep).send().await?.watch().await?;

        Ok(())

    }
}
