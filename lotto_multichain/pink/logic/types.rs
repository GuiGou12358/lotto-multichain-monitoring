use ink::prelude::string::String;

pub type RegistrationContractId = u128;
pub type DrawNumber = u32;
pub type Number = u16;
pub type WasmContractId = [u8; 32];
pub type EvmContractId = [u8; 20];
pub type AccountId32 = [u8; 32];
pub type AccountId20 = [u8; 20];
pub type Hash = [u8; 32];

#[derive(scale::Encode, scale::Decode, Debug, Clone)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum ContractConfig {
    //WasmContractConfig(String, u8, u8, WasmContractId, Option<[u8; 32]>),
    Wasm(WasmContractConfig),
    //EvmContractConfig(String, EvmContractId, Option<[u8; 32]>),
    Evm(EvmContractConfig),
}

#[derive(scale::Encode, scale::Decode, Debug, Clone)]
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

#[derive(scale::Encode, scale::Decode, Debug, Clone)]
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

#[derive(scale::Encode, scale::Decode, Debug, Clone)]
pub struct RaffleConfig {
    pub nb_numbers: u8,
    pub min_number: Number,
    pub max_number: Number,
}

