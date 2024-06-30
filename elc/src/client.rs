use crate::client_state::ClientState;
use crate::commitment::{keccak256, verify_signature};
use crate::consensus_state::ConsensusState;
use crate::errors::Error;
use crate::header::EthHeader;
use crate::internal_prelude::*;
use crate::message::Header;
use crate::types::{gen_state_id, Address, H256};
use core::time::Duration;
use light_client::commitments::{
    EmittedState, TrustingPeriodContext, UpdateStateProxyMessage, ValidationContext,
    VerifyMembershipProxyMessage,
};
use light_client::{
    types::{ClientId, Height, Time},
    CreateClientResult, HostClientReader, LightClient, UpdateStateData, VerifyMembershipResult,
    VerifyNonMembershipResult,
};

pub struct BesuQBFTLightClient;

impl LightClient for BesuQBFTLightClient {
    fn client_type(&self) -> String {
        "hb-qbft".to_string()
    }

    fn latest_height(
        &self,
        ctx: &dyn light_client::HostClientReader,
        client_id: &light_client::types::ClientId,
    ) -> Result<light_client::types::Height, light_client::Error> {
        let client_state: ClientState = ctx.client_state(client_id)?.try_into()?;
        Ok(client_state.latest_height)
    }

    fn create_client(
        &self,
        _: &dyn light_client::HostClientReader,
        any_client_state: light_client::types::Any,
        any_consensus_state: light_client::types::Any,
    ) -> Result<light_client::CreateClientResult, light_client::Error> {
        let client_state = ClientState::try_from(any_client_state.clone())?;
        let consensus_state = ConsensusState::try_from(any_consensus_state)?;

        client_state.validate()?;
        consensus_state.validate()?;

        let height = client_state.latest_height;

        let timestamp = consensus_state.timestamp;
        let state_id = gen_state_id(client_state, consensus_state)?;
        Ok(CreateClientResult {
            height,
            message: UpdateStateProxyMessage {
                prev_height: None,
                prev_state_id: None,
                post_height: height,
                post_state_id: state_id,
                emitted_states: vec![EmittedState(height, any_client_state)],
                timestamp,
                context: ValidationContext::Empty,
            }
            .into(),
            prove: false,
        })
    }

    fn update_client(
        &self,
        ctx: &dyn light_client::HostClientReader,
        client_id: light_client::types::ClientId,
        client_message: light_client::types::Any,
    ) -> Result<light_client::UpdateClientResult, light_client::Error> {
        let header: Header = client_message.try_into()?;
        let client_state: ClientState = ctx.client_state(&client_id)?.try_into()?;
        let trusted_consensus_state: ConsensusState = ctx
            .consensus_state(&client_id, &header.trusted_height)?
            .try_into()?;

        let eth_header = EthHeader::parse(header.besu_header_rlp.as_slice())?;
        let commit_hash = eth_header.commit_hash()?;

        self.verify_commit_seals_trusting(
            &trusted_consensus_state.validators,
            &header.seals,
            commit_hash,
        )?;
        self.verify_commit_seals_untrusting(
            &eth_header.extra.validators,
            &header.seals,
            commit_hash,
        )?;

        let storage_root = client_state.verify_account_storage(
            header.account_state_proof,
            eth_header.state_root,
            &client_state.ibc_store_address,
        )?;

        let mut new_client_state = client_state.clone();
        let height = Height::new(
            client_state.latest_height.revision_number(),
            eth_header
                .number
                .try_into()
                .map_err(Error::FromUint64Error)?,
        );
        if client_state.latest_height < height {
            new_client_state.latest_height = height;
        }
        let timestamp: u128 = eth_header
            .timestamp
            .try_into()
            .map_err(Error::FromUint128Error)?;
        let new_consensus_state = ConsensusState {
            timestamp: Time::from_unix_timestamp_nanos(timestamp * 1_000_000_000u128)
                .map_err(Error::Time)?,
            root: storage_root,
            validators: eth_header.extra.validators,
        };

        let validation_context = if client_state.trusting_period.is_zero() {
            ValidationContext::Empty
        } else {
            ValidationContext::TrustingPeriod(TrustingPeriodContext::new(
                client_state.trusting_period,
                // TODO make this configurable
                Duration::from_secs(30),
                new_consensus_state.timestamp,
                trusted_consensus_state.timestamp,
            ))
        };
        validation_context
            .validate(ctx.host_timestamp())
            .map_err(Error::Commitments)?;

        Ok(UpdateStateData {
            new_any_client_state: new_client_state.clone().into(),
            new_any_consensus_state: new_consensus_state.clone().into(),
            height,
            message: UpdateStateProxyMessage {
                prev_height: Some(header.trusted_height),
                prev_state_id: Some(gen_state_id(client_state, trusted_consensus_state)?),
                post_height: height,
                post_state_id: gen_state_id(new_client_state, new_consensus_state.clone())?,
                emitted_states: Default::default(),
                timestamp: new_consensus_state.timestamp,
                context: validation_context,
            },
            prove: true,
        }
        .into())
    }

