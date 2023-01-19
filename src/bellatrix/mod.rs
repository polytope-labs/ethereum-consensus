//! This module provides an implementation of the `bellatrix` fork
//! of the consensus spec. The primary entrypoints should be one of
//! the "presets" like `mainnet` or `minimal`.
mod beacon_block;
mod beacon_state;
mod blinded_beacon_block;
mod block_processing;
mod epoch_processing;
mod execution;
mod fork;
mod fork_choice;
mod genesis;
mod helpers;
mod presets;
mod state_transition_bellatrix;

mod state_transition;
pub use state_transition::{
    block_processing::*, epoch_processing::*, helpers::*, slot_processing::*, *,
};

pub use beacon_block::*;
pub use beacon_state::*;
pub use blinded_beacon_block::*;
pub use execution::*;
pub use fork::*;
pub use fork_choice::*;
pub use genesis::*;
pub use presets::Preset;

pub use crate::altair::{
    SyncAggregate, SyncAggregatorSelectionData, SyncCommittee, PARTICIPATION_FLAG_WEIGHTS,
    PROPOSER_WEIGHT, SYNC_REWARD_WEIGHT, TIMELY_HEAD_FLAG_INDEX, TIMELY_SOURCE_FLAG_INDEX,
    TIMELY_TARGET_FLAG_INDEX, WEIGHT_DENOMINATOR,
};
pub use crate::phase0::{
    Attestation, AttestationData, AttesterSlashing, BeaconBlockHeader, Checkpoint, Deposit,
    DepositData, DepositMessage, Eth1Block, Eth1Data, Fork, ForkData, HistoricalBatchAccumulator,
    IndexedAttestation, ProposerSlashing, SignedBeaconBlockHeader, SignedVoluntaryExit,
    SigningData, Validator, VoluntaryExit, BASE_REWARDS_PER_EPOCH, DEPOSIT_CONTRACT_TREE_DEPTH,
    JUSTIFICATION_BITS_LENGTH,
};

pub mod mainnet {
    pub use super::presets::mainnet::*;
}

pub mod minimal {
    pub use super::presets::minimal::*;
}
