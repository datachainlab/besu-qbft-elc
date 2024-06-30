use crate::errors::Error;
use crate::internal_prelude::*;
use besu_qbft_proto::ibc::{
    core::client::v1::Height as RawHeight, lightclients::qbft::v1::Header as RawHeader,
};
use light_client::types::proto::protobuf::Protobuf;
use light_client::types::{Any, Height};
use prost::Message;

pub const BESU_QBFT_HEADER_TYPE_URL: &str = "/ibc.lightclients.qbft.v1.Header";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ClientMessage {
    Header(Header),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Header {
    pub besu_header_rlp: Vec<u8>,
    pub seals: Vec<Vec<u8>>,
    pub trusted_height: Height,
    pub account_state_proof: Vec<u8>,
}

impl From<Header> for RawHeader {
    fn from(value: Header) -> Self {
        RawHeader {
            besu_header_rlp: value.besu_header_rlp,
            seals: value.seals,
            trusted_height: Some(RawHeight {
                revision_number: value.trusted_height.revision_number(),
                revision_height: value.trusted_height.revision_height(),
            }),
            account_state_proof: value.account_state_proof,
        }
    }
}

impl From<Header> for Any {
    fn from(value: Header) -> Self {
        let raw_header = RawHeader::from(value);
        let value = raw_header.encode_to_vec();
        Any::new(BESU_QBFT_HEADER_TYPE_URL.to_string(), value)
    }
}

impl TryFrom<RawHeader> for Header {
    type Error = Error;

    fn try_from(value: RawHeader) -> Result<Self, Self::Error> {
        let trusted_height = value
            .trusted_height
            .ok_or(Error::InvalidHeaderZeroTrustedHeight)?;
        Ok(Header {
            besu_header_rlp: value.besu_header_rlp,
            seals: value.seals,
            trusted_height: Height::new(
                trusted_height.revision_number,
                trusted_height.revision_height,
            ),
            account_state_proof: value.account_state_proof,
        })
    }
}

impl TryFrom<Any> for Header {
    type Error = Error;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        let type_url = value.type_url.as_str();
        let value = value.value.as_ref();

        match type_url {
            BESU_QBFT_HEADER_TYPE_URL => {
                let raw_header = RawHeader::decode(value).map_err(Error::Decode)?;
                Header::try_from(raw_header)
            }
            _ => Err(Error::UnexpectedClientType(type_url.to_string())),
        }
    }
}

impl From<ClientMessage> for Any {
    fn from(value: ClientMessage) -> Self {
        match value {
            ClientMessage::Header(header) => header.into(),
        }
    }
}

impl TryFrom<Any> for ClientMessage {
    type Error = Error;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        let type_url = value.type_url.as_str();
        let value = value.value.as_ref();

        match type_url {
            BESU_QBFT_HEADER_TYPE_URL => {
                let raw_header = RawHeader::decode(value).map_err(Error::Decode)?;
                let header = Header::try_from(raw_header)?;
                Ok(ClientMessage::Header(header))
            }
            _ => Err(Error::UnexpectedClientType(type_url.to_string())),
        }
    }
}

impl Protobuf<Any> for ClientMessage {}
