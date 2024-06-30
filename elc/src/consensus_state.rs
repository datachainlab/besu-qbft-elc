use crate::errors::Error;
use crate::internal_prelude::*;
use crate::types::{Address, H256};
use besu_qbft_proto::ibc::lightclients::qbft::v1::ConsensusState as RawConsensusState;
use light_client::types::{proto::google::protobuf::Any as ProtoAny, Any, Time};
use prost::Message;
use serde::{Deserialize, Serialize};

pub const BESU_QBFT_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.lightclients.qbft.v1.ConsensusState";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusState {
    pub timestamp: Time,
    pub root: H256,
    pub validators: Vec<Address>,
}

impl From<ConsensusState> for RawConsensusState {
    fn from(value: ConsensusState) -> Self {
        RawConsensusState {
            timestamp: value.timestamp.as_unix_timestamp_secs() as u64,
            root: value.root.to_be_bytes_vec(),
            validators: value.validators.iter().map(|v| v.to_vec()).collect(),
        }
    }
}

impl TryFrom<RawConsensusState> for ConsensusState {
    type Error = Error;

    fn try_from(value: RawConsensusState) -> Result<Self, Self::Error> {
        Ok(ConsensusState {
            timestamp: Time::from_unix_timestamp_nanos(value.timestamp as u128 * 1_000_000_000)?,
            root: H256::try_from_be_slice(&value.root)
                .ok_or_else(|| Error::InvalidConsensusStateRootSize(value.root.len()))?,
            validators: value
                .validators
                .iter()
                .map(|v| v.as_slice().try_into().unwrap())
                .collect(),
        })
    }
}

impl TryFrom<Any> for ConsensusState {
    type Error = Error;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            BESU_QBFT_CONSENSUS_STATE_TYPE_URL => RawConsensusState::decode(&*value.value)
                .map_err(Error::Decode)
                .and_then(Self::try_from),
            url => Err(Error::UnexpectedClientType(url.to_string())),
        }
    }
}

impl From<ConsensusState> for Any {
    fn from(value: ConsensusState) -> Self {
        ProtoAny {
            type_url: BESU_QBFT_CONSENSUS_STATE_TYPE_URL.to_string(),
            value: RawConsensusState::from(value).encode_to_vec(),
        }
        .into()
    }
}

impl ConsensusState {
    pub fn validate(&self) -> Result<(), Error> {
        if self.root.as_uint().is_zero() {
            return Err(Error::InvalidConsensusStateZeroRoot);
        }
        Ok(())
    }
}
