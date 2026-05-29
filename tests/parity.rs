#![cfg(feature = "e2e")]

#[macro_use]
#[path = "e2e_modules/harness.rs"]
mod harness;

#[path = "parity/stake-advanced.rs"]
mod stake_advanced;
