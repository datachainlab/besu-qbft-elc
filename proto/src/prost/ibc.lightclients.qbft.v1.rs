#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(bytes = "vec", tag = "1")]
    pub chain_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub ibc_store_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub latest_height: ::core::option::Option<
        super::super::super::core::client::v1::Height,
    >,
    /// duration in seconds
    /// if this is set to 0, the client will not verify the header's timestamp is within the trusting period
    #[prost(uint64, tag = "4")]
    pub trusting_period: u64,
    /// duration in seconds
    #[prost(uint64, tag = "5")]
    pub max_clock_drift: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    #[prost(uint64, tag = "1")]
    pub timestamp: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", repeated, tag = "3")]
    pub validators: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    /// RLP encoded header of Besu, which does not include the seals in the extra data
    #[prost(bytes = "vec", tag = "1")]
    pub besu_header_rlp: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub seals: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(message, optional, tag = "3")]
    pub trusted_height: ::core::option::Option<
        super::super::super::core::client::v1::Height,
    >,
    #[prost(bytes = "vec", tag = "4")]
    pub account_state_proof: ::prost::alloc::vec::Vec<u8>,
}
