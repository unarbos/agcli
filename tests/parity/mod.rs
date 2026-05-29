#[cfg(feature = "e2e")]
#[macro_use]
#[path = "../e2e_modules/harness.rs"]
pub mod harness;

#[cfg(feature = "e2e")]
pub mod balance_transfer;

#[cfg(feature = "e2e")]
pub mod governance;

#[cfg(feature = "e2e")]
pub mod misc;

#[cfg(feature = "e2e")]
pub mod stake_basic;

#[cfg(feature = "e2e")]
pub mod wallet;

#[cfg(feature = "e2e")]
pub mod root;
