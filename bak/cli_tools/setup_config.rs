use super::{
    cli::{Cli, ConfigItem, ConfigSubcommand},
    setup_config_file::ConfigFile,
};
use alloy::primitives::Address;
use std::str::FromStr;

pub struct Config {
    pub profile: String,
    pub rpc_node_url: Option<String>,
    pub rpc_bundler_url: Option<String>,
    pub entry_point_addr: Option<Address>,
}

#[derive(Debug, thiserror::Error)]
pub enum SettingNotSpecifiedError {
    #[error("rpc-node-url is not specified, check your profile settings")]
    RpcNodeUrl,

    #[error("rpc-bundler-url is not specified, check your profile settings")]
    RpcBundlerUrl,

    #[error("entry-point-addr is not specified, check you profile settings")]
    EntryPointAddr,
}

impl Config {
    /// Loads configuration from CLI and configuration file.
    pub fn new(cli: &Cli) -> eyre::Result<Self> {
        let profile_name = cli.profile.clone();
        let config_file = ConfigFile::read(profile_name.clone())?;

        Ok(Self {
            profile: profile_name,
            rpc_node_url: cli
                .rpc_node_url
                .clone()
                .or(config_file.rpc_node_url)
                .or(config_file.rpc_bundler_url.clone()), // For the case when the bundler RPC would also support other RPC calls except erc4337
            entry_point_addr: cli
                .entry_point_addr
                .clone()
                .or(config_file.entry_point_addr),
            rpc_bundler_url: cli.rpc_bundler_url.clone().or(config_file.rpc_bundler_url),
        })
    }

    pub fn get_rpc_node_url(&self) -> Result<&str, SettingNotSpecifiedError> {
        self.rpc_node_url
            .as_deref()
            .ok_or(SettingNotSpecifiedError::RpcNodeUrl)
    }

    pub fn get_entry_point_addr(&self) -> Result<&Address, SettingNotSpecifiedError> {
        self.entry_point_addr
            .as_ref()
            .ok_or(SettingNotSpecifiedError::EntryPointAddr)
    }

    pub fn get_rpc_bundler_url(&self) -> Result<&str, SettingNotSpecifiedError> {
        self.rpc_bundler_url
            .as_deref()
            .ok_or(SettingNotSpecifiedError::RpcBundlerUrl)
    }
}

pub fn run_config_command(cli: &Cli, cmd: &ConfigSubcommand) -> eyre::Result<()> {
    let mut cfg_file = ConfigFile::read(cli.profile.clone())?;

    match cmd {
        ConfigSubcommand::Get { key } => match key {
            Some(key) => match key {
                ConfigItem::RpcNodeUrl => {
                    println!("rpc-node-url = {:?}", cfg_file.rpc_node_url)
                }
                ConfigItem::EntryPointAddr => {
                    println!("entry-point-addr = {:?}", cfg_file.entry_point_addr)
                }
                ConfigItem::RpcBundlerUrl => {
                    println!("rpc-bundler-url = {:?}", cfg_file.rpc_node_url)
                }
            },
            None => {
                println!("{}", toml::to_string_pretty(&cfg_file)?);
            }
        },
        ConfigSubcommand::Set { key, value } => {
            match key {
                ConfigItem::RpcNodeUrl => cfg_file.rpc_node_url = Some(value.parse()?),
                ConfigItem::EntryPointAddr => {
                    cfg_file.entry_point_addr = Some(Address::from_str(value)?)
                }
                ConfigItem::RpcBundlerUrl => cfg_file.rpc_bundler_url = Some(value.parse()?),
            };
            cfg_file.write(cli.profile.clone())?;
        }
        ConfigSubcommand::Unset { key } => {
            match key {
                ConfigItem::RpcNodeUrl => cfg_file.rpc_node_url = None,
                ConfigItem::EntryPointAddr => cfg_file.entry_point_addr = None,
                ConfigItem::RpcBundlerUrl => cfg_file.rpc_bundler_url = None,
            };
            cfg_file.write(cli.profile.clone())?;
        }
    }

    Ok(())
}
