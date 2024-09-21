use alloy::primitives::{Bytes, FixedBytes};
use alloy::sol;
use alloy::sol_types::{SolCall, SolValue};
use serde::{Deserialize, Serialize};

pub const SINGLE_EXECUTION_MODE: ModeCode = ModeCode(FixedBytes([0x00; 32]));
pub const BATCH_EXECUTION_MODE: ModeCode = ModeCode({
    let mut bytes = [0x00; 32];
    bytes[0] = 0x01;
    FixedBytes(bytes)
});

sol! {
    #[derive(Debug, PartialEq, Eq)]
    type ModeCode is bytes32;
    type CallType is bytes1;
    type ExecType is bytes1;
    type ModeSelector is bytes4;
    type ModePayload is bytes22;

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Execution {
        address target;
        uint256 value;
        bytes callData;
    }

    #[derive(Debug, PartialEq, Eq)]
    contract ERC7579Account {
        function execute(ModeCode mode, bytes calldata executionCalldata) external;
        function installModule(uint256 moduleTypeId, address module, bytes calldata initData) external;
        function uninstallModule(uint256 moduleTypeId, address module, bytes calldata deInitData) external;
        function isModuleInstalled(uint256 moduleTypeId, address module, bytes calldata additionalContext) external view returns (bool);
        function accountId() external view returns (string memory accountImplementationId);
    }
}

pub trait ExecutionBuilder {
    fn encode_executions(self) -> Bytes;
}

impl ExecutionBuilder for Vec<Execution> {
    fn encode_executions(self) -> Bytes {
        match self.len() {
            1 => {
                let mode = SINGLE_EXECUTION_MODE;
                let execution_data = Execution::abi_encode_packed(&self[0]);

                let calldata = ERC7579Account::executeCall {
                    mode: mode.into(),
                    executionCalldata: execution_data.into(),
                };
                Bytes::from(calldata.abi_encode())
            }
            _ => {
                let mode = BATCH_EXECUTION_MODE;
                let mut result = Vec::new();
                for execution in self {
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
    }
}