use thiserror::Error;

#[derive(Error, Debug)]
pub enum TypeSafeError {
    #[error("ReqwestError")]
    ReqwestError(#[from] reqwest::Error),
}

pub type Result<T, E = TypeSafeError> = core::result::Result<T, E>;
