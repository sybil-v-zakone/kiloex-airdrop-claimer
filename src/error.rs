use alloy::{
    consensus::TxType,
    hex,
    network::{Ethereum, UnbuiltTransactionError},
    providers::{MulticallError, PendingTransactionError},
    transports::{RpcError as RpcErr, TransportErrorKind},
};
use std::{string::ParseError, time::SystemTimeError};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Sign(#[from] alloy::signers::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error(transparent)]
    Timestamp(#[from] SystemTimeError),

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    #[error("Failed to parse proof: {0}")]
    ProofParse(#[from] serde_json::Error),

    #[error("transaction was sent but failed")]
    TransactionFailed,

    #[error(transparent)]
    Rpc(#[from] RpcErr<TransportErrorKind>),

    #[error(transparent)]
    PendingTx(#[from] PendingTransactionError),

    #[error(transparent)]
    Parse(#[from] ParseError),

    #[error(transparent)]
    FromHex(#[from] hex::FromHexError),

    #[error(transparent)]
    UnbuiltTx(#[from] Box<UnbuiltTransactionError<Ethereum>>),

    #[error("tx type `{0}` is not supported")]
    UnexpectedTxType(TxType),

    #[error(transparent)]
    Contract(#[from] alloy::contract::Error),

    #[error(transparent)]
    Multicall(#[from] MulticallError),
}
