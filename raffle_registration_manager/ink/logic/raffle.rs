use crate::error::{RaffleError, RaffleError::*};
use crate::{Number, DrawNumber, RegistrationContractId};
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
    WaitingResults,
    WaitingWinners,
    Closed,
}

#[openbrush::trait_definition]
pub trait Raffle: Storage<Data> {
    /// Open the registrations
    fn add_registration_contract(&mut self, registration_contract: RegistrationContractId) -> Result<(), RaffleError> {
        // check the status
        self.check_registrations_status(Status::NotStarted)?;

        // add the new contract
        self.data::<Data>().registration_contracts.push(registration_contract);
        // add the default status for this added contract
        self.data::<Data>().registration_contracts_status.insert(registration_contract, &Status::NotStarted);

        Ok(())
    }

    /// start
    fn start(&mut self) -> Result<(), RaffleError> {
        // check the status
        self.check_registrations_status(Status::Started)?;

        self.data::<Data>().status = Status::RegistrationsOpen;

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
        self.check_registrations_status(self.get_status())?;

        // increment the draw number
        let new_draw_number = self.data::<Data>().draw_number.checked_add(1).ok_or(AddOverFlow)?;

        self.data::<Data>().draw_number = new_draw_number;
        self.data::<Data>().status = Status::RegistrationsOpen;

        Ok(new_draw_number)
    }

    /// Close the registrations
    fn close_registrations(&mut self) -> Result<DrawNumber, RaffleError> {
        // check the status
        self.check_registrations_status(Status::RegistrationsOpen)?;

        // update the status
        self.data::<Data>().status = Status::RegistrationsClosed;
        Ok(self.data::<Data>().draw_number)
    }

    /// Save the status for given registration contracts
    /// return the contracts not synchronized yet
    fn check_registrations_status(
        &mut self,
        status: Status,
    ) -> Result<(), RaffleError> {
        // check the status in the manager
        if self.data::<Data>().status != status {
            return Err(IncorrectStatus);
        }

        // check the status in all  registration contracts
        for i in 0..self.data::<Data>().registration_contracts.len() {
            let contract_id = self.data::<Data>().registration_contracts[i];
            let contract_status = self.data::<Data>().registration_contracts_status.get(contract_id);
            if contract_status != Some(status) {
                return Err(IncorrectStatus);
            }
        }

        Ok(())
    }

    /// Save the status for given registration contracts
    /// return the contracts not synchronized yer
    fn save_registrations_status(
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
            self.data::<Data>().registration_contracts_status.insert(registration_contract, &status);
        }

        // contract not synchronized yet
        let mut not_synchronized_contracts = Vec::new();

        for i in 0..self.data::<Data>().registration_contracts.len() {
            let contract_id = self.data::<Data>().registration_contracts[i];
            let contract_status = self.data::<Data>().registration_contracts_status.get(contract_id);
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
    fn get_registration_contract_status(&self, registration_contract: RegistrationContractId) -> Option<Status> {
        self.data::<Data>().registration_contracts_status.get(registration_contract)
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
        if self.data::<Data>().status != Status::WaitingResults {
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
