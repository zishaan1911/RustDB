use rustdb_server::security::crypto::aes_gcm::AesGcmContext;

/// ================================
/// AES-GCM TEST SUITE
/// ================================

#[test]
fn test_aes_gcm_roundtrip()
{
    let key = AesGcmContext::generate_key();
    let ctx = AesGcmContext::new(key);

    let plaintext = b"hello aes-gcm";
    let (ciphertext, nonce, tag) = ctx.encrypt(plaintext)
        .expect("AES-GCM encryption failed");

    let decrypted = ctx.decrypt(&ciphertext, &nonce, &tag)
        .expect("AES-GCM decryption failed");

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_aes_gcm_tamper()
{
    let key = AesGcmContext::generate_key();
    let ctx = AesGcmContext::new(key);

    let plaintext = b"tamper test";
    let (mut ciphertext, nonce, tag) = ctx.encrypt(plaintext)
        .expect("AES-GCM encryption failed");

    ciphertext[0] ^= 1;

    assert!(ctx.decrypt(&ciphertext, &nonce, &tag).is_err(),
        "Tampering must be detected by AES-GCM decryption");
}

#[test]
fn test_aes_gcm_in_place()
{
    let key = AesGcmContext::generate_key();
    let ctx = AesGcmContext::new(key);

    let original = b"in place test".to_vec();
    let mut buffer = original.clone();

    let (nonce, tag) = ctx.encrypt_in_place(&mut buffer, b"")
        .expect("AES-GCM in-place encryption failed");

    ctx.decrypt_in_place(&mut buffer, &nonce, b"", &tag)
        .expect("AES-GCM in-place decryption failed");

    assert_eq!(buffer, original);
}
