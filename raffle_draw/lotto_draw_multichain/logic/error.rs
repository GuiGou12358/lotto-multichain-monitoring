
#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RaffleDrawError {
    BadOrigin,
    ClientNotConfigured,
    InvalidKeyLength,
    InvalidAddressLength,
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
