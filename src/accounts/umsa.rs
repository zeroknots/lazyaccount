use alloy::primitives::{address, b256, Address, bytes, Bytes, FixedBytes, B256, U256};
use alloy::contract::SolCallBuilder;
use alloy::sol;
use std::error::Error as StdError;
use crate::types::{Foo, HttpProvider, RootProviderType};
use async_trait::async_trait;
use std::sync::Arc;


sol!(
    #[derive( Debug)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    UMSA,
    "src/abi/umsa.json"
);

sol!(
    #[derive( Debug)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    UMSAFactory,
    "src/abi/umsa_factory.json"
);

// sol!(
//     #[derive( Debug)]
//     #[allow(missing_docs)]
//     #[sol(rpc)]
//     UMSABootstrap,
//     "src/abi/bootstrap.json"
// );

#[derive( Debug)]
pub struct Factory {
    pub factory: Option<Arc<UMSAFactory::UMSAFactoryInstance<HttpProvider>>>,
    // pub account_impl: Option<UMSA::UMSAInstance<T,P>>,
    // pub bootstrap: Option<UMSABootstrap>,

}

#[async_trait]
pub trait FactoryDeploy<T,P> {
    async fn new( provider: Arc<RootProviderType>) -> Result<Factory<T,P>, Box<dyn StdError>>;
}


#[async_trait]
impl <T,P> FactoryDeploy for Factory<T,P>{
    async fn new( provider: Arc<RootProviderType>) -> Result<Factory<T,P>, Box<dyn StdError>> {
        let account_impl = UMSA::deploy(&provider).await?;
        let account = *account_impl.address();
        let factory = UMSAFactory::deploy(&provider, account).await?;

        Ok(Factory {
            factory: Some(factory),
            account_impl: Some(account_impl),
            // bootstrap: Some(UMSABootstrap::deploy(&provider ))
        })
    }
}



// #[async_trait]
// pub trait AccountCreation {
//     async fn get_initcode(factory: Factory) -> Bytes;
// }
//
// #[async_trait]
// impl AccountCreation for Factory {
//     async fn get_initcode(salt:B256, factory: Factory) -> Bytes {
//         Bytes::from(init_code.abi_encode())
//     }
//
// }
