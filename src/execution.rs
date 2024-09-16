use crate::erc4337::{
    create_default_packed_user_operation, get_nonce, ERC7579Account, EntryPoint, Execution,
    ModeCode, BATCH_EXECUTION_MODE, SINGLE_EXECUTION_MODE,
};
use crate::smart_account::RootProviderType;
use alloy::primitives::aliases::U192;
use alloy::primitives::{Address, Bytes, U256};
use alloy::rpc::types::{PackedUserOperation, SendUserOperation};
use alloy::sol_types::{SolCall, SolValue};
use async_trait::async_trait;
use std::convert::Infallible;
use std::error::Error;
use std::fmt::Debug;
use std::marker::PhantomData;

pub struct SingleExecution;
pub struct BatchExecution;

pub struct UserOpsBuilder<State> {
    _phantom: PhantomData<State>,
    executions: Vec<Execution>,
}

impl UserOpsBuilder<()> {
    pub(crate) fn new() -> Self {
        Self {
            _phantom: PhantomData,
            executions: Vec::new(),
        }
    }

    pub fn add_execution(
        mut self,
        target: Address,
        value: U256,
        calldata: Bytes,
    ) -> UserOpsBuilder<SingleExecution> {
        self.executions.push(Execution {
            target,
            value,
            callData: calldata,
        });

        UserOpsBuilder {
            _phantom: PhantomData,
            executions: self.executions,
        }
    }
}

impl UserOpsBuilder<SingleExecution> {
    pub(crate) fn add_execution(
        mut self,
        target: Address,
        value: U256,
        calldata: Bytes,
    ) -> UserOpsBuilder<BatchExecution> {
        self.executions.push(Execution {
            target,
            value,
            callData: calldata,
        });

        UserOpsBuilder {
            _phantom: PhantomData,
            executions: self.executions,
        }
    }
}

impl UserOpsBuilder<BatchExecution> {
    pub(crate) fn add_execution(
        mut self,
        target: Address,
        value: U256,
        calldata: Bytes,
    ) -> UserOpsBuilder<BatchExecution> {
        self.executions.push(Execution {
            target,
            value,
            callData: calldata,
        });
        self
    }
}

trait HasAnyExecution {}
trait HasSingleExecution {}
impl HasAnyExecution for SingleExecution {}
impl HasSingleExecution for SingleExecution {}
trait HasBatchExecution {}
impl HasSingleExecution for BatchExecution {}
impl HasAnyExecution for BatchExecution {}
impl HasBatchExecution for BatchExecution {}

pub trait EncodeExecutions {
    fn encode_executions(self) -> Bytes;
}

impl EncodeExecutions for UserOpsBuilder<SingleExecution> {
    fn encode_executions(self) -> Bytes {
        let mode: ModeCode = SINGLE_EXECUTION_MODE;
        let mut result: Vec<u8> = Vec::new();
        let execution_data = Execution::abi_encode_packed(&self.executions[0]);
        result.extend(execution_data.into_iter());

        let calldata = ERC7579Account::executeCall {
            mode: mode.into(),
            executionCalldata: result.into(),
        };
        Bytes::from(calldata.abi_encode())
    }
}

impl EncodeExecutions for UserOpsBuilder<BatchExecution> {
    fn encode_executions(self) -> Bytes {
        let mode: ModeCode = BATCH_EXECUTION_MODE;
        let mut result: Vec<u8> = Vec::new();
        for execution in self.executions {
            let execution_data = Execution::abi_encode(&execution);
            result.extend(execution_data);
        }
        let calldata = ERC7579Account::executeCall {
            mode: mode.into(),
            executionCalldata: result.into(),
        };
        Bytes::from(calldata.abi_encode())
    }
}

#[async_trait]
pub trait BuildUserOp: EncodeExecutions {
    async fn build(
        self,
        sender: Address,
        nonce: &mut impl NonceProvider,
    ) -> eyre::Result<SendUserOperation>;
}

#[async_trait]
impl BuildUserOp for UserOpsBuilder<SingleExecution> {
    async fn build(
        mut self,
        sender: Address,
        nonce: &mut impl NonceProvider,
    ) -> eyre::Result<SendUserOperation> {
        Ok(SendUserOperation::EntryPointV07(PackedUserOperation {
            nonce: nonce.nonce().await?,
            sender,
            call_data: self.encode_executions(),
            ..create_default_packed_user_operation()
        }))
    }
}

#[async_trait]
impl BuildUserOp for UserOpsBuilder<BatchExecution> {
    async fn build(
        mut self,
        sender: Address,
        nonce: &mut impl NonceProvider,
    ) -> eyre::Result<SendUserOperation> {
        Ok(SendUserOperation::EntryPointV07(PackedUserOperation {
            nonce: nonce.nonce().await?,
            sender,
            call_data: self.encode_executions(),
            ..create_default_packed_user_operation()
        }))
    }
}

/// The nonce provider trait is defined by a single operation [`Self::nonce`].
#[async_trait]
pub trait NonceProvider: Send + Sync {
    /// The associated error type for the nonce fetching operation.
    type Error: Error + Send + Sync + 'static;

    /// Fetches a nonce from an [`EntryPoint`] smart contract.
    async fn nonce(&mut self) -> Result<U256, Self::Error>;
}

/// Provides nonce by fetching it from a smart contract.
pub struct Erc4337Nonce {
    provider: RootProviderType,
    entry_point: Address,
    validator_module: Address,
    sender: Address,
}

impl Erc4337Nonce {
    pub fn new(
        provider: RootProviderType,
        entry_point: Address,
        validator_module: Address,
        sender: Address,
    ) -> Self {
        Self {
            provider,
            entry_point,
            validator_module,
            sender,
        }
    }
}

#[async_trait]
impl NonceProvider for Erc4337Nonce {
    type Error = alloy_contract::Error;

    async fn nonce(&mut self) -> Result<U256, Self::Error> {
        let mut key_bytes = [0u8; 24];
        key_bytes[4..24].copy_from_slice(self.validator_module.as_slice());
        let key = U192::from_be_bytes(key_bytes);

        let key = key & U192::MAX;
        let contract = EntryPoint::new(self.entry_point, self.provider.clone());
        let EntryPoint::getNonceReturn { nonce } =
            contract.getNonce(self.sender, key).call().await?;

        Ok(nonce)
    }
}

#[cfg(test)]
#[async_trait]
impl NonceProvider for U256 {
    type Error = Infallible;

    async fn nonce(&mut self) -> Result<U256, Self::Error> {
        Ok(*self)
    }
}

pub async fn prepare_user_operation(
    sender: Address,
    nonce: &mut impl NonceProvider,
    mut executions: Vec<Execution>,
    // _signature: Bytes, // TODO mostly the signature would also be required
) -> eyre::Result<SendUserOperation> {
    // TODO verify that all necessary steps required for the correct execution where done
    let e = executions.pop().unwrap();
    let mut builder = UserOpsBuilder::new().add_execution(e.target, e.value, e.callData);
    builder.build(sender, nonce).await
}
