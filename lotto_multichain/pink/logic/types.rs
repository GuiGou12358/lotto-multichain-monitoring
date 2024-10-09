extern crate alloc;

use alloc::vec::Vec;
use ink::prelude::string::String;

pub type RegistrationContractId = u128;
pub type DrawNumber = u32;
pub type Number = u16;
pub type WasmContractId = [u8; 32];
pub type EvmContractId = [u8; 20];
pub type AccountId32 = [u8; 32];
pub type AccountId20 = [u8; 20];
pub type Hash = [u8; 32];

#[derive(scale::Encode, scale::Decode, Debug)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct WasmContractConfig {
    /// The RPC endpoint of the target blockchain
    pub rpc: String,
    pub pallet_id: u8,
    pub call_id: u8,
    /// The rollup anchor address on the target blockchain
    pub contract_id: WasmContractId,
    /// Key for sending out the rollup meta-tx. None to fallback to the wallet based auth.
    pub sender_key: Option<[u8; 32]>,
}

#[derive(scale::Encode, scale::Decode, Debug)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct EvmContractConfig {
    /// The RPC endpoint of the target blockchain
    pub rpc: String,
    /// The rollup anchor address on the target blockchain
    pub contract_id: EvmContractId,
    /// Key for sending out the rollup meta-tx. None to fallback to the wallet based auth.
    pub sender_key: Option<[u8; 32]>,
}
/*
/// Message to request the lotto lotto_draw or the list of winners
/// message pushed in the queue by the Ink! smart contract and read by the offchain rollup
#[derive(Eq, PartialEq, Clone, Debug, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct LottoRequestMessage {
    /// lotto_draw number
    pub raffle_id: RaffleId,
    /// request
    pub request: Request,
}

#[derive(Eq, PartialEq, Clone, Debug, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum Request {
    /// request to complete the lotto draw on all chains
    CompleteAllRaffles,
    /// request to lotto_draw the n number between min and max values
    /// arg1: number of numbers for the lotto_draw
    /// arg2:  smallest number for the lotto_draw
    /// arg2:  biggest number for the lotto_draw
    DrawNumbers(u8, Number, Number),
    /// request to check if there is a winner for the given numbers
    CheckWinners(Vec<Number>),
}

/// Message sent to provide the lotto lotto_draw or the list of winners
/// response pushed in the queue by the offchain rollup and read by the Ink! smart contract
#[derive(scale::Encode, scale::Decode, Debug)]
pub struct LottoResponseMessage {
    /// initial request
    pub request: LottoRequestMessage, // TODO hash the request
    /// response
    pub response: Response,
}

#[derive(Eq, PartialEq, Clone, scale::Encode, scale::Decode, Debug)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum Response {
    // waiting synchronization between contracts
    WaitingSynchronization,
    /// list of hashes for all raffles completed on different chain
    CompletedRaffles(Vec<Hash>),
    /// list of numbers
    Numbers(Vec<Number>), // TODO add the salt hash
    /// list of winners
    Winners(Vec<AccountId32>, Vec<AccountId20>), // TODO manage AccountId20
}
 */

#[derive(Debug, Eq, PartialEq, Copy, Clone, scale::Encode, scale::Decode)]
pub struct RaffleConfig {
    pub nb_numbers: u8,
    pub min_number: Number,
    pub max_number: Number,
}

