#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub type DrawNumber = u32;
pub type Number = u16;

pub mod config;
pub mod error;
pub mod raffle;

