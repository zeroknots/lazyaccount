use crate::{
    account_builder::AccountBuilder, cli_tools::setup_config::Config, erc4337::Execution,
    execution::prepare_user_operation, module_ops::ModuleAction,
};
use alloy::{primitives::Address, rpc::types::SendUserOperation};
use alloy_provider::{ext::Erc4337Api, ProviderBuilder};
use url::Url;

pub type RootProviderType =
    alloy::providers::RootProvider<alloy::transports::http::Http<alloy::transports::http::Client>>;

pub struct SmartAccount {
    rpc_node_provider: RootProviderType,
    rpc_bundler_provider: RootProviderType,
    entry_point: Address,
}

// TODO consider to add some trait interface because it seems like in action methods we would prepare SendUserOperation and call send_user_operation on bundler in all cases
impl SmartAccount {
    pub fn new(config: Config) -> eyre::Result<Self> {
        let rpc_url = Url::parse(config.get_rpc_node_url()?)?;

        let provider = ProviderBuilder::new().on_http(rpc_url);

        let rpc_bundler_url = Url::parse(config.get_rpc_bundler_url()?)?;
        let bundler = ProviderBuilder::new().on_http(rpc_bundler_url);

        let entry_point_addr = config.get_entry_point_addr()?.clone();

        Ok(SmartAccount {
            rpc_node_provider: provider,
            rpc_bundler_provider: bundler,
            entry_point: entry_point_addr,
        })
    }

    pub async fn create_account(
        &self,
        account_builder: Box<dyn AccountBuilder>,
    ) -> eyre::Result<()> {
        let user_operations: SendUserOperation = account_builder
            .build_user_operations(&self.rpc_node_provider, self.entry_point.clone())
            .await?;

        self.rpc_bundler_provider
            .send_user_operation(user_operations, self.entry_point.clone())
            .await?;

        Ok(())
    }

    pub async fn execute_operation(
        &self,
        sender: &Address,
        validator_module: &Address,
        execution: Vec<Execution>,
    ) -> eyre::Result<()> {
        let user_operations: SendUserOperation = prepare_user_operation(
            &self.rpc_node_provider,
            &self.entry_point,
            validator_module,
            sender,
            execution,
        )
        .await?;

        self.rpc_bundler_provider
            .send_user_operation(user_operations, self.entry_point.clone())
            .await?;

        Ok(())
    }

    // TODO finalize the interface
    pub async fn execute_module_operation(
        &self,
        _sender: &Address,
        _action: &ModuleAction,
    ) -> eyre::Result<()> {
        todo!()
    }
}
