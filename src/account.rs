use crate::accounts::safe::{
    Safe7579HelperImpl, SAFE7579LAUNCHPAD_ADDR, SAFE_IMPL_ADDR, SAFE_PROXY_FACTORY,
};
use crate::erc4337::{ERC7579Account, EntryPoint, PackedUserOperation, ENTRYPOINT_ADDR};
use crate::types::{Foo, RootProviderType};
use alloy::contract::SolCallBuilder;
use alloy::network::{Ethereum, EthereumWallet};
use alloy::primitives::aliases::U192;
use alloy::primitives::{address, bytes, Address, Bytes, FixedBytes, U128, U256};
use alloy::providers::fillers::{
    ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller,
};
use alloy::providers::{ProviderBuilder, RootProvider};
use alloy::rpc::types::{SendUserOperation, UserOperation};
use alloy::sol;
use alloy::transports::http::reqwest::Url;
use alloy_provider::ext::Erc4337Api;
use alloy_provider::Provider;
use async_trait::async_trait;
use serde::Deserialize;
use std::error::Error as StdError;
use std::sync::Arc;

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
    pub provider: Option<Arc<RootProviderType>>,
    pub init_code: Option<Bytes>,
    pub url_provider: Option<Arc<Foo<'a>>>,
}

impl<'a> SmartAccount<'a> {
    pub fn new() -> SmartAccount<'a> {
        let account = SmartAccount {
            account_type: AccountType::Safe7579,
            address: None,
            url_provider: None,
            provider: None,
            init_code: None,
        };
        account
    }
    pub fn with_url(mut self, url: Url, wallet: &'a EthereumWallet) -> Self {
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(url);

        self.url_provider = Some(Arc::new(provider));
        self
    }
    pub fn with_provider(mut self, provider: Arc<RootProviderType>) -> Self {
        self.provider = Some(provider);
        self
    }
    pub fn with_init_code(mut self, init_code: Bytes) -> Self {
        self.init_code = Some(init_code);
        self
    }
}

#[async_trait]
pub trait BaseAccount {
    async fn get_nonce(&self, validator_module: Address) -> Result<U256, Box<dyn StdError>>;
    async fn send_user_op(
        &self,
        userop: PackedUserOperation,
    ) -> Result<FixedBytes<32>, Box<dyn StdError>>;
}

#[async_trait]
impl<'a> BaseAccount for SmartAccount<'a> {
    async fn get_nonce(&self, validator_module: Address) -> Result<U256, Box<dyn StdError>> {
        let mut key_bytes = [0u8; 24];
        key_bytes[4..24].copy_from_slice(&validator_module.as_slice());
        let key = U192::from_be_bytes(key_bytes);
        // Truncate to 192 bits (24 bytes)
        let key = key & U192::MAX; // Equivalent to uint192 in Solidity
        let ep: Address = address!("0000000071727De22E5E9d8BAf0edAc6f37da032");
        let contract = EntryPoint::new(ep, self.provider.as_ref().unwrap());
        let EntryPoint::getNonceReturn { nonce } =
            // TODO: fix the unwrap_or to actually point the counterfactual
            contract.getNonce(self.address.unwrap_or(Address::ZERO), key).call().await?;
        println!("Nonce: {:?}", nonce);
        let nonce = U256::from(0);
        Ok(nonce)
    }

    async fn send_user_op(
        &self,
        mut userop: PackedUserOperation,
    ) -> Result<FixedBytes<32>, Box<dyn StdError>> {
        let contract = EntryPoint::new(ENTRYPOINT_ADDR, self.url_provider.as_ref().unwrap());

        if let Some(init_code) = &self.init_code {
            userop.initCode = init_code.clone();
        }

        let tx_hash = contract
            .handleOps(vec![userop], ENTRYPOINT_ADDR)
            .gas(100000)
            .max_fee_per_gas(200000000000)
            .max_priority_fee_per_gas(1500000000)
            .send()
            .await?
            .watch()
            .await?;

        println!("Transaction hash: {:?}", tx_hash);

        Ok(tx_hash)
    }
}
