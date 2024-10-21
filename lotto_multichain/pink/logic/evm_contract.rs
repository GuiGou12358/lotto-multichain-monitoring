extern crate alloc;

use crate::error::RaffleDrawError::{self, *};
use crate::types::*;
use alloc::boxed::Box;
use alloc::vec::Vec;
use ethabi::{ParamType, Token};

use crate::raffle_registration_contract::{
    RaffleRegistrationContract, RaffleRegistrationStatus, RequestForAction,
};
use kv_session::traits::KvSession;
use phat_offchain_rollup::{clients::evm::EvmRollupClient, Action};
use pink_extension::ResultExt;
use pink_web3::keys::pink::KeyPair;

pub struct EvmContract {
    config: EvmContractConfig,
}

impl EvmContract {
    pub fn new(config: Option<EvmContractConfig>) -> Result<Self, RaffleDrawError> {
        let config = config.ok_or(EvmContractNotConfigured)?;
        Ok(Self { config })
    }

    fn connect(&self) -> Result<EvmRollupClient, RaffleDrawError> {
        let contract_id: sp_core::H160 = self.config.contract_id.into();
        let result = EvmRollupClient::new(&self.config.rpc, contract_id)
            .log_err("failed to create rollup client");

        match result {
            Ok(client) => Ok(client),
            Err(e) => {
                pink_extension::error!("Error : {:?}", e);
                ink::env::debug_println!("Error : {:?}", e);
                Err(FailedToCreateClient)
            }
        }
    }
}

impl RaffleRegistrationContract for EvmContract {
    fn do_action(
        &self,
        expected_draw_number: Option<DrawNumber>,
        expected_status: Option<RaffleRegistrationStatus>,
        action: RequestForAction,
        attest_key: &[u8; 32],
    ) -> Result<bool, RaffleDrawError> {
        // connect to the contract
        let mut client = self.connect()?;

        let correct_status = match expected_status {
            Some(expected_status) => {
                let status = get_status(&mut client)?.ok_or(StatusUnknown)?;
                status == expected_status
            }
            None => true,
        };

        let correct_draw_number = match expected_draw_number {
            Some(expected_draw_number) => {
                let draw_number = get_draw_number(&mut client)?.ok_or(DrawNumberUnknown)?;
                draw_number == expected_draw_number
            }
            None => true,
        };

        if correct_draw_number && correct_status {
            // the contract is already synchronized
            return Ok(true);
        }

        // synchronize the contract =>  Attach an action to the tx
        let action = encode_request(&action)?;
        client.action(Action::Reply(action));
        // submit the transaction
        maybe_submit_tx(client, attest_key, self.config.sender_key.as_ref())?;

        Ok(false)
    }
}

