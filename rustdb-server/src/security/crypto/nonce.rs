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

/// All types of nonces
/// Printable for testing and debugging purposes
#[derive(Debug)]
pub enum NonceType
{
    Secure(usize),
    Hex(usize),
    Base64(usize),
    UrlSafeBase64(usize),
    Uuid,
    Timestamp(usize),
    Counter,
    Hybrid(usize),
    AesGcm,
    XChaCha20,
}

/// All return types of nonces
/// Printable for testing and debugging purposes
#[derive(Debug)]
pub enum NonceResult
{
    String(String),
    Bytes(Vec<u8>),
    U64(u64),
}

/// Central nonce generator
/// Dispatches to all available nonce types
pub fn generate_nonce(nonce_type: NonceType) -> NonceResult
{
    match nonce_type
    {
        // Cryptographic random nonce
        NonceType::Secure(size) =>
        {
            NonceResult::Bytes(generate_secure_nonce(size))
        }

        // Hex encoded nonce
        NonceType::Hex(size) =>
        {
            NonceResult::String(generate_hex_nonce(size))
        }

        // Base64 encoded nonce
        NonceType::Base64(size) =>
        {
            NonceResult::String(generate_base64_nonce(size))
        }

        // URL-safe Base64 nonce
        NonceType::UrlSafeBase64(size) =>
        {
            NonceResult::String(generate_urlsafe_nonce(size))
        }

        // UUID v4 nonce
        NonceType::Uuid =>
        {
            NonceResult::String(generate_uuid_nonce())
        }

        // Timestamp + random nonce
        NonceType::Timestamp(random_size) =>
        {
            NonceResult::String(generate_timestamp_nonce(random_size))
        }

        // Atomic counter nonce
        NonceType::Counter =>
        {
            NonceResult::U64(generate_counter_nonce())
        }

        // Hybrid nonce generator
        NonceType::Hybrid(random_size) =>
        {
            NonceResult::String(generate_hybrid_nonce(random_size))
        }

        // AES-GCM nonce (96-bit)
        NonceType::AesGcm =>
        {
            NonceResult::Bytes(generate_aes_gcm_nonce().to_vec())
        }

        // XChaCha20 nonce (192-bit)
        NonceType::XChaCha20 =>
        {
            NonceResult::Bytes(generate_xchacha20_nonce().to_vec())
        }
    }
}

/// Temporary function to generate all nonce types for testing
/// Can be removed later, or used in benchmarks
pub fn generate_all_nonces_temp()
{
    let secure_nonce = generate_nonce(NonceType::Secure(16));
    let hex_nonce = generate_nonce(NonceType::Hex(16));
    let base64_nonce = generate_nonce(NonceType::Base64(16));
    let urlsafe_nonce = generate_nonce(NonceType::UrlSafeBase64(16));
    let uuid_nonce = generate_nonce(NonceType::Uuid);
    let timestamp_nonce = generate_nonce(NonceType::Timestamp(16));
    let counter_nonce = generate_nonce(NonceType::Counter);
    let hybrid_nonce = generate_nonce(NonceType::Hybrid(16));
    let aes_gcm_nonce = generate_nonce(NonceType::AesGcm);
    let xchacha20_nonce = generate_nonce(NonceType::XChaCha20);

    // Prevent unused warnings (optional, for testing)
    fn print_nonce_result(label: &str, result: NonceResult)
    {
        match result
        {
            NonceResult::String(value) => println!("{}: {}", label, value),
            NonceResult::Bytes(value) => println!("{}: {:?}", label, value),
            NonceResult::U64(value) => println!("{}: {}", label, value),
        }
    }

    print_nonce_result("secure_nonce", secure_nonce);
    print_nonce_result("hex_nonce", hex_nonce);
    print_nonce_result("base64_nonce", base64_nonce);
    print_nonce_result("urlsafe_nonce", urlsafe_nonce);
    print_nonce_result("uuid_nonce", uuid_nonce);
    print_nonce_result("timestamp_nonce", timestamp_nonce);
    print_nonce_result("counter_nonce", counter_nonce);
    print_nonce_result("hybrid_nonce", hybrid_nonce);
    print_nonce_result("aes_gcm_nonce", aes_gcm_nonce);
    print_nonce_result("xchacha20_nonce", xchacha20_nonce);
}
