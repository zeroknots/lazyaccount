// use alloy_contract::SolCallBuilder;
// use alloy_primitives::{address, keccak256, Address, FixedBytes, Bytes, b256, B256, U256};
// use alloy_provider::Provider;
// use alloy_sol_types::{sol_data::*, SolInterface};
// use alloy_sol_types::{abi, sol};
// use alloy_sol_types::{sol_data::*, SolValue};
// use alloy_sol_types::SolCall;
//
// use alloy_provider::network;
use crate::account::SmartAccount;
use crate::erc4337::{
    ERC7579Account, Execution, ModeCode, BATCH_EXECUTION_MODE, SINGLE_EXECUTION_MODE,
};
use alloy::contract::SolCallBuilder;
use alloy::network::{Ethereum, EthereumWallet};
use alloy::primitives::{address, Address, Bytes, U256};
use alloy::providers::ProviderBuilder;
use alloy::sol;
use alloy::sol_types::{abi, sol_data::*, sol_data::*, SolCall, SolInterface, SolValue};
use alloy::transports::http::reqwest::Url;
use async_trait::async_trait;
use serde::Deserialize;
use std::error::Error as StdError;
use std::sync::Arc;

pub trait ExecutionHelper {
    fn encode_execution(&self, executions: Vec<Execution>) -> Bytes;
    fn install_module(
        &self,
        module_type: Vec<U256>,
        module: Address,
        init_data: Bytes,
    ) -> ERC7579Account::ERC7579AccountCalls;
}

impl<'a> ExecutionHelper for SmartAccount<'a> {
    fn encode_execution(&self, executions: Vec<Execution>) -> Bytes {
        let mode: ModeCode;
        let mut result: Vec<u8> = Vec::new();

        match executions.len() {
            0 => {
                panic!("No executions to encode")
            }
            1 => {
                let tmp = Execution::abi_encode_packed(&executions[0]);
                result.extend(tmp);
                mode = SINGLE_EXECUTION_MODE;
            }
            _ => {
                mode = BATCH_EXECUTION_MODE;
                for execution in executions {
                    let tmp = Execution::abi_encode(&execution);
                    result.extend(tmp);
                }
            }
        }

        let calldata = ERC7579Account::executeCall {
            mode: mode.into(),
            executionCalldata: result.into(),
        };
        Bytes::from(calldata.abi_encode())
    }

    fn install_module(
        &self,
        module_type: Vec<U256>,
        module: Address,
        init_data: Bytes,
    ) -> ERC7579Account::ERC7579AccountCalls {
        match module_type.len() {
            0 => {
                panic!("No module type to encode")
            }
            1 => ERC7579Account::ERC7579AccountCalls::installModule(
                ERC7579Account::installModuleCall {
                    moduleTypeId: module_type[0],
                    module,
                    initData: init_data,
                },
            ),
            _ => {
                panic!("Multiple module types not supported")
            }
        }
    }
}
