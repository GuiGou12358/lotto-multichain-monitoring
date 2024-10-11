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
    RegistrationsOpen,
    RegistrationsClosed,
    ResultsReceived,
}

#[openbrush::trait_definition]
pub trait Raffle: Storage<Data> {
    /// start (the config cannot be updated anymore)
    fn start(&mut self) -> Result<(), RaffleError> {
        // check the status
        if self.data::<Data>().status != Status::NotStarted {
            return Err(IncorrectStatus);
        }

        self.data::<Data>().status = Status::Started;

        Ok(())
    }

    /// Open the registrations
    fn open_registrations(&mut self, draw_number: DrawNumber) -> Result<(), RaffleError> {
        // check the status
        if self.data::<Data>().status != Status::Started
            && self.data::<Data>().status != Status::ResultsReceived
        {
            return Err(IncorrectStatus);
        }

        self.data::<Data>().draw_number = draw_number;
        self.data::<Data>().status = Status::RegistrationsOpen;

        Ok(())
    }

    /// Close the registrations
    fn close_registrations(&mut self, draw_number: DrawNumber) -> Result<(), RaffleError> {
        // check the status
        if self.data::<Data>().status != Status::RegistrationsOpen {
            return Err(IncorrectStatus);
        }
        // check the draw number
        if self.data::<Data>().draw_number != draw_number {
            return Err(IncorrectDrawNumber);
        }
        // update the status
        self.data::<Data>().status = Status::RegistrationsClosed;
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
        if self.data::<Data>().status != Status::RegistrationsClosed {
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
        if self.data::<Data>().status != Status::RegistrationsOpen {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_contract::lotto_contract::Contract;

    #[ink::test]
    fn test_start() {
        let mut contract = Contract::new();

        assert_eq!(contract.get_status(), Status::NotStarted);
        assert_eq!(contract.get_draw_number(), 0);

        contract.start().expect("Fail to start");

        assert_eq!(contract.get_status(), Status::Started);
        assert_eq!(contract.get_draw_number(), 0);
    }

    #[ink::test]
    fn test_open_registrations() {
        let mut contract = Contract::new();

        assert_eq!(contract.open_registrations(10), Err(IncorrectStatus));

        contract.start().expect("Fail to start");

        contract
            .open_registrations(10)
            .expect("Fail to open the registrations");
        assert_eq!(contract.get_status(), Status::RegistrationsOpen);
        assert_eq!(contract.get_draw_number(), 10);

        assert_eq!(contract.open_registrations(10), Err(IncorrectStatus));
        assert_eq!(contract.open_registrations(11), Err(IncorrectStatus));
    }

    #[ink::test]
    fn test_close_registrations() {
        let mut contract = Contract::new();

        assert_eq!(contract.close_registrations(10), Err(IncorrectStatus));

        contract.start().expect("Fail to start");

        assert_eq!(contract.close_registrations(10), Err(IncorrectStatus));

        contract
            .open_registrations(10)
            .expect("Fail to open the registrations");

        assert_eq!(contract.close_registrations(9), Err(IncorrectDrawNumber));
        assert_eq!(contract.close_registrations(11), Err(IncorrectDrawNumber));

        contract
            .close_registrations(10)
            .expect("Fail to close the registrations");
        assert_eq!(contract.get_status(), Status::RegistrationsClosed);
        assert_eq!(contract.get_draw_number(), 10);
    }

    #[ink::test]
    fn test_save_results() {
        let mut contract = Contract::new();

        assert_eq!(
            contract.save_results(10, vec![], vec![]),
            Err(IncorrectStatus)
        );

        contract.start().expect("Fail to start");

        assert_eq!(
            contract.save_results(10, vec![], vec![]),
            Err(IncorrectStatus)
        );

        contract
            .open_registrations(10)
            .expect("Fail to open the registrations");

        assert_eq!(
            contract.save_results(10, vec![], vec![]),
            Err(IncorrectStatus)
        );

        contract
            .close_registrations(10)
            .expect("Fail to close the registrations");

        assert_eq!(
            contract.save_results(9, vec![], vec![]),
            Err(IncorrectDrawNumber)
        );
        assert_eq!(
            contract.save_results(11, vec![], vec![]),
            Err(IncorrectDrawNumber)
        );

        contract
            .save_results(10, vec![], vec![])
            .expect("Fail to save the results");
        assert_eq!(contract.get_status(), Status::ResultsReceived);
        assert_eq!(contract.get_draw_number(), 10);
    }

    #[ink::test]
    fn test_can_participate() {
        let mut contract = Contract::new();

        assert_eq!(contract.can_participate(), Err(IncorrectStatus));

        contract.start().expect("Fail to start");

        assert_eq!(contract.can_participate(), Err(IncorrectStatus));

        contract
            .open_registrations(10)
            .expect("Fail to open the registrations");

        contract.can_participate().expect("Fail to participate");

        contract
            .close_registrations(10)
            .expect("Fail to close the registrations");

        assert_eq!(contract.can_participate(), Err(IncorrectStatus));

        contract
            .save_results(10, vec![], vec![])
            .expect("Fail to save the results");

        assert_eq!(contract.can_participate(), Err(IncorrectStatus));
    }

    #[ink::test]
    fn test_reopen_after_results() {
        let mut contract = Contract::new();
        contract.start().expect("Fail to start");
        contract
            .open_registrations(10)
            .expect("Fail to open the registrations");
        contract
            .close_registrations(10)
            .expect("Fail to close the registrations");
        contract
            .save_results(10, vec![], vec![])
            .expect("Fail to save the results");

        contract
            .open_registrations(13)
            .expect("Fail to open the registrations");
        assert_eq!(contract.get_status(), Status::RegistrationsOpen);
        assert_eq!(contract.get_draw_number(), 13);
    }

    #[ink::test]
    fn test_full() {
        let mut contract = Contract::new();
        contract.start().expect("Fail to start");
        contract
            .open_registrations(1)
            .expect("Fail to open the Registrations");
        contract
            .close_registrations(1)
            .expect("Fail to open the Registrations");
        contract
            .save_results(1, vec![], vec![])
            .expect("Fail to save the results");
        contract
            .open_registrations(2)
            .expect("Fail to open the Registrations");
        contract
            .close_registrations(2)
            .expect("Fail to open the Registrations");
        contract
            .save_results(2, vec![], vec![])
            .expect("Fail to save the results");
    }
}
