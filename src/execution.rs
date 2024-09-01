use crate::erc4337::{
    create_default_packed_user_operation, get_nonce, ERC7579Account, Execution, ModeCode,
    BATCH_EXECUTION_MODE, SINGLE_EXECUTION_MODE,
};
use crate::smart_account::RootProviderType;
use alloy::primitives::{Address, Bytes};
use alloy::rpc::types::SendUserOperation;
use alloy::sol_types::{SolCall, SolValue};

trait EncodeExecution {
    fn encode_execution(self) -> Bytes;
}

impl EncodeExecution for Vec<Execution> {
    fn encode_execution(self) -> Bytes {
        let mode: ModeCode;
        let mut result: Vec<u8> = Vec::new();

        match self.len() {
            0 => {
                panic!("No executions to encode")
            }
            1 => {
                mode = SINGLE_EXECUTION_MODE;
                let execution_data = Execution::abi_encode_packed(&self[0]);
                result.extend(execution_data.into_iter());
            }
            _ => {
                mode = BATCH_EXECUTION_MODE;
                for execution in self {
                    let execution_data = Execution::abi_encode(&execution);
                    result.extend(execution_data);
                }
            }
        }

        let calldata = ERC7579Account::executeCall {
            mode: mode.into(),
            executionCalldata: result.into(),
        };
        Bytes::from(calldata.abi_encode())
    }
}

pub async fn prepare_user_operation(
    provider: &RootProviderType,
    entry_point: &Address,
    validator_module: &Address,
    sender: &Address,
    executions: Vec<Execution>,
    // _signature: Bytes, // TODO mostly the signature would also be required
) -> eyre::Result<SendUserOperation> {
    // TODO verify that all necessary steps required for the correct execution where done

    let mut packed_operation = create_default_packed_user_operation();

    packed_operation.nonce = get_nonce(
        provider,
        entry_point.clone(),
        validator_module.clone(),
        sender.clone(),
    )
    .await?;
    packed_operation.sender = sender.clone();
    packed_operation.call_data = executions.encode_execution();

    Ok(SendUserOperation::EntryPointV07(packed_operation))
}
