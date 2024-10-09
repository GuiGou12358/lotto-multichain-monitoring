extern crate alloc;

use crate::error::RaffleDrawError::{self, *};
use crate::types::*;
use alloc::boxed::Box;
use alloc::vec::Vec;
use ethabi::{ParamType, Token};

use phat_offchain_rollup::{clients::evm::EvmRollupClient, Action};
use pink_extension::ResultExt;
use pink_web3::keys::pink::KeyPair;

#[derive(Debug)]
enum ActionSecondary {
    /// complete the raffle
    CompleteRaffle(RaffleId),
    /// notify the raffle there is a winner or not for the given raffle id
    /// if the bool is true, it means there is at least one winner in all contracts and the raffles must be stopped
    /// if the bool is false, it means there is no winner and the raffle must continue
    /// the list of accounts is the winners who participated with this contract
    SetWinners(RaffleId, bool, Vec<AccountId20>),
}

pub struct EvmContract {
    config: EvmContractConfig,
}

impl EvmContract {
    pub fn new(config: Option<EvmContractConfig>) -> Result<Self, RaffleDrawError> {
        let config = config.ok_or(EvmContractNotConfigured)?;
        Ok(Self { config })
    }

    pub fn complete_raffle(
        &self,
        raffle_id: RaffleId,
        attest_key: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, RaffleDrawError> {
        self.do_action(ActionSecondary::CompleteRaffle(raffle_id), attest_key)
    }

    pub fn send_raffle_result(
        &self,
        raffle_id: RaffleId,
        has_winner: bool,
        winners: Vec<AccountId20>,
        attest_key: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, RaffleDrawError> {
        self.do_action(
            ActionSecondary::SetWinners(raffle_id, has_winner, winners),
            attest_key,
        )
    }

    pub fn do_action(
        &self,
        action: ActionSecondary,
        attest_key: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, RaffleDrawError> {
        // connect to the contract
        let mut client = connect(&self.config)?;

        // Attach an action to the tx:
        let action = encode_action(&action)?;
        client.action(Action::Reply(action));

        // submit the transaction
        maybe_submit_tx(client, &attest_key, self.config.sender_key.as_ref())
    }
}

fn connect(config: &EvmContractConfig) -> Result<EvmRollupClient, RaffleDrawError> {
    let contract_id: sp_core::H160 = config.contract_id.into();
    let result =
        EvmRollupClient::new(&config.rpc, contract_id).log_err("failed to create rollup client");

    match result {
        Ok(client) => Ok(client),
        Err(e) => {
            pink_extension::error!("Error : {:?}", e);
            ink::env::debug_println!("Error : {:?}", e);
            Err(FailedToCreateClient)
        }
    }
}

fn decode_request(raw_data: &[u8]) -> Result<LottoRequestMessage, RaffleDrawError> {
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
    let Ok(decoded) = ethabi::decode(
        &[ParamType::Uint(32), ParamType::Uint(32)],
        &raw_data[0..64],
    ) else {
        return Err(FailedToDecodeRequest);
    };

    let [Token::Uint(raffle_id), Token::Uint(request_type)] = decoded.as_slice() else {
        return Err(FailedToDecodeRequest);
    };

    ink::env::debug_println!(
        "Received request with raffle_id {raffle_id} and request_type {request_type}"
    );
    let raffle_id = raffle_id.as_u32();
    let request_type = request_type.as_u32();

    if request_type == 0 {
        // DRAW_NUMBERS
        ink::env::debug_println!("Draw numbers ...");
        // Decode by ethabi (uint8, uint , uint)
        let Ok(request_decoded) = ethabi::decode(
            &[
                ParamType::Uint(32),
                ParamType::Uint(32),
                ParamType::Uint(32),
                ParamType::Uint(32),
                ParamType::Uint(32),
            ],
            &raw_data,
        ) else {
            return Err(FailedToDecodeRequest);
        };
        let [Token::Uint(_), Token::Uint(_), Token::Uint(nb_numbers), Token::Uint(min_number), Token::Uint(max_number)] =
            request_decoded.as_slice()
        else {
            return Err(FailedToDecodeRequest);
        };
        ink::env::debug_println!(
            "Received request to draw {nb_numbers} numbers between {min_number} and {max_number}"
        );

        let nb_numbers = nb_numbers.as_u32() as u8;
        let min_number = min_number.as_u128() as Number;
        let max_number = max_number.as_u128() as Number;

        return Ok(LottoRequestMessage {
            raffle_id,
            request: Request::DrawNumbers(nb_numbers, min_number, max_number),
        });
    }
    if request_type == 1 {
        // CHECK_WINNERS
        ink::env::debug_println!("Check winners ...");
        // Decode by ethabi (uint[])
        let Ok(request_decoded) = ethabi::decode(
            &[
                ParamType::Uint(32),
                ParamType::Uint(32),
                ParamType::Array(Box::new(ParamType::Uint(32))),
            ],
            &raw_data,
        ) else {
            return Err(FailedToDecodeRequest);
        };
        let [Token::Uint(_), Token::Uint(_), Token::Array(ref numbers)] =
            request_decoded.as_slice()
        else {
            return Err(FailedToDecodeRequest);
        };
        // TODO implement try UINT -> Number
        let numbers: Vec<Number> = numbers
            .into_iter()
            .map(|n: &ethabi::Token| {
                if let ethabi::Token::Uint(v) = n {
                    v.as_u128() as Number
                } else {
                    0
                }
            })
            .collect();

        ink::env::debug_println!("Received request to check winners for numbers {numbers:?}");

        return Ok(LottoRequestMessage {
            raffle_id,
            request: Request::CheckWinners(numbers),
        });
    }

    Err(FailedToDecodeRequest)
}

fn encode_response(message: &LottoResponseMessage) -> Result<Vec<u8>, RaffleDrawError> {
    ink::env::debug_println!("Response Message: {message:?}");

    let raffle_id = message.request.raffle_id;

    const RESPONSE_DRAW_NUMBERS: u8 = 0;
    const RESPONSE_CHECK_WINNERS: u8 = 1;

    let encoded = match (&message.request.request, &message.response) {
        (
            Request::DrawNumbers(nb_numbers, smallest_number, biggest_number),
            Response::Numbers(numbers),
        ) => {
            let numbers: Vec<Token> = numbers
                .into_iter()
                .map(|n: &Number| Token::Uint((*n).into()))
                .collect();

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
            let response = ethabi::encode(&[Token::Array(numbers)]);
            ethabi::encode(&[
                Token::Uint(raffle_id.into()),
                Token::Uint(RESPONSE_DRAW_NUMBERS.into()),
                Token::Bytes(request),
                Token::Bytes(response),
            ])
        }
        (
            Request::CheckWinners(ref numbers),
            Response::Winners(substrate_addresses, evm_addresses),
        ) => {
            let numbers: Vec<Token> = numbers
                .into_iter()
                .map(|n: &Number| Token::Uint((*n).into()))
                .collect();
            let winners = Vec::new();
            // TODO manage winners AccountId20
            //let winners : Vec<Token> = winners.into_iter().map(|a: &AccountId20| Token::Address((*a).into())).collect();
            /*
                          const request = abiCoder.encode(['uint[]'], [[33, 47, 5, 6]]);
                          const response = abiCoder.encode(['address[]'], [[]]);
                          const action = abiCoder.encode(['uint', 'uint8', 'bytes', 'bytes'], [raffleId, CHECK_WINNERS, request, response]);
            */
            let request = ethabi::encode(&[Token::Array(numbers)]);
            let response = ethabi::encode(&[Token::Array(winners)]);

            ethabi::encode(&[
                Token::Uint(raffle_id.into()),
                Token::Uint(RESPONSE_CHECK_WINNERS.into()),
                Token::Bytes(request),
                Token::Bytes(response),
            ])
        }
        _ => return Err(FailedToEncodeResponse),
    };
    Ok(encoded)
}

fn encode_action(action: &ActionSecondary) -> Result<Vec<u8>, RaffleDrawError> {
    ink::env::debug_println!("Action Message: {action:?}");

    const RESPONSE_COMPLETE_RAFFLE: u8 = 0;

    let encoded = match &action {
        ActionSecondary::CompleteRaffle(raffle_id) => {
            let raffle_id = *raffle_id as u128;
            ethabi::encode(&[
                Token::Uint(raffle_id.into()),
                Token::Uint(RESPONSE_COMPLETE_RAFFLE.into()),
            ])
        }
        _ => return Err(FailedToEncodeResponse),
    };
    Ok(encoded)
}

fn maybe_submit_tx(
    client: EvmRollupClient,
    attest_key: &[u8; 32],
    sender_key: Option<&[u8; 32]>,
) -> Result<Option<Vec<u8>>, RaffleDrawError> {
    let maybe_submittable = client
        .commit()
        .log_err("failed to commit")
        .map_err(|_| FailedToCommitTx)?;

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
    use crate::types::Request::{CheckWinners, DrawNumbers};

    #[ink::test]
    fn encode_response() {
        let _ = env_logger::try_init();
        pink_extension_runtime::mock_ext::mock_all_ext();

        let raffle_id = 3;
        let numbers = vec![43, 50, 2, 15];

        let response = LottoResponseMessage {
            request: LottoRequestMessage {
                raffle_id,
                request: Request::DrawNumbers(4, 1, 50),
            },
            response: Response::Numbers(numbers.clone()),
        };
        let encoded_response =
            super::encode_response(&response).expect("Failed to encode response");
        ink::env::debug_println!("Reply response numbers: {encoded_response:02x?}");
        let expected : Vec<u8> = hex::decode("0000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000003200000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000002b00000000000000000000000000000000000000000000000000000000000000320000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000f").expect("hex decode failed");
        assert_eq!(expected, encoded_response);

        let response = LottoResponseMessage {
            request: LottoRequestMessage {
                raffle_id,
                request: Request::CheckWinners(numbers),
            },
            response: Response::Winners(vec![], vec![]),
        };
        let encoded_response =
            super::encode_response(&response).expect("Failed to encode response");
        ink::env::debug_println!("Reply response winners: {encoded_response:02x?}");
        let expected : Vec<u8> = hex::decode("000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000002b00000000000000000000000000000000000000000000000000000000000000320000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000f000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000").expect("hex decode failed");
        assert_eq!(expected, encoded_response);
    }

    #[ink::test]
    fn decode_message_draw_numbers() {
        let encoded_message : Vec<u8> = hex::decode("00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000032").expect("hex decode failed");
        let message =
            super::decode_request(encoded_message.as_slice()).expect("Error to decode message");
        ink::env::debug_println!("message: {message:?}");
        assert_eq!(2, message.raffle_id);
        assert_eq!(DrawNumbers(4, 1, 50), message.request);
    }

    #[ink::test]
    fn decode_message_check_winners() {
        let encoded_message : Vec<u8> = hex::decode("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000021000000000000000000000000000000000000000000000000000000000000002f00000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000006").expect("hex decode failed");
        let message =
            super::decode_request(encoded_message.as_slice()).expect("Error to decode message");
        ink::env::debug_println!("message: {message:?}");
        assert_eq!(1, message.raffle_id);
        assert_eq!(CheckWinners(vec![33, 47, 5, 6]), message.request);
    }

    #[ink::test]
    fn decode_array() {
        let raw : Vec<u8> = hex::decode("000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000021000000000000000000000000000000000000000000000000000000000000002f00000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000006").expect("hex decode failed");
        let request_decoded =
            ethabi::decode(&[ParamType::Array(Box::new(ParamType::Uint(32)))], &raw)
                .expect("Error 1 to decode message");

        let [Token::Array(ref numbers)] = request_decoded.as_slice() else {
            assert!(false, "Error 2 to decode message");
            return Ok(());
        };
        let numbers: Vec<u32> = numbers
            .into_iter()
            .map(|n: &ethabi::Token| {
                if let ethabi::Token::Uint(v) = n {
                    v.as_u32()
                } else {
                    0
                }
            })
            .collect();

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
