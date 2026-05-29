#![allow(clippy::needless_borrow)]

#[macro_use]
#[path = "e2e_modules/harness.rs"]
mod e2e_harness;

mod parity_cases {
    use super::e2e_harness::*;

    include!("parity/subnet-hyperparams.rs");
}
