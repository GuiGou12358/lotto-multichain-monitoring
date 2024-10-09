#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;
extern crate core;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod lotto_draw_evm {
    use alloc::vec::Vec;
    use alloc::boxed::Box;
    use ink::prelude::{format, string::String};
    use phat_offchain_rollup::{clients::evm::EvmRollupClient, Action};
    use pink_extension::chain_extension::signing;
    use pink_extension::{debug, error, http_post, info, vrf, ResultExt};
    use pink_web3::keys::pink::KeyPair;
    use pink_kv_session::traits::QueueSession;
    use ethabi::{ParamType, Token};
    use sp_core::H160;
    use scale::{Decode, Encode};
    use serde::Deserialize;
    use serde_json_core;

    pub type RaffleId = u128;
    pub type Number = u128;
    pub type ContractId = [u8; 20];
    pub type AccountId20 = [u8; 20];

    /// Message to request the lotto lotto_draw or the list of winners
    /// message pushed in the queue by the Ink! smart contract and read by the offchain rollup
    #[derive(Eq, PartialEq, Clone, Debug, scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct LottoRequestMessage {
        /// lotto_draw number
        raffle_id: RaffleId,
        /// request
        request: Request,
    }

    #[derive(Eq, PartialEq, Clone, Debug, scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum Request {
        /// request to lotto_draw the n number between min and max values
        /// arg1: number of numbers for the lotto_draw
        /// arg2:  smallest number for the lotto_draw
        /// arg2:  biggest number for the lotto_draw
        DrawNumbers(u32, Number, Number),
        /// request to check if there is a winner for the given numbers
        CheckWinners(Vec<Number>),
    }

    /// Message sent to provide the lotto lotto_draw or the list of winners
    /// response pushed in the queue by the offchain rollup and read by the Ink! smart contract
    #[derive(Encode, Decode, Debug)]
    struct LottoResponseMessage {
        /// initial request
        request: LottoRequestMessage,
        /// response
        response: Response,
    }

    #[derive(Eq, PartialEq, Clone, scale::Encode, scale::Decode, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum Response {
        /// list of numbers
        Numbers(Vec<Number>),
        /// list of winners
        Winners(Vec<AccountId20>),
    }

    /// DTO use for serializing and deserializing the json
    #[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
    pub struct IndexerResponse<'a> {
        #[serde(borrow)]
        data: IndexerResponseData<'a>,
    }

    #[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
    #[allow(non_snake_case)]
    struct IndexerResponseData<'a> {
        #[serde(borrow)]
        participations: Participations<'a>,
    }

    #[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
    struct Participations<'a> {
        #[serde(borrow)]
        nodes: Vec<ParticipationNode<'a>>,
    }

    #[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
    #[allow(non_snake_case)]
    struct ParticipationNode<'a> {
        accountId: &'a str,
    }

    #[ink(storage)]
    pub struct Lotto {
        owner: AccountId,
        /// config to send the data to the ink! smart contract
        consumer_config: Option<Config>,
        /// indexer endpoint
        indexer_url: Option<String>,
        /// Key for signing the rollup tx.
        attest_key: [u8; 32],
    }

    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    struct Config {
        /// The RPC endpoint of the target blockchain
        rpc: String,
        /// The rollup anchor address on the target blockchain
        contract_id: ContractId,
        /// Key for sending out the rollup meta-tx. None to fallback to the wallet based auth.
        sender_key: Option<[u8; 32]>,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq)]
    #[repr(u8)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        BadOrigin,
        ClientNotConfigured,
        InvalidKeyLength,
        InvalidAddressLength,
        NoRequestInQueue,
        FailedToCreateClient,
        FailedToCommitTx,
        FailedToCallRollup,
        FailedToDecode,
        FailedToGetStorage,
        FailedToEncodeResponse,
        // error when checking the winners
        NoNumber,
        IndexerNotConfigured,
        HttpRequestFailed,
        InvalidResponseBody,
        InvalidSs58Address,
        // error when drawing the numbers
        MinGreaterThanMax,
        AddOverFlow,
        SubOverFlow,
        DivByZero,
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

    impl Lotto {
        #[ink(constructor)]
        pub fn default() -> Self {
            const NONCE: &[u8] = b"lotto";
            let private_key = signing::derive_sr25519_key(NONCE);

            Self {
                owner: Self::env().caller(),
                attest_key: private_key[..32].try_into().expect("Invalid Key Length"),
                consumer_config: None,
                indexer_url: None,
            }
        }

        /// Get the owner of the contract
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Get the attestor address used by this rollup
        #[ink(message)]
        pub fn get_attest_address(&self) -> Vec<u8> {
            signing::get_public_key(&self.attest_key, signing::SigType::Sr25519)
        }

        /// Get the ecdsa address used by this rollup in the meta transaction
        #[ink(message)]
        pub fn get_attest_ecdsa_address(&self) -> Vec<u8> {
            use ink::env::hash;
            let input = signing::get_public_key(&self.attest_key, signing::SigType::Ecdsa);
            let mut output = <hash::Blake2x256 as hash::HashOutput>::Type::default();
            ink::env::hash_bytes::<hash::Blake2x256>(&input, &mut output);
            output.to_vec()
        }

        /// Get the sender address used by this rollup (in case of meta-transaction)
        #[ink(message)]
        pub fn get_sender_address(&self) -> Option<Vec<u8>> {
            if let Some(Some(sender_key)) =
                self.consumer_config.as_ref().map(|c| c.sender_key.as_ref())
            {
                let sender_key = signing::get_public_key(sender_key, signing::SigType::Sr25519);
                Some(sender_key)
            } else {
                None
            }
        }

        /// Set attestor key.
        ///
        /// For dev purpose.
        #[ink(message)]
        pub fn set_attest_key(&mut self, attest_key: Option<Vec<u8>>) -> Result<()> {
            self.attest_key = match attest_key {
                Some(key) => key.try_into().or(Err(ContractError::InvalidKeyLength))?,
                None => {
                    const NONCE: &[u8] = b"attest_key";
                    let private_key = signing::derive_sr25519_key(NONCE);
                    private_key[..32]
                        .try_into()
                        .or(Err(ContractError::InvalidKeyLength))?
                }
            };
            Ok(())
        }

        /// Gets the config of the target consumer contract
        #[ink(message)]
        pub fn get_target_contract(&self) -> Option<(String, ContractId)> {
            self.consumer_config
                .as_ref()
                .map(|c| (c.rpc.clone(), c.contract_id))
        }

        /// Configures the target consumer contract (admin only)
        #[ink(message)]
        pub fn config_target_contract(
            &mut self,
            rpc: String,
            contract_id: Vec<u8>,
            sender_key: Option<Vec<u8>>,
        ) -> Result<()> {
            self.ensure_owner()?;
            self.consumer_config = Some(Config {
                rpc,
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
            let mut client = connect(config)?;

            // Get a request if presents
            let raw_data = client
                .session()
                .pop()
                .log_err("answer_request: failed to read queue")
                .or(Err(ContractError::FailedToGetStorage))?
                .ok_or(ContractError::NoRequestInQueue)?;

            let request = decode_message(&raw_data)?;
            let response = self.handle_request(request)?;
            let action = encode_response(&response)?;

            // Attach an action to the tx by:
            client.action(Action::Reply(action));

            maybe_submit_tx(client, &self.attest_key, config.sender_key.as_ref())
        }

        fn handle_request(&self, message: LottoRequestMessage) -> Result<LottoResponseMessage> {
            let response = match message.request {
                Request::DrawNumbers(nb_numbers, smallest_number, biggest_number) => self
                    .inner_get_numbers(
                        message.raffle_id,
                        nb_numbers,
                        smallest_number,
                        biggest_number,
                    )
                    .map(Response::Numbers)?,
                Request::CheckWinners(ref numbers) => self
                    .inner_get_winners(message.raffle_id, numbers)
                    .map(Response::Winners)?,
            };

            Ok(LottoResponseMessage {
                request: message,
                response,
            })
        }

        /// Verify if the winning numbers for a raffle are valid (only for past raffles)
        #[ink(message)]
        pub fn verify_numbers(
            &self,
            contract_id: ContractId,
            raffle_id: RaffleId,
            //nb_numbers: u8,
            nb_numbers: u32,
            smallest_number: Number,
            biggest_number: Number,
            numbers: Vec<Number>,
        ) -> Result<bool> {
            let config = self.ensure_client_configured()?;

            // check if the target contract is correct
            if contract_id != config.contract_id {
                return Err(ContractError::InvalidContractId);
            }

/*
            let mut client = connect(config)?;
            const LAST_RAFFLE_FOR_VERIF: u32 = ink::selector_id!("LAST_RAFFLE_FOR_VERIF");

            let last_raffle: RaffleId = client
                .session()
                .get(&LAST_RAFFLE_FOR_VERIF)
                .log_err("verify numbers: last raffle unknown")?
                .ok_or(ContractError::CurrentRaffleUnknown)?;

            // verify the winning numbers only for the past raffles
            if raffle_id > last_raffle {
                return Err(ContractError::UnauthorizedRaffle);
            }

 */

            self.inner_verify_numbers(
                raffle_id,
                nb_numbers,
                smallest_number,
                biggest_number,
                numbers,
            )
        }

        pub fn inner_verify_numbers(
            &self,
            raffle_id: RaffleId,
            nb_numbers: u32,
            smallest_number: Number,
            biggest_number: Number,
            numbers: Vec<Number>,
        ) -> Result<bool> {
            let winning_numbers =
                self.inner_get_numbers(raffle_id, nb_numbers, smallest_number, biggest_number)?;
            if winning_numbers.len() != numbers.len() {
                return Ok(false);
            }

            for n in &numbers {
                if !winning_numbers.contains(n) {
                    return Ok(false);
                }
            }

            Ok(true)
        }

        fn inner_get_numbers(
            &self,
            raffle_id: RaffleId,
            nb_numbers: u32,
            smallest_number: Number,
            biggest_number: Number,
        ) -> Result<Vec<Number>> {
            info!(
                "Request received for raffle {raffle_id} - draw {nb_numbers} numbers between {smallest_number} and {biggest_number}"
            );

            let contract_id = self.ensure_client_configured()?.contract_id;

            if smallest_number > biggest_number {
                return Err(ContractError::MinGreaterThanMax);
            }

            let mut numbers = Vec::new();
            let mut i: u8 = 0;

            while numbers.len() < nb_numbers as usize {
                // build a salt for this lotto_draw number
                let mut salt: Vec<u8> = Vec::new();
                salt.extend_from_slice(&i.to_be_bytes());
                salt.extend_from_slice(&raffle_id.to_be_bytes());
                salt.extend_from_slice(&contract_id);

                // lotto_draw the number
                let number = self.inner_get_number(salt, smallest_number, biggest_number)?;
                // check if the number has already been drawn
                if !numbers.iter().any(|&n| n == number) {
                    // the number has not been drawn yet => we added it
                    numbers.push(number);
                }
                //i += 1;
                i = i.checked_add(1).ok_or(ContractError::AddOverFlow)?;
            }

            info!("Numbers: {numbers:?}");

            Ok(numbers)
        }

        fn inner_get_number(&self, salt: Vec<u8>, min: Number, max: Number) -> Result<Number> {
            let output = vrf(&salt);
            // keep only 8 bytes to compute the random u64
            let mut arr = [0x00; 8];
            arr.copy_from_slice(&output[0..8]);
            let rand_u64 = u64::from_le_bytes(arr);

            // r = rand_u64() % (max - min + 1) + min
            // use u128 because (max - min + 1) can be equal to (U64::MAX - 0 + 1)
            let a = (max as u128)
                .checked_sub(min as u128)
                .ok_or(ContractError::SubOverFlow)?
                .checked_add(1u128)
                .ok_or(ContractError::AddOverFlow)?;
            //let b = (rand_u64 as u128) % a;
            let b = (rand_u64 as u128).checked_rem_euclid(a).ok_or(ContractError::DivByZero)?;
            let r = b
                .checked_add(min as u128)
                .ok_or(ContractError::AddOverFlow)?;

            Ok(r as Number)
        }

        fn inner_get_winners(
            &self,
            raffle_id: RaffleId,
            numbers: &Vec<Number>,
        ) -> Result<Vec<AccountId20>> {
            info!(
                "Request received to get the winners for raffle id {raffle_id} and numbers {numbers:?} "
            );

            if numbers.is_empty() {
                return Err(ContractError::NoNumber);
            }

            // check if the endpoint is configured
            let indexer_endpoint = self.ensure_indexer_configured()?;

            // build the headers
            let headers = alloc::vec![
                ("Content-Type".into(), "application/json".into()),
                ("Accept".into(), "application/json".into())
            ];
            // build the filter
            let mut filter = format!(
                r#"filter:{{and:[{{numRaffle:{{equalTo:\"{}\"}}}}"#,
                raffle_id
            );
            for n in numbers {
                let f = format!(r#",{{numbers:{{contains:\"{}\"}}}}"#, n);
                filter.push_str(&f);
            }
            filter.push_str("]}");

            // build the body
            let body = format!(
                r#"{{"query" : "{{participations({}){{ nodes {{ accountId }} }} }}"}}"#,
                filter
            );

            debug!("body: {body}");

            // query the indexer
            let resp = http_post!(indexer_endpoint, body, headers);

            // check the result
            if resp.status_code != 200 {
                ink::env::debug_println!("status code {}", resp.status_code);
                return Err(ContractError::HttpRequestFailed);
            }

            // parse the result
            let result: IndexerResponse = serde_json_core::from_slice(resp.body.as_slice())
                .or(Err(ContractError::InvalidResponseBody))?
                .0;

            // add the winners
            let mut winners = Vec::new();
            for w in result.data.participations.nodes.iter() {
                // build the accountId from the string address
                let account_id : AccountId20 = hex::decode(w.accountId)
                    .expect("hex decode failed")
                    .try_into()
                    .expect("incorrect length");
                winners.push(account_id);
            }

            info!("Winners: {winners:02x?}");

            Ok(winners)
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
        fn ensure_client_configured(&self) -> Result<&Config> {
            self.consumer_config
                .as_ref()
                .ok_or(ContractError::ClientNotConfigured)
        }

        fn ensure_indexer_configured(&self) -> Result<&String> {
            self.indexer_url
                .as_ref()
                .ok_or(ContractError::IndexerNotConfigured)
        }
    }

    fn connect(config: &Config) -> Result<EvmRollupClient> {
        let contract_id : H160 = config.contract_id.into();
        let result = EvmRollupClient::new(
            &config.rpc,
            contract_id,
        )
            .log_err("failed to create rollup client");

        match result {
            Ok(client) => Ok(client),
            Err(e) => {
                error!("Error : {:?}", e);
                ink::env::debug_println!("Error : {:?}", e);
                Err(ContractError::FailedToCreateClient)
            }
        }
    }

    fn decode_message(raw_data: &[u8]) -> Result<LottoRequestMessage> {

        ink::env::debug_println!("Received raw request: {raw_data:02x?}");

        /*
        (uint _raffleId, RequestType _requestType, bytes memory _request, bytes memory _response) = abi.decode(_action, (uint, RequestType, bytes, bytes));

        require(_requestType == RequestType.DRAW_NUMBERS ||  _requestType == RequestType.CHECK_WINNERS, "cannot parse action");
        if (_requestType == RequestType.DRAW_NUMBERS){
            (uint8 _nbNumbers, uint _minNumber, uint _maxNumber) = abi.decode(_request, (uint8, uint , uint));
            (uint[] memory _numbers) = abi.decode(_response, (uint[]));
            _innerSetResults(_raffleId, _nbNumbers, _minNumber, _maxNumber, _numbers);
        } else if (_requestType == RequestType.CHECK_WINNERS){
            (uint[] memory _numbers) = abi.decode(_request, (uint[]));
            (address[] memory _winners) = abi.decode(_response, (address[]));
            _innerSetWinners(_raffleId, _numbers, _winners);
        }
         */

        // Decode the queue data by ethabi (uint, uint8, bytes)
        let Ok(decoded) = ethabi::decode(&[ParamType::Uint(32), ParamType::Uint(32)], &raw_data[0..64]) else {
            return Err(ContractError::FailedToDecode);
        };

        let [Token::Uint(raffle_id), Token::Uint(request_type)] = decoded.as_slice() else {
            return Err(ContractError::FailedToDecode);
        };

        ink::env::debug_println!("Received request with raffle_id {raffle_id} and request_type {request_type}");
        let raffle_id = raffle_id.as_u128();
        let request_type = request_type.as_u32();

        if request_type == 0 { // DRAW_NUMBERS
            ink::env::debug_println!("Draw numbers ...");
            // Decode by ethabi (uint8, uint , uint)
            let Ok(request_decoded) = ethabi::decode(&[ParamType::Uint(32), ParamType::Uint(32), ParamType::Uint(32), ParamType::Uint(32), ParamType::Uint(32)], &raw_data) else {
                return Err(ContractError::FailedToDecode);
            };
            let [Token::Uint(_), Token::Uint(_), Token::Uint(nb_numbers), Token::Uint(min_number), Token::Uint(max_number)] = request_decoded.as_slice() else {
                return Err(ContractError::FailedToDecode);
            };
            ink::env::debug_println!("Received request to draw {nb_numbers} numbers between {min_number} and {max_number}");

            let nb_numbers = nb_numbers.as_u32();
            let min_number = min_number.as_u128();
            let max_number = max_number.as_u128();

            return Ok(LottoRequestMessage {
                raffle_id,
                request: Request::DrawNumbers(nb_numbers, min_number, max_number),
            });
        }
        if request_type == 1 { // CHECK_WINNERS
            ink::env::debug_println!("Check winners ...");
            // Decode by ethabi (uint[])
            let Ok(request_decoded) = ethabi::decode(&[ParamType::Uint(32), ParamType::Uint(32), ParamType::Array(Box::new(ParamType::Uint(32)))], &raw_data) else {
                return Err(ContractError::FailedToDecode);
            };
            let [Token::Uint(_), Token::Uint(_), Token::Array(ref numbers)] = request_decoded.as_slice() else {
                return Err(ContractError::FailedToDecode);
            };
            let numbers : Vec<Number> = numbers.into_iter().map(|n: &ethabi::Token| {if let ethabi::Token::Uint(v) = n { v.as_u128() } else { 0 } }).collect();

            ink::env::debug_println!("Received request to check winners for numbers {numbers:?}");

            return Ok(LottoRequestMessage {
                raffle_id,
                request: Request::CheckWinners(numbers),
            });
        }

        Err(ContractError::FailedToDecode)
    }


    fn encode_response(message: &LottoResponseMessage) -> Result<Vec<u8>> {

        ink::env::debug_println!("Response Message: {message:?}");

        let raffle_id = message.request.raffle_id;

        const RESPONSE_DRAW_NUMBERS : u8 = 0;
        const RESPONSE_CHECK_WINNERS : u8 = 1;

        let encoded = match (&message.request.request, &message.response) {
            (Request::DrawNumbers(nb_numbers, smallest_number, biggest_number), Response::Numbers(numbers)) => {
                let numbers : Vec<Token> = numbers.into_iter().map(|n: &Number| Token::Uint((*n).into())).collect();

/*
                const request = abiCoder.encode(['uint8', 'uint', 'uint'], [4, 1, 50]);
                const response = abiCoder.encode(['uint[]'], [[40, 50, 2, 15]]);
                const action = abiCoder.encode(['uint', 'uint8', 'bytes', 'bytes'], [raffleId, DRAW_NUMBERS, request, response]);
 */
                let request = ethabi::encode(&[
                    Token::Uint((*nb_numbers).into()),
                    Token::Uint((*smallest_number).into()),
                    Token::Uint((*biggest_number).into()),
                ]);
                let response = ethabi::encode(&[
                    Token::Array(numbers),
                ]);
                ethabi::encode(&[
                    Token::Uint(raffle_id.into()),
                    Token::Uint(RESPONSE_DRAW_NUMBERS.into()),
                    Token::Bytes(request),
                    Token::Bytes(response),
                ])
            },
            (Request::CheckWinners(ref numbers), Response::Winners(winners)) => {
                let numbers : Vec<Token> = numbers.into_iter().map(|n: &Number| Token::Uint((*n).into())).collect();
                let winners : Vec<Token> = winners.into_iter().map(|a: &AccountId20| Token::Address((*a).into())).collect();
/*
                const request = abiCoder.encode(['uint[]'], [[33, 47, 5, 6]]);
                const response = abiCoder.encode(['address[]'], [[]]);
                const action = abiCoder.encode(['uint', 'uint8', 'bytes', 'bytes'], [raffleId, CHECK_WINNERS, request, response]);
  */
                let request = ethabi::encode(&[
                    Token::Array(numbers),
                ]);
                let response = ethabi::encode(&[
                    Token::Array(winners),
                ]);

                ethabi::encode(&[
                    Token::Uint(raffle_id.into()),
                    Token::Uint(RESPONSE_CHECK_WINNERS.into()),
                    Token::Bytes(request),
                    Token::Bytes(response),
                ])
            },
            _ => return Err(ContractError::FailedToEncodeResponse),
        };
        Ok(encoded)
    }

    fn maybe_submit_tx(
        client: EvmRollupClient,
        attest_key: &[u8; 32],
        sender_key: Option<&[u8; 32]>,
    ) -> Result<Option<Vec<u8>>> {
        let maybe_submittable = client
            .commit()
            .log_err("failed to commit")
            .map_err(|_| ContractError::FailedToCommitTx)?;

        if let Some(submittable) = maybe_submittable {
            let attest_pair = KeyPair::from(*attest_key);
            let tx_id = if let Some(sender_key) = sender_key {
                // Prefer to meta-tx
                let sender_pair = KeyPair::from(*sender_key);
                submittable
                    .submit_meta_tx(&attest_pair, &sender_pair)
                    .log_err("failed to submit rollup meta-tx")?
            } else {
                // Fallback to account-based authentication
                submittable
                    .submit(attest_pair)
                    .log_err("failed to submit rollup tx")?
            };
            return Ok(Some(tx_id));
        }
        Ok(None)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        struct EnvVars {
            /// The RPC endpoint of the target blockchain
            rpc: String,
            /// The rollup anchor address on the target blockchain
            contract_id: ContractId,
            /// When we want to manually set the attestor key for signing the message (only dev purpose)
            attest_key: Vec<u8>,
            /// When we want to use meta tx
            sender_key: Option<Vec<u8>>,
        }

        fn get_env(key: &str) -> String {
            std::env::var(key).expect("env not found")
        }

        fn config() -> EnvVars {
            dotenvy::dotenv().ok();
            let rpc = get_env("RPC");
            let contract_id: ContractId = hex::decode(get_env("CONTRACT_ID"))
                .expect("hex decode failed")
                .try_into()
                .expect("incorrect length");
            let attest_key = hex::decode(get_env("ATTEST_KEY")).expect("hex decode failed");
            let sender_key = std::env::var("SENDER_KEY")
                .map(|s| hex::decode(s).expect("hex decode failed"))
                .ok();

            EnvVars {
                rpc: rpc.to_string(),
                contract_id: contract_id.into(),
                attest_key,
                sender_key,
            }
        }

        fn init_contract() -> Lotto {
            let EnvVars {
                rpc,
                contract_id,
                attest_key,
                sender_key,
            } = config();

            let mut lotto = Lotto::default();
            lotto
                .config_target_contract(rpc, contract_id.into(), sender_key)
                .unwrap();

            lotto
                .config_indexer("https://query.substrate.fi/lotto-subquery-shibuya".to_string())
                .unwrap();
            lotto.set_attest_key(Some(attest_key)).unwrap();

            lotto
        }

        #[ink::test]
        fn test_get_numbers() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let lotto = init_contract();

            let raffle_id = 1;
            let nb_numbers = 5;
            let smallest_number = 1;
            let biggest_number = 50;

            let result = lotto
                .inner_get_numbers(raffle_id, nb_numbers, smallest_number, biggest_number)
                .unwrap();
            assert_eq!(nb_numbers as usize, result.len());
            for &n in result.iter() {
                assert!(n >= smallest_number);
                assert!(n <= biggest_number);
            }

            ink::env::debug_println!("random numbers: {result:?}");
        }

        #[ink::test]
        fn test_get_numbers_from_1_to_5() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let lotto = init_contract();

            let raffle_id = 1;
            let nb_numbers = 5;
            let smallest_number = 1;
            let biggest_number = 5;

            let result = lotto
                .inner_get_numbers(raffle_id, nb_numbers, smallest_number, biggest_number)
                .unwrap();
            assert_eq!(nb_numbers as usize, result.len());
            for &n in result.iter() {
                assert!(n >= smallest_number);
                assert!(n <= biggest_number);
            }

            ink::env::debug_println!("random numbers: {result:?}");
        }

        #[ink::test]
        fn test_with_different_draw_num() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let lotto = init_contract();

            let nb_numbers = 5;
            let smallest_number = 1;
            let biggest_number = 50;

            let mut results = Vec::new();

            for i in 0..100 {
                let result = lotto
                    .inner_get_numbers(i, nb_numbers, smallest_number, biggest_number)
                    .unwrap();
                // this result must be different from the previous ones
                results.iter().for_each(|r| assert_ne!(result, *r));

                // same request message means same result
                let result_2 = lotto
                    .inner_get_numbers(i, nb_numbers, smallest_number, biggest_number)
                    .unwrap();
                assert_eq!(result, result_2);

                results.push(result);
            }
        }

        #[ink::test]
        fn test_verify_numbers() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let lotto = init_contract();

            let raffle_id = 1;
            let nb_numbers = 5;
            let smallest_number = 1;
            let biggest_number = 50;

            let numbers = lotto
                .inner_get_numbers(raffle_id, nb_numbers, smallest_number, biggest_number)
                .unwrap();

            assert_eq!(
                Ok(true),
                lotto.inner_verify_numbers(
                    raffle_id,
                    nb_numbers,
                    smallest_number,
                    biggest_number,
                    numbers.clone()
                )
            );

            assert_eq!(
                Ok(false),
                lotto.inner_verify_numbers(
                    raffle_id + 1,
                    nb_numbers,
                    smallest_number,
                    biggest_number,
                    numbers.clone()
                )
            );
        }

        #[ink::test]
        fn test_verify_numbers_with_bad_contract_id() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let mut lotto = init_contract();

            let raffle_id = 1;
            let nb_numbers = 5;
            let smallest_number = 1;
            let biggest_number = 50;

            let numbers = lotto
                .inner_get_numbers(raffle_id, nb_numbers, smallest_number, biggest_number)
                .unwrap();

            assert_eq!(
                Ok(true),
                lotto.inner_verify_numbers(
                    raffle_id,
                    nb_numbers,
                    smallest_number,
                    biggest_number,
                    numbers.clone()
                )
            );

            let target_contract = lotto.get_target_contract().unwrap();

            let bad_contract_id: ContractId = [0; 20];
            lotto
                .config_target_contract(
                    target_contract.0,
                    bad_contract_id.to_vec(),
                    None,
                )
                .unwrap();

            assert_eq!(
                Ok(false),
                lotto.inner_verify_numbers(
                    raffle_id,
                    nb_numbers,
                    smallest_number,
                    biggest_number,
                    numbers.clone()
                )
            );
        }

        #[ink::test]
        fn test_get_winners() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let lotto = init_contract();

            let draw_num = 2;
            let numbers = vec![15, 1, 44, 28];

            let winners = lotto.inner_get_winners(draw_num, &numbers).unwrap();
            ink::env::debug_println!("winners: {winners:?}");
        }

        #[ink::test]
        fn test_no_winner() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let lotto = init_contract();

            let draw_num = 0;
            let numbers = vec![150, 1, 44, 2800];

            let winners = lotto.inner_get_winners(draw_num, &numbers).unwrap();
            assert_eq!(0, winners.len());
        }

        #[ink::test]
        fn test_no_number() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let lotto = init_contract();

            let draw_num = 0;
            let numbers = vec![];

            let result = lotto.inner_get_winners(draw_num, &numbers);
            assert_eq!(Err(ContractError::NoNumber), result);
        }

        #[ink::test]
        #[ignore = "The target contract must be deployed on the Substrate node and a random number request must be submitted"]
        fn answer_request() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            ink::env::debug_println!("init contract before");
            let lotto = init_contract();

            ink::env::debug_println!("init contract after");

            let r = lotto.answer_request().expect("failed to answer request");
            ink::env::debug_println!("answer request: {r:?}");
        }


        #[ink::test]
        fn encode_response() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let raffle_id = 3;
            let numbers = vec![43, 50, 2, 15];

            let response = LottoResponseMessage {
                request: LottoRequestMessage {raffle_id, request: Request::DrawNumbers(4, 1, 50)},
                response: Response::Numbers(numbers.clone()),
            };
            let encoded_response = super::encode_response(&response).expect("Failed to encode response");
            ink::env::debug_println!("Reply response numbers: {encoded_response:02x?}");
            let expected : Vec<u8> = hex::decode("0000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000003200000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000002b00000000000000000000000000000000000000000000000000000000000000320000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000f").expect("hex decode failed");
            assert_eq!(expected, encoded_response);

            let response = LottoResponseMessage {
                request: LottoRequestMessage {raffle_id, request: Request::CheckWinners(numbers)},
                response: Response::Winners(vec![]),
            };
            let encoded_response = super::encode_response(&response).expect("Failed to encode response");
            ink::env::debug_println!("Reply response winners: {encoded_response:02x?}");
            let expected : Vec<u8> = hex::decode("000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000002b00000000000000000000000000000000000000000000000000000000000000320000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000f000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000").expect("hex decode failed");
            assert_eq!(expected, encoded_response);

        }

        #[ink::test]
        fn decode_message_draw_numbers() {
            let encoded_message : Vec<u8> = hex::decode("00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000032").expect("hex decode failed");
            let message = super::decode_message(encoded_message.as_slice()).expect("Error to decode message");
            ink::env::debug_println!("message: {message:?}");
            assert_eq!(2, message.raffle_id);
            assert_eq!(Request::DrawNumbers(4, 1, 50), message.request);
        }

        #[ink::test]
        fn decode_message_check_winners() {
            let encoded_message : Vec<u8> = hex::decode("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000021000000000000000000000000000000000000000000000000000000000000002f00000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000006").expect("hex decode failed");
            let message = super::decode_message(encoded_message.as_slice()).expect("Error to decode message");
            ink::env::debug_println!("message: {message:?}");
            assert_eq!(1, message.raffle_id);
            assert_eq!(Request::CheckWinners(vec![33, 47, 5, 6]), message.request);
        }

        #[ink::test]
        fn decode_array() {
            let raw : Vec<u8> = hex::decode("000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000021000000000000000000000000000000000000000000000000000000000000002f00000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000006").expect("hex decode failed");
            let request_decoded = ethabi::decode(&[ParamType::Array(Box::new(ParamType::Uint(32)))], &raw).expect("Error 1 to decode message");

            let [Token::Array(ref numbers)] = request_decoded.as_slice() else {
                assert!(false, "Error 2 to decode message");
                return Ok(());
            };
            let numbers : Vec<u32> = numbers.into_iter().map(|n: &ethabi::Token| {if let ethabi::Token::Uint(v) = n { v.as_u32() } else { 0 } }).collect();

            assert_eq!(vec![33u32, 47, 5, 6], numbers);

        }

        #[ink::test]
        fn encode_array() {

            let mut numbers = Vec::new();
            numbers.push(Token::Uint(33u32.into()));
            numbers.push(Token::Uint(47u32.into()));
            numbers.push(Token::Uint(5u32.into()));
            numbers.push(Token::Uint(6u32.into()));

            let array_encoded = ethabi::encode(&[Token::Array(numbers)]);

            let expected : Vec<u8> = hex::decode("000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000021000000000000000000000000000000000000000000000000000000000000002f00000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000006").expect("hex decode failed");
            assert_eq!(expected, array_encoded);
        }


    }
}
