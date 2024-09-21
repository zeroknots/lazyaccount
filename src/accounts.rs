use alloy::network::Network;
use alloy::primitives::{Address, Bytes, FixedBytes, Log, U256};
use alloy::providers::Provider;
use alloy::rpc::types::{PackedUserOperation, TransactionReceipt};
use alloy::sol_types::{SolCall, SolValue};
use alloy::transports::Transport;
use alloy_provider::ext::Erc4337Api;
use alloy_provider::ProviderBuilder;

use async_trait::async_trait;

use super::erc7579::Execution;
use crate::cli::BaseArgs;
use crate::erc4337::{EntryPointApi, PackedUserOperationBuilder, ENTRYPOINT};
use crate::erc7579::{ERC7579Account, ERC7579ViewAccountApi, ExecutionBuilder};
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

    // TODO: support multiple validators
    pub async fn get_nonce(&self) -> Result<U256> {
        let validator = self.validators[0];
        let key = crate::address_to_key(&validator);
        self.rpc.get_nonce(self.account_address, key).await
    }

    pub async fn execute(
        &self,
        executions: Vec<Execution>,
        _validator_index: usize,
    ) -> Result<FixedBytes<32>> {
        let nonce = self.get_nonce().await?;
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

    pub async fn install_module(
        &self,
        module_type_id: U256,
        module: Address,
        init_data: Bytes,
    ) -> Result<FixedBytes<32>> {
        let nonce = self.get_nonce().await?;
        println!("module_type_id: {:?}", module_type_id);
        println!("init_data: {:?}", init_data);

        let call = ERC7579Account::installModuleCall {
            moduleTypeId: module_type_id,
            module,
            initData: init_data,
        };
        let mut call_data = Vec::new();
        call_data.extend(ERC7579Account::installModuleCall::SELECTOR.to_vec());
        call_data.extend(call.moduleTypeId.abi_encode());
        call_data.extend(call.module.abi_encode());

        // Do not encode the `initData`
        call_data.extend(call.initData.to_vec());

        // let call_data = call.abi_encode();
        let user_op = PackedUserOperation::default()
            .with_call_data(Bytes::from(call_data))
            .with_sender(self.account_address)
            .with_nonce(nonce);

        println!(
            "Submitting {:?}",
            serde_json::to_string_pretty(&user_op)
                .unwrap()
                .replace("\n", "")
        );

        let user_op_hash = self
            .bundler
            .client()
            .request("eth_sendUserOperation", (user_op, ENTRYPOINT))
            .await?;

        println!("Submitted user operation: {:?}", user_op_hash);
        Ok(user_op_hash)
    }

    pub async fn is_module_installed(
        &self,
        module_type_id: U256,
        module: Address,
        additional_context: Bytes,
    ) -> Result<bool> {
        self.rpc
            .is_module_installed(
                self.account_address,
                module_type_id,
                module,
                additional_context,
            )
            .await
    }

    pub async fn account_id(&self, module_manager: Address) -> Result<String> {
        self.rpc.account_id(module_manager).await
    }
}

/// TODO: fix alloy issue upstream
/// Represents the receipt of a user operation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationReceipt {
    pub user_operation: PackedUserOperation,
    pub transaction_hash: Bytes,
    pub block_hash: Bytes,
    pub block_number: U256,
    pub entry_point: Address,
}
