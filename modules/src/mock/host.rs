//! Host chain types and methods, used by context mock.

use tendermint::time::Time;
use tendermint_testgen::light_block::TmLightBlock;
use tendermint_testgen::{Generator, LightBlock as TestgenLightBlock};
use time::OffsetDateTime;

use crate::clients::ics07_tendermint::consensus_state::ConsensusState as TMConsensusState;
use crate::clients::ics07_tendermint::header::Header as TMHeader;
use crate::core::ics02_client::client_consensus::AnyConsensusState;
use crate::core::ics02_client::header::AnyHeader;
use crate::core::ics24_host::identifier::ChainId;
use crate::mock::header::MockHeader;
use crate::prelude::*;
use crate::timestamp::Timestamp;
use crate::Height;

/// Defines the different types of host chains that a mock context can emulate.
/// The variants are as follows:
/// - `Mock` defines that the context history consists of `MockHeader` blocks.
/// - `SyntheticTendermint`: the context has synthetically-generated Tendermint (light) blocks.
/// See also the `HostBlock` enum to get more insights into the underlying block type.
#[derive(Clone, Debug, Copy)]
pub enum HostType {
    Mock,
    SyntheticTendermint,
}

/// Depending on `HostType` (the type of host chain underlying a context mock), this enum defines
/// the type of blocks composing the history of the host chain.
#[derive(Clone, Debug)]
pub enum HostBlock {
    Mock(MockHeader),
    SyntheticTendermint(Box<TmLightBlock>),
}

impl HostBlock {
    /// Returns the height of a block.
    pub fn height(&self) -> Height {
        match self {
            HostBlock::Mock(header) => header.height(),
            HostBlock::SyntheticTendermint(light_block) => Height::new(
                ChainId::chain_version(light_block.signed_header.header.chain_id.as_str()),
                light_block.signed_header.header.height.value(),
            ),
        }
    }

    /// Generates a new block at `height` for the given chain identifier and chain type.
    pub fn generate_block(chain_id: ChainId, chain_type: HostType, height: u64) -> HostBlock {
        match chain_type {
            HostType::Mock => HostBlock::Mock(MockHeader {
                height: Height::new(chain_id.version(), height),
                timestamp: Timestamp::now(),
            }),
            HostType::SyntheticTendermint => {
                HostBlock::SyntheticTendermint(Box::new(Self::generate_tm_block(chain_id, height)))
            }
        }
    }

    pub fn generate_tm_block(chain_id: ChainId, height: u64) -> TmLightBlock {
        // Sleep is required otherwise the generator produces blocks with the
        // same timestamp as two block can be generated per second.
        let ten_millis = core::time::Duration::from_millis(1000);
        std::thread::sleep(ten_millis);
        let time: Time = OffsetDateTime::now_utc().try_into().unwrap();
        let time = time.duration_since(Time::unix_epoch()).unwrap().as_secs();

        TestgenLightBlock::new_default_with_time_and_chain_id(chain_id.to_string(), time, height)
            .generate()
            .unwrap()
    }
}

impl From<TmLightBlock> for AnyConsensusState {
    fn from(light_block: TmLightBlock) -> Self {
        let cs = TMConsensusState::from(light_block.signed_header.header);
        AnyConsensusState::Tendermint(cs)
    }
}

impl From<HostBlock> for AnyConsensusState {
    fn from(any_block: HostBlock) -> Self {
        match any_block {
            HostBlock::Mock(mock_header) => mock_header.into(),
            HostBlock::SyntheticTendermint(light_block) => (*light_block).into(),
        }
    }
}

impl From<HostBlock> for AnyHeader {
    fn from(any_block: HostBlock) -> Self {
        match any_block {
            HostBlock::Mock(mock_header) => mock_header.into(),
            HostBlock::SyntheticTendermint(light_block_box) => {
                // Conversion from TMLightBlock to AnyHeader
                AnyHeader::Tendermint((*light_block_box).into())
            }
        }
    }
}

impl From<TmLightBlock> for TMHeader {
    fn from(light_block: TmLightBlock) -> Self {
        // TODO: This conversion is incorrect for `trusted_height` and `trusted_validator_set`.
        TMHeader {
            signed_header: light_block.signed_header,
            validator_set: light_block.validators,
            trusted_height: Default::default(),
            trusted_validator_set: light_block.next_validators,
        }
    }
}
