#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[openbrush::implementation(AccessControl)]
#[openbrush::contract]
pub mod lotto_contract {
    use crate::{
        config, config::*, raffle_manager, raffle_manager::*,
        raffle_registration::*,
    };
    use openbrush::traits::Storage;
    use phat_rollup_anchor_ink::traits::{rollup_anchor, rollup_anchor::*};

    // Contract storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        access: access_control::Data,
        #[storage_field]
        rollup_anchor: rollup_anchor::Data,
        #[storage_field]
        config: config::Data,
        #[storage_field]
        raffle_manager: raffle_manager::Data,
    }

    impl RollupAnchor for Contract {}
    impl RaffleConfig for Contract {}
    impl Raffle for Contract {}
    impl RaffleManager for Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl rollup_anchor::MessageHandler for Contract {
        fn on_message_received(&mut self, _action: Vec<u8>) -> Result<(), RollupAnchorError> {
            Ok(())
        }
    }

    impl rollup_anchor::EventBroadcaster for Contract {
        fn emit_event_message_queued(&self, _id: u32, _data: Vec<u8>) {}
        fn emit_event_message_processed_to(&self, _id: u32) {}
    }
}
