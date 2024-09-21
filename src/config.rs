//! Configuration for the execution module

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

use crate::erc7579::Execution;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub entrypoint: Address,
    pub executions: Vec<Execution>,
}
