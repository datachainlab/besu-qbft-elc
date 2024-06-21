#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::result_large_err)]
#![allow(clippy::large_enum_variant)]

use light_client::LightClientRegistry;
extern crate alloc;

pub mod client;
pub mod client_state;
pub mod commitment;
pub mod consensus_state;
pub mod errors;
pub mod header;
pub mod message;
pub mod types;

mod internal_prelude {
    pub use alloc::boxed::Box;
    pub use alloc::format;
    pub use alloc::string::{String, ToString};
    pub use alloc::vec;
    pub use alloc::vec::Vec;
}
use client_state::BESU_QBFT_CLIENT_STATE_TYPE_URL;
use internal_prelude::*;

pub fn register_implementations(registry: &mut dyn LightClientRegistry) {
    registry
        .put_light_client(
            BESU_QBFT_CLIENT_STATE_TYPE_URL.to_string(),
            Box::new(client::BesuQBFTLightClient),
        )
        .unwrap()
}
