extern crate alloc;

use crate::error::RaffleDrawError::{self, *};
use crate::types::*;
use alloc::boxed::Box;
use alloc::vec::Vec;
use ethabi::{ParamType, Token};

use phat_offchain_rollup::{clients::evm::EvmRollupClient, Action};
use pink_extension::ResultExt;
use pink_web3::keys::pink::KeyPair;
use crate::raffle_registration_contract::{RaffleRegistrationContract, RaffleRegistrationStatus, RequestForAction};

pub struct EvmContract {
    config: EvmContractConfig,
}

impl EvmContract {
    pub fn new(config: Option<EvmContractConfig>) -> Result<Self, RaffleDrawError> {
        let config = config.ok_or(EvmContractNotConfigured)?;
        Ok(Self { config })
    }
}


impl RaffleRegistrationContract for EvmContract {

    fn get_status(
        &self
    ) -> Option<RaffleRegistrationStatus> {
        // use kv store
        None // TODO
    }

    fn get_draw_number(
        &self
    ) -> Option<DrawNumber> {
        // use kv store
        None // TODO
    }

    fn do_action(
        &self,
        action: RequestForAction,
        attest_key: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, RaffleDrawError> {
        // connect to the contract
        let mut client = connect(&self.config)?;

        // Attach an action to the tx:
        let action = encode_request(&action)?;
        client.action(Action::Reply(action));

        // submit the transaction
        maybe_submit_tx(client, attest_key, self.config.sender_key.as_ref())
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

fn encode_request(request: &RequestForAction) -> Result<Vec<u8>, RaffleDrawError> {
    ink::env::debug_println!("Action Message: {request:?}");

    const REQUEST_SET_CONFIG: u8 = 0;
    const REQUEST_OPEN_REGISTRATIONS: u8 = 1;
    const REQUEST_CLOSE_REGISTRATIONS: u8 = 2;
    const REQUEST_SET_RESULTS: u8 = 3;

    let encoded = match &request {
        RequestForAction::SetConfig(config) => {
            let nb_numbers = config.nb_numbers as u128;
            let min_number = config.min_number as u128;
            let max_number = config.max_number as u128;
            ethabi::encode(&[
                Token::Uint(REQUEST_SET_CONFIG.into()),
                Token::Uint(nb_numbers.into()),
                Token::Uint(min_number.into()),
                Token::Uint(max_number.into()),
            ])
        }
        RequestForAction::CloseRegistrations(draw_number) => {
            let draw_number = *draw_number as u128;
            ethabi::encode(&[
                Token::Uint(REQUEST_CLOSE_REGISTRATIONS.into()),
                Token::Uint(draw_number.into()),
            ])
        }
        RequestForAction::OpenRegistrations(draw_number) => {
            let draw_number = *draw_number as u128;
            ethabi::encode(&[
                Token::Uint(REQUEST_OPEN_REGISTRATIONS.into()),
                Token::Uint(draw_number.into()),
            ])
        }
        RequestForAction::SetResults(draw_number, _, _) => {
            let draw_number = *draw_number as u128;
            // TODO manage winners and results
            ethabi::encode(&[
                Token::Uint(REQUEST_SET_RESULTS.into()),
                Token::Uint(draw_number.into()),
            ])
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




#[cfg(test)]
mod tests {
    use super::*;


    #[ink::test]
    fn encode_response() {

        let nb_numbers : u8= 4;
        let min_number : Number = 0;
        let max_number : Number = 50;
        let config = RaffleConfig { nb_numbers, min_number, max_number };

        let raffle_id = 3;
        let numbers = vec![43, 50, 2, 15];

        let request = RequestForAction::SetConfig(config);

        let encoded_request=
            super::encode_request(&request).expect("Failed to encode request");
        ink::env::debug_println!("EncodedRequest response numbers: {encoded_request:02x?}");
        /*
        let expected : Vec<u8> = hex::decode("0000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000003200000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000002b00000000000000000000000000000000000000000000000000000000000000320000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000f").expect("hex decode failed");
        assert_eq!(expected, encode_request);
         */
    }

    /*

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

     */

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

