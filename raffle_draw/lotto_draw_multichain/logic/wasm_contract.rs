extern crate alloc;

use crate::error::RaffleDrawError::{self, *};
use crate::types::*;
use alloc::boxed::Box;
use alloc::vec::Vec;
use phat_offchain_rollup::clients::ink::{Action, InkRollupClient};

use scale::{Decode, Encode};
use pink_extension::ResultExt;
use pink_web3::keys::pink::KeyPair;

#[derive(Eq, PartialEq, Clone, scale::Encode, scale::Decode, Debug)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
enum ActionSecondary {
    /// complete the raffle
    CompleteRaffle(RaffleId),
    /// notify the raffle there is a winner or not for the given raffle id
    /// if the bool is true, it means there is at least one winner in all contracts and the raffles must be stopped
    /// if the bool is false, it means there is no winner and the raffle must continue
    /// the list of accounts is the winners who participated with this contract
    SetWinners(RaffleId, bool, Vec<AccountId32>),
}

pub struct WasmContract {
    config: WasmContractConfig,
}

impl WasmContract {
    pub fn new(config: Option<WasmContractConfig>) -> Result<Self, RaffleDrawError> {
        let config = config.ok_or(WasmContractNotConfigured)?;
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
        winners: Vec<AccountId32>,
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
        let mut client = Self::connect(&self.config)?;

        // Attach an action to the tx:
        client.action(Action::Reply(scale::Encode::encode(&action)));

        // submit the transaction
        Self::maybe_submit_tx(client, &attest_key, self.config.sender_key.as_ref())
    }

    pub fn connect(config: &WasmContractConfig) -> Result<InkRollupClient, RaffleDrawError> {
        let result = InkRollupClient::new(
            &config.rpc,
            config.pallet_id,
            config.call_id,
            &config.contract_id,
        )
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

    pub fn maybe_submit_tx(
        client: InkRollupClient,
        attest_key: &[u8; 32],
        sender_key: Option<&[u8; 32]>,
    ) -> Result<Option<Vec<u8>>, RaffleDrawError> {
        let maybe_submittable = client
            .commit()
            .log_err("failed to commit")
            .map_err(|_| FailedToCommitTx)?;

        if let Some(submittable) = maybe_submittable {
            let tx_id = if let Some(sender_key) = sender_key {
                // Prefer to meta-tx
                submittable
                    .submit_meta_tx(attest_key, sender_key)
                    .log_err("failed to submit rollup meta-tx")?
            } else {
                // Fallback to account-based authentication
                submittable
                    .submit(attest_key)
                    .log_err("failed to submit rollup tx")?
            };
            return Ok(Some(tx_id));
        }
        Ok(None)
    }
}



#[cfg(test)]
mod tests {
    use super::*;


    #[ink::test]
    fn encode_response_numbers() {
        let _ = env_logger::try_init();
        pink_extension_runtime::mock_ext::mock_all_ext();

        //Request received for raffle 6 - draw 4 numbers between 1 and 50
        // Numbers: [4, 49, 41, 16]

        let raffle_id = 6;
        let numbers = vec![4, 49, 41, 16];

        let response = LottoResponseMessage {
            request: LottoRequestMessage {raffle_id, request: Request::DrawNumbers(4, 1, 50)},
            response: Response::Numbers(numbers.clone()),
        };
        let encoded_response = response.encode();
        ink::env::debug_println!("Reply response numbers: {encoded_response:02x?}");

        let response = LottoResponseMessage {
            request: LottoRequestMessage {raffle_id, request: Request::CheckWinners(numbers)},
            response: Response::Winners(vec![]),
        };
        let encoded_response = response.encode();
        ink::env::debug_println!("Reply response winners: {encoded_response:02x?}");

    }


    #[ink::test]
    fn encode_keys() {

        const QUEUE_PREFIX : &[u8] = b"q/";

        const QUEUE_HEAD_KEY : &[u8] = b"_head";
        let head_key = [QUEUE_PREFIX, QUEUE_HEAD_KEY].concat();
        ink::env::debug_println!("queue head key: {head_key:02x?}");

        const QUEUE_TAIL_KEY : &[u8] = b"_tail";
        let tail_key = [QUEUE_PREFIX, QUEUE_TAIL_KEY].concat();
        ink::env::debug_println!("queue tail key: {tail_key:02x?}");

        let id: u32 = 11;
        let key = [QUEUE_PREFIX, &id.encode()].concat();
        ink::env::debug_println!("queue key: {key:02x?}");

    }

    #[ink::test]
    fn decode_message() {
        let encoded_message : Vec<u8> = hex::decode("0600000001100400310029001000").expect("hex decode failed");
        let message = LottoRequestMessage::decode(&mut encoded_message.as_slice());
        ink::env::debug_println!("message: {message:?}");

        let encoded_message : Vec<u8> = hex::decode("07000000000401003200").expect("hex decode failed");
        let message = LottoRequestMessage::decode(&mut encoded_message.as_slice());
        ink::env::debug_println!("message: {message:?}");

    }


}
