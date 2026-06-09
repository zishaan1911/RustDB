// Minimal stub for the crypto module to unblock CI.
// Replace with real implementations as needed.

#![allow(dead_code)]

pub fn init() {
    // initialization placeholder
}

pub fn checksum(data: &[u8]) -> u32 {
    // placeholder implementation using a simple xor-based checksum
    // replace with crc32fast or real crypto functions as needed
    let mut acc: u32 = 0;
    for &b in data {
        acc = acc.wrapping_add(b as u32);
    }
    acc
}
