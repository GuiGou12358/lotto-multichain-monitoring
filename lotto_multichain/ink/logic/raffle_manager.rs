use crate::error::{RaffleError, RaffleError::*};
use crate::{DrawNumber, Number, RegistrationContractId};
use ink::prelude::vec::Vec;
use ink::storage::Mapping;
use openbrush::traits::{AccountId, Storage};

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    draw_number: DrawNumber,
    status: Status,
    registration_contracts: Vec<RegistrationContractId>,
    registration_contracts_status: Mapping<RegistrationContractId, Status>,
    results: Mapping<DrawNumber, Vec<Number>>,
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
    Started,
    RegistrationsOpen,
    RegistrationsClosed,
    //WaitingResults,
    WaitingWinners,
    Closed,
}

#[openbrush::trait_definition]
pub trait RaffleManager: Storage<Data> {
    /// Open the registrations
    fn add_registration_contract(
        &mut self,
        registration_contract: RegistrationContractId,
    ) -> Result<(), RaffleError> {
        // check the status
        self.check_registration_contracts_status(Status::NotStarted)?;

        // add the new contract
        self.data::<Data>()
            .registration_contracts
            .push(registration_contract);
        // add the default status for this added contract
        self.data::<Data>()
            .registration_contracts_status
            .insert(registration_contract, &Status::NotStarted);

        Ok(())
    }

    /// start
    fn start(&mut self, previous_draw_number: DrawNumber) -> Result<(), RaffleError> {
        // check the status
        self.check_registration_contracts_status(Status::NotStarted)?;

        self.data::<Data>().draw_number = previous_draw_number;
        self.data::<Data>().status = Status::Started;

        Ok(())
    }

    /// Open the registrations
    fn open_registrations(&mut self) -> Result<DrawNumber, RaffleError> {
        // check the status
        if self.data::<Data>().status != Status::Started
            && self.data::<Data>().status != Status::Closed
        {
            return Err(IncorrectStatus);
        }
        // check the status
        self.check_registration_contracts_status(self.get_status())?;

        // increment the draw number
        let new_draw_number = self
            .data::<Data>()
            .draw_number
            .checked_add(1)
            .ok_or(AddOverFlow)?;

        self.data::<Data>().draw_number = new_draw_number;
        self.data::<Data>().status = Status::RegistrationsOpen;

        Ok(new_draw_number)
    }

    /// Return true if the registrations can be closed
    fn can_close_registrations(&self) -> bool {
        self.check_registration_contracts_status(Status::RegistrationsOpen)
            .is_ok()
    }

    /// Close the registrations
    fn close_registrations(&mut self) -> Result<DrawNumber, RaffleError> {
        // check the status
        self.check_registration_contracts_status(Status::RegistrationsOpen)?;

        // update the status
        self.data::<Data>().status = Status::RegistrationsClosed;
        Ok(self.data::<Data>().draw_number)
    }

    /// Save the status for given registration contracts
    /// return the contracts not synchronized yet
    fn check_registration_contracts_status(&self, status: Status) -> Result<(), RaffleError> {
        // check the status in the manager
        if self.data::<Data>().status != status {
            return Err(IncorrectStatus);
        }

        // check the status in all  registration contracts
        for i in 0..self.data::<Data>().registration_contracts.len() {
            let contract_id = self.data::<Data>().registration_contracts[i];
            let contract_status = self
                .data::<Data>()
                .registration_contracts_status
                .get(contract_id);
            if contract_status != Some(status) {
                return Err(IncorrectStatus);
            }
        }

        Ok(())
    }

