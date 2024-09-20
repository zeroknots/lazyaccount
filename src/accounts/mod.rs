use alloy::network::Network;
use alloy::primitives::aliases::U192;
use alloy::primitives::{address, bytes, Address, Bytes, B256, U256};
use alloy::providers::Provider;
use alloy::rpc::types::{PackedUserOperation, SendUserOperation};
use alloy::transports::Transport;
use alloy_provider::ext::Erc4337Api;
use std::marker::PhantomData;

use async_trait::async_trait;

use super::erc7579::Execution;
use crate::erc4337::{EntryPointApi, ENTRYPOINT};
use crate::erc7579::ExecutionBuilder;
use crate::RootProviderType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountType {
    Nexus,
    Safe7579,
    Kernel,
}

#[derive(Debug, Clone)]
pub struct SmartAccount {
    account_address: Option<Address>,
    init_code: Option<Bytes>,
    validators: Option<Vec<Address>>,
    account_type: AccountType,
    bundler: Box<RootProviderType>,
    rpc: Box<RootProviderType>,
    is_initialized: bool,
}

#[derive(Debug, Clone)]
pub struct SmartAccountConfig {
    validators: Option<Vec<Address>>,
    account_type: AccountType,
}

#[async_trait]
pub trait SmartAccountBuilder<N, T>: Send + Sync {
    async fn connect(
        &self,
        account_address: Option<Address>,
        account_type: AccountType,
        bundler: Box<RootProviderType>,
        rpc: Box<RootProviderType>,
    ) -> eyre::Result<SmartAccount>;

    async fn is_contract(&self, account: Address) -> eyre::Result<bool>;
}

#[async_trait]
impl<N, T, P> SmartAccountBuilder<N, T> for P
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N>,
{
    async fn is_contract(&self, account: Address) -> eyre::Result<bool> {
        let code = self.get_code_at(account).await?;
        if code.len() > 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn connect(
        &self,
        account_address: Option<Address>,
        account_type: AccountType,
        bundler: Box<RootProviderType>,
        rpc: Box<RootProviderType>,
    ) -> eyre::Result<SmartAccount> {
        let is_initialized = match account_address {
            Some(addr) => self.is_contract(addr).await?,
            None => false,
        };

        let smart_account = SmartAccount {
            account_address,
            init_code: None,
            validators: None,
            account_type,
            bundler,
            is_initialized,
            rpc,
        };
        Ok(smart_account)
    }
}

impl SmartAccount {
    // fn get_validator_nonce(&self, validator: Address) -> eyre::Result<U256> {
    //     let nonce = self.rpc.get_nonce_for_validator(validator).await?;
    //     Ok(nonce)
    // }

    pub async fn execute(
        &self,
        provider: RootProviderType,
        validator: Address,
        nonce: U256,
        executions: Vec<Execution>,
    ) -> eyre::Result<()> {
        let call_data = executions.encode_executions();
        // first 20 bytes of nonce is validator address, the rest is sequence number

        // let mut key = [0u8; 24];
        // key[4..].copy_from_slice(validator.as_slice());

        // let real_nonce = provider
        //     .get_nonce(self.account_address.unwrap(), U192::from_be_bytes(key))
        //     .await?;

        // println!("real nonce: {:?}", real_nonce.to_be_bytes_vec());

        // let mut nonce = [0u8; 32];
        // nonce[..24].copy_from_slice(&key);
        // nonce[24..].copy_from_slice(&0_u64.to_be_bytes()[..]);

        let user_op = SendUserOperation::EntryPointV07(PackedUserOperation {
            sender: self.account_address.unwrap(),
            nonce,
            factory: None,
            factory_data: None,
            call_data,
            call_gas_limit: U256::from(1000000),
            verification_gas_limit: U256::from(1000000),
            pre_verification_gas: U256::from(1000000),
            max_fee_per_gas: U256::from(1000000000),
            max_priority_fee_per_gas: U256::from(1000000000),
            paymaster: None,
            paymaster_verification_gas_limit: Some(U256::from(1000000)),
            paymaster_post_op_gas_limit: Some(U256::from(1000000)),
            paymaster_data: None,
            signature: Bytes::default(),
        });

        // sign user op

        println!("{:?}", user_op);
        let operation_gas = provider
            .estimate_user_operation_gas(user_op.clone(), ENTRYPOINT)
            .await;

        println!("{:?}", operation_gas);

        provider.send_user_operation(user_op, ENTRYPOINT).await?;

        Ok(())
    }
}
