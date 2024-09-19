use crate::smart_account::RootProviderType;
use alloy::primitives::aliases::U192;
use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use alloy::rpc::types::PackedUserOperation;
use alloy::sol;

sol! {
    #[derive(Debug, PartialEq, Eq)]
    type ModeCode is bytes32;
    type CallType is bytes1;
    type ExecType is bytes1;
    type ModeSelector is bytes4;
    type ModePayload is bytes22;

    #[derive(Debug, PartialEq, Eq)]
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
    }
}

sol! {
    #[sol(rpc)]
    contract EntryPoint {
        function getNonce(address sender, uint192 key) external view returns (uint256 nonce);
    }
}

pub async fn get_nonce(
    provider: &RootProviderType,
    entry_point: Address,
    validator_module: Address,
    sender: Address,
) -> eyre::Result<U256> {
    let mut key_bytes = [0u8; 24];
    key_bytes[4..24].copy_from_slice(&validator_module.as_slice());
    let key = U192::from_be_bytes(key_bytes);

    let key = key & U192::MAX;
    let contract = EntryPoint::new(entry_point, provider.clone());
    let EntryPoint::getNonceReturn { nonce } = contract.getNonce(sender, key).call().await?;

    println!("Nonce: {:?}", nonce);

    let nonce = U256::from(0);
    Ok(nonce)
}

pub const SINGLE_EXECUTION_MODE: ModeCode = ModeCode(FixedBytes([0x00; 32]));
pub const BATCH_EXECUTION_MODE: ModeCode = ModeCode({
    let mut bytes = [0x00; 32];
    bytes[0] = 0x01;
    FixedBytes(bytes)
});

