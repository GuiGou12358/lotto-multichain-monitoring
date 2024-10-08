#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[openbrush::implementation(Ownable, AccessControl, Upgradeable)]
#[openbrush::contract]
pub mod lotto_contract {
    use ink::prelude::vec::Vec;
    use lotto_registration::{
        config, config::*, error::*, raffle, raffle::*, Number, DrawNumber
    };
    use openbrush::contracts::access_control::*;
    use openbrush::contracts::ownable::*;
    use openbrush::{modifiers, traits::Storage};
    use phat_rollup_anchor_ink::traits::{
        meta_transaction, meta_transaction::*, rollup_anchor, rollup_anchor::*,
    };

    const LOTTO_MANAGER_ROLE: RoleType = ink::selector_id!("LOTTO_MANAGER");


    /// Event emitted when the config is received
    #[ink(event)]
    pub struct ConfigUpdated {
        #[ink(topic)]
        contract_id: AccountId,
        config: Config,
    }

    /// Event emitted when the participation is registered
    #[ink(event)]
    pub struct ParticipationRegistered {
        #[ink(topic)]
        contract_id: AccountId,
        #[ink(topic)]
        draw_number: DrawNumber,
        #[ink(topic)]
        participant: AccountId,
        numbers: Vec<Number>,
    }

    /// Event emitted when the registrations are open
    #[ink(event)]
    pub struct RegistrationsOpen {
        #[ink(topic)]
        contract_id: AccountId,
        #[ink(topic)]
        draw_number: DrawNumber,
    }

    /// Event emitted when the registrations are closed
    #[ink(event)]
    pub struct RegistrationsClosed {
        #[ink(topic)]
        contract_id: AccountId,
        #[ink(topic)]
        draw_number: DrawNumber,
    }

    /// Event emitted when the results are received
    #[ink(event)]
    pub struct ResultsReceived {
        #[ink(topic)]
        contract_id: AccountId,
        #[ink(topic)]
        draw_number: DrawNumber,
        numbers: Vec<Number>,
        winners: Vec<AccountId>,
    }

