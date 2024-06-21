use core::time::Duration;

use crate::{
    internal_prelude::*,
    types::{Address, H256},
};
use displaydoc::Display;
use light_client::{types::Time, LightClientSpecificError};

#[derive(Debug, Display)]
pub enum Error {
    /// invalid client state: height is zero
    InvalidClientStateZeroHeight,
    /// invalid client state: ibc store address is zero
    InvalidClientStateZeroIbcStoreAddress,
    /// invalid consensus state: state root is zero
    InvalidConsensusStateZeroRoot,
    /// invalid consensus state: state root size is not 32 but {0}
    InvalidConsensusStateRootSize(usize),

    /// invalid header: trusted height is zero
    InvalidHeaderZeroTrustedHeight,

    /// invalid rlp format: not list: `{0:?}``
    InvalidRLPFormatNotList(Vec<u8>),

    /// invalid state root length: `{0}`
    InvalidStateRootLength(usize),
    /// invalid block number length: `{0}`
    InvalidBlockNumberLength(usize),
    /// invalid block timestamp length: `{0}`
    InvalidBlockTimestampLength(usize),
    /// invalid validator address length: `{0}`
    InvalidValidatorAddressLength(usize),

    /// from uint to u64 error: `{0}`
    FromUint64Error(ruint::FromUintError<u64>),
    /// from uint to u128 error: `{0}`
    FromUint128Error(ruint::FromUintError<u128>),

    /// unexpected client type: `{0}`
    UnexpectedClientType(String),

    /// account not found: state_root={0:?} address={1:?}
    AccountNotFound(H256, Address),
    /// account storage root mismatch: expected={0:?} actual={1:?}
    AccountStorageRootMismatch(H256, H256),
    /// invalid account storage root: {0:?}
    InvalidAccountStorageRoot(Vec<u8>),

    /// out of trusting period: current_timestamp={current_timestamp} trusting_period_end={trusting_period_end}
    OutOfTrustingPeriod {
        current_timestamp: Time,
        trusting_period_end: Time,
    },
    /// header is coming from future: current_timestamp={current_timestamp} clock_drift={clock_drift:?} header_timestamp={header_timestamp}
    HeaderFromFuture {
        current_timestamp: Time,
        clock_drift: Duration,
        header_timestamp: Time,
    },
    /// invalid header extra size: `{0}`
    InvalidHeaderExtraSize(usize),
    /// invalid header extra: contains committed seals: header={0:?}
    HeaderExtraContainsCommittedSeals(Vec<u8>),

    /// lcp time error: `{0}`
    Time(light_client::types::TimeError),
    /// lcp commitments error: `{0}`
    Commitments(light_client::commitments::Error),
    /// ethereum light client error: `{0}`
    EthereumLightClient(ethereum_light_client_verifier::errors::Error),
    /// proto decode error: `{0}`
    Decode(prost::DecodeError),
    /// rlp decode error: `{0}`
    Rlp(rlp::DecoderError),
}

impl LightClientSpecificError for Error {}

impl From<light_client::commitments::Error> for Error {
    fn from(value: light_client::commitments::Error) -> Self {
        Self::Commitments(value)
    }
}

impl From<light_client::types::TimeError> for Error {
    fn from(value: light_client::types::TimeError) -> Self {
        Self::Time(value)
    }
}

impl From<ethereum_light_client_verifier::errors::Error> for Error {
    fn from(value: ethereum_light_client_verifier::errors::Error) -> Self {
        Self::EthereumLightClient(value)
    }
}

impl From<rlp::DecoderError> for Error {
    fn from(value: rlp::DecoderError) -> Self {
        Self::Rlp(value)
    }
}
