use crate::accounts::safe::Safe7579Data;
use crate::smart_account::RootProviderType;
use alloy::primitives::Address;
use alloy::rpc::types::SendUserOperation;
use async_trait::async_trait;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, clap::ValueEnum)]
pub enum AccountType {
    Safe7579,
    Kernel,
}

impl AccountType {
    pub fn create_account_builder(
        &self,
    ) -> eyre::Result<Box<dyn AccountBuilder>> {
        let file = File::open(data_path)?;
        let reader = BufReader::new(file);

        match self {
            AccountType::Safe7579 => {
                let data: Safe7579Data = serde_json::from_reader(reader)?;
                Ok(Box::new(data))
            }
            AccountType::Kernel => {
                let data: KernelData = serde_json::from_reader(reader)?;
                Ok(Box::new(data))
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct KernelData {
    // TODO add appropriate fields
}

#[async_trait]
pub trait AccountBuilder {
    async fn build_user_operations(
        self: Box<Self>,
        provider: &RootProviderType,
        entry_point: Address,
    ) -> eyre::Result<SendUserOperation>;
}

// should be moved to Kernel account file
#[async_trait]
impl AccountBuilder for KernelData {
    async fn build_user_operations(
        self: Box<Self>,
        _provider: &RootProviderType,
        _entry_point: Address,
    ) -> eyre::Result<SendUserOperation> {
        todo!()
    }
}
