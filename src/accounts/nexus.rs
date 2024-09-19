use crate::erc4337::{EntryPointApi, PackedUserOperationBuilder};
use alloy::network::Network;
use alloy::primitives::aliases::U192;
use alloy::providers::Provider;
use alloy::transports::Transport;
use alloy::{
    primitives::{address, bytes, Address, Bytes, B256, U256},
    rpc::types::PackedUserOperation,
};
use async_trait::async_trait;
use serde::Deserialize;

use super::{AccountBuilder, AccountType, SmartAccount};
//
// #[async_trait]
// pub trait AccountSpecific<N, T>: Send + Sync {
//     async fn get_nonce_for_validator(
//         &self,
//         sender: Address,
//         validator_module: Address,
//     ) -> eyre::Result<U256>;
// }
//
// pub struct NexusAccount {
// }
//
// #[async_trait]
// impl<N, T, P> AccountSpecific<N, T> for NexusAccount<P>
// where
//     N: Network,
//     T: Transport + Clone,
//     P: Provider<T, N>,
// {
//     async fn get_nonce_for_validator(
//         &self,
//         sender: Address,
//         validator_module: Address,
//     ) -> eyre::Result<U256> {
//         let mut key_bytes = [0u8; 24];
//         key_bytes[4..24].copy_from_slice(&validator_module.as_slice());
//         let key = U192::from_be_bytes(key_bytes);
//
//         let nonce = self.account.provider.get_nonce(sender, key).await?;
//         Ok(nonce)
//     }
// }

impl<P> AccountBuilder<P> for SmartAccount<P>
where
    P: Provider + Send + Sync,
{
    async fn new(provider:P) -> eyre::Result<SmartAccount<P>> {

        let account:SmartAccount<P> = SmartAccount {
            account_address: None,
            init_code: None,
            validators: None,
            provider,
            account_type: AccountType::Nexus,
        };
        Ok(account)

    } }



