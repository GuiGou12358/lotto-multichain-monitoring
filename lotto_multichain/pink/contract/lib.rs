#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;
extern crate core;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod lotto_draw_multichain {
    use alloc::vec::Vec;
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use lotto_draw_logic::draw::Draw;
    use lotto_draw_logic::error::RaffleDrawError;
    use lotto_draw_logic::error::RaffleDrawError::AddOverFlow;
    use lotto_draw_logic::evm_contract::EvmContract;
    use lotto_draw_logic::indexer::Indexer;
    use lotto_draw_logic::types::*;
    use lotto_draw_logic::wasm_contract::WasmContract;
    use phat_offchain_rollup::clients::ink::Action;
    use pink_extension::chain_extension::signing;
    use pink_extension::{error, info, ResultExt};
    use scale::{Decode, Encode};
    use lotto_draw_logic::raffle_manager_contract::{LottoManagerRequestMessage, LottoManagerResponseMessage};

    #[ink(storage)]
    pub struct Lotto {
        owner: AccountId,
        /// config to send the data to the wasm or evm smart contract
        primary_consumer: Option<WasmContractConfig>,
        secondary_consumers: Mapping<u8, EvmContractConfig>,
        secondary_consumers_keys: Vec<u8>,
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
                primary_consumer: None,
                secondary_consumers: Mapping::default(),
                secondary_consumers_keys: Vec::default(),
                indexer_url: None,
            }
        }

        /// Gets the owner of the contract
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Gets the attestor address used by this rollup
        #[ink(message)]
        pub fn get_attest_address(&self) -> Vec<u8> {
            signing::get_public_key(&self.attest_key, signing::SigType::Sr25519)
        }

        /// Gets the ecdsa address used by this rollup in the meta transaction
        #[ink(message)]
        pub fn get_attest_ecdsa_address(&self) -> Vec<u8> {
            use ink::env::hash;
            let input = signing::get_public_key(&self.attest_key, signing::SigType::Ecdsa);
            let mut output = <hash::Blake2x256 as hash::HashOutput>::Type::default();
            ink::env::hash_bytes::<hash::Blake2x256>(&input, &mut output);
            output.to_vec()
        }

        /// Gets the sender address used by this rollup (in case of meta-transaction)
        #[ink(message)]
        pub fn get_primary_sender_address(&self) -> Option<Vec<u8>> {
            if let Some(Some(sender_key)) = self
                .primary_consumer
                .as_ref()
                .map(|c| c.sender_key.as_ref())
            {
                let sender_key = signing::get_public_key(sender_key, signing::SigType::Sr25519);
                Some(sender_key)
            } else {
                None
            }
        }

        /// Gets the sender address used by this rollup (in case of meta-transaction)
        #[ink(message)]
        pub fn get_secondary_sender_address(&self, key: u8) -> Option<Vec<u8>> {
            if let Some(Some(sender_key)) = self.secondary_consumers.get(key).map(|c| c.sender_key)
            {
                let sender_key = signing::get_public_key(&sender_key, signing::SigType::Sr25519);
                Some(sender_key)
            } else {
                None
            }
        }

        /// Gets the config of the target consumer contract
        #[ink(message)]
        pub fn get_primary_consumer(&self) -> Option<(String, u8, u8, WasmContractId)> {
            self.primary_consumer
                .as_ref()
                .map(|c| (c.rpc.clone(), c.pallet_id, c.call_id, c.contract_id))
        }

        /// Gets the config of the target consumer contract
        #[ink(message)]
        pub fn get_secondary_consumer(&self, key: u8) -> Option<(String, EvmContractId)> {
            self.secondary_consumers
                .get(key)
                .map(|c| (c.rpc.clone(), c.contract_id))
        }

        /// Configures the target consumer contract (admin only)
        #[ink(message)]
        pub fn set_primary_consumer(
            &mut self,
            rpc: String,
            pallet_id: u8,
            call_id: u8,
            contract_id: Vec<u8>,
            sender_key: Option<Vec<u8>>,
        ) -> Result<()> {
            self.ensure_owner()?;
            self.primary_consumer = Some(WasmContractConfig {
                rpc,
                pallet_id,
                call_id,
                contract_id: contract_id
                    .try_into()
                    .or(Err(ContractError::InvalidAddressLength))?,
                sender_key: match sender_key {
                    Some(key) => Some(key.try_into().or(Err(ContractError::InvalidKeyLength))?),
                    None => None,
                },
            });
            Ok(())
        }

        #[ink(message)]
        pub fn set_secondary_consumer(
            &mut self,
            key: u8,
            config: Option<EvmContractConfig>,
        ) -> Result<()> {
            self.ensure_owner()?;
            match config {
                None => {
                    if let Some(index) =
                        self.secondary_consumers_keys.iter().position(|k| *k == key)
                    {
                        self.secondary_consumers.remove(key);
                        self.secondary_consumers_keys.remove(index);
                    }
                }
                Some(c) => {
                    self.secondary_consumers.insert(key, &c);
                    if self
                        .secondary_consumers_keys
                        .iter()
                        .position(|k| *k == key)
                        .is_none()
                    {
                        self.secondary_consumers_keys.push(key);
                    }
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
        pub fn answer_request(&self) -> Result<Option<Vec<u8>>> {
            let config = self.ensure_client_configured()?;
            let mut client = WasmContract::connect(config)?;

            // Get a request if presents
            let request: LottoManagerRequestMessage = client
                .pop()
                .log_err("answer_request: failed to read queue")?
                .ok_or(ContractError::NoRequestInQueue)?;

            ink::env::debug_println!("Received request: {request:02x?}");

            let response = self.handle_request(request)?;
            // Attach an action to the tx by:
            client.action(Action::Reply(response.encode()));

            let tx = WasmContract::maybe_submit_tx(
                client,
                &self.attest_key,
                config.sender_key.as_ref(),
            )?;
            Ok(tx)
        }

        fn handle_request(&self, message: LottoManagerRequestMessage) -> Result<LottoManagerResponseMessage> {
            let raffle_id = message.raffle_id;

            let response = match message.request {
                Request::CompleteAllRaffles => {
                    match self.inner_complete_all_raffles(raffle_id)? {
                        (true, hashes) => Response::CompletedRaffles(hashes),
                        (false, _) => Response::WaitingSynchronization,
                    }
                    /*
                    if all_raffles_completed {
                        Response::CompletedRaffles(hashes)
                    } else {
                        Response::WaitingSynchronization
                    }

                     */
                }
                Request::DrawNumbers(nb_numbers, smallest_number, biggest_number) => self
                    .inner_get_numbers(raffle_id, nb_numbers, smallest_number, biggest_number)
                    .map(Response::Numbers)?,
                Request::CheckWinners(ref numbers) => {
                    let indexer = Indexer::new(self.get_indexer_url())?;
                    indexer.query_winners(raffle_id, numbers).map(
                        |(substrate_addresses, evm_addresses)| {
                            Response::Winners(substrate_addresses, evm_addresses)
                        },
                    )?
                }
            };

            Ok(LottoResponseMessage {
                request: message,
                response,
            })
        }

        /// Verify if the winning numbers for a raffle are valid (only for past raffles)
        ///
        #[ink(message)]
        pub fn verify_numbers(
            &self,
            contract_id: WasmContractId,
            raffle_id: RaffleId,
            hashes: Vec<lotto_draw_logic::types::Hash>,
            nb_numbers: u8,
            smallest_number: Number,
            biggest_number: Number,
            numbers: Vec<Number>,
        ) -> Result<bool> {
            let config = self.ensure_client_configured()?;

            // check if the target contract is correct
            if contract_id != config.contract_id {
                return Err(ContractError::InvalidContractId);
            }

            let mut client = WasmContract::connect(config)?;

            const LAST_RAFFLE_FOR_VERIF: u32 = ink::selector_id!("LAST_RAFFLE_FOR_VERIF");

            let last_raffle: RaffleId = client
                .get(&LAST_RAFFLE_FOR_VERIF)
                .log_err("verify numbers: last raffle unknown")?
                .ok_or(ContractError::CurrentRaffleUnknown)?;

            // verify the winning numbers only for the past raffles
            if raffle_id > last_raffle {
                return Err(ContractError::UnauthorizedRaffle);
            }

            let draw = Draw::new(nb_numbers, smallest_number, biggest_number)?;
            let result = draw.verify_numbers(raffle_id, hashes, numbers)?;
            Ok(result)
        }

        fn inner_get_numbers(
            &self,
            raffle_id: RaffleId,
            nb_numbers: u8,
            smallest_number: Number,
            biggest_number: Number,
        ) -> Result<Vec<Number>> {
            info!(
                "Request received for raffle {raffle_id} - draw {nb_numbers} numbers between {smallest_number} and {biggest_number}"
            );

            let indexer = Indexer::new(self.get_indexer_url())?;
            let hashes = indexer.query_hashes(raffle_id)?;

            let draw = Draw::new(nb_numbers, smallest_number, biggest_number)?;
            let result = draw.get_numbers(raffle_id, hashes)?;
            Ok(result)
        }

        fn inner_complete_all_raffles(
            &self,
            raffle_id: RaffleId,
        ) -> Result<(bool, Vec<lotto_draw_logic::types::Hash>)> {
            info!("Synchronize raffle {raffle_id} - complete all raffles");

            let indexer = Indexer::new(self.get_indexer_url())?;
            let hashes = indexer.query_hashes(raffle_id)?;

            let expected_nb_hashes = self
                .secondary_consumers_keys
                .len()
                .checked_add(1)
                .ok_or(AddOverFlow)?;
            if hashes.len() == expected_nb_hashes {
                // we already have all hashes, it means all raffles are completed
                return Ok((true, hashes));
            }

            for (_i, key) in self.secondary_consumers_keys.iter().enumerate() {
                // TODO complete the contract raffle only if we don't have the hash

                // get the config linked to this contract
                let config = self.secondary_consumers.get(key);
                // complete the raffle
                let contract = EvmContract::new(config)?;
                contract.complete_raffle(raffle_id, &self.attest_key)?;
            }

            // we have to wait
            Ok((false, hashes))
        }

        fn inner_propagate_result_in_all_raffles(&self, raffle_id: RaffleId) -> Result<bool> {
            info!("Synchronize raffle {raffle_id} - propagate result");

            for (_i, key) in self.secondary_consumers_keys.iter().enumerate() {
                // get the config linked to this contract
                let config = self.secondary_consumers.get(key);
                // encode the reply
                let contract = EvmContract::new(config)?;
                // TODO check the winners
                contract.send_raffle_result(raffle_id, false, Vec::new(), &self.attest_key)?;
            }

            Ok(true)
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
        fn ensure_client_configured(&self) -> Result<&WasmContractConfig> {
            self.primary_consumer
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
            contract_id: WasmContractId,
            /// When we want to manually set the attestor key for signing the message (only dev purpose)
            //attest_key: Vec<u8>,
            /// When we want to use meta tx
            sender_key: Option<Vec<u8>>,
        }

        fn get_env(key: &str) -> String {
            std::env::var(key).expect("env not found")
        }

        fn config() -> EnvVars {
            dotenvy::dotenv().ok();
            let rpc = get_env("RPC");
            let pallet_id: u8 = get_env("PALLET_ID").parse().expect("u8 expected");
            let call_id: u8 = get_env("CALL_ID").parse().expect("u8 expected");
            let contract_id: WasmContractId = hex::decode(get_env("CONTRACT_ID"))
                .expect("hex decode failed")
                .try_into()
                .expect("incorrect length");
            //let attest_key = hex::decode(get_env("ATTEST_KEY")).expect("hex decode failed");
            let sender_key = std::env::var("SENDER_KEY")
                .map(|s| hex::decode(s).expect("hex decode failed"))
                .ok();

            EnvVars {
                rpc: rpc.to_string(),
                pallet_id,
                call_id,
                contract_id: contract_id.into(),
                //attest_key,
                sender_key,
            }
        }

        fn init_contract() -> Lotto {
            let EnvVars {
                rpc,
                pallet_id,
                call_id,
                contract_id,
                //attest_key,
                sender_key,
            } = config();

            let mut lotto = Lotto::default();
            lotto
                .set_primary_consumer(rpc, pallet_id, call_id, contract_id.into(), sender_key)
                .unwrap();

            lotto
                .config_indexer("https://query.substrate.fi/lotto-subquery-shibuya".to_string())
                .unwrap();
            //lotto.set_attest_key(Some(attest_key)).unwrap();

            lotto
        }

        #[ink::test]
        #[ignore = "The target contract must be deployed on the Substrate node and a random number request must be submitted"]
        fn answer_request() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let lotto = init_contract();

            let r = lotto.answer_request().expect("failed to answer request");
            ink::env::debug_println!("answer request: {r:?}");
        }
    }
}
