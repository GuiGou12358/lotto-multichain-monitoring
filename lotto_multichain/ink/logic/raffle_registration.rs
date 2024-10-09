use crate::error::{RaffleError, RaffleError::*};
use crate::{DrawNumber, Number};
use ink::prelude::vec::Vec;
use openbrush::traits::{AccountId, Storage};

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    draw_number: DrawNumber,
    status: Status,
}

#[derive(Default, Debug, Eq, PartialEq, Copy, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum Status {
    #[default]
    NotStarted,
    Started,
    RegistrationOpen,
    RegistrationClosed,
    ResultsReceived,
}

#[openbrush::trait_definition]
pub trait Raffle: Storage<Data> {
    /// Open the registrations
    fn open_registrations(&mut self, draw_number: DrawNumber) -> Result<(), RaffleError> {
        // check the status
        if self.data::<Data>().status != Status::Started
            && self.data::<Data>().status != Status::ResultsReceived
        {
            return Err(IncorrectStatus);
        }

        self.data::<Data>().draw_number = draw_number;
        self.data::<Data>().status = Status::RegistrationOpen;

        Ok(())
    }

    /// Close the registrations
    fn close_registrations(&mut self, draw_number: DrawNumber) -> Result<(), RaffleError> {
        // check the status
        if self.data::<Data>().status != Status::RegistrationOpen {
            return Err(IncorrectStatus);
        }
        // check the draw number
        if self.data::<Data>().draw_number != draw_number {
            return Err(IncorrectDrawNumber);
        }
        // update the status
        self.data::<Data>().status = Status::RegistrationClosed;
        Ok(())
    }

    /// save the results for the draw number.
    fn save_results(
        &mut self,
        draw_number: DrawNumber,
        _numbers: Vec<Number>,
        _winners: Vec<AccountId>,
    ) -> Result<(), RaffleError> {
        // check the status
        if self.data::<Data>().status != Status::RegistrationClosed {
            return Err(IncorrectStatus);
        }
        // check the draw number
        if self.data::<Data>().draw_number != draw_number {
            return Err(IncorrectDrawNumber);
        }

        self.data::<Data>().status = Status::ResultsReceived;
        Ok(())
    }

    /// check if the registrations are open
    fn can_participate(&mut self) -> Result<(), RaffleError> {
        // check the status
        if self.data::<Data>().status != Status::RegistrationOpen {
            return Err(IncorrectStatus);
        }

        Ok(())
    }

    #[ink(message)]
    fn get_draw_number(&self) -> DrawNumber {
        self.data::<Data>().draw_number
    }

    #[ink(message)]
    fn get_status(&self) -> Status {
        self.data::<Data>().status
    }

}
