use base64::{engine::general_purpose, Engine as _};
use std::collections::HashSet;
use uuid::Uuid;

use rustdb_server::security::crypto::nonce::{
    self,
    NonceType,
    NonceResult,
};

/// ================================
/// NONCE TEST SUITE
/// ================================

#[test]
fn test_nonce_generation_and_formats()
{
    match nonce::generate_nonce(NonceType::Secure(16))
    {
        NonceResult::Bytes(v) => assert_eq!(v.len(), 16),
        _ => panic!("Secure nonce must return bytes"),
    }

    match nonce::generate_nonce(NonceType::Hex(16))
    {
        NonceResult::String(s) => assert_eq!(s.len(), 32),
        _ => panic!("Hex nonce must return a string"),
    }

    match nonce::generate_nonce(NonceType::Base64(16))
    {
        NonceResult::String(s) =>
        {
            let decoded = general_purpose::STANDARD_NO_PAD
                .decode(&s)
                .expect("Base64 nonce should decode successfully");

            assert_eq!(decoded.len(), 16);
        }
        _ => panic!("Base64 nonce must return a string"),
    }

    match nonce::generate_nonce(NonceType::UrlSafeBase64(16))
    {
        NonceResult::String(s) =>
        {
            let decoded = general_purpose::URL_SAFE_NO_PAD
                .decode(&s)
                .expect("URL-safe Base64 nonce should decode successfully");

            assert_eq!(decoded.len(), 16);
        }
        _ => panic!("UrlSafeBase64 nonce must return a string"),
    }

    match nonce::generate_nonce(NonceType::Uuid)
    {
        NonceResult::String(s) =>
        {
            assert_eq!(s.len(), 36);
            Uuid::parse_str(&s).expect("UUID nonce must be valid UUID v4");
        }
        _ => panic!("UUID nonce must return a string"),
    }

    match nonce::generate_nonce(NonceType::Timestamp(16))
    {
        NonceResult::String(s) =>
        {
            let parts: Vec<&str> = s.splitn(2, '-').collect();
            assert_eq!(parts.len(), 2);
            assert!(!parts[0].is_empty(), "Timestamp prefix must not be empty");
            assert_eq!(parts[1].len(), 32);
        }
        _ => panic!("Timestamp nonce must return a string"),
    }

    match nonce::generate_nonce(NonceType::Hybrid(16))
    {
        NonceResult::String(s) =>
        {
            let parts: Vec<&str> = s.splitn(3, '-').collect();
            assert_eq!(parts.len(), 3);
            assert!(!parts[0].is_empty(), "Hybrid nonce timestamp must not be empty");
            assert!(!parts[1].is_empty(), "Hybrid nonce counter must not be empty");
            assert_eq!(parts[2].len(), 32);
        }
        _ => panic!("Hybrid nonce must return a string"),
    }

    match nonce::generate_nonce(NonceType::AesGcm)
    {
        NonceResult::Bytes(v) => assert_eq!(v.len(), 12),
        _ => panic!("AES-GCM nonce must return bytes"),
    }

    match nonce::generate_nonce(NonceType::XChaCha20)
    {
        NonceResult::Bytes(v) => assert_eq!(v.len(), 24),
        _ => panic!("XChaCha20 nonce must return bytes"),
    }
}

#[test]
fn test_counter_nonce_monotonic()
{
    let first = match nonce::generate_nonce(NonceType::Counter)
    {
        NonceResult::U64(v) => v,
        _ => panic!("Counter nonce must return u64"),
    };

    let second = match nonce::generate_nonce(NonceType::Counter)
    {
        NonceResult::U64(v) => v,
        _ => panic!("Counter nonce must return u64"),
    };

    assert!(second > first, "Counter nonce should be monotonic");
}

#[test]
fn test_aes_gcm_nonce_uniqueness()
{
    let mut seen = HashSet::new();

    for _ in 0..2000
    {
        let nonce_bytes = match nonce::generate_nonce(NonceType::AesGcm)
        {
            NonceResult::Bytes(v) => v,
            _ => panic!("AES-GCM nonce must return bytes"),
        };

        assert!(seen.insert(nonce_bytes), "AES-GCM nonce collision detected");
    }
}