    /// Save the status for given registration contracts
    /// return the contracts not synchronized yer
    fn save_registration_contracts_status(
        &mut self,
        draw_number: DrawNumber,
        status: Status,
        registration_contracts: Vec<RegistrationContractId>,
    ) -> Result<Vec<RegistrationContractId>, RaffleError> {
        // check the status
        if self.data::<Data>().status != status {
            return Err(IncorrectStatus);
        }
        // check the draw number
        if self.data::<Data>().draw_number != draw_number {
            return Err(IncorrectDrawNumber);
        }

        for registration_contract in &registration_contracts {
            self.data::<Data>()
                .registration_contracts_status
                .insert(registration_contract, &status);
        }

        // contract not synchronized yet
        let mut not_synchronized_contracts = Vec::new();

        for i in 0..self.data::<Data>().registration_contracts.len() {
            let contract_id = self.data::<Data>().registration_contracts[i];
            let contract_status = self
                .data::<Data>()
                .registration_contracts_status
                .get(contract_id);
            if contract_status != Some(status) {
                not_synchronized_contracts.push(contract_id);
            }
        }

        Ok(not_synchronized_contracts)
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
    fn get_registration_contracts(&self) -> Vec<RegistrationContractId> {
        self.data::<Data>().registration_contracts.clone()
    }

    #[ink(message)]
    fn get_registration_contract_status(
        &self,
        registration_contract: RegistrationContractId,
    ) -> Option<Status> {
        self.data::<Data>()
            .registration_contracts_status
            .get(registration_contract)
    }

    #[ink(message)]
    fn get_results(&self, draw_number: DrawNumber) -> Option<Vec<Number>> {
        self.data::<Data>().results.get(draw_number)
    }

    #[ink(message)]
    fn get_winners(&self, draw_number: DrawNumber) -> Option<Vec<AccountId>> {
        self.data::<Data>().winners.get(draw_number)
    }

    /// save the results for the current raffle.
    fn set_results(
        &mut self,
        draw_number: DrawNumber,
        results: Vec<Number>,
    ) -> Result<(), RaffleError> {
        // check the raffle number
        if self.data::<Data>().draw_number != draw_number {
            return Err(IncorrectDrawNumber);
        }

        // check the status
        if self.data::<Data>().status != Status::RegistrationsClosed {
            return Err(IncorrectStatus);
        }

        match self.data::<Data>().results.get(draw_number) {
            Some(_) => Err(ExistingResults),
            None => {
                // save the results
                self.data::<Data>().results.insert(draw_number, &results);
                // update the status
                self.data::<Data>().status = Status::WaitingWinners;
                Ok(())
            }
        }
    }

    /// check if the saved results are the same as the ones given in parameter
    fn ensure_same_results(
        &mut self,
        draw_number: DrawNumber,
        numbers: &[Number],
    ) -> Result<(), RaffleError> {
        // get the correct results for the given raffle
        let result = self
            .data::<Data>()
            .results
            .get(draw_number)
            .ok_or(DifferentResults)?;

        if result.len() != numbers.len() {
            return Err(DifferentResults);
        }

        for i in 0..numbers.len() {
            if numbers[i] != result[i] {
                return Err(DifferentResults);
            }
        }

        Ok(())
    }

    /// save the winners for the current raffle.
    fn set_winners(
        &mut self,
        draw_number: DrawNumber,
        winners: Vec<AccountId>,
    ) -> Result<(), RaffleError> {
        // check the raffle number
        if self.data::<Data>().draw_number != draw_number {
            return Err(IncorrectDrawNumber);
        }

        // check the status
        if self.data::<Data>().status != Status::WaitingWinners {
            return Err(IncorrectStatus);
        }

        match self.data::<Data>().winners.get(draw_number) {
            Some(_) => Err(ExistingWinners),
            None => {
                // save the result
                self.data::<Data>().winners.insert(draw_number, &winners);
                // update the status
                self.data::<Data>().status = Status::Closed;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_contract::lotto_contract::Contract;

    #[ink::test]
    fn test_add_registrations_contract() {
        let mut contract = Contract::new();

        contract
            .add_registration_contract(100)
            .expect("Fail to add registrations contract");
        contract
            .add_registration_contract(101)
            .expect("Fail to add registrations contract");
        contract
            .add_registration_contract(102)
            .expect("Fail to add registrations contract");

        assert_eq!(contract.get_status(), Status::NotStarted);
        assert_eq!(contract.get_draw_number(), 0);
        assert_eq!(contract.get_registration_contracts(), vec![100, 101, 102]);
        assert_eq!(
            contract.get_registration_contract_status(100),
            Some(Status::NotStarted)
        );
        assert_eq!(
            contract.get_registration_contract_status(101),
            Some(Status::NotStarted)
        );
        assert_eq!(
            contract.get_registration_contract_status(102),
            Some(Status::NotStarted)
        );
        assert_eq!(contract.get_registration_contract_status(103), None);
    }

    #[ink::test]
    fn test_start() {
        let mut contract = Contract::new();

        assert_eq!(contract.get_status(), Status::NotStarted);
        assert_eq!(contract.get_draw_number(), 0);

        contract.start(1).expect("Fail to start");

        assert_eq!(contract.get_status(), Status::Started);
        assert_eq!(contract.get_draw_number(), 1);

        // we cannot add registration contract when it started
        assert_eq!(contract.add_registration_contract(1), Err(IncorrectStatus));
    }

    #[ink::test]
    fn test_open_registrations() {
        let mut contract = Contract::new();

        assert_eq!(contract.open_registrations(), Err(IncorrectStatus));

        contract.start(0).expect("Fail to start");

        contract
            .open_registrations()
            .expect("Fail to open the registrations");
        assert_eq!(contract.get_status(), Status::RegistrationsOpen);
        assert_eq!(contract.get_draw_number(), 1);

        assert_eq!(contract.open_registrations(), Err(IncorrectStatus));
    }

    #[ink::test]
    fn test_close_registrations() {
        let mut contract = Contract::new();

        assert_eq!(false, contract.can_close_registrations());
        assert_eq!(contract.close_registrations(), Err(IncorrectStatus));

        contract.start(0).expect("Fail to start");

        assert_eq!(false, contract.can_close_registrations());
        assert_eq!(contract.close_registrations(), Err(IncorrectStatus));

        contract
            .open_registrations()
            .expect("Fail to open the registrations");

        assert_eq!(true, contract.can_close_registrations());
        contract
            .close_registrations()
            .expect("Fail to close the registrations");
        assert_eq!(contract.get_status(), Status::RegistrationsClosed);
        assert_eq!(contract.get_draw_number(), 1);
    }

    #[ink::test]
    fn test_set_results() {
        let mut contract = Contract::new();

        contract.start(0).expect("Fail to start");
        contract
            .open_registrations()
            .expect("Fail to open the registrations");

        assert_eq!(contract.set_results(1, vec![]), Err(IncorrectStatus));

        contract
            .close_registrations()
            .expect("Fail to close the registrations");

        assert_eq!(contract.set_results(0, vec![]), Err(IncorrectDrawNumber));
        assert_eq!(contract.set_results(2, vec![]), Err(IncorrectDrawNumber));

        assert_eq!(contract.get_status(), Status::RegistrationsClosed);
        assert_eq!(contract.get_draw_number(), 1);
        contract
            .set_results(1, vec![1, 2, 3, 4])
            .expect("Fail to save the results");
        assert_eq!(contract.get_status(), Status::WaitingWinners);
        assert_eq!(contract.get_draw_number(), 1);

        assert_eq!(contract.get_results(1), Some(vec![1, 2, 3, 4]));
        assert_eq!(contract.get_results(0), None);
        assert_eq!(contract.get_results(2), None);
    }

    #[ink::test]
    fn test_ensure_same_results() {
        let mut contract = Contract::new();

        contract.start(0).expect("Fail to start");
        contract
            .open_registrations()
            .expect("Fail to open the registrations");
        contract
            .close_registrations()
            .expect("Fail to close the registrations");
        contract
            .set_results(1, vec![1, 2, 3, 4])
            .expect("Fail to save the results");

        contract
            .ensure_same_results(1, vec![1, 2, 3, 4].as_slice())
            .expect("Check same results failed");
        assert_eq!(
            contract.ensure_same_results(1, vec![1, 2, 3, 5].as_slice()),
            Err(DifferentResults)
        );
        assert_eq!(
            contract.ensure_same_results(2, vec![1, 2, 3, 4].as_slice()),
            Err(DifferentResults)
        );
    }

    #[ink::test]
    fn test_set_winners() {
        let mut contract = Contract::new();

        contract.start(0).expect("Fail to start");
        contract
            .open_registrations()
            .expect("Fail to open the registrations");

        assert_eq!(contract.set_winners(1, vec![]), Err(IncorrectStatus));

        contract
            .close_registrations()
            .expect("Fail to close the registrations");

        assert_eq!(contract.set_winners(1, vec![]), Err(IncorrectStatus));

        contract
            .set_results(1, vec![1, 2, 3, 4])
            .expect("Fail to save the results");

        assert_eq!(contract.set_winners(0, vec![]), Err(IncorrectDrawNumber));
        assert_eq!(contract.set_winners(2, vec![]), Err(IncorrectDrawNumber));

        assert_eq!(contract.get_status(), Status::WaitingWinners);
        assert_eq!(contract.get_draw_number(), 1);
        contract
            .set_winners(1, vec![])
            .expect("Fail to save the winners");

        assert_eq!(contract.get_status(), Status::Closed);
        assert_eq!(contract.get_draw_number(), 1);

        assert_eq!(contract.get_winners(1), Some(vec![]));
        assert_eq!(contract.get_results(0), None);
        assert_eq!(contract.get_results(2), None);
    }

    #[ink::test]
    fn test_reopen_after_results_and_winners() {
        let mut contract = Contract::new();
        contract.start(0).expect("Fail to start");
        contract
            .open_registrations()
            .expect("Fail to open the registrations");
        contract
            .close_registrations()
            .expect("Fail to close the registrations");
        contract
            .set_results(1, vec![1, 2, 3, 4])
            .expect("Fail to save the results");
        contract
            .set_winners(1, vec![])
            .expect("Fail to save the winners");

        contract
            .open_registrations()
            .expect("Fail to open the registrations");
        assert_eq!(contract.get_status(), Status::RegistrationsOpen);
        assert_eq!(contract.get_draw_number(), 2);
    }

    #[ink::test]
    fn test_registration_contracts_status() {
        let mut contract = Contract::new();

        contract
            .add_registration_contract(100)
            .expect("Fail to add registrations contract");
        contract
            .add_registration_contract(101)
            .expect("Fail to add registrations contract");
        contract
            .add_registration_contract(102)
            .expect("Fail to add registrations contract");

        contract
            .check_registration_contracts_status(Status::NotStarted)
            .expect("Check status failed");

        // start
        contract.start(0).expect("Fail to start");

        assert_eq!(contract.get_status(), Status::Started);
        assert_eq!(
            contract.get_registration_contract_status(100),
            Some(Status::NotStarted)
        );
        assert_eq!(
            contract.get_registration_contract_status(101),
            Some(Status::NotStarted)
        );
        assert_eq!(
            contract.get_registration_contract_status(102),
            Some(Status::NotStarted)
        );

        assert_eq!(
            contract.check_registration_contracts_status(Status::NotStarted),
            Err(IncorrectStatus)
        );
        assert_eq!(
            contract.check_registration_contracts_status(Status::Started),
            Err(IncorrectStatus)
        );

        assert_eq!(contract.get_draw_number(), 0);
        // cannot save the draw number doesn't match
        assert_eq!(
            contract.save_registration_contracts_status(1, Status::Started, vec![100, 102]),
            Err(IncorrectDrawNumber)
        );

        contract
            .save_registration_contracts_status(0, Status::Started, vec![100, 102])
            .expect("Save status failed");
        assert_eq!(contract.get_status(), Status::Started);
        assert_eq!(
            contract.get_registration_contract_status(100),
            Some(Status::Started)
        );
        assert_eq!(
            contract.get_registration_contract_status(101),
            Some(Status::NotStarted)
        );
        assert_eq!(
            contract.get_registration_contract_status(102),
            Some(Status::Started)
        );

        assert_eq!(
            contract.check_registration_contracts_status(Status::NotStarted),
            Err(IncorrectStatus)
        );
        assert_eq!(
            contract.check_registration_contracts_status(Status::Started),
            Err(IncorrectStatus)
        );

        contract
            .save_registration_contracts_status(0, Status::Started, vec![101])
            .expect("Save status failed");

        contract
            .check_registration_contracts_status(Status::Started)
            .expect("Check status failed");

        // open the registrations
        contract
            .open_registrations()
            .expect("Fail to open the Registrations");
        assert_eq!(
            contract.check_registration_contracts_status(Status::RegistrationsOpen),
            Err(IncorrectStatus)
        );

        contract
            .save_registration_contracts_status(1, Status::RegistrationsOpen, vec![100, 101, 102])
            .expect("Save status failed");
        contract
            .check_registration_contracts_status(Status::RegistrationsOpen)
            .expect("Check status failed");

        // cannot save an incorrect
        assert_eq!(
            contract.save_registration_contracts_status(
                1,
                Status::RegistrationsClosed,
                vec![100, 101, 102]
            ),
            Err(IncorrectStatus)
        );

        // close the registrations
        contract
            .close_registrations()
            .expect("Fail to close the Registrations");
        assert_eq!(
            contract.check_registration_contracts_status(Status::RegistrationsClosed),
            Err(IncorrectStatus)
        );

        contract
            .save_registration_contracts_status(1, Status::RegistrationsClosed, vec![100, 101, 102])
            .expect("Save status failed");
        contract
            .check_registration_contracts_status(Status::RegistrationsClosed)
            .expect("Check status failed");
    }

    #[ink::test]
    fn test_full() {
        let mut contract = Contract::new();

        contract
            .add_registration_contract(100)
            .expect("Fail to add registrations contract");
        contract
            .add_registration_contract(101)
            .expect("Fail to add registrations contract");
        contract
            .add_registration_contract(102)
            .expect("Fail to add registrations contract");

        // start
        contract.start(0).expect("Fail to start");
        contract
            .save_registration_contracts_status(0, Status::Started, vec![100, 101, 102])
            .expect("Save status failed");
        // first draw
        contract
            .open_registrations()
            .expect("Fail to open the Registrations");
        contract
            .save_registration_contracts_status(1, Status::RegistrationsOpen, vec![100, 101, 102])
            .expect("Save status failed");
        contract
            .close_registrations()
            .expect("Fail to open the Registrations");
        contract
            .save_registration_contracts_status(1, Status::RegistrationsClosed, vec![100, 101, 102])
            .expect("Save status failed");
        contract
            .set_results(1, vec![1, 2, 3, 4])
            .expect("Fail to save the results");
        contract
            .set_winners(1, vec![])
            .expect("Fail to save the winners");
        contract
            .save_registration_contracts_status(1, Status::Closed, vec![100, 101, 102])
            .expect("Save status failed");
        // second draw
        contract
            .open_registrations()
            .expect("Fail to open the Registrations");
        contract
            .save_registration_contracts_status(2, Status::RegistrationsOpen, vec![100, 101, 102])
            .expect("Save status failed");
        contract
            .close_registrations()
            .expect("Fail to open the Registrations");
        contract
            .save_registration_contracts_status(2, Status::RegistrationsClosed, vec![100, 101, 102])
            .expect("Save status failed");
        contract
            .set_results(2, vec![10, 35, 8, 10])
            .expect("Fail to save the results");
        contract
            .set_winners(2, vec![])
            .expect("Fail to save the winners");
        contract
            .save_registration_contracts_status(2, Status::Closed, vec![100, 101, 102])
            .expect("Save status failed");
    }
}
