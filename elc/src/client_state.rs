use crate::commitment::{
    calculate_ibc_commitment_storage_key, decode_eip1184_rlp_proof, keccak256,
};
use crate::internal_prelude::*;
use crate::types::{Address, H256};
use crate::{errors::Error, types::U256};
use besu_qbft_proto::ibc::{
    core::client::v1::Height as RawHeight, lightclients::qbft::v1::ClientState as RawClientState,
};
use core::time::Duration;
use ethereum_light_client_verifier::execution::ExecutionVerifier;
use light_client::types::proto::google::protobuf::Any as ProtoAny;
use light_client::types::{Any, Height};
use prost::Message;
use serde::{Deserialize, Serialize};

pub const BESU_QBFT_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.qbft.v1.ClientState";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientState {
    pub chain_id: U256,
    pub ibc_store_address: Address,
    pub latest_height: Height,
    pub trusting_period: Duration,
    pub max_clock_drift: Duration,
    #[serde(skip)]
    pub execution_verifier: ExecutionVerifier,
}

impl From<ClientState> for RawClientState {
    fn from(value: ClientState) -> Self {
        RawClientState {
            chain_id: value.chain_id.to_be_bytes_vec(),
            ibc_store_address: value.ibc_store_address.to_vec(),
            latest_height: if value.latest_height.is_zero() {
                None
            } else {
                Some(RawHeight {
                    revision_number: value.latest_height.revision_number(),
                    revision_height: value.latest_height.revision_height(),
                })
            },
            trusting_period: value.trusting_period.as_secs(),
            max_clock_drift: value.max_clock_drift.as_secs(),
        }
    }
}

impl TryFrom<RawClientState> for ClientState {
    type Error = Error;

    fn try_from(value: RawClientState) -> Result<Self, Self::Error> {
        Ok(ClientState {
            chain_id: U256::from_be_slice(&value.chain_id),
            ibc_store_address: value
                .ibc_store_address
                .as_slice()
                .try_into()
                .map_err(Error::SliceToArrayConversionError)?,
            latest_height: value.latest_height.map_or(Height::zero(), |height| {
                Height::new(height.revision_number, height.revision_height)
            }),
            trusting_period: Duration::from_secs(value.trusting_period),
            max_clock_drift: Duration::from_secs(value.max_clock_drift),
            execution_verifier: ExecutionVerifier::default(),
        })
    }
}

impl TryFrom<Any> for ClientState {
    type Error = Error;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            BESU_QBFT_CLIENT_STATE_TYPE_URL => RawClientState::decode(&*value.value)
                .map_err(Error::Decode)
                .and_then(Self::try_from),
            url => Err(Error::UnexpectedClientType(url.to_string())),
        }
    }
}

impl From<ClientState> for Any {
    fn from(value: ClientState) -> Self {
        ProtoAny {
            type_url: BESU_QBFT_CLIENT_STATE_TYPE_URL.to_string(),
            value: RawClientState::from(value).encode_to_vec(),
        }
        .into()
    }
}

impl ClientState {
    pub fn verify_account_storage(
        &self,
        proof: Vec<u8>,
        root: H256,
        path: &Address,
    ) -> Result<H256, Error> {
        let proof = decode_eip1184_rlp_proof(&proof)?;
        match self.execution_verifier.verify_account(
            root.to_be_bytes().into(),
            // unwrap is safe because the address is 20 bytes
            &(path.as_slice().try_into().unwrap()),
            proof,
        )? {
            Some(account) => Ok(H256::try_from_be_slice(account.storage_root.as_bytes())
                .ok_or_else(|| {
                    Error::InvalidAccountStorageRoot(account.storage_root.0.to_vec())
                })?),
            None => Err(Error::AccountNotFound(root, *path)),
        }
    }

    pub fn verify_membership(
        &self,
        proof: Vec<u8>,
        root: H256,
        path: String,
        value: Vec<u8>,
    ) -> Result<(), Error> {
        let proof = decode_eip1184_rlp_proof(&proof)?;
        let key = calculate_ibc_commitment_storage_key(path.as_bytes());

        self.execution_verifier.verify_membership(
            root.to_be_bytes().into(),
            key.to_be_bytes_vec().as_slice(),
            rlp::encode(&trim_left_zero(keccak256(&value).as_slice())).as_ref(),
            proof,
        )?;

        Ok(())
    }

    pub fn verify_non_membership(
        &self,
        proof: Vec<u8>,
        root: H256,
        path: String,
    ) -> Result<(), Error> {
        let proof = decode_eip1184_rlp_proof(&proof)?;
        let key = calculate_ibc_commitment_storage_key(path.as_bytes());

        self.execution_verifier.verify_non_membership(
            root.to_be_bytes().into(),
            key.to_be_bytes_vec().as_slice(),
            proof,
        )?;

        Ok(())
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.latest_height.is_zero() {
            return Err(Error::InvalidClientStateZeroHeight);
        }
        if self.ibc_store_address == Address::default() {
            return Err(Error::InvalidClientStateZeroIbcStoreAddress);
        }
        Ok(())
    }
}

pub fn canonicalize_client_state(client_state: ClientState) -> ClientState {
    let mut client_state = client_state;
    client_state.latest_height = Height::zero();
    client_state
}

fn trim_left_zero(value: &[u8]) -> &[u8] {
    let mut pos = 0;
    for v in value {
        if *v != 0 {
            break;
        }
        pos += 1;
    }
    &value[pos..]
}
