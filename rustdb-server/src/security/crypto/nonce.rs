use base64::{engine::general_purpose, Engine as _};
use rand::rngs::OsRng;
use rand::RngCore;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

static NONCE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Cryptographic random nonce generator
/// Returns: Vec<u8>
/// Size: user-defined (bytes)
pub fn generate_secure_nonce(size: usize) -> Vec<u8>
{
    let mut nonce = vec![0u8; size];

    OsRng.fill_bytes(&mut nonce);

    nonce
}

/// Hex encoded nonce
/// Returns: String
/// Size: size * 2 characters
pub fn generate_hex_nonce(size: usize) -> String
{
    let nonce = generate_secure_nonce(size);

    let mut out = vec![0u8; size * 2];

    const LUT: &[u8; 16] = b"0123456789abcdef";

    for (i, b) in nonce.iter().enumerate()
    {
        out[i * 2] = LUT[(b >> 4) as usize];
        out[i * 2 + 1] = LUT[(b & 0x0F) as usize];
    }

    unsafe
    {
        String::from_utf8_unchecked(out)
    }
}

/// Base64 encoded nonce
/// Returns: String
/// Size: ~4 * ceil(size / 3)
pub fn generate_base64_nonce(size: usize) -> String
{
    let nonce = generate_secure_nonce(size);

    general_purpose::STANDARD_NO_PAD.encode(nonce)
}

/// URL-safe Base64 nonce
/// Returns: String
/// Size: ~4 * ceil(size / 3)
pub fn generate_urlsafe_nonce(size: usize) -> String
{
    let nonce = generate_secure_nonce(size);

    general_purpose::URL_SAFE_NO_PAD.encode(nonce)
}

/// UUID v4 nonce
/// Returns: String
/// Size: 36 characters
pub fn generate_uuid_nonce() -> String
{
    Uuid::new_v4().to_string()
}

/// Timestamp + random nonce
/// Returns: String
/// Format: <timestamp_ns>-<random_hex>
pub fn generate_timestamp_nonce(random_size: usize) -> String
{
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let random = generate_hex_nonce(random_size);

    let mut out = String::with_capacity(64 + random_size * 2);

    use std::fmt::Write;

    let _ = write!(out, "{}-{}", timestamp, random);

    out
}

/// Atomic counter nonce
/// Returns: u64
/// Size: 8 bytes
pub fn generate_counter_nonce() -> u64
{
    NONCE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Hybrid nonce generator (recommended for distributed systems)
/// Combines:
/// - timestamp (ns)
/// - atomic counter
/// - cryptographic random
///
/// Returns: String
/// Format: <timestamp_ns>-<counter>-<random_hex>
pub fn generate_hybrid_nonce(random_size: usize) -> String
{
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let counter = generate_counter_nonce();

    let random = generate_hex_nonce(random_size);

    let mut out = String::with_capacity(64 + random_size * 2);

    use std::fmt::Write;

    let _ = write!(out, "{}-{}-{}", timestamp, counter, random);

    out
}

/// AES-GCM nonce (96-bit)
/// Returns: [u8; 12]
/// Size: 12 bytes
pub fn generate_aes_gcm_nonce() -> [u8; 12]
{
    let mut nonce = [0u8; 12];

    OsRng.fill_bytes(&mut nonce);

    nonce
}

/// XChaCha20 nonce (192-bit)
/// Returns: [u8; 24]
/// Size: 24 bytes
pub fn generate_xchacha20_nonce() -> [u8; 24]
{
    let mut nonce = [0u8; 24];

    OsRng.fill_bytes(&mut nonce);

    nonce
}

/// Temporary function to generate all nonce types for testing
/// Can be removed later, or used in benchmarks
pub fn generate_all_nonces_temp()
{
    // Raw cryptographic nonce
    let secure_nonce = generate_secure_nonce(16);

    // Hex encoded nonce
    let hex_nonce = generate_hex_nonce(16);

    // Base64 encoded nonce
    let base64_nonce = generate_base64_nonce(16);

    // URL-safe Base64 nonce
    let urlsafe_nonce = generate_urlsafe_nonce(16);

    // UUID nonce
    let uuid_nonce = generate_uuid_nonce();

    // Timestamp nonce
    let timestamp_nonce = generate_timestamp_nonce(16);

    // Counter nonce
    let counter_nonce = generate_counter_nonce();

    // Hybrid nonce
    let hybrid_nonce = generate_hybrid_nonce(16);

    // AES-GCM nonce (12 bytes)
    let aes_gcm_nonce = generate_aes_gcm_nonce();

    // XChaCha20 nonce (24 bytes)
    let xchacha20_nonce = generate_xchacha20_nonce();

    // Prevent unused warnings (optional, for testing)
    println!("{:?}", secure_nonce);
    println!("{}", hex_nonce);
    println!("{}", base64_nonce);
    println!("{}", urlsafe_nonce);
    println!("{}", uuid_nonce);
    println!("{}", timestamp_nonce);
    println!("{}", counter_nonce);
    println!("{}", hybrid_nonce);
    println!("{:?}", aes_gcm_nonce);
    println!("{:?}", xchacha20_nonce);
}