#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[openbrush::implementation(Ownable, AccessControl, Upgradeable)]
#[openbrush::contract]
pub mod lotto_registration_manager_contract {
    use ink::codegen::{EmitEvent, Env};
    use ink::prelude::vec::Vec;
    use lotto::{
        config, config::*, error::*, raffle_manager, raffle_manager::*, DrawNumber, Number,
        RegistrationContractId,
    };
    use openbrush::contracts::access_control::*;
    use openbrush::contracts::ownable::*;
    use openbrush::{modifiers, traits::Storage};
    use phat_rollup_anchor_ink::traits::{
        meta_transaction, meta_transaction::*, rollup_anchor, rollup_anchor::*,
    };

    const LOTTO_MANAGER_ROLE: RoleType = ink::selector_id!("LOTTO_MANAGER");

    /// Event emitted when the lotto is started
    #[ink(event)]
    pub struct LottoStarted {
        config: Config,
    }

    /// Event emitted when the registrations are open
    #[ink(event)]
    pub struct RegistrationOpen {
        #[ink(topic)]
        draw_number: DrawNumber,
    }

    /// Event emitted when the registrations are closed
    #[ink(event)]
    pub struct RegistrationClosed {
        #[ink(topic)]
        draw_number: DrawNumber,
    }

    /// Event emitted when the raffle result is received
    #[ink(event)]
    pub struct ResultReceived {
        #[ink(topic)]
        draw_number: DrawNumber,
        numbers: Vec<Number>,
    }

    /// Event emitted when the winners are revealed
    #[ink(event)]
    pub struct WinnersRevealed {
        #[ink(topic)]
        draw_number: DrawNumber,
        winners: Vec<AccountId>,
    }

    /// Event emitted when the lotto is closed
    #[ink(event)]
    pub struct LottoClosed {}

    /// Errors occurred in the contract
    #[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        AccessControlError(AccessControlError),
        RaffleError(RaffleError),
        RollupAnchorError(RollupAnchorError),
        CannotBeClosedYet,
        NoResult,
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

    /// convertor from RollupAnchorError to ContractError
    impl From<RollupAnchorError> for ContractError {
        fn from(error: RollupAnchorError) -> Self {
            ContractError::RollupAnchorError(error)
        }
    }

    /// convertor from ContractError to RollupAnchorError
    impl From<ContractError> for RollupAnchorError {
        fn from(error: ContractError) -> Self {
            ink::env::debug_println!("Error: {:?}", error);
            RollupAnchorError::UnsupportedAction
        }
    }

    /// Message to synchronize the contracts, to request the lotto draw and get the list of winners.
    /// message pushed in the queue by this contract and read by the offchain rollup
    #[derive(scale::Encode, scale::Decode, Eq, PartialEq, Clone, Debug)]
    pub enum LottoManagerRequestMessage {
        /// request to propagate the config to all given contracts
        PropagateConfig(Config, Vec<RegistrationContractId>),
        /// request to open the registrations to all given contracts
        OpenRegistrations(DrawNumber, Vec<RegistrationContractId>),
        /// request to close the registrations to all given contracts
        CloseRegistrations(DrawNumber, Vec<RegistrationContractId>),
        /// request to draw the numbers based on the config
        DrawNumbers(DrawNumber, Config),
        /// request to check if there is a winner for the given numbers
        CheckWinners(DrawNumber, Vec<Number>),
        /// request to propagate the results to all given contracts
        PropagateResults(
            DrawNumber,
            Vec<Number>,
            Vec<AccountId>,
            Vec<RegistrationContractId>,
        ),
    }

