use crate::error::{RaffleError, RaffleError::*};
use crate::{DrawNumber, Number};
use ink::prelude::vec::Vec;
use openbrush::traits::AccountId;
use phat_rollup_anchor_ink::traits::rollup_anchor::RollupAnchor;
use scale::{Decode, Encode};

const STATUS: u32 = ink::selector_id!("STATUS");
const DRAW_NUMBER: u32 = ink::selector_id!("DRAW_NUMBER");

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
pub trait Raffle: RollupAnchor {
    /// start (the config cannot be updated anymore)
    fn start(&mut self) -> Result<(), RaffleError> {
        // check the status
        if self.get_status()? != Status::NotStarted {
            return Err(IncorrectStatus);
        }

        self.set_status(Status::Started);

        Ok(())
    }

    /// Open the registrations
    fn open_registrations(&mut self, draw_number: DrawNumber) -> Result<(), RaffleError> {
        // check the status
        let status = self.get_status()?;
        if status != Status::Started && status != Status::ResultsReceived {
            return Err(IncorrectStatus);
        }

        self.set_draw_number(draw_number);
        self.set_status(Status::RegistrationsOpen);

        Ok(())
    }

    /// Close the registrations
    fn close_registrations(&mut self, draw_number: DrawNumber) -> Result<(), RaffleError> {
        // check the status
        if self.get_status()? != Status::RegistrationsOpen {
            return Err(IncorrectStatus);
        }
        // check the draw number
        if self.get_draw_number()? != draw_number {
            return Err(IncorrectDrawNumber);
        }
        // update the status
        self.set_status(Status::RegistrationsClosed);
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
        if self.get_status()? != Status::RegistrationsClosed {
            return Err(IncorrectStatus);
        }
        // check the draw number
        if self.get_draw_number()? != draw_number {
            return Err(IncorrectDrawNumber);
        }

        self.set_status(Status::ResultsReceived);
        Ok(())
    }

    /// check if the registrations are open
    fn check_can_participate(&mut self) -> Result<(), RaffleError> {
        // check the status
        if !self.can_participate() {
            return Err(IncorrectStatus);
        }

        Ok(())
    }

    /// check if the user can participate are open
    #[ink(message)]
    fn can_participate(&mut self) -> bool {
        self.get_status() == Ok(Status::RegistrationsOpen)
    }

    #[ink(message)]
    fn get_draw_number(&self) -> Result<DrawNumber, RaffleError> {
        match RollupAnchor::get_value(self, DRAW_NUMBER.encode()) {
            Some(v) => DrawNumber::decode(&mut v.as_slice()).map_err(|_| FailedToDecode),
            _ => Ok(0),
        }
    }

    fn set_draw_number(&mut self, draw_number: DrawNumber) {
        RollupAnchor::set_value(self, &DRAW_NUMBER.encode(), Some(&draw_number.encode()));
    }

    #[ink(message)]
    fn get_status(&self) -> Result<Status, RaffleError> {
        match RollupAnchor::get_value(self, STATUS.encode()) {
            Some(v) => Status::decode(&mut v.as_slice()).map_err(|_| FailedToDecode),
            _ => Ok(Status::NotStarted),
        }
    }

    fn set_status(&mut self, status: Status) {
        RollupAnchor::set_value(self, &STATUS.encode(), Some(&status.encode()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_contract::lotto_contract::Contract;

    #[ink::test]
    fn test_start() {
        let mut contract = Contract::new();

        assert_eq!(contract.get_status(), Ok(Status::NotStarted));
        assert_eq!(contract.get_draw_number(), Ok(0));

        contract.start().expect("Fail to start");

        assert_eq!(contract.get_status(), Ok(Status::Started));
        assert_eq!(contract.get_draw_number(), Ok(0));
    }

    #[ink::test]
    fn test_open_registrations() {
        let mut contract = Contract::new();

        assert_eq!(contract.open_registrations(10), Err(IncorrectStatus));

        contract.start().expect("Fail to start");

        contract
            .open_registrations(10)
            .expect("Fail to open the registrations");
        assert_eq!(contract.get_status(), Ok(Status::RegistrationsOpen));
        assert_eq!(contract.get_draw_number(), Ok(10));

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
        assert_eq!(contract.get_status(), Ok(Status::RegistrationsClosed));
        assert_eq!(contract.get_draw_number(), Ok(10));
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
        assert_eq!(contract.get_status(), Ok(Status::ResultsReceived));
        assert_eq!(contract.get_draw_number(), Ok(10));
    }

    #[ink::test]
    fn test_can_participate() {
        let mut contract = Contract::new();

        assert_eq!(contract.can_participate(), false);

        contract.start().expect("Fail to start");

        assert_eq!(contract.can_participate(), false);
        assert_eq!(contract.check_can_participate(), Err(IncorrectStatus));

        contract
            .open_registrations(10)
            .expect("Fail to open the registrations");

        assert!(contract.can_participate());
        contract
            .check_can_participate()
            .expect("Check Participations Failed");

        contract
            .close_registrations(10)
            .expect("Fail to close the registrations");

        assert_eq!(contract.can_participate(), false);

        contract
            .save_results(10, vec![], vec![])
            .expect("Fail to save the results");

        assert_eq!(contract.can_participate(), false);
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
        assert_eq!(contract.get_status(), Ok(Status::RegistrationsOpen));
        assert_eq!(contract.get_draw_number(), Ok(13));
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
