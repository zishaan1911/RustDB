use rustdb_server::security::crypto::aes_gcm::XChaCha20Context;

/// ================================
/// XChaCha20-Poly1305 TEST SUITE
/// ================================

#[test]
fn test_xchacha20_roundtrip()
{
    let key = XChaCha20Context::generate_key();
    let ctx = XChaCha20Context::new(key);

    let plaintext = b"hello xchacha20";
    let (ciphertext, nonce, tag) = ctx.encrypt(plaintext)
        .expect("XChaCha20 encryption failed");

    let decrypted = ctx.decrypt(&ciphertext, &nonce, &tag)
        .expect("XChaCha20 decryption failed");

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_xchacha20_tamper()
{
    let key = XChaCha20Context::generate_key();
    let ctx = XChaCha20Context::new(key);

    let plaintext = b"xchacha20 tamper test";
    let (mut ciphertext, nonce, tag) = ctx.encrypt(plaintext)
        .expect("XChaCha20 encryption failed");

    ciphertext[0] ^= 1;

    assert!(ctx.decrypt(&ciphertext, &nonce, &tag).is_err(),
        "Tampering must be detected by XChaCha20 decryption");
}

#[test]
fn test_xchacha20_in_place()
{
    let key = XChaCha20Context::generate_key();
    let ctx = XChaCha20Context::new(key);

    let original = b"in place xchacha20".to_vec();
    let mut buffer = original.clone();

    let (nonce, tag) = ctx.encrypt_in_place(&mut buffer, b"")
        .expect("XChaCha20 in-place encryption failed");

    ctx.decrypt_in_place(&mut buffer, &nonce, b"", &tag)
        .expect("XChaCha20 in-place decryption failed");

    assert_eq!(buffer, original);
}
