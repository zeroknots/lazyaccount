use alloy::primitives::{address, b256, Address, Bytes, FixedBytes, B256, U256};
use crate::account::SmartAccount;
use alloy::sol;
use async_trait::async_trait;
use std::error::Error as StdError;

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
    #[derive(Debug)]
    struct PackedUserOperation {
        address sender;
        uint256 nonce;
        bytes initCode;
        bytes callData;
        bytes32 accountGasLimits;
        uint256 preVerificationGas;
        bytes32 gasFees;
        bytes paymasterAndData;
        bytes signature;
    }

    #[sol(rpc)]
    contract EntryPoint {
        function handleOps(PackedUserOperation[] calldata ops, address payable beneficiary) external;
        function getNonce(address sender, uint192 key) external view returns (uint256 nonce);
    }
}

const ENTRYPOINT_ADDR: Address = address!("0000000071727De22E5E9d8BAf0edAc6f37da032");

pub const SINGLE_EXECUTION_MODE: ModeCode = ModeCode(FixedBytes([0x00; 32]));
pub const BATCH_EXECUTION_MODE: ModeCode = ModeCode({
    let mut bytes = [0x00; 32];
    bytes[0] = 0x01;
    FixedBytes(bytes)
});

impl PackedUserOperation {
    pub fn new() -> PackedUserOperation {
        PackedUserOperation {
            sender: Address::default(),
            nonce: U256::from(0),
            initCode: Bytes::default(),
            callData: Bytes::default(),
            accountGasLimits: b256!(
                "0000000000000000000000000000000000000000000000000000000000000010"
            ),
            preVerificationGas: U256::from(100000),
            gasFees: b256!("0000000000000000000000000000000000000000000000000000000000000010"),
            paymasterAndData: Bytes::default(),
            signature: Bytes::default(),
        }
    }
    pub fn with_sender(mut self, sender: Address) -> Self {
        self.sender = sender;
        self
    }

    pub fn with_nonce(mut self, nonce: U256) -> Self {
        self.nonce = nonce;
        self
    }
    pub fn with_init_code(mut self, init_code: Bytes) -> Self {
        self.initCode = init_code;
        self
    }
    pub fn with_calldata(mut self, callData: Bytes) -> Self {
        self.callData = callData;
        self
    }
    pub fn with_account_gas_limits(mut self, account_gas_limits: B256) -> Self {
        self.accountGasLimits = account_gas_limits;
        self
    }
    pub fn with_pre_verification_gas(mut self, pre_verification_gas: U256) -> Self {
        self.preVerificationGas = pre_verification_gas;
        self
    }
    pub fn with_gas_fees(mut self, gas_fees: B256) -> Self {
        self.gasFees = gas_fees;
        self
    }
    pub fn with_paymaster_and_data(mut self, paymaster_and_data: Bytes) -> Self {
        self.paymasterAndData = paymaster_and_data;
        self
    }
    pub fn with_signature(mut self, signature: Bytes) -> Self {
        self.signature = signature;
        self
    }
}

//
// #[async_trait]
// pub trait Bundler {
//     async fn send_user_op(&self, userop: PackedUserOperation) -> Result<(), Box<dyn StdError>>;
// }
//
// #[async_trait]
// impl<'a> Bundler for SmartAccount<'a> {
//     async fn send_user_op(&self, userop: PackedUserOperation) -> Result<(), Box<dyn StdError>> {
//         let ep: Address = address!("0000000071727De22E5E9d8BAf0edAc6f37da032");
//         let contract = EntryPoint::new(ep, self.provider.as_ref().unwrap());
//
//         let tx_hash = contract
//             .handleOps(vec![userop], ep)
//             .gas(100000)
//             .max_fee_per_gas(200000000000)
//             .max_priority_fee_per_gas(1500000000)
//             .send()
//             .await?
//             .watch()
//             .await?;
//
//         println!("{:?}", tx_hash);
//
//         Ok(())
//     }
// }
