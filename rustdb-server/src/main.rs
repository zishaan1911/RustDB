
/// Temporary main function for testing purposes. Will be removed in the future.
/// Please leave this function as is, as it is used for testing and benchmarking nonce generation.
/// If you want to test something, just modify it. Some output will be generated through the `generate_all_nonces_temp` function, which generates all types of nonces.
/// Dont remove it, as this will generate unused warninges.
/// Thanks :) ~ Silas
mod security;

use crate::security::crypto::nonce;
use crate::security::crypto::nonce::NonceType;

fn main()
{
    nonce::generate_all_nonces_temp();

    println!("{:?}", nonce::generate_nonce(NonceType::Hybrid(16)));
}