    fn verify_membership(
        &self,
        ctx: &dyn light_client::HostClientReader,
        client_id: light_client::types::ClientId,
        prefix: light_client::commitments::CommitmentPrefix,
        path: String,
        value: Vec<u8>,
        proof_height: light_client::types::Height,
        proof: Vec<u8>,
    ) -> Result<light_client::VerifyMembershipResult, light_client::Error> {
        let (client_state, consensus_state) = Self::validate_args(ctx, client_id, proof_height)?;

        client_state.verify_membership(proof, consensus_state.root, path.clone(), value.clone())?;

        Ok(VerifyMembershipResult {
            message: VerifyMembershipProxyMessage::new(
                prefix,
                path,
                Some(keccak256(&value)),
                proof_height,
                gen_state_id(client_state, consensus_state)?,
            ),
        })
    }

    fn verify_non_membership(
        &self,
        ctx: &dyn light_client::HostClientReader,
        client_id: light_client::types::ClientId,
        prefix: light_client::commitments::CommitmentPrefix,
        path: String,
        proof_height: light_client::types::Height,
        proof: Vec<u8>,
    ) -> Result<light_client::VerifyNonMembershipResult, light_client::Error> {
        let (client_state, consensus_state) = Self::validate_args(ctx, client_id, proof_height)?;

        client_state.verify_non_membership(proof, consensus_state.root, path.clone())?;

        Ok(VerifyNonMembershipResult {
            message: VerifyMembershipProxyMessage::new(
                prefix,
                path,
                None,
                proof_height,
                gen_state_id(client_state, consensus_state)?,
            ),
        })
    }
}

impl BesuQBFTLightClient {
    fn validate_args(
        ctx: &dyn HostClientReader,
        client_id: ClientId,
        proof_height: Height,
    ) -> Result<(ClientState, ConsensusState), light_client::Error> {
        let client_state: ClientState = ctx.client_state(&client_id)?.try_into()?;

        let consensus_state: ConsensusState =
            ctx.consensus_state(&client_id, &proof_height)?.try_into()?;

        Ok((client_state, consensus_state))
    }

    fn verify_commit_seals_trusting(
        &self,
        trusted_validators: &[Address],
        committed_seals: &[Vec<u8>],
        commit_hash: H256,
    ) -> Result<(), Error> {
        let mut marked = vec![false; trusted_validators.len()];
        let mut success = 0;
        for seal in committed_seals {
            if seal.is_empty() {
                continue;
            }

            let addr = verify_signature(commit_hash, seal)?;
            if let Some(pos) = trusted_validators.iter().position(|v| v == &addr) {
                if !marked[pos] {
                    marked[pos] = true;
                    success += 1;
                }
            }
        }
        if success * 3 <= trusted_validators.len() * 2 {
            panic!("success * 3 <= trusted_validators.len() * 2");
        }
        Ok(())
    }

    /// CONTRACT: the order of `committed_seals` must be corresponding to the order of `validators`
    fn verify_commit_seals_untrusting(
        &self,
        untrusted_validators: &[Address],
        committed_seals: &[Vec<u8>],
        commit_hash: H256,
    ) -> Result<(), Error> {
        if untrusted_validators.len() != committed_seals.len() {
            panic!("untrusted_validators.len() != committed_seals.len()");
        }

        let mut success = 0;
        for (validator, seal) in untrusted_validators.iter().zip(committed_seals.iter()) {
            if seal.is_empty() {
                continue;
            }

            let addr = verify_signature(commit_hash, seal)?;
            if addr == *validator {
                success += 1;
            }
        }

        if success * 3 <= untrusted_validators.len() * 2 {
            panic!("success * 3 <= untrusted_validators.len() * 2");
        }
        Ok(())
    }
}
