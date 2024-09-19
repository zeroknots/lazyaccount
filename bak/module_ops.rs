use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, clap::ValueEnum)]
pub enum ModuleAction {
    Install,
    Uninstall,
}

//     fn install_module(
//         module_type: Vec<U256>,
//         module: Address,
//         init_data: Bytes,
//     ) -> ERC7579Account::ERC7579AccountCalls {
//         match module_type.len() {
//             0 => {
//                 panic!("No module type to encode")
//             }
//             1 => ERC7579Account::ERC7579AccountCalls::installModule(
//                 ERC7579Account::installModuleCall {
//                     moduleTypeId: module_type[0],
//                     module,
//                     initData: init_data,
//                 },
//             ),
//             _ => {
//                 panic!("Multiple module types not supported")
//             }
//         }
//     }
