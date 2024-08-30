use crate::account::SmartAccount;
use crate::types::{Foo, HttpProvider, RootProviderType};
use alloy::contract::SolCallBuilder;
use alloy::primitives::{address, b256, bytes, Address, Bytes, FixedBytes, B256, U256};
use alloy::sol;
use alloy::sol_types::{abi, SolCall, SolConstructor, SolEnum, SolType};
use async_trait::async_trait;
use std::error::Error as StdError;
use std::sync::Arc;

use self::Safe7579::Safe7579Instance;
use self::Safe7579Launchpad::Safe7579LaunchpadInstance;
use self::SafeProxyFactory::SafeProxyFactoryCalls;

sol!(
    #[derive(Debug)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    Safe7579,
    "src/artifacts/safe7579.json"
);
pub const SAFE7579_ADDR: Address = address!("7579F9feedf32331C645828139aFF78d517d0001");

sol!(
    #[derive(Debug)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    Safe7579Launchpad,
    "src/artifacts/safe7579Launchpad.json"
);
pub const SAFE7579LAUNCHPAD_ADDR: Address = address!("75796e975bD270d487Be50b4e9797780360400ff");

sol!(
    #[derive(Debug)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    Safe,
    "src/artifacts/safe.json"
);
pub const SAFE_IMPL_ADDR: Address = address!("29fcB43b46531BcA003ddC8FCB67FFE91900C762");

sol!(
    #[derive(Debug)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    SafeProxyFactory,
    "src/artifacts/safeProxyFactory.json"
);
pub const SAFE_PROXY_FACTORY: Address = address!("4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67");

sol!(
    #[derive(Debug)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    SafeProxy,
    "src/artifacts/safeProxy.json"
);

sol! {
    #[derive(Debug)]
    struct PackedFactoryCall {
        address factory;
        bytes data;
    }
}

pub const EMPTY_MODULE_INIT: Safe7579Launchpad::ModuleInit = Safe7579Launchpad::ModuleInit {
    module: Address::ZERO,
    initData: bytes!(""),
};

#[async_trait]
pub trait Safe7579Helper {
    async fn make_account(
        provider: Arc<RootProviderType>,
        salt: B256,
        owners: Vec<Address>,
        validator: Vec<Address>,
    ) -> Result<(Bytes, Address), Box<dyn StdError>>;
}

#[derive(Debug)]
pub struct Safe7579HelperData {
    pub proxy_call: Bytes,
    pub factory_initializer: Bytes,
}

pub struct Safe7579HelperImpl;

