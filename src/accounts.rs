use alloy::network::Network;
use alloy::primitives::{Address, Bytes, FixedBytes};
use alloy::providers::Provider;
use alloy::rpc::types::PackedUserOperation;
use alloy::transports::Transport;
use alloy_provider::ProviderBuilder;

use async_trait::async_trait;

use super::erc7579::Execution;
use crate::cli::BaseArgs;
use crate::erc4337::{EntryPointApi, PackedUserOperationBuilder, ENTRYPOINT};
use crate::erc7579::ExecutionBuilder;
use crate::Result;
use crate::RootProviderType;

/// The type of account to be used
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountType {
    Nexus,
    Safe7579,
    Kernel,
}

/// The smart account, which is used to execute the user operations
#[derive(Debug, Clone)]
pub struct SmartAccount {
    account_address: Address,
    init_code: Bytes,
    validators: Vec<Address>,
    account_type: AccountType,
    bundler: Box<RootProviderType>,
    rpc: Box<RootProviderType>,
    paymaster: Option<Address>,
    is_initialized: bool,
}

#[async_trait]
pub trait SmartAccountBuilder<N, T>: Send + Sync {
    async fn connect(
        &self,
        account_address: Address,
        account_type: AccountType,
        bundler: Box<RootProviderType>,
        rpc: Box<RootProviderType>,
    ) -> Result<SmartAccount>;

    async fn is_contract(&self, account: Address) -> bool;
}

#[async_trait]
impl<N, T, P> SmartAccountBuilder<N, T> for P
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N>,
{
    async fn is_contract(&self, account: Address) -> bool {
        if let Ok(code) = self.get_code_at(account).await {
            return code.len() > 0;
        }
        false
    }

    /// Connect to the smart account
    async fn connect(
        &self,
        account_address: Address,
        account_type: AccountType,
        bundler: Box<RootProviderType>,
        rpc: Box<RootProviderType>,
    ) -> Result<SmartAccount> {
        let is_initialized = self.is_contract(account_address).await;

        let smart_account = SmartAccount {
            account_address,
            init_code: Bytes::default(),
            validators: Vec::new(),
            account_type,
            bundler,
            is_initialized,
            rpc,
            paymaster: None,
        };
        Ok(smart_account)
    }
}

impl SmartAccount {
    pub async fn from_base_args(base_args: BaseArgs) -> Result<SmartAccount> {
        let bundler = ProviderBuilder::new().on_http(base_args.bundler);
        let rpc = ProviderBuilder::new().on_http(base_args.client);

        let account_address = base_args.account;
        let account_type = AccountType::Safe7579;

        let mut account = rpc
            .connect(
                account_address,
                account_type,
                Box::new(bundler),
                Box::new(rpc.clone()),
            )
            .await?;

        // TODO: use builder
        account.validators = vec![base_args.validator];

        Ok(account)
    }

    pub async fn execute(
        &self,
        executions: Vec<Execution>,
        _validator_index: usize,
    ) -> Result<FixedBytes<32>> {
        // TODO: support multiple validators
        let validator = self.validators[0];
        let key = crate::address_to_key(&validator);
        let nonce = self.rpc.get_nonce(self.account_address, key).await?;
        let call_data = executions.encode_executions();

        let user_op = PackedUserOperation::default()
            .with_call_data(call_data)
            .with_sender(self.account_address)
            .with_nonce(nonce);

        println!(
            "Submitting {:?}",
            serde_json::to_string_pretty(&user_op)
                .unwrap()
                .replace("\n", "")
        );

        // TODO: fix upstream issue where `send_user_operation` result is not properly deserialized and uncomment lines below

        // self.bundler
        //     .send_user_operation(SendUserOperation::EntryPointV07(user_op), ENTRYPOINT)
        //     .await?;
        let user_op_hash = self
            .bundler
            .client()
            .request("eth_sendUserOperation", (user_op, ENTRYPOINT))
            .await?;

        println!("Submitted user operation: {:?}", user_op_hash);
        Ok(user_op_hash)
    }
}
