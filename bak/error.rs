#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Execution value should feet into U256")]
    ExecutionValueOverflow,
    #[error("No validator address provided")]
    ValidatorNotFound,
}
