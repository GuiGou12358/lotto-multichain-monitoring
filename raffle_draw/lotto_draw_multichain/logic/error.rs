
#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RaffleDrawError {
    BadOrigin,
    ClientNotConfigured,
    InvalidKeyLength,
    InvalidAddressLength,
    EvmContractNotConfigured,
    WasmContractNotConfigured,
    FailedToDecodeRequest,
    FailedToEncodeResponse,
    FailedToEncodeAction,
    NoRequestInQueue,
    FailedToCreateClient,
    FailedToCommitTx,
    FailedToCallRollup,
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

impl From<phat_offchain_rollup::Error> for RaffleDrawError {
    fn from(error: phat_offchain_rollup::Error) -> Self {
        pink_extension::error!("error in the rollup: {:?}", error);
        ink::env::debug_println!("Error : {:?}", error);
        RaffleDrawError::FailedToCallRollup
    }
}
