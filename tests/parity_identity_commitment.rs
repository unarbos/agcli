#![allow(clippy::needless_borrow)]

#[macro_use]
#[path = "e2e_modules/harness.rs"]
mod e2e_harness;

#[path = "parity.rs"]
mod parity;

#[tokio::test]
async fn parity_identity_commitment() {
    parity::identity_commitment::parity_identity_commitment().await;
}
