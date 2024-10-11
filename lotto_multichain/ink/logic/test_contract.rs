#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[openbrush::contract]
pub mod lotto_contract {
    use crate::{
        config, config::*, raffle_manager, raffle_manager::*, raffle_registration,
        raffle_registration::*,
    };
    use openbrush::traits::Storage;

    /// Contract storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        config: config::Data,
        #[storage_field]
        raffle_registration: raffle_registration::Data,
        #[storage_field]
        raffle_manager: raffle_manager::Data,
    }

    impl RaffleConfig for Contract {}
    impl Raffle for Contract {}
    impl RaffleManager for Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self::default()
        }
    }
}