    /// Offchain rollup response
    #[derive(scale::Encode, scale::Decode)]
    pub enum LottoManagerResponseMessage {
        /// The config is propagated to the given contract ids.
        /// arg2: list of contracts where the config is propagated
        /// Arg2 : Hash of config
        ConfigPropagated(Vec<RegistrationContractId>, Hash),
        /// The registration is open for the given contract ids.
        /// arg1: draw number
        /// arg2: list of contracts where the registration is open
        RegistrationsOpen(DrawNumber, Vec<RegistrationContractId>),
        /// The registration is closed for the given contract ids.
        /// arg1: draw number
        /// arg2: list of contracts where the registration is closed
        RegistrationsClosed(DrawNumber, Vec<RegistrationContractId>),
        /// Return the winning numbers
        /// arg1: draw number
        /// arg2: winning numbers
        /// arg3: hash of salt used for vrf
        WinningNumbers(DrawNumber, Vec<Number>, Hash),
        /// Return the list of winners
        /// arg1: draw number
        /// arg2: winners
        /// arg3: hash of winning numbers
        Winners(DrawNumber, Vec<AccountId>, Hash),
        /// The results are propagated to the given contract ids.
        /// arg1: draw number
        /// arg2: list of contracts where the results are propagated
        /// arg3: hash of results
        ResultsPropagated(DrawNumber, Vec<RegistrationContractId>, Hash),
    }

