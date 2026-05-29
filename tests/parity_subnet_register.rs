#![cfg(feature = "e2e")]

#[macro_use]
#[path = "e2e_modules/harness.rs"]
mod harness;

#[path = "parity/subnet-register.rs"]
mod subnet_register;
