use std::marker::PhantomData;
use alloy::network::Network;
use alloy::providers::Provider;
use alloy::rpc::types::{SendUserOperation, PackedUserOperation};
use alloy::transports::Transport;
use alloy::{
    primitives::{address, bytes, Address, Bytes, B256, U256},
    rpc::types::PackedUserOperation,
};

use async_trait::async_trait;

use crate::RootProviderType;
use super::erc7579::Execution;

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
    ) -> eyre::Result<SmartAccount>;

    async fn is_contract(&self, account:Address) -> eyre::Result<bool>;
}

#[async_trait]
impl<N, T, P> SmartAccountBuilder<N, T> for P
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N>,
{

    async fn is_contract(&self, account:Address) -> eyre::Result<bool> {
        let code = self.get_code_at(account).await?;
        if code.len() > 0 {
            Ok(true)
        }
        else {
            Ok(false)
        }
    }

    async fn connect(
        &self,
        account_address: Option<Address>,
        account_type: AccountType,
        bundler: Box<RootProviderType>,
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
            is_initialized
        };
        Ok(smart_account)
    }
}




impl SmartAccount {
    async fn execute(
        &self,
        validator: Address,
        executions: Vec<Execution>,
    ) -> eyre::Result<()> {

        let user_op = SendUserOperation::EntryPointV07(PackedUserOperation {
            sender: self.account_address.unwrap(),
            nonce: U256::from(0),
            factory: Address::ZERO,
            factory_data: Bytes::default(),
            call_data: Bytes::default(),
            call_gas_limit: U256::from(1000000),
            verification_gas_limit: U256::from(1000000),
            pre_verification_gas: U256::from(1000000),
            max_fee_per_gas: U256::from(1000000000),
            max_priority_fee_per_gas: U256::from(1000000000),
            paymaster: Address::ZERO,
            paymaster_verification_gas_limit: U256::from(1000000),
            paymaster_post_op_gas_limit: U256::from(1000000),
            paymaster_data: Bytes::default(),
            signature: Bytes::default(),
        });





        Ok(())


    }


}