    // Contract storage
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
        raffle_manager: raffle_manager::Data,
        block_number_close_registrations: BlockNumber,
    }

    impl RaffleConfig for Contract {}
    impl RaffleManager for Contract {}

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
            // grant the role manager
            AccessControl::grant_role(&mut instance, LOTTO_MANAGER_ROLE, Some(caller))
                .expect("Should grant the role LOTTO_MANAGER_ROLE");
            instance
        }

        #[ink(message)]
        #[openbrush::modifiers(access_control::only_role(LOTTO_MANAGER_ROLE))]
        pub fn set_config(&mut self, config: Config) -> Result<(), ContractError> {
            // check the status, we can set the config only when the raffle is not started yet
            let status = RaffleManager::get_status(self);
            if status != Status::NotStarted {
                return Err(ContractError::RaffleError(RaffleError::IncorrectStatus));
            }

            // update the config
            RaffleConfig::set_config(self, config)?;

            Ok(())
        }

        #[ink(message)]
        #[openbrush::modifiers(access_control::only_role(LOTTO_MANAGER_ROLE))]
        pub fn add_registration_contract(
            &mut self,
            registration_contract: RegistrationContractId,
        ) -> Result<(), ContractError> {
            // add registration contract
            RaffleManager::add_registration_contract(self, registration_contract)?;
            // TODO develop delete/update

            Ok(())
        }

        #[ink(message)]
        #[openbrush::modifiers(access_control::only_role(LOTTO_MANAGER_ROLE))]
        pub fn start(&mut self, previous_draw_number: DrawNumber) -> Result<(), ContractError> {
            // start
            RaffleManager::start(self, previous_draw_number)?;
            // propagate the config in all given contracts
            let config = RaffleConfig::ensure_config(self)?;

            // emmit the event
            self.env().emit_event(LottoStarted { config });

            let registration_contracts = RaffleManager::get_registration_contracts(self);
            let message =
                LottoManagerRequestMessage::PropagateConfig(config, registration_contracts);
            RollupAnchor::push_message(self, &message)?;

            Ok(())
        }

        #[ink(message)]
        pub fn can_close_registrations(&self) -> bool {
            // check the status of all contracts
            if !RaffleManager::can_close_registrations(self) {
                return false;
            }

            // check the block number
            let block_number = self.env().block_number();
            block_number >= self.block_number_close_registrations
        }

        #[ink(message)]
        pub fn close_registrations(&mut self) -> Result<(), ContractError> {
            // check if we can close the registrations
            if !self.can_close_registrations() {
                return Err(ContractError::CannotBeClosedYet);
            }
            // close the registrations in the manager
            let draw_number = RaffleManager::close_registrations(self)?;

            // emmit the event
            self.env().emit_event(RegistrationClosed { draw_number });

            // close the registrations in all contracts
            let registration_contracts = RaffleManager::get_registration_contracts(self);
            let message =
                LottoManagerRequestMessage::CloseRegistrations(draw_number, registration_contracts);
            RollupAnchor::push_message(self, &message)?;

            Ok(())
        }

        #[ink(message)]
        pub fn has_pending_message(&self) -> bool {
            let tail = RollupAnchor::get_queue_tail(self).unwrap_or_default();
            let head = RollupAnchor::get_queue_head(self).unwrap_or_default();
            tail > head
        }

        fn handle_started(
            &mut self,
            registration_contracts: Vec<RegistrationContractId>,
        ) -> Result<(), ContractError> {
            let not_synchronized_contracts = RaffleManager::save_registration_contracts_status(
                self,
                RaffleManager::get_draw_number(self),
                Status::Started,
                registration_contracts,
            )?;

            if !not_synchronized_contracts.is_empty() {
                // synchronized missing contracts and wait
                let config = RaffleConfig::ensure_config(self)?;
                let message =
                    LottoManagerRequestMessage::PropagateConfig(config, not_synchronized_contracts);
                RollupAnchor::push_message(self, &message)?;
                return Ok(());
            }

            // open the registration
            self.inner_open_registrations()?;

            Ok(())
        }

        fn inner_open_registrations(&mut self) -> Result<(), ContractError> {
            // open the registrations in the manager
            let draw_number = RaffleManager::open_registrations(self)?;

            // emmit the event
            self.env().emit_event(RegistrationOpen { draw_number });

            // open the registrations in all given contracts
            let registration_contracts = RaffleManager::get_registration_contracts(self);
            let message =
                LottoManagerRequestMessage::OpenRegistrations(draw_number, registration_contracts);
            RollupAnchor::push_message(self, &message)?;

            Ok(())
        }

        fn handle_registrations_open(
            &mut self,
            draw_number: DrawNumber,
            registration_contracts: Vec<RegistrationContractId>,
        ) -> Result<(), ContractError> {
            let not_synchronized_contracts = RaffleManager::save_registration_contracts_status(
                self,
                draw_number,
                Status::RegistrationsOpen,
                registration_contracts,
            )?;

            if !not_synchronized_contracts.is_empty() {
                // synchronized missing contracts and wait
                let message = LottoManagerRequestMessage::OpenRegistrations(
                    draw_number,
                    not_synchronized_contracts,
                );
                RollupAnchor::push_message(self, &message)?;
                return Ok(());
            }

            // all contracts are synchronized
            // we can close the registration in X block
            let block_number = self.env().block_number();
            self.block_number_close_registrations = block_number
                .checked_add(0)
                .ok_or(RaffleError::AddOverFlow)?;

            Ok(())
        }

        fn handle_registrations_closed(
            &mut self,
            draw_number: DrawNumber,
            registration_contracts: Vec<RegistrationContractId>,
        ) -> Result<(), ContractError> {
            let not_synchronized_contracts = RaffleManager::save_registration_contracts_status(
                self,
                draw_number,
                Status::RegistrationsClosed,
                registration_contracts,
            )?;

            if !not_synchronized_contracts.is_empty() {
                // synchronized missing contracts and wait
                let message = LottoManagerRequestMessage::CloseRegistrations(
                    draw_number,
                    not_synchronized_contracts,
                );
                RollupAnchor::push_message(self, &message)?;
                return Ok(());
            }

            // if all contracts are synchronized, we can request the draw numbers
            let config = RaffleConfig::ensure_config(self)?;
            // TODO get the hash when the registration is closed
            let message = LottoManagerRequestMessage::DrawNumbers(draw_number, config);
            RollupAnchor::push_message(self, &message)?;

            Ok(())
        }

        fn handle_winning_numbers(
            &mut self,
            draw_number: DrawNumber,
            //config: Config,
            numbers: Vec<Number>,
        ) -> Result<(), ContractError> {
            // TODO check if the config used to select the number is correct
            //RaffleConfig::ensure_same_config(self, &config)?;

            // check if the numbers are correct
            RaffleConfig::check_numbers(self, &numbers)?;

            // set the result
            RaffleManager::set_results(self, draw_number, numbers.clone())?;

            // emmit the event
            self.env().emit_event(ResultReceived {
                draw_number,
                numbers: numbers.clone(),
            });

            // request to check the winners
            let message = LottoManagerRequestMessage::CheckWinners(draw_number, numbers);
            RollupAnchor::push_message(self, &message)?;

            Ok(())
        }

        fn handle_winners(
            &mut self,
            draw_number: DrawNumber,
            //numbers: Vec<Number>,
            winners: Vec<AccountId>,
        ) -> Result<(), ContractError> {
            // TODO check if the winners were selected based on the correct numbers
            //RaffleManager::ensure_same_results(self, raffle_id, &numbers)?;

            // set the winners in the raffle
            RaffleManager::set_winners(self, draw_number, winners.clone())?;

            // emmit the event
            self.env().emit_event(WinnersRevealed {
                draw_number,
                winners: winners.clone(),
            });

            // propagate the results in all contracts
            let numbers =
                RaffleManager::get_results(self, draw_number).ok_or(ContractError::NoResult)?;
            let registration_contracts = RaffleManager::get_registration_contracts(self);
            let message = LottoManagerRequestMessage::PropagateResults(
                draw_number,
                numbers,
                winners.clone(),
                registration_contracts,
            );
            RollupAnchor::push_message(self, &message)?;

            Ok(())
        }

        fn handle_results_propagated(
            &mut self,
            draw_number: DrawNumber,
            registration_contracts: Vec<RegistrationContractId>,
        ) -> Result<(), ContractError> {
            let not_synchronized_contracts = RaffleManager::save_registration_contracts_status(
                self,
                draw_number,
                Status::Closed,
                registration_contracts,
            )?;

            let winners = RaffleManager::get_winners(self, draw_number).unwrap_or_default();

            if !not_synchronized_contracts.is_empty() {
                // synchronized missing contracts and wait
                let numbers =
                    RaffleManager::get_results(self, draw_number).ok_or(ContractError::NoResult)?;
                let message = LottoManagerRequestMessage::PropagateResults(
                    draw_number,
                    numbers,
                    winners.clone(),
                    not_synchronized_contracts,
                );
                RollupAnchor::push_message(self, &message)?;
                return Ok(());
            }

            // if all contracts are synchronized, we can request the draw numbers
            if winners.is_empty() {
                // if there is no winner, we can open the registrations for the next draw number
                self.inner_open_registrations()?;
            }

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
            let response: LottoManagerResponseMessage = scale::Decode::decode(&mut &action[..])
                .or(Err(RollupAnchorError::FailedToDecode))?;

            match response {
                LottoManagerResponseMessage::ConfigPropagated(contract_ids, _hash) => {
                    self.handle_started(contract_ids)?
                }
                LottoManagerResponseMessage::RegistrationsOpen(draw_number, contract_ids) => {
                    self.handle_registrations_open(draw_number, contract_ids)?
                }
                LottoManagerResponseMessage::RegistrationsClosed(draw_number, contract_ids) => {
                    self.handle_registrations_closed(draw_number, contract_ids)?
                }
                LottoManagerResponseMessage::ResultsPropagated(
                    draw_number,
                    contract_ids,
                    _hash,
                ) => self.handle_results_propagated(draw_number, contract_ids)?,
                LottoManagerResponseMessage::WinningNumbers(draw_number, numbers, _hash) => {
                    self.handle_winning_numbers(draw_number, numbers)?
                }
                LottoManagerResponseMessage::Winners(draw_number, winners, _hash) => {
                    self.handle_winners(draw_number, winners)?
                }
            }

            Ok(())
        }
    }

    /// Event emitted when a message is pushed in the queue
    #[ink(event)]
    pub struct MessageQueued {
        #[ink(topic)]
        id: u32,
        data: Vec<u8>,
    }

    /// Event emitted when a message is processed
    #[ink(event)]
    pub struct MessageProcessedTo {
        #[ink(topic)]
        id: u32,
    }

    impl rollup_anchor::EventBroadcaster for Contract {
        fn emit_event_message_queued(&self, id: u32, data: Vec<u8>) {
            self.env().emit_event(MessageQueued { id, data });
        }
        fn emit_event_message_processed_to(&self, id: u32) {
            self.env().emit_event(MessageProcessedTo { id });
        }
    }

    impl meta_transaction::EventBroadcaster for Contract {
        fn emit_event_meta_tx_decoded(&self) {
            // do nothing
        }
    }
}