    /// Errors occurred in the contract
    #[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        AccessControlError(AccessControlError),
        RaffleError(RaffleError),
        RollupAnchorError(RollupAnchorError),
        TransferError,
    }

    /// convertor from AccessControlError to ContractError
    impl From<AccessControlError> for ContractError {
        fn from(error: AccessControlError) -> Self {
            ContractError::AccessControlError(error)
        }
    }

    /// convertor from RaffleError to ContractError
    impl From<RaffleError> for ContractError {
        fn from(error: RaffleError) -> Self {
            ContractError::RaffleError(error)
        }
    }

    /// convertor from RaffleError to ContractError
    impl From<RollupAnchorError> for ContractError {
        fn from(error: RollupAnchorError) -> Self {
            ContractError::RollupAnchorError(error)
        }
    }

    /// convertor from RaffleError to ContractError
    impl From<ContractError> for RollupAnchorError {
        fn from(error: ContractError) -> Self {
            ink::env::debug_println!("Error: {:?}", error);
            RollupAnchorError::UnsupportedAction
        }
    }

    /// Message to request for action
    /// Message sent by the offchain rollup to the Ink! smart contract
    #[derive(scale::Encode, scale::Decode)]
    /*
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
     */
    pub enum RequestForAction {
        SetConfig(Config),
        OpenRegistrations(DrawNumber),
        CloseRegistrations(DrawNumber),
        SetResults(DrawNumber, Vec<Number>, Vec<AccountId>),
    }

    /// Contract storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        access: access_control::Data,
        #[storage_field]
        rollup_anchor: rollup_anchor::Data,
        #[storage_field]
        meta_transaction: meta_transaction::Data,
        #[storage_field]
        config: config::Data,
        #[storage_field]
        lotto: raffle::Data,
        /*
        #[storage_field]
        reward: reward::Data,
         */
    }

    impl RaffleConfig for Contract {}
    impl Raffle for Contract {}
    //impl RewardManager for Contract {}

    impl RollupAnchor for Contract {}
    impl MetaTransaction for Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            // set the owner of this contract
            ownable::Internal::_init_with_owner(&mut instance, caller);
            // set the admin of this contract
            access_control::Internal::_init_with_admin(&mut instance, Some(caller));
            instance
        }

        #[ink(message)]
        pub fn participate(&mut self, numbers: Vec<Number>) -> Result<(), ContractError> {
            // check if the numbers are correct
            RaffleConfig::check_numbers(self, &numbers)?;
            // check if the user can participate (raffle is open)
            Raffle::can_participate(self)?;
            // save the participation with an event
            let participant = Self::env().caller();
            let contract_id = Self::env().account_id();
            let draw_number = Raffle::get_draw_number(self);
            self.env().emit_event(ParticipationRegistered {
                contract_id,
                draw_number,
                participant,
                numbers,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn participate_batch(
            &mut self,
            numbers: Vec<Vec<Number>>,
        ) -> Result<(), ContractError> {
            // check if the numbers are correct
            for n in numbers {
                self.participate(n)?;
            }

            Ok(())
        }

        #[ink(message)]
        #[openbrush::modifiers(access_control::only_role(LOTTO_MANAGER_ROLE))]
        pub fn set_config(&mut self, config: Config) -> Result<(), ContractError> {
            self.inner_set_config(config)
        }

        fn inner_set_config(&mut self, config: Config) -> Result<(), ContractError> {
            // check the status, we can set the config only when the raffle is not started yet
            let status = Raffle::get_status(self);
            if status != Status::NotStarted {
                return Err(ContractError::RaffleError(RaffleError::IncorrectStatus));
            }

            // update the config
            RaffleConfig::set_config(self, config)?;

            // emit the event
            let contract_id = Self::env().account_id();
            self.env().emit_event(ConfigUpdated { contract_id, config });

            Ok(())
        }

        #[ink(message)]
        #[openbrush::modifiers(access_control::only_role(LOTTO_MANAGER_ROLE))]
        pub fn open_registrations(&mut self, draw_number: DrawNumber) -> Result<(), ContractError> {
            self.inner_open_registrations(draw_number)
        }

        fn inner_open_registrations(&mut self, draw_number: DrawNumber) -> Result<(), ContractError> {
            // Open the registrations
            Raffle::open_registrations(self, draw_number)?;

            // emit the event
            let contract_id = Self::env().account_id();
            self.env().emit_event(RegistrationsOpen { contract_id, draw_number });

            Ok(())
        }

        #[ink(message)]
        #[openbrush::modifiers(access_control::only_role(LOTTO_MANAGER_ROLE))]
        pub fn close_registrations(&mut self, draw_number: DrawNumber) -> Result<(), ContractError> {
            self.inner_close_registrations(draw_number)
        }

        fn inner_close_registrations(&mut self, draw_number: DrawNumber) -> Result<(), ContractError> {
            // Close the registrations
            Raffle::close_registrations(self, draw_number)?;

            // emit the event
            let contract_id = Self::env().account_id();
            self.env().emit_event(RegistrationsClosed { contract_id, draw_number });

            Ok(())
        }

        fn inner_set_results(
            &mut self,
            draw_number: DrawNumber,
            numbers: Vec<Number>,
            winners: Vec<AccountId>,
        ) -> Result<(), ContractError> {

            // check if the numbers satisfies the config
            RaffleConfig::check_numbers(self, &numbers)?;

            // save the results
            Raffle::set_results(self, draw_number, winners.clone())?;

            // emmit the event
            let contract_id = Self::env().account_id();
            self.env().emit_event(ResultsReceived {
                contract_id,
                draw_number,
                numbers: numbers.clone(),
                winners: winners.clone(),
            });

            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn register_attestor(
            &mut self,
            account_id: AccountId,
        ) -> Result<(), AccessControlError> {
            AccessControl::grant_role(self, ATTESTOR_ROLE, Some(account_id))?;
            Ok(())
        }

        #[ink(message)]
        pub fn get_attestor_role(&self) -> RoleType {
            ATTESTOR_ROLE
        }

        #[ink(message)]
        pub fn get_manager_role(&self) -> RoleType {
            LOTTO_MANAGER_ROLE
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn terminate_me(&mut self) -> Result<(), ContractError> {
            self.env().terminate_contract(self.env().caller());
        }

        #[ink(message)]
        #[openbrush::modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn withdraw(&mut self, value: Balance) -> Result<(), ContractError> {
            let caller = Self::env().caller();
            self.env()
                .transfer(caller, value)
                .map_err(|_| ContractError::TransferError)?;
            Ok(())
        }
    }

    impl rollup_anchor::MessageHandler for Contract {
        fn on_message_received(&mut self, action: Vec<u8>) -> Result<(), RollupAnchorError> {

            // parse the response
            let request: RequestForAction = scale::Decode::decode(&mut &action[..])
                .or(Err(RollupAnchorError::FailedToDecode))?;

            match request {
                RequestForAction::SetConfig(config) => {
                    self.inner_set_config(config)?;
                }
                RequestForAction::OpenRegistrations(draw_number) => {
                    self.inner_open_registrations(draw_number)?;
                }
                RequestForAction::CloseRegistrations(draw_number) => {
                    self.inner_close_registrations(draw_number)?;
                }
                RequestForAction::SetResults(draw_number, numbers, winners) => {
                    self.inner_set_results(draw_number, numbers, winners)?
                }
            }

            Ok(())
        }
    }

    impl rollup_anchor::EventBroadcaster for Contract {
        fn emit_event_message_queued(&self, _id: u32, _data: Vec<u8>) {
            // nothing because the message queue is not used in this contract
        }
        fn emit_event_message_processed_to(&self, _id: u32) {
            // nothing because an event is already emitted in the different methods
        }
    }

    impl meta_transaction::EventBroadcaster for Contract {
        fn emit_event_meta_tx_decoded(&self) {
            // do nothing, we don't care
        }
    }
}
