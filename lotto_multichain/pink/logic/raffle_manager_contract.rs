extern crate alloc;

use crate::types::{AccountId32, DrawNumber, Hash, Number, RaffleConfig, RegistrationContractId};
use alloc::vec::Vec;

#[derive(scale::Encode, scale::Decode)]
pub enum RaffleManagerStatus {
    NotStarted,
    Started,
    RegistrationsOpen,
    RegistrationsClosed,
    WaitingResults,
    WaitingWinners,
    Closed,
}

/// message pushed in the queue by the Raffle Manager contract and read by the offchain rollup
#[derive(scale::Encode, scale::Decode, Debug)]
pub enum LottoManagerRequestMessage {
    PropagateConfig(RaffleConfig, Vec<RegistrationContractId>),
    OpenRegistrations(DrawNumber, Vec<RegistrationContractId>),
    CloseRegistrations(DrawNumber, Vec<RegistrationContractId>),
    PropagateResults(
        DrawNumber,
        Vec<Number>,
        Vec<AccountId32>,
        Vec<RegistrationContractId>,
    ),

    /// request to lotto_draw the n number between min and max values
    /// arg1: draw number
    /// arg2: number of numbers for the lotto_draw
    /// arg3:  smallest number for the lotto_draw
    /// arg4:  biggest number for the lotto_draw
    DrawNumbers(DrawNumber, u8, Number, Number),
    /// request to check if there is a winner for the given numbers
    CheckWinners(DrawNumber, Vec<Number>),
}

/// response pushed in the queue by the offchain rollup and read by the Raffle Manager contract
#[derive(scale::Encode, scale::Decode)]
pub enum LottoManagerResponseMessage {
    ///
    /// arg1: list of contracts where the config is propagated
    /// arg2: hash of config
    ConfigPropagated(Vec<RegistrationContractId>, Hash),
    ///
    /// arg1: draw number
    /// arg2: list of contracts where the registrations are open
    RegistrationsOpen(DrawNumber, Vec<RegistrationContractId>),
    ///
    /// arg1: draw number
    /// arg2: list of contracts where the registrations are closed
    RegistrationsClosed(DrawNumber, Vec<RegistrationContractId>),
    /// Return the winning numbers
    /// arg1: draw number
    /// arg2: winning numbers
    /// arg3: salt used for vrf
    WinningNumbers(DrawNumber, Vec<Number>, Hash),
    /// Return the list of winners
    /// arg1: draw number
    /// arg2: winners
    /// arg3: hash of winning numbers
    Winners(DrawNumber, Vec<AccountId32>, Hash),
    ///
    /// arg1: draw number
    /// arg2: list of contracts where the results are propagated
    /// arg3: hash of results
    ResultsPropagated(DrawNumber, Vec<RegistrationContractId>, Hash),
}

pub trait RaffleManagerContract {
    fn get_raffle_manager_status(&self) -> Option<RaffleManagerStatus>;

    /*
    fn get_request(&self) -> Result<Option<LottoManagerRequestMessage>, RaffleDrawError>;

    fn send_response(
        &mut self,
        response: LottoManagerResponseMessage,
        attest_key: &[u8],
    ) -> Result<Option<Vec<u8>>, RaffleDrawError>;

     */
}
