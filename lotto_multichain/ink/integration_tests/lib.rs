#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[cfg(all(test, feature = "e2e-tests"))]
mod raffle_manager;

#[cfg(all(test, feature = "e2e-tests"))]
mod raffle_registration;