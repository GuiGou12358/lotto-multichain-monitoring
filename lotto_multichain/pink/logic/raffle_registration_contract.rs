extern crate alloc;

use crate::error::RaffleDrawError;
use crate::types::{AccountId32, DrawNumber, Number, RaffleConfig};
use alloc::vec::Vec;

#[derive(scale::Encode, scale::Decode, Eq, PartialEq, Clone, Copy, Debug)]
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
        target_draw_number: Option<DrawNumber>,
        target_status: Option<RaffleRegistrationStatus>,
        action: RequestForAction,
        attest_key: &[u8; 32],
    ) -> Result<bool, RaffleDrawError>;
}
