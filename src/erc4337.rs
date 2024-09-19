use alloy::network::Network;
use alloy::primitives::aliases::U192;
use alloy::primitives::{address, Address, Bytes, U256};
use alloy::providers::Provider;
use alloy::rpc::types::PackedUserOperation;
use alloy::sol;
use alloy::transports::Transport;

sol! {
    #[sol(rpc)]
    contract EntryPoint {
        function getNonce(address sender, uint192 key) external view returns (uint256 nonce);
    }
}
const ENTRYPOINT: Address = address!("0000000071727De22E5E9d8BAf0edAc6f37da032");

pub trait EntryPointApi<N, T>: Send + Sync {
    async fn get_nonce(&self, sender: Address, key: U192) -> eyre::Result<U256>;
}

impl<N, T, P> EntryPointApi<N, T> for P
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N>,
{
    async fn get_nonce(&self, account: Address, key: U192) -> eyre::Result<U256> {
        let contract = EntryPoint::new(ENTRYPOINT, self);
        let EntryPoint::getNonceReturn { nonce } = contract.getNonce(account, key).call().await?;
        println!("Nonce: {:?}", nonce);
        Ok(nonce)
    }
}

pub trait PackedUserOperationBuilder {
    fn default() -> Self;
    fn with_sender(self, sender: Address) -> Self;
    fn with_nonce(self, nonce: U256) -> Self;
    fn with_call_data(self, call_data: Bytes) -> Self;
    fn set_signature(self, signature: Bytes) -> Self;
}

impl PackedUserOperationBuilder for PackedUserOperation {
    fn default() -> Self {
        PackedUserOperation {
            sender: Address::ZERO,
            nonce: U256::from(0),
            factory: Address::ZERO,
            factory_data: Bytes::default(),
            call_data: Bytes::default(),
            call_gas_limit: U256::from(10000000u64),
            verification_gas_limit: U256::from(10000000u64),
            pre_verification_gas: U256::from(10000000u64),
            max_fee_per_gas: U256::from(10000u64),
            max_priority_fee_per_gas: U256::from(10000u64),
            paymaster: Address::ZERO,
            paymaster_verification_gas_limit: U256::from(10000000u64),
            paymaster_post_op_gas_limit: U256::from(10000000u64),
            paymaster_data: Bytes::default(),
            signature: Bytes::default(),
        }
    }
    fn with_call_data(mut self, call_data: Bytes) -> Self {
        self.call_data = call_data;
        self
    }
    fn with_sender(mut self, sender: Address) -> Self {
        self.sender = sender;
        self
    }
    fn with_nonce(mut self, nonce: U256) -> Self {
        self.nonce = nonce;
        self
    }
    fn set_signature(mut self, signature: Bytes) -> Self {
        self.signature = signature;
        self
    }
}
