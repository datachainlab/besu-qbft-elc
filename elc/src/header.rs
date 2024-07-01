use crate::commitment::keccak256;
use crate::errors::Error;
use crate::types::Address;
use crate::{
    internal_prelude::*,
    types::{H256, U256},
};
use rlp::Rlp;

pub const ETH_HEADER_STATE_ROOT_INDEX: usize = 3;
pub const ETH_HEADER_NUMBER_INDEX: usize = 8;
pub const ETH_HEADER_TIMESTAMP_INDEX: usize = 11;
pub const ETH_HEADER_EXTRA_INDEX: usize = 12;

#[derive(Debug, Clone, PartialEq)]
pub struct EthHeader {
    pub(crate) bytes: Vec<u8>,

    pub state_root: H256,
    pub number: U256,
    pub timestamp: U256,

    pub extra: QbftExtra,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct QbftExtra {
    pub vanity_data: Vec<u8>,
    pub validators: Vec<Address>,
    pub vote: Vec<u8>,
    pub round: u32,
    pub committed_seals: Vec<Vec<u8>>,
}

impl QbftExtra {
    pub fn decode(bz: &[u8]) -> Result<Self, Error> {
        let extra = Rlp::new(bz);
        let mut it = extra.iter();
        if it.len() != 5 {
            return Err(Error::InvalidHeaderExtraSize(it.len()));
        }
        // unwrap is safe here because we have checked the length
        let vanity_data: Vec<u8> = it.next().unwrap().as_val()?;
        let validators: Vec<Vec<u8>> = it.next().unwrap().as_list()?;
        let raw_vote = it.next().unwrap().as_raw();
        let round: u32 = it.next().unwrap().as_val()?;
        let committed_seals: Vec<Vec<u8>> = it.next().unwrap().as_list()?;

        let addrs = validators
            .into_iter()
            .map(|v| {
                Address::try_from(v.as_slice())
                    .map_err(|_| Error::InvalidValidatorAddressLength(v.len()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            vanity_data: vanity_data.to_vec(),
            validators: addrs,
            vote: raw_vote.to_vec(),
            round,
            committed_seals,
        })
    }
}

impl EthHeader {
    /// header_rlp: RLP encoded header
    pub fn parse(header_rlp: &[u8]) -> Result<Self, Error> {
        let rlp = Rlp::new(header_rlp);
        let state_root = {
            let v: Vec<u8> = rlp.at(ETH_HEADER_STATE_ROOT_INDEX)?.as_val()?;
            H256::try_from_be_slice(v.as_slice())
                .ok_or_else(|| Error::InvalidStateRootLength(v.len()))?
        };

        let number = {
            let v: Vec<u8> = rlp.at(ETH_HEADER_NUMBER_INDEX)?.as_val()?;
            U256::try_from_be_slice(v.as_slice())
                .ok_or_else(|| Error::InvalidBlockNumberLength(v.len()))?
        };

        let timestamp = {
            let v: Vec<u8> = rlp.at(ETH_HEADER_TIMESTAMP_INDEX)?.as_val()?;
            U256::try_from_be_slice(v.as_slice())
                .ok_or_else(|| Error::InvalidBlockTimestampLength(v.len()))?
        };

        let extra_bz: Vec<u8> = rlp.at(ETH_HEADER_EXTRA_INDEX)?.as_val()?;
        let extra = QbftExtra::decode(extra_bz.as_slice())?;

        Ok(EthHeader {
            bytes: header_rlp.to_vec(),
            state_root,
            number,
            timestamp,
            extra,
        })
    }

    pub fn commit_hash(&self) -> Result<H256, Error> {
        if !self.extra.committed_seals.is_empty() {
            return Err(Error::HeaderExtraContainsCommittedSeals(self.bytes.clone()));
        }
        Ok(H256::from_be_bytes(keccak256(&self.bytes)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commitment::verify_signature;
    use hex_literal::hex;

    #[test]
    fn test_parse_header() {
        // i = 0: header extra contains committed seals
        // i = 1: header extra does not contain committed seals
        let headers = vec![
            hex!("f9033fa00af93e70b1c6d3974a88a42eb70bb61adbd523bfac0c83027ba4637c52746a0fa01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794ee3353e587cfa91625a1adaef308a726de3803d3a0166ed98eea93ab2b6f6b1a425526994adc2d675bf9a0d77d600ed1e02d8f77dfa056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b901000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001821d688347b76080846640618bb90147f90144a00000000000000000000000000000000000000000000000000000000000000000f85494647bfdd19655e51e69d35454ff3a92f8828e630294a5c8416b9d13417b45b45ada76408f39d1e504ef94b92e91f4dcc9d28503be521afa2a8fbf3c1acf6094ee3353e587cfa91625a1adaef308a726de3803d3c001f8c9b841bc7633fd65570f610a595086e9a34e5bf6aacfb67b8f8cd01852e6b285147f046a50577b49378b86723ac9b456ef59ef7ab57cda7139d807f10f58e8cb10c67600b841ad1defc2b0b4a48158cff24778bb5ba4d9f373c171022ab0a42e37bdb0d4025718434d303a8d94df56ef9ad5219be9f27b2f67179a7fb82d3323dde29546f9f701b841e233d3670dd97c715f72b440eeb1ccb1e22c8c23f6ab470c46c99c2d0ee6509f0341a42e0e4569782557e93c3815e8ca4294043595f69a90f73f135de8ecf41e00a063746963616c2062797a616e74696e65206661756c7420746f6c6572616e6365880000000000000000").to_vec(),
            hex!("f90273a00af93e70b1c6d3974a88a42eb70bb61adbd523bfac0c83027ba4637c52746a0fa01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794ee3353e587cfa91625a1adaef308a726de3803d3a0166ed98eea93ab2b6f6b1a425526994adc2d675bf9a0d77d600ed1e02d8f77dfa056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b901000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001821d688347b76080846640618bb87cf87aa00000000000000000000000000000000000000000000000000000000000000000f85494647bfdd19655e51e69d35454ff3a92f8828e630294a5c8416b9d13417b45b45ada76408f39d1e504ef94b92e91f4dcc9d28503be521afa2a8fbf3c1acf6094ee3353e587cfa91625a1adaef308a726de3803d3c001c0a063746963616c2062797a616e74696e65206661756c7420746f6c6572616e6365880000000000000000").to_vec(),
        ];
        for (i, bz) in headers.into_iter().enumerate() {
            let header = EthHeader::parse(&bz);
            assert!(header.is_ok(), "{:?}", header);
            let header = header.unwrap();

            assert_eq!(header.number, U256::from(7528u64));
            assert_eq!(
                header.state_root,
                H256::try_from_be_slice(
                    hex!("166ed98eea93ab2b6f6b1a425526994adc2d675bf9a0d77d600ed1e02d8f77df")
                        .as_slice()
                )
                .unwrap()
            );
            assert_eq!(header.timestamp, U256::from(1715495307u64));

            assert_eq!(header.extra.round, 1);

            assert_eq!(header.extra.validators.len(), 4);
            assert_eq!(
                header.extra.validators[0],
                hex!("647bfdd19655e51e69d35454ff3a92f8828e6302")
            );
            assert_eq!(
                header.extra.validators[1],
                hex!("a5c8416b9d13417b45b45ada76408f39d1e504ef")
            );
            assert_eq!(
                header.extra.validators[2],
                hex!("b92e91f4dcc9d28503be521afa2a8fbf3c1acf60")
            );
            assert_eq!(
                header.extra.validators[3],
                hex!("ee3353e587cfa91625a1adaef308a726de3803d3")
            );

            if i == 0 {
                assert_eq!(header.extra.committed_seals.len(), 3);
                assert_eq!(header.extra.committed_seals[0], hex!("bc7633fd65570f610a595086e9a34e5bf6aacfb67b8f8cd01852e6b285147f046a50577b49378b86723ac9b456ef59ef7ab57cda7139d807f10f58e8cb10c67600"));
                assert_eq!(header.extra.committed_seals[1], hex!("ad1defc2b0b4a48158cff24778bb5ba4d9f373c171022ab0a42e37bdb0d4025718434d303a8d94df56ef9ad5219be9f27b2f67179a7fb82d3323dde29546f9f701"));
                assert_eq!(header.extra.committed_seals[2], hex!("e233d3670dd97c715f72b440eeb1ccb1e22c8c23f6ab470c46c99c2d0ee6509f0341a42e0e4569782557e93c3815e8ca4294043595f69a90f73f135de8ecf41e00"));
                assert!(header.commit_hash().is_err());
            } else {
                assert_eq!(header.extra.committed_seals.len(), 0);
                let hash = header.commit_hash().unwrap();
                assert_eq!(
                    hash,
                    H256::from_be_bytes(hex!(
                        "75ea184f58cd3f0ef89032a069df01f07ec524ef3a85cf6d3e424d62130c0a32"
                    ))
                );
            }
        }
    }

    #[test]
    fn test_parse_header_extra() {
        let extra_bz = hex!("f87ea00000000000000000000000000000000000000000000000000000000000000000d594cc4b2d4fbb236d5207b37a5cf739b8491b2b717cc080f843b84192782505a9fcf7d7352298df515282905a9e99c44a7dcffe30b75e9df44660aa76f1ed27e2b27e29712d62e7415ebb4e700e1d64fdc8bed818562391b5421d3500");
        let extra = QbftExtra::decode(&extra_bz).unwrap();
        assert_eq!(extra.round, 0);
        assert_eq!(extra.vanity_data.len(), 32);
        assert_eq!(extra.validators.len(), 1);
        assert_eq!(extra.committed_seals.len(), 1);

        let extra_bz = hex!("f90144a00000000000000000000000000000000000000000000000000000000000000000f85494647bfdd19655e51e69d35454ff3a92f8828e630294a5c8416b9d13417b45b45ada76408f39d1e504ef94b92e91f4dcc9d28503be521afa2a8fbf3c1acf6094ee3353e587cfa91625a1adaef308a726de3803d3c001f8c9b841bc7633fd65570f610a595086e9a34e5bf6aacfb67b8f8cd01852e6b285147f046a50577b49378b86723ac9b456ef59ef7ab57cda7139d807f10f58e8cb10c67600b841ad1defc2b0b4a48158cff24778bb5ba4d9f373c171022ab0a42e37bdb0d4025718434d303a8d94df56ef9ad5219be9f27b2f67179a7fb82d3323dde29546f9f701b841e233d3670dd97c715f72b440eeb1ccb1e22c8c23f6ab470c46c99c2d0ee6509f0341a42e0e4569782557e93c3815e8ca4294043595f69a90f73f135de8ecf41e00");
        let extra = QbftExtra::decode(&extra_bz).unwrap();
        assert_eq!(extra.round, 1);
        assert_eq!(extra.vanity_data.len(), 32);
        assert_eq!(extra.validators.len(), 4);
        assert_eq!(extra.committed_seals.len(), 3);
    }

    #[test]
    fn test_verify_committed_seals() {
        let headers = vec![
            hex!("f9033fa00af93e70b1c6d3974a88a42eb70bb61adbd523bfac0c83027ba4637c52746a0fa01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794ee3353e587cfa91625a1adaef308a726de3803d3a0166ed98eea93ab2b6f6b1a425526994adc2d675bf9a0d77d600ed1e02d8f77dfa056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b901000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001821d688347b76080846640618bb90147f90144a00000000000000000000000000000000000000000000000000000000000000000f85494647bfdd19655e51e69d35454ff3a92f8828e630294a5c8416b9d13417b45b45ada76408f39d1e504ef94b92e91f4dcc9d28503be521afa2a8fbf3c1acf6094ee3353e587cfa91625a1adaef308a726de3803d3c001f8c9b841bc7633fd65570f610a595086e9a34e5bf6aacfb67b8f8cd01852e6b285147f046a50577b49378b86723ac9b456ef59ef7ab57cda7139d807f10f58e8cb10c67600b841ad1defc2b0b4a48158cff24778bb5ba4d9f373c171022ab0a42e37bdb0d4025718434d303a8d94df56ef9ad5219be9f27b2f67179a7fb82d3323dde29546f9f701b841e233d3670dd97c715f72b440eeb1ccb1e22c8c23f6ab470c46c99c2d0ee6509f0341a42e0e4569782557e93c3815e8ca4294043595f69a90f73f135de8ecf41e00a063746963616c2062797a616e74696e65206661756c7420746f6c6572616e6365880000000000000000").to_vec(),
            hex!("f90273a00af93e70b1c6d3974a88a42eb70bb61adbd523bfac0c83027ba4637c52746a0fa01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794ee3353e587cfa91625a1adaef308a726de3803d3a0166ed98eea93ab2b6f6b1a425526994adc2d675bf9a0d77d600ed1e02d8f77dfa056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b901000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001821d688347b76080846640618bb87cf87aa00000000000000000000000000000000000000000000000000000000000000000f85494647bfdd19655e51e69d35454ff3a92f8828e630294a5c8416b9d13417b45b45ada76408f39d1e504ef94b92e91f4dcc9d28503be521afa2a8fbf3c1acf6094ee3353e587cfa91625a1adaef308a726de3803d3c001c0a063746963616c2062797a616e74696e65206661756c7420746f6c6572616e6365880000000000000000").to_vec(),
        ];
        let header = EthHeader::parse(&headers[0]).unwrap();
        assert_eq!(header.extra.committed_seals.len(), 3);
        let commit_hash = EthHeader::parse(&headers[1])
            .unwrap()
            .commit_hash()
            .unwrap();

        let validators = header.extra.validators;

        let mut signers = vec![];
        for seal in header.extra.committed_seals.iter() {
            let addr = verify_signature(commit_hash, seal).unwrap();
            assert!(validators.contains(&addr));
            assert!(!signers.contains(&addr));
            signers.push(addr);
        }
        assert_eq!(signers.len(), 3);
    }
}