#[async_trait]
impl Safe7579Helper for Safe7579HelperImpl {
    async fn make_account(
        provider: Arc<RootProviderType>,
        salt: B256,
        owners: Vec<Address>,
        validators: Vec<Address>,
    ) -> Result<(Bytes, Address), Box<dyn StdError>> {
        // let validators_init: Vec<Safe7579Launchpad::ModuleInit> = validators
        //     .into_iter()
        //     .map(|validator| Safe7579Launchpad::ModuleInit {
        //         module: validator,
        //         initData: Bytes::from(""),
        //     })
        //     .collect();

        // let safe_setup_call: Bytes = Safe7579Launchpad::initSafe7579Call {
        //     safe7579: SAFE7579_ADDR,
        //     executors: vec![EMPTY_MODULE_INIT],
        //     fallbacks: vec![EMPTY_MODULE_INIT],
        //     hooks: vec![EMPTY_MODULE_INIT],
        //     attesters: owners.clone(),
        //     threshold: 1,
        // }
        // .abi_encode()
        // .into();

        // let launchpad_init_call = Safe7579Launchpad::InitData {
        //     singleton: SAFE_IMPL_ADDR,
        //     owners,
        //     threshold: U256::from(1),
        //     setupTo: SAFE7579LAUNCHPAD_ADDR,
        //     setupData: safe_setup_call,
        //     safe7579: SAFE7579_ADDR,
        //     validators: validators_init,
        //     callData: Bytes::from(""),
        // };

        // let safe7579_launchpad = Safe7579Launchpad::new(SAFE7579LAUNCHPAD_ADDR, provider);

        // let Safe7579Launchpad::hashReturn { initHash } = safe7579_launchpad
        //     .hash(launchpad_init_call.clone())
        //     .call()
        //     .await?;

        // println!(" Init hash {:?}", initHash);

        // let factory_initializer_bytes: Bytes = Safe7579Launchpad::preValidationSetupCall {
        //     initHash,
        //     to: Address::ZERO,
        //     preInit: Bytes::from(""),
        // }
        // .abi_encode()
        // .into();

        // let proxy_call = SafeProxyFactory::createProxyWithNonceCall {
        //     _singleton: SAFE7579LAUNCHPAD_ADDR,
        //     initializer: factory_initializer_bytes.clone(),
        //     saltNonce: salt.into(),
        // }
        // .abi_encode();

        let Safe7579HelperData {
            proxy_call,
            factory_initializer,
        } = Safe7579HelperImpl::make_factory_data(provider.clone(), salt, owners, validators)
            .await?;

        let safe7579_launchpad = Safe7579Launchpad::new(SAFE7579LAUNCHPAD_ADDR, provider);

        let safe_proxy_bytecode = &SafeProxy::BYTECODE;
        let Safe7579Launchpad::predictSafeAddressReturn { safeProxy } = safe7579_launchpad
            .predictSafeAddress(
                SAFE7579LAUNCHPAD_ADDR,
                SAFE_PROXY_FACTORY,
                safe_proxy_bytecode.clone(),
                salt.into(),
                factory_initializer,
            )
            .call()
            .await?;

        println!("Predicted Account {:?}", safeProxy);
        Ok((
            Bytes::from(PackedFactoryCall::abi_encode(&PackedFactoryCall {
                factory: SAFE_PROXY_FACTORY,
                data: proxy_call,
            })),
            safeProxy,
        ))
    }
}

impl Safe7579HelperImpl {
    pub async fn make_factory_data(
        provider: Arc<RootProviderType>,
        salt: B256,
        owners: Vec<Address>,
        validators: Vec<Address>,
    ) -> Result<Safe7579HelperData, Box<dyn StdError>> {
        let validators_init: Vec<Safe7579Launchpad::ModuleInit> = validators
            .into_iter()
            .map(|validator| Safe7579Launchpad::ModuleInit {
                module: validator,
                initData: Bytes::from(""),
            })
            .collect();

        let safe_setup_call: Bytes = Safe7579Launchpad::initSafe7579Call {
            safe7579: SAFE7579_ADDR,
            executors: vec![EMPTY_MODULE_INIT],
            fallbacks: vec![EMPTY_MODULE_INIT],
            hooks: vec![EMPTY_MODULE_INIT],
            attesters: owners.clone(),
            threshold: 1,
        }
        .abi_encode()
        .into();

        let launchpad_init_call = Safe7579Launchpad::InitData {
            singleton: SAFE_IMPL_ADDR,
            owners,
            threshold: U256::from(1),
            setupTo: SAFE7579LAUNCHPAD_ADDR,
            setupData: safe_setup_call,
            safe7579: SAFE7579_ADDR,
            validators: validators_init,
            callData: Bytes::from(""),
        };

        let safe7579_launchpad = Safe7579Launchpad::new(SAFE7579LAUNCHPAD_ADDR, provider);

        let Safe7579Launchpad::hashReturn { initHash } = safe7579_launchpad
            .hash(launchpad_init_call.clone())
            .call()
            .await?;

        let factory_initializer_bytes: Bytes = Safe7579Launchpad::preValidationSetupCall {
            initHash,
            to: Address::ZERO,
            preInit: Bytes::from(""),
        }
        .abi_encode()
        .into();

        let proxy_call = SafeProxyFactory::createProxyWithNonceCall {
            _singleton: SAFE7579LAUNCHPAD_ADDR,
            initializer: factory_initializer_bytes.clone(),
            saltNonce: salt.into(),
        }
        .abi_encode()
        .into();

        Ok(Safe7579HelperData {
            proxy_call,
            factory_initializer: factory_initializer_bytes,
        })
    }
}