fn encode_request(request: &RequestForAction) -> Result<Vec<u8>, RaffleDrawError> {
    ink::env::debug_println!("Action Message: {request:?}");

    const REQUEST_SET_CONFIG: u8 = 0;
    const REQUEST_OPEN_REGISTRATIONS: u8 = 1;
    const REQUEST_CLOSE_REGISTRATIONS: u8 = 2;
    const REQUEST_SET_RESULTS: u8 = 3;

    let encoded = match &request {
        RequestForAction::SetConfigAndStart(config, contract_id) => {
            let nb_numbers = config.nb_numbers as u128;
            let min_number = config.min_number as u128;
            let max_number = config.max_number as u128;
            let contract_id = *contract_id as u128;
            let body = ethabi::encode(&[
                Token::Uint(nb_numbers.into()),
                Token::Uint(min_number.into()),
                Token::Uint(max_number.into()),
                Token::Uint(contract_id.into()),
            ]);
            ethabi::encode(&[Token::Uint(REQUEST_SET_CONFIG.into()), Token::Bytes(body)])
        }
        RequestForAction::OpenRegistrations(draw_number) => {
            let draw_number = *draw_number as u128;
            let body = ethabi::encode(&[Token::Uint(draw_number.into())]);
            ethabi::encode(&[
                Token::Uint(REQUEST_OPEN_REGISTRATIONS.into()),
                Token::Bytes(body),
            ])
        }
        RequestForAction::CloseRegistrations(draw_number) => {
            let draw_number = *draw_number as u128;
            let body = ethabi::encode(&[Token::Uint(draw_number.into())]);
            ethabi::encode(&[
                Token::Uint(REQUEST_CLOSE_REGISTRATIONS.into()),
                Token::Bytes(body),
            ])
        }
        RequestForAction::SetResults(draw_number, ref numbers, ref winners) => {
            let draw_number = *draw_number as u128;
            let numbers: Vec<Token> = numbers
                .into_iter()
                .map(|n: &Number| Token::Uint((*n).into()))
                .collect();
            let winners: Vec<Token> = Vec::new(); // winners.into_iter().map(|a: &AccountId20| Token::Address((*a).into())).collect();
                                                  // TODO manage winners
            let body = ethabi::encode(&[
                Token::Uint(draw_number.into()),
                Token::Array(numbers),
                Token::Array(winners),
            ]);
            ethabi::encode(&[Token::Uint(REQUEST_SET_RESULTS.into()), Token::Bytes(body)])
        }
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

const DRAW_NUMBER: &[u8] = "DRAW_NUMBER".as_bytes();
const STATUS: &[u8] = "STATUS".as_bytes();

fn get_draw_number(client: &mut EvmRollupClient) -> Result<Option<DrawNumber>, RaffleDrawError> {
    let raw_value = client
        .session()
        .get(DRAW_NUMBER)
        .log_err("Draw number unknown in kv store")
        .map_err(|_| DrawNumberUnknown)?;

    let result = match raw_value {
        Some(raw) => Some(decode_draw_number(raw.as_slice())?),
        None => None,
    };

    Ok(result)
}

fn decode_draw_number(raw: &[u8]) -> Result<DrawNumber, RaffleDrawError> {
    let tokens = ethabi::decode(&[ParamType::Uint(32)], raw)
        .log_err("Fail to decode draw number in kv store")
        .map_err(|_| FailedToDecodeDrawNumber)?;
    let [Token::Uint(draw_number)] = tokens.as_slice() else {
        return Err(FailedToDecodeDrawNumber);
    };
    Ok(draw_number.as_u32())
}

fn get_status(
    client: &mut EvmRollupClient,
) -> Result<Option<RaffleRegistrationStatus>, RaffleDrawError> {
    let raw_value = client
        .session()
        .get(STATUS)
        .log_err("Status unknown in kv store")
        .map_err(|_| StatusUnknown)?;

    let result = match raw_value {
        Some(raw) => Some(decode_status(raw.as_slice())?),
        None => None,
    };
    Ok(result)
}

fn decode_status(raw: &[u8]) -> Result<RaffleRegistrationStatus, RaffleDrawError> {
    let tokens = ethabi::decode(&[ParamType::Uint(32)], raw)
        .log_err("Fail to decode status in kv store")
        .map_err(|_| FailedToDecodeStatus)?;
    let [Token::Uint(status)] = tokens.as_slice() else {
        return Err(FailedToDecodeStatus);
    };
    let status = match status.as_u32() {
        0 => RaffleRegistrationStatus::NotStarted,
        1 => RaffleRegistrationStatus::Started,
        2 => RaffleRegistrationStatus::RegistrationOpen,
        3 => RaffleRegistrationStatus::RegistrationClosed,
        4 => RaffleRegistrationStatus::ResultsReceived,
        _ => return Err(FailedToDecodeStatus),
    };

    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ink::test]
    fn encode_request_set_config_and_start() {
        let nb_numbers: u8 = 4;
        let min_number: Number = 1;
        let max_number: Number = 50;
        let config = RaffleConfig {
            nb_numbers,
            min_number,
            max_number,
        };

        let registration_id = 33;

        let request = RequestForAction::SetConfigAndStart(config, registration_id);

        let encoded_request = encode_request(&request).expect("Failed to encode request");
        ink::env::debug_println!("Encoded request: {encoded_request:02x?}");

        let expected : Vec<u8> = hex::decode("0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000320000000000000000000000000000000000000000000000000000000000000021")
            .expect("hex decode failed");
        assert_eq!(expected, encoded_request);
    }

    #[ink::test]
    fn encode_request_open_registrations() {
        let draw_number = 11;

        let request = RequestForAction::OpenRegistrations(draw_number);

        let encoded_request = encode_request(&request).expect("Failed to encode request");
        ink::env::debug_println!("Encoded request: {encoded_request:02x?}");

        let expected : Vec<u8> = hex::decode("000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000b")
            .expect("hex decode failed");
        assert_eq!(expected, encoded_request);
    }

    #[ink::test]
    fn encode_request_close_registrations() {
        let draw_number = 11;

        let request = RequestForAction::CloseRegistrations(draw_number);

        let encoded_request = encode_request(&request).expect("Failed to encode request");
        ink::env::debug_println!("Encoded request: {encoded_request:02x?}");

        let expected : Vec<u8> = hex::decode("000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000b")
            .expect("hex decode failed");
        assert_eq!(expected, encoded_request);
    }

    #[ink::test]
    fn encode_request_set_results() {
        let draw_number = 11;
        let numbers = vec![33, 47, 5, 6];
        let winners = vec![];

        let request = RequestForAction::SetResults(draw_number, numbers, winners);

        let encoded_request = encode_request(&request).expect("Failed to encode request");
        ink::env::debug_println!("Encoded request: {encoded_request:02x?}");

        let expected : Vec<u8> = hex::decode("000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000000b0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000021000000000000000000000000000000000000000000000000000000000000002f000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000")
            .expect("hex decode failed");
        assert_eq!(expected, encoded_request);
    }

    #[ink::test]
    fn decode_status() {
        let raw: Vec<u8> =
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .expect("hex decode failed");
        let status = super::decode_status(raw.as_slice()).expect("Fail to decode status");
        assert_eq!(status, RaffleRegistrationStatus::NotStarted);

        let raw: Vec<u8> =
            hex::decode("0000000000000000000000000000000000000000000000000000000000000001")
                .expect("hex decode failed");
        let status = super::decode_status(raw.as_slice()).expect("Fail to decode status");
        assert_eq!(status, RaffleRegistrationStatus::Started);

        let raw: Vec<u8> =
            hex::decode("0000000000000000000000000000000000000000000000000000000000000002")
                .expect("hex decode failed");
        let status = super::decode_status(raw.as_slice()).expect("Fail to decode status");
        assert_eq!(status, RaffleRegistrationStatus::RegistrationOpen);

        let raw: Vec<u8> =
            hex::decode("0000000000000000000000000000000000000000000000000000000000000003")
                .expect("hex decode failed");
        let status = super::decode_status(raw.as_slice()).expect("Fail to decode status");
        assert_eq!(status, RaffleRegistrationStatus::RegistrationClosed);

        let raw: Vec<u8> =
            hex::decode("0000000000000000000000000000000000000000000000000000000000000004")
                .expect("hex decode failed");
        let status = super::decode_status(raw.as_slice()).expect("Fail to decode status");
        assert_eq!(status, RaffleRegistrationStatus::ResultsReceived);
    }

    #[ink::test]
    fn decode_draw_number() {
        let raw: Vec<u8> =
            hex::decode("000000000000000000000000000000000000000000000000000000000000000b")
                .expect("hex decode failed");
        let draw_number =
            super::decode_draw_number(raw.as_slice()).expect("Fail to decode draw number");
        assert_eq!(draw_number, 11);
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
