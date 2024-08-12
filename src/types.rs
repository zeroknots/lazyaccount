use alloy::network::{Ethereum, EthereumWallet};
use alloy::providers::fillers::{
    ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller,
};

pub type HttpProvider<'a> = alloy::providers::fillers::FillProvider<
    alloy::providers::fillers::JoinFill<
        alloy::providers::Identity,
        alloy::providers::fillers::WalletFiller<&'a EthereumWallet>,
    >,
    alloy::providers::RootProvider<alloy::transports::http::Http<alloy::transports::http::Client>>,
    alloy::transports::http::Http<alloy::transports::http::Client>,
    alloy::network::Ethereum,
>;
pub type RootProviderType<'a> =
    alloy::providers::RootProvider<alloy::transports::http::Http<alloy::transports::http::Client>>;

pub type Foo<'a> = FillProvider<
    JoinFill<
        JoinFill<
            JoinFill<JoinFill<alloy::providers::Identity, GasFiller>, NonceFiller>,
            ChainIdFiller,
        >,
        WalletFiller<&'a EthereumWallet>,
    >,
    alloy::providers::RootProvider<alloy::transports::http::Http<alloy::transports::http::Client>>,
    alloy::transports::http::Http<alloy::transports::http::Client>,
    alloy::network::Ethereum,
>;
