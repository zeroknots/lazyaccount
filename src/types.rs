//! Some types used in the project

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

use crate::erc7579::Execution;

/// Configuration for the execution file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Executions {
    pub entrypoint: Address,
    pub executions: Vec<Execution>,
}
