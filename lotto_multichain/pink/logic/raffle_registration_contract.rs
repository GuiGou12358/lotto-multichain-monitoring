extern crate alloc;

use crate::error::RaffleDrawError;
use crate::types::{AccountId32, DrawNumber, Number, RaffleConfig};
use alloc::vec::Vec;

#[derive(scale::Encode, scale::Decode, Eq, PartialEq, Clone, Copy)]
pub enum RaffleRegistrationStatus {
    NotStarted,
    Started,
    RegistrationOpen,
    RegistrationClosed,
    ResultsReceived,
}

/// Message sent by the offchain rollup to the Raffle Registration Contracts
#[derive(scale::Encode, scale::Decode, Debug, Clone)]
pub enum RequestForAction {
    SetConfig(RaffleConfig),
    OpenRegistrations(DrawNumber),
    CloseRegistrations(DrawNumber),
    SetResults(DrawNumber, Vec<Number>, Vec<AccountId32>),
}

pub trait RaffleRegistrationContract {

    fn do_action(
        &self,
        expected_draw_number: DrawNumber,
        expected_status: RaffleRegistrationStatus,
        action: RequestForAction,
        attest_key: &[u8; 32],
    ) -> Result<bool, RaffleDrawError>;
}
