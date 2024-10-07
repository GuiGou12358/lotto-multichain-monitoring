
#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RaffleError {
    IncorrectDrawNumber,
    IncorrectStatus,
    IncorrectConfig,
    ConfigNotSet,
    DifferentConfig,
    IncorrectNbNumbers,
    IncorrectNumbers,
    DifferentResults,
    ExistingResults,
    ExistingWinners,
    TransferError,
    AddOverFlow,
    SubOverFlow,
    DivByZero,
    NoReward,
}
