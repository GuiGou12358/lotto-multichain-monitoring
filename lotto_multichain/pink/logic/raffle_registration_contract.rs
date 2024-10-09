use crate::error::RaffleDrawError::{self, *};
use crate::types::{AccountId32, DrawNumber, Number, RaffleConfig};

#[derive(scale::Encode, scale::Decode)]
pub enum RaffleRegistrationStatus {
    NotStarted,
    Started,
    RegistrationOpen,
    RegistrationClosed,
    ResultsReceived,
}

/// Message sent by the offchain rollup to the Raffle Registration Contracts
#[derive(scale::Encode, scale::Decode)]
pub enum RequestForAction {
    SetConfig(RaffleConfig),
    OpenRegistrations(DrawNumber),
    CloseRegistrations(DrawNumber),
    SetResults(DrawNumber, Vec<Number>, Vec<AccountId32>),
}


pub trait RaffleRegistrationContract {

    fn get_raffle_registration_status(&self) -> Option<RaffleRegistrationStatus>;

    fn do_action(
        &self,
        action: RequestForAction,
        attest_key: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, RaffleDrawError>;

}