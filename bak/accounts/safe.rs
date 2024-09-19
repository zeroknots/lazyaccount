use crate::account_builder::AccountBuilder;
use crate::erc4337::{create_default_packed_user_operation, get_nonce};
use crate::error::AppError;
use crate::smart_account::RootProviderType;
use alloy::primitives::{address, bytes, Address, Bytes, B256, U256};
use alloy::rpc::types::SendUserOperation;
use alloy::sol;
use alloy::sol_types::{SolCall, SolType};
use async_trait::async_trait;
use serde::Deserialize;

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

#[allow(dead_code)]
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

#[derive(Debug)]
pub struct Safe7579CreationData {
    pub proxy_call: Bytes,
    pub factory_initializer: Bytes,
}

#[derive(Deserialize, Debug)]
pub struct Safe7579Data {
    salt: B256,
    owners: Vec<Address>,
    validator: Vec<Address>,
    safe_account: Option<Address>,
}

#[async_trait]
impl AccountBuilder for Safe7579Data {
    async fn build_user_operations(
        self: Box<Self>,
        provider: &RootProviderType,
        entry_point: Address,
    ) -> eyre::Result<SendUserOperation> {
        let (init_code, safe7579_account) = make_account(
            provider,
            self.salt.clone(),
            self.safe_account.clone(),
            self.owners.clone(),
            self.validator.clone(),
        )
        .await?;

        let mut packed_operation = create_default_packed_user_operation();

        // TODO consider to move population logic out from here or think about some generic interface
        packed_operation.nonce = get_nonce(
            provider,
            entry_point,
            self.validator
                .get(0)
                .ok_or(AppError::ValidatorNotFound)?
                .clone(),
            safe7579_account,
        )
        .await?;
        packed_operation.sender = safe7579_account;
        packed_operation.factory = SAFE_PROXY_FACTORY;
        packed_operation.factory_data = init_code;

        Ok(SendUserOperation::EntryPointV07(packed_operation))
    }
}

async fn make_account(
    provider: &RootProviderType,
    salt: B256,
    safe_account: Option<Address>,
    owners: Vec<Address>,
    validators: Vec<Address>,
) -> eyre::Result<(Bytes, Address)> {
    let Safe7579CreationData {
        proxy_call,
        factory_initializer,
    } = make_factory_data(provider, salt, safe_account, owners, validators).await?;

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

pub async fn make_factory_data(
    provider: &RootProviderType,
    salt: B256,
    safe_account: Option<Address>,
    owners: Vec<Address>,
    validators: Vec<Address>,
) -> eyre::Result<Safe7579CreationData> {
    let safe_account = if let Some(addr) = safe_account {
        addr
    } else {
        // TODO handle the case when the Safe account have not been created
        Address::ZERO
    };

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
        singleton: safe_account, // TODO double check whether it is the correct place for the user's safe account address
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

    Ok(Safe7579CreationData {
        proxy_call,
        factory_initializer: factory_initializer_bytes,
    })
}
