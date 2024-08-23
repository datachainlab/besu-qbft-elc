use crate::errors::Error;
use crate::internal_prelude::*;
use crate::types::{Address, H256, U256};
use hex_literal::hex;
use libsecp256k1::{curve::Scalar, Message, PublicKey, RecoveryId, Signature};
use rlp::Rlp;
use tiny_keccak::Keccak;

/// decode rlp format `List<List>` to `Vec<List>`
pub fn decode_eip1184_rlp_proof(proof: &[u8]) -> Result<Vec<Vec<u8>>, Error> {
    let r = Rlp::new(proof);
    if r.is_list() {
        Ok(r.into_iter()
            .map(|r| {
                let elems: Vec<Vec<u8>> = r.as_list()?;
                Ok(rlp::encode_list::<Vec<u8>, Vec<u8>>(&elems).into())
            })
            .collect::<Result<Vec<Vec<u8>>, Error>>()?)
    } else {
        Err(Error::InvalidRLPFormatNotList(proof.to_vec()))
    }
}

pub fn calculate_ibc_commitment_storage_key(path: &[u8]) -> U256 {
    let h = keccak256(
        &[
            keccak256(path).as_slice(),
            hex!("1ee222554989dda120e26ecacf756fe1235cd8d726706b57517715dde4f0c900").as_slice(),
        ]
        .concat(),
    );
    U256::from_be_slice(&h)
}

pub fn keccak256(bz: &[u8]) -> [u8; 32] {
    let mut keccak = Keccak::new_keccak256();
    let mut result = [0u8; 32];
    keccak.update(bz);
    keccak.finalize(result.as_mut());
    result
}

pub fn verify_signature(sign_hash: H256, signature: &[u8]) -> Result<Address, Error> {
    if signature.len() != 65 {
        return Err(Error::InvalidSignatureLength(signature.len()));
    }
    let mut s = Scalar::default();
    let _ = s.set_b32(&sign_hash.to_be_bytes());

    let sig = Signature::parse_overflowing_slice(&signature[..64])?;
    let rid = RecoveryId::parse(signature[64])?;
    let signer: PublicKey = libsecp256k1::recover(&Message(s), &sig, &rid)?;
    Ok(address_from_pubkey(&signer))
}

pub fn address_from_pubkey(pubkey: &PublicKey) -> Address {
    let mut address = [0u8; 20];
    let hash = keccak256(&pubkey.serialize()[1..]);
    address.copy_from_slice(&hash[12..]);
    address
}
