use crate::{
    client_state::{canonicalize_client_state, ClientState},
    consensus_state::ConsensusState,
    errors::Error,
};
use light_client::commitments::{gen_state_id_from_any, StateID};

pub type U256 = ruint::aliases::U256;

pub type H256 = ruint::aliases::B256;

pub type Address = [u8; 20];

pub fn gen_state_id(
    client_state: ClientState,
    consensus_state: ConsensusState,
) -> Result<StateID, Error> {
    Ok(gen_state_id_from_any(
        &canonicalize_client_state(client_state).into(),
        &consensus_state.into(),
    )?)
}
