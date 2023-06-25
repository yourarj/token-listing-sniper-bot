use ethers::{providers::ProviderError, signers::WalletError};
use std::env;

use thiserror::Error;

/// Main application Error
#[derive(Error, Debug)]
pub enum BlockBotError {
    #[error("Envionment setup error")]
    EnvSetupFailed(#[from] EnvSetUpError),
}

#[derive(Error, Debug)]
pub enum EnvSetUpError {
    #[error("enviroment variable {0} set")]
    EnvVarNotFound(String, env::VarError),
    // #[error("enviroment variable {0} set")]
    // EllipticCurve(#[from] elliptic_curve::Error),
    #[error("Invalid hex adress")]
    InvalidAddress(#[from] rustc_hex::FromHexError),
    #[error("Invalid provider")]
    InvalidProvider(#[from] ProviderError),
    #[error("Invalid Wallet")]
    InvalidWallet(#[from] WalletError),
}
