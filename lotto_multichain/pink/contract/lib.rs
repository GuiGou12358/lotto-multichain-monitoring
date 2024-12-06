#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;
extern crate core;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod lotto_draw_multichain {
    use alloc::boxed::Box;
    use alloc::vec;
    use alloc::vec::Vec;
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use lotto_draw_logic::draw::Draw;
    use lotto_draw_logic::error::RaffleDrawError;
    use lotto_draw_logic::evm_contract::EvmContract;
    use lotto_draw_logic::indexer::Indexer;
    use lotto_draw_logic::raffle_manager_contract::{
        LottoManagerRequestMessage, LottoManagerResponseMessage,
    };
    use lotto_draw_logic::raffle_registration_contract::{
        RaffleRegistrationContract, RaffleRegistrationStatus, RequestForAction,
    };
    use lotto_draw_logic::types::*;
    use lotto_draw_logic::wasm_contract::WasmContract;
    use phat_offchain_rollup::clients::ink::Action;
    use pink_extension::chain_extension::signing;
    use pink_extension::{error, info, ResultExt};
    use scale::{Decode, Encode};

    #[ink(storage)]
    pub struct Lotto {
        owner: AccountId,
        /// config for raffle manager contract
        raffle_manager: Option<ContractConfig>,
        /// config for raffle registrations contracts
        raffle_registrations: Mapping<RegistrationContractId, ContractConfig>,
        /// indexer endpoint
        indexer_url: Option<String>,
        /// Key for signing the rollup tx.
        attest_key: [u8; 32],
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq)]
    #[repr(u8)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        RaffleDrawError(RaffleDrawError),
        BadOrigin,
        ClientNotConfigured,
        InvalidKeyLength,
        InvalidAddressLength,
        NoRequestInQueue,
        FailedToCallRollup,
        // error when verify the numbers
        InvalidContractId,
        CurrentRaffleUnknown,
        UnauthorizedRaffle,
        UnknownDrawNumber,
        UnknownRegistrationStatus,
        MissingRegistrationContract,
        EvmRaffleManagerNotImplemented,
    }

    type Result<T> = core::result::Result<T, ContractError>;

    impl From<phat_offchain_rollup::Error> for ContractError {
        fn from(error: phat_offchain_rollup::Error) -> Self {
            error!("error in the rollup: {:?}", error);
            ContractError::FailedToCallRollup
        }
    }

    impl From<RaffleDrawError> for ContractError {
        fn from(error: RaffleDrawError) -> Self {
            ContractError::RaffleDrawError(error)
        }
    }

    impl Lotto {
        #[ink(constructor)]
        pub fn default() -> Self {
            const NONCE: &[u8] = b"lotto";
            let private_key = signing::derive_sr25519_key(NONCE);
            Self {
                owner: Self::env().caller(),
                attest_key: private_key[..32].try_into().expect("Invalid Key Length"),
                raffle_manager: None,
                raffle_registrations: Mapping::default(),
                indexer_url: None,
            }
        }

        /// Gets the owner of the contract
        #[ink(message)]
             pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Set attestor key.
        ///
        /// For dev purpose.
        #[ink(message)]
        pub fn set_attest_key(&mut self, attest_key: Option<Vec<u8>>) -> Result<()> {
            self.attest_key = match attest_key {
                Some(key) => key.try_into().or(Err(ContractError::InvalidKeyLength))?,
                None => {
                    const NONCE: &[u8] = b"lotto";
                    let private_key = signing::derive_sr25519_key(NONCE);
                    private_key[..32]
                        .try_into()
                        .or(Err(ContractError::InvalidKeyLength))?
                }
            };
            Ok(())
        }

        /// Gets the attestor address used by this rollup (for substrate tx)
        #[ink(message)]
        pub fn get_attest_address_substrate(&self) -> Vec<u8> {
            signing::get_public_key(&self.attest_key, signing::SigType::Sr25519)
        }

        /// Gets the ecdsa address used by this rollup in the meta transaction (for evm tx)
        #[ink(message)]
        pub fn get_attest_address_evm(&self) -> Vec<u8> {
            Self::get_evm_address(&self.attest_key)
        }

        /// Gets the ecdsa address used by this rollup in the meta transaction (for substrate tx)
        #[ink(message)]
        pub fn get_attest_ecdsa_address_substrate(&self) -> Vec<u8> {
            use ink::env::hash;
            let input = signing::get_public_key(&self.attest_key, signing::SigType::Ecdsa);
            let mut output = <hash::Blake2x256 as hash::HashOutput>::Type::default();
            ink::env::hash_bytes::<hash::Blake2x256>(&input, &mut output);
            output.to_vec()
        }

        fn get_substrate_address(key: &[u8]) -> Vec<u8> {
            signing::get_public_key(key, signing::SigType::Sr25519)
        }

        fn get_evm_address(key: &[u8]) -> Vec<u8> {
            let public_key: [u8; 33] = signing::get_public_key(key, signing::SigType::Ecdsa)
                .try_into()
                .expect("Public key should be of length 33");
            let mut address = [0u8; 20];
            ink::env::ecdsa_to_eth_address(&public_key, &mut address).expect("Get address of ecdsa failed");
            address.to_vec()
        }

        fn display_config(config: &ContractConfig) -> ContractConfig {
            match config {
                ContractConfig::Wasm(c) =>
                    ContractConfig::Wasm(WasmContractConfig {
                        rpc : c.rpc.clone(),
                        pallet_id: c.pallet_id,
                        call_id: c.call_id,
                        contract_id: c.contract_id,
                        sender_key : c.sender_key.as_ref().map(|key| Self::get_substrate_address(key)
                            .try_into()
                            .expect("incorrect length")),
                    }),
                ContractConfig::Evm(c) =>
                    ContractConfig::Evm(EvmContractConfig {
                        rpc: c.rpc.clone(),
                        contract_id: c.contract_id,
                        sender_key: c.sender_key.as_ref()
                            .map(
                                |key| {
                                    let mut address: Vec<u8> = Vec::new();
                                    address.extend_from_slice([0u8; 12].as_slice());
                                    address.extend_from_slice(Self::get_evm_address(key).as_slice());
                                    address.try_into().expect("incorrect length")
                                }
                            ),
                    }),
            }
        }

        /// Gets the config of the target consumer contract
        #[ink(message)]
        pub fn get_config_raffle_manager(&self) -> Option<ContractConfig> {
            self.raffle_manager.as_ref().map(Self::display_config)
        }

        /// Gets the config of the target consumer contract
        #[ink(message)]
        pub fn get_config_raffle_registrations(
            &self,
            contract_id: RegistrationContractId,
        ) -> Option<ContractConfig> {
            self.raffle_registrations.get(contract_id).as_ref().map(Self::display_config)
        }

        /// Configures the target consumer contract (admin only)
        #[ink(message)]
        pub fn set_config_raffle_manager(&mut self, config: Option<ContractConfig>) -> Result<()> {
            self.ensure_owner()?;
            self.raffle_manager = config;
            Ok(())
        }

        #[ink(message)]
        pub fn set_config_raffle_registrations(
            &mut self,
            contract_id: RegistrationContractId,
            config: Option<ContractConfig>,
        ) -> Result<()> {
            self.ensure_owner()?;
            match config {
                None => {
                    self.raffle_registrations.remove(contract_id);
                }
                Some(c) => {
                    self.raffle_registrations.insert(contract_id, &c);
                }
            }
            Ok(())
        }


        /// Gets the config to target the indexer
        #[ink(message)]
        pub fn get_indexer_url(&self) -> Option<String> {
            self.indexer_url.clone()
        }

        /// Configures the indexer (admin only)
        #[ink(message)]
        pub fn config_indexer(&mut self, indexer_url: String) -> Result<()> {
            self.ensure_owner()?;
            self.indexer_url = Some(indexer_url);
            Ok(())
        }

        /// Transfers the ownership of the contract (admin only)
        #[ink(message)]
        pub fn transfer_ownership(&mut self, new_owner: AccountId) -> Result<()> {
            self.ensure_owner()?;
            self.owner = new_owner;
            Ok(())
        }

        /// Processes a request by a rollup transaction
        #[ink(message)]
        pub fn answer_request(&self) -> Result<Vec<(RegistrationContractId, Option<Vec<u8>>)>> {
            let config = self.ensure_client_configured()?;

            let (mut client, manager_contract_id, sender_key) = match config {
                ContractConfig::Wasm(config) => (WasmContract::connect(config)?, config.contract_id , config.sender_key),
                ContractConfig::Evm(_config) => {
                    return Err(ContractError::EvmRaffleManagerNotImplemented)
                }
            };

            // Get a request if presents
            let request: LottoManagerRequestMessage = client
                .pop()
                .log_err("answer_request: failed to read queue")?
                .ok_or(ContractError::NoRequestInQueue)?;

            ink::env::debug_println!("Received request: {request:02x?}");

            // read the status and the draw number to include them in the where clause (optimistic locking)
            /*
            let status = lotto_draw_logic::wasm_contract::get_manager_status(&mut client)?;
            let draw_number = lotto_draw_logic::wasm_contract::get_manager_draw_number(&mut client)?;
            ink::env::debug_println!("manager status : {status:?}");
            ink::env::debug_println!("manager draw_number : {draw_number:?}");
             */

            let (r, mut txs) = self.handle_request(request, manager_contract_id)?;
            if let Some(response) = r {
                let encoded_response = response.encode();
                ink::env::debug_println!("Manager encoded response: {encoded_response:02x?}");
                // Attach an action to the tx by:
                client.action(Action::Reply(response.encode()));
                let tx = WasmContract::maybe_submit_tx(client, &self.attest_key, sender_key.as_ref())?;
                ink::env::debug_println!("tx: {tx:02x?}");
                txs.push((0, tx));
            } else {
                txs.push((0, None));
            }
            Ok(txs)
        }

        fn hash_input<T: scale::Encode>(
            input: &T
        ) -> lotto_draw_logic::types::Hash {
            use ink::env::hash;
            // encode and hash the input for verification by the manager
            let encoded_input = input.encode();
            let mut hash_encoded_input = <hash::Blake2x256 as hash::HashOutput>::Type::default();
            ink::env::hash_bytes::<hash::Blake2x256>(&encoded_input, &mut hash_encoded_input);
            hash_encoded_input.into()
        }

        fn handle_request(
            &self,
            message: LottoManagerRequestMessage,
            manager_contract_id: WasmContractId
        ) -> Result<(Option<LottoManagerResponseMessage>, Vec<(RegistrationContractId, Option<Vec<u8>>)>)> {
            let response = match message {
                LottoManagerRequestMessage::PropagateConfig(config, ref contract_ids) => {
                    let (synchronized_contracts, txs) = self.inner_do_action(
                        RequestForAction::SetConfigAndStart(config.clone(), 0),
                        contract_ids,
                    )?;
                    let response = if synchronized_contracts.is_empty(){
                        None
                    } else {
                        // encode and hash the input for verification by the manager
                        let hash = Self::hash_input(&config);
                        Some(LottoManagerResponseMessage::ConfigPropagated(synchronized_contracts, hash))
                    };
                    (response, txs)
                }
                LottoManagerRequestMessage::OpenRegistrations(draw_number, ref contract_ids) => {
                    let (synchronized_contracts, txs) = self.inner_do_action(
                        RequestForAction::OpenRegistrations(draw_number),
                        contract_ids,
                    )?;
                    let response = if synchronized_contracts.is_empty(){
                        None
                    } else {
                        Some(LottoManagerResponseMessage::RegistrationsOpen(
                            draw_number,
                            synchronized_contracts,
                        ))
                    };
                    (response, txs)
                }
                LottoManagerRequestMessage::CloseRegistrations(draw_number, ref contract_ids) => {
                    let  (synchronized_contracts, txs) = self.inner_do_action(
                        RequestForAction::CloseRegistrations(draw_number),
                        contract_ids,
                    )?;
                    let response = if synchronized_contracts.is_empty(){
                        None
                    } else {
                        Some(LottoManagerResponseMessage::RegistrationsClosed(
                            draw_number,
                            synchronized_contracts,
                        ))
                    };
                    (response, txs)
                }
                LottoManagerRequestMessage::DrawNumbers(draw_number, ref config) => {
                    let numbers = self.inner_get_numbers(
                        manager_contract_id,
                        draw_number,
                        config.nb_numbers,
                        config.min_number,
                        config.max_number,
                    )?;
                    // encode and hash the input for verification by the manager
                    let hash = Self::hash_input(&config);
                    (Some(LottoManagerResponseMessage::WinningNumbers(draw_number, numbers, hash)), Vec::new())
                }
                LottoManagerRequestMessage::CheckWinners(draw_number, ref numbers) => {
                    let indexer = Indexer::new(self.get_indexer_url())?;
                    let winners = indexer.query_winners(draw_number, numbers)?;
                    /*.map(
                        |(substrate_addresses, evm_addresses)| {
                            LottoManagerResponseMessage::Winners(substrate_addresses, evm_addresses)
                        },
                    )?
                    */
                    // encode and hash the input for verification by the manager
                    let hash = Self::hash_input(numbers);
                    (Some(LottoManagerResponseMessage::Winners(draw_number, winners.0, hash)), Vec::new())
                }
                LottoManagerRequestMessage::PropagateResults(
                    draw_number,
                    ref numbers,
                    ref winners,
                    ref contract_ids,
                ) => {
                    let  (synchronized_contracts, txs) = self.inner_do_action(
                        RequestForAction::SetResults(
                            draw_number,
                            numbers.to_vec(),
                            winners.to_vec(),
                        ),
                        contract_ids,
                    )?;
                    let response = if synchronized_contracts.is_empty(){
                        None
                    } else {
                        // encode and hash the input for verification by the manager
                        let hash = Self::hash_input(numbers);
                        Some(LottoManagerResponseMessage::ResultsPropagated(
                            draw_number,
                            synchronized_contracts,
                            hash,
                        ))
                    };
                    (response, txs)
                }
            };

            Ok(response)
        }

        fn inner_do_action(
            &self,
            request: RequestForAction,
            contract_ids: &[RegistrationContractId],
        ) -> Result<(Vec<RegistrationContractId>, Vec<(RegistrationContractId, Option<Vec<u8>>)>)> {
            let mut synchronized_contracts = Vec::new();
            let mut txs = Vec::new();

            // get the status and draw number matching with this action
            let (target_draw_number, target_status) = match request {
                RequestForAction::SetConfigAndStart(_, _) => {
                    (None, Some(RaffleRegistrationStatus::Started))
                }
                RequestForAction::OpenRegistrations(draw_number) => (
                    Some(draw_number),
                    Some(RaffleRegistrationStatus::RegistrationOpen),
                ),
                RequestForAction::CloseRegistrations(draw_number) => (
                    Some(draw_number),
                    Some(RaffleRegistrationStatus::RegistrationClosed),
                ),
                RequestForAction::SetResults(draw_number, _, _) => (
                    Some(draw_number),
                    Some(RaffleRegistrationStatus::ResultsReceived),
                ),
            };

            // iterate on contract_ids
            for contract_id in contract_ids {
                // get the config linked to this contract
                let contract_config = self
                    .raffle_registrations
                    .get(contract_id)
                    .ok_or(ContractError::MissingRegistrationContract)?;
                // build the object to reach this contract
                let contract: Box<dyn RaffleRegistrationContract> = match contract_config {
                    ContractConfig::Wasm(config) => {
                        WasmContract::new(Some(config)).map(Box::new)?
                    }
                    ContractConfig::Evm(config) => EvmContract::new(Some(config)).map(Box::new)?,
                };
                // for the action SetConfigAndStart, we have to override the registration contract id
                let request = match &request {
                    RequestForAction::SetConfigAndStart(config, _) => {
                        &RequestForAction::SetConfigAndStart(config.clone(), *contract_id)
                    }
                    _ => &request,
                };

                // check the status and draw number and do the action is the contract is not synchronized
                let (sync, tx) = contract.do_action(
                    target_draw_number,
                    target_status,
                    request.clone(),
                    &self.attest_key,
                )?;
                if sync {
                    // the contract is synchronized
                    synchronized_contracts.push(*contract_id);
                }
                txs.push((*contract_id, tx));

            }
            // return the list of synchronized contracts
            Ok((synchronized_contracts, txs))
        }


        /// Send a request to Manager to close the registrations
        #[ink(message)]
        pub fn close_registrations(&self) -> Result<Option<Vec<u8>>> {
            let config = self.ensure_client_configured()?;
            let (mut client, sender_key) = match config {
                ContractConfig::Wasm(config) => (WasmContract::connect(config)?, config.sender_key),
                ContractConfig::Evm(_config) => {
                    return Err(ContractError::EvmRaffleManagerNotImplemented)
                }
            };
            // send the request to the manager
            client.action(Action::Reply(LottoManagerResponseMessage::CloseRegistrations().encode()));

            let tx = WasmContract::maybe_submit_tx(client, &self.attest_key, sender_key.as_ref())?;
            ink::env::debug_println!("tx: {tx:02x?}");
            Ok(tx)
        }

        /// Verify if the winning numbers for a raffle are valid (only for past raffles)
        ///
        #[ink(message)]
        pub fn verify_numbers(
            &self,
            draw_number: DrawNumber,
            hashes: Vec<lotto_draw_logic::types::Hash>,
            nb_numbers: u8,
            smallest_number: Number,
            biggest_number: Number,
            numbers: Vec<Number>,
        ) -> Result<bool> {

           let config = self.ensure_client_configured()?;

            let (mut client, contract_id) = match config {
                ContractConfig::Wasm(config) => (WasmContract::connect(config)?, config.contract_id),
                ContractConfig::Evm(_config) => {
                    return Err(ContractError::EvmRaffleManagerNotImplemented)
                }
            };

           const LAST_RAFFLE_FOR_VERIF: u32 = ink::selector_id!("LAST_RAFFLE_FOR_VERIF");

           let last_raffle: DrawNumber = client
               .get(&LAST_RAFFLE_FOR_VERIF)
               .log_err("verify numbers: last raffle unknown")?
               .ok_or(ContractError::CurrentRaffleUnknown)?;

           // verify the winning numbers only for the past raffles
           if draw_number > last_raffle {
               return Err(ContractError::UnauthorizedRaffle);
           }

            let draw = Draw::new(nb_numbers, smallest_number, biggest_number)?;
            let result = draw.verify_numbers(contract_id, draw_number, hashes, numbers)?;
            Ok(result)
        }

        fn inner_get_numbers(
            &self,
            contract_id: WasmContractId,
            draw_number: DrawNumber,
            nb_numbers: u8,
            smallest_number: Number,
            biggest_number: Number,
        ) -> Result<Vec<Number>> {
            info!(
                "Draw number {draw_number} - Request received for draw {nb_numbers} numbers between {smallest_number} and {biggest_number}"
            );

            let indexer = Indexer::new(self.get_indexer_url())?;
            //let hashes = indexer.query_hashes(draw_number)?;
            let hashes = vec![]; // TODO implement get hashes

            let draw = Draw::new(nb_numbers, smallest_number, biggest_number)?;
            let result = draw.get_numbers(contract_id, draw_number, hashes)?;
            Ok(result)
        }

        /// Returns BadOrigin error if the caller is not the owner
        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() == self.owner {
                Ok(())
            } else {
                Err(ContractError::BadOrigin)
            }
        }

        /// Returns the config reference or raise the error `ClientNotConfigured`
        fn ensure_client_configured(&self) -> Result<&ContractConfig> {
            self.raffle_manager
                .as_ref()
                .ok_or(ContractError::ClientNotConfigured)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        struct EnvVars {
            /// The RPC endpoint of the target blockchain
            rpc: String,
            pallet_id: u8,
            call_id: u8,
            /// The rollup anchor address on the target blockchain
            manager_contract_id: WasmContractId,
            /// The rollup anchor address on the target blockchain
            lotto_contract_id: WasmContractId,
            /// When we want to manually set the attestor key for signing the message (only dev purpose)
            attest_key: Option<Vec<u8>>,
            /// When we want to use meta tx
            sender_key: Option<Vec<u8>>,
        }

        fn get_env(key: &str) -> Option<String> {
            match std::env::var(key) {
                Ok(k) => Some(k),
                _ => {
                    ink::env::debug_println!("Key {key} not found");
                    None
                }
            }
        }

        fn config() -> EnvVars {
            dotenvy::dotenv().ok();
            let rpc = get_env("RPC").unwrap();
            let pallet_id: u8 = get_env("PALLET_ID").unwrap().parse().expect("u8 expected");
            let call_id: u8 = get_env("CALL_ID").unwrap().parse().expect("u8 expected");
            let manager_contract_id: WasmContractId =
                hex::decode(get_env("MANAGER_CONTRACT_ID").unwrap())
                    .expect("hex decode failed")
                    .try_into()
                    .expect("incorrect length");
            let lotto_contract_id: WasmContractId =
                hex::decode(get_env("LOTTO_CONTRACT_ID").unwrap())
                    .expect("hex decode failed")
                    .try_into()
                    .expect("incorrect length");
            let attest_key =
                get_env("ATTEST_KEY").map(|s| hex::decode(s).expect("hex decode failed"));
            let sender_key =
                get_env("SENDER_KEY").map(|s| hex::decode(s).expect("hex decode failed"));

            EnvVars {
                rpc: rpc.to_string(),
                pallet_id,
                call_id,
                manager_contract_id: manager_contract_id.into(),
                lotto_contract_id: lotto_contract_id.into(),
                attest_key,
                sender_key,
            }
        }

        fn init_contract() -> Lotto {
            let EnvVars {
                rpc,
                pallet_id,
                call_id,
                manager_contract_id,
                lotto_contract_id,
                attest_key,
                sender_key,
            } = config();

            let mut lotto = Lotto::default();
            let sender_key = match sender_key {
                Some(k) => Some(k.try_into().expect("fatal sender key")),
                None => None,
            };

            let manager_config = WasmContractConfig {
                rpc: rpc.clone(),
                pallet_id,
                call_id,
                contract_id: manager_contract_id.into(),
                sender_key: sender_key.clone(),
            };

            let registration_contract_config_10 = WasmContractConfig {
                rpc,
                pallet_id,
                call_id,
                contract_id: lotto_contract_id.into(),
                sender_key,
            };

            lotto
                .set_config_raffle_manager(Some(ContractConfig::Wasm(manager_config)))
                .unwrap();

            lotto
                .set_config_raffle_registrations(
                    100,
                    Some(ContractConfig::Wasm(registration_contract_config_10)),
                )
                .unwrap();

            if let Some(attest_key) = attest_key {
                lotto.set_attest_key(Some(attest_key)).unwrap();
            }

            lotto
                .config_indexer("https://query.substrate.fi/lotto-subquery-shibuya".to_string())
                .unwrap();

            lotto
        }

        #[ink::test]
        fn test_display_config() {
            pink_extension_runtime::mock_ext::mock_all_ext();

            let mut lotto = Lotto::default();

            let manager_config = WasmContractConfig {
                rpc: "https://rpc.shibuya.astar.network".to_string(),
                pallet_id: 70,
                call_id: 6,
                contract_id: hex::decode("40dbd8dedbabc995f5758d62cbd48c53246f895bcffd1ca2325c2a2eea624610")
                    .expect("hex decode failed")
                    .try_into()
                    .expect("Incorrect Length"),
                sender_key: Some(hex::decode("ea21cc675ba1c0108cda39829a1f3c00d8ec36ea03b164d2ec206a2ba8848e3d")
                    .expect("hex decode failed")
                    .try_into()
                    .expect("Incorrect Length")),
            };

            lotto
                .set_config_raffle_manager(Some(ContractConfig::Wasm(manager_config.clone())))
                .unwrap();

            let registration_contract_config_10 = WasmContractConfig {
                rpc: "https://rpc.shibuya.astar.network".to_string(),
                pallet_id: 70,
                call_id: 6,
                contract_id: hex::decode("fad3389c75170b01767b014e357e508cbecc0cf3966b7dbfcf434369db62f134")
                    .expect("hex decode failed")
                    .try_into()
                    .expect("Incorrect Length"),
                sender_key: Some(hex::decode("ea21cc675ba1c0108cda39829a1f3c00d8ec36ea03b164d2ec206a2ba8848e3d")
                    .expect("hex decode failed")
                    .try_into()
                    .expect("Incorrect Length")),
            };

            lotto
                .set_config_raffle_registrations(
                    10,
                    Some(ContractConfig::Wasm(registration_contract_config_10.clone())),
                )
                .unwrap();


            let registration_contract_config_11 = EvmContractConfig {
                rpc: "https://rpc.shibuya.astar.network".to_string(),
                contract_id: hex::decode("53121246949Cad4E771F42EacF950a35B3f727ca")
                    .expect("hex decode failed")
                    .try_into()
                    .expect("Incorrect Length"),
                sender_key: Some(hex::decode("ea21cc675ba1c0108cda39829a1f3c00d8ec36ea03b164d2ec206a2ba8848e3d")
                    .expect("hex decode failed")
                    .try_into()
                    .expect("Incorrect Length")),
            };

            lotto
                .set_config_raffle_registrations(
                    11,
                    Some(ContractConfig::Evm(registration_contract_config_11.clone())),
                )
                .unwrap();

            if let Some(ContractConfig::Wasm(config)) = lotto.get_config_raffle_manager() {
                assert_eq!(config.rpc, manager_config.rpc);
                assert_eq!(config.pallet_id, manager_config.pallet_id);
                assert_eq!(config.call_id, manager_config.call_id);
                assert_eq!(config.contract_id, manager_config.contract_id);
                assert_ne!(config.sender_key, manager_config.sender_key);
            } else {
                assert!(false);
            }

            if let Some(ContractConfig::Wasm(config)) = lotto.get_config_raffle_registrations(10) {
                assert_eq!(config.rpc, registration_contract_config_10.rpc);
                assert_eq!(config.pallet_id, registration_contract_config_10.pallet_id);
                assert_eq!(config.call_id, registration_contract_config_10.call_id);
                assert_eq!(config.contract_id, registration_contract_config_10.contract_id);
                assert_ne!(config.sender_key, registration_contract_config_10.sender_key);
            } else {
                assert!(false);
            }

            if let Some(ContractConfig::Evm(config)) = lotto.get_config_raffle_registrations(11) {
                assert_eq!(config.rpc, registration_contract_config_11.rpc);
                assert_eq!(config.contract_id, registration_contract_config_11.contract_id);
                assert_ne!(config.sender_key, registration_contract_config_11.sender_key);
            } else {
                assert!(false);
            }


        }

        #[ink::test]
        #[ignore = "The target contract must be deployed on the Substrate node and a random number request must be submitted"]
        fn answer_request() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let lotto = init_contract();

            let attestor_address = lotto.get_attest_address_substrate();
            ink::env::debug_println!("attestor address: {attestor_address:02x?}");
            let attestor_ecdsa_address = lotto.get_attest_ecdsa_address_substrate();
            ink::env::debug_println!("attestor ecdsa address: {attestor_ecdsa_address:02x?}");

            let r = lotto.answer_request().expect("failed to answer request");
            ink::env::debug_println!("answer request: {r:?}");
        }

    }
}
