use crate::error::{RaffleError, RaffleError::*};
use crate::DrawNumber;
use ink::prelude::vec::Vec;
use ink::storage::Mapping;
use openbrush::traits::{AccountId, Storage};

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    draw_number: DrawNumber,
    status: Status,
    winners: Mapping<DrawNumber, Vec<AccountId>>,
}

#[derive(Default, Debug, Eq, PartialEq, Copy, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum Status {
    #[default]
    NotStarted,
    Ongoing, // TODO rename with RegistrationOpen
    WaitingResults, // TODO rename RegistrationClosed
    Closed,  // TODO rename ResultReceived in the manager
}

#[openbrush::trait_definition]
pub trait Raffle: Storage<Data> {
    /// Open the registrations
    fn open_registrations(
        &mut self,
        draw_number: DrawNumber,
    ) -> Result<(), RaffleError> {
        // check the status
        if self.data::<Data>().status != Status::NotStarted
            && self.data::<Data>().status != Status::Closed
        {
            return Err(IncorrectStatus);
        }

        self.data::<Data>().draw_number = draw_number;
        self.data::<Data>().status = Status::Ongoing;

        Ok(())
    }

    /// Close the registrations
    fn close_registrations(
        &mut self,
        draw_number: DrawNumber,
    ) -> Result<(), RaffleError> {
        // check the status
        if self.data::<Data>().status != Status::Ongoing {
            return Err(IncorrectStatus);
        }
        // check the draw number
        if self.data::<Data>().draw_number != draw_number {
            return Err(IncorrectDrawNumber);
        }
        // update the status
        self.data::<Data>().status = Status::WaitingResults;
        Ok(())
    }

    /// save the winners for the draw number.
    fn set_results(
        &mut self,
        draw_number: DrawNumber,
        winners: Vec<AccountId>,
    ) -> Result<(), RaffleError> {
        // check the status
        if self.data::<Data>().status != Status::WaitingResults {
            return Err(IncorrectStatus);
        }
        // check the draw number
        if self.data::<Data>().draw_number != draw_number {
            return Err(IncorrectDrawNumber);
        }

        match self.data::<Data>().winners.get(draw_number) {
            Some(_) => Err(ExistingWinners),
            None => {
                // save the result
                self.data::<Data>().winners.insert(draw_number, &winners); // TODO check if we need it
                // update the status
                self.data::<Data>().status = Status::Closed;
                Ok(())
            }
        }
    }

    /// check if the registrations are open
    fn can_participate(&mut self) -> Result<(), RaffleError> {
        // check the status
        if self.data::<Data>().status != Status::Ongoing {
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

    #[ink(message)]
    fn get_winners(&self, draw_number: DrawNumber) -> Option<Vec<AccountId>> {
        self.data::<Data>().winners.get(draw_number)
    }
}
