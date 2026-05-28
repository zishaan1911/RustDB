use aes_gcm::{
    aead::{Aead, AeadInPlace, KeyInit, Payload},
    Aes256Gcm, Nonce,
};

use chacha20poly1305::{
    aead::Payload as ChaChaPayload,
    XChaCha20Poly1305,
    XNonce,
};

use rand::rngs::OsRng;
use rand::RngCore;

use crate::security::crypto::nonce::{
    generate_nonce,
    NonceResult,
    NonceType,
};

use std::sync::Arc;

/// AES-GCM context with cached cipher
/// Cipher is initialized once and reused
pub struct AesGcmContext
{
    cipher: Aes256Gcm,
}

impl AesGcmContext
{
    /// Creates a new AES-GCM context
    /// Key size: 32 bytes (AES-256)
    pub fn new(key: [u8; 32]) -> Self
    {
        let cipher = Aes256Gcm::new_from_slice(&key)
            .expect("AES-256 key must be 32 bytes");

        Self
        {
            cipher,
        }
    }

    /// Generates a secure random AES-256 key
    /// Returns: [u8; 32]
    pub fn generate_key() -> [u8; 32]
    {
        let mut key = [0u8; 32];

        OsRng.fill_bytes(&mut key);

        key
    }

    /// Internal nonce generator
    #[inline]
    fn generate_nonce() -> [u8; 12]
    {
        match generate_nonce(NonceType::AesGcm)
        {
            NonceResult::Bytes(bytes) =>
            {
                let mut nonce = [0u8; 12];

                nonce.copy_from_slice(&bytes);

                nonce
            }

            _ =>
            {
                panic!("Invalid AES-GCM nonce type");
            }
        }
    }
}

/// Standard allocation API
impl AesGcmContext
{
    /// Encrypts plaintext using AES-256-GCM
    ///
    /// Returns:
    /// (ciphertext, nonce, tag)
    pub fn encrypt(
        &self,
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, [u8; 12], [u8; 16]), aes_gcm::Error>
    {
        let nonce = Self::generate_nonce();

        let nonce_ref = Nonce::from_slice(&nonce);

        let payload = Payload
        {
            msg: plaintext,
            aad: &[],
        };

        let ciphertext_with_tag = self.cipher.encrypt(
            nonce_ref,
            payload,
        )?;

        let split_index = ciphertext_with_tag.len() - 16;

        let (ciphertext, tag) = ciphertext_with_tag.split_at(split_index);

        Ok((
            ciphertext.to_vec(),
            nonce,
            tag.try_into().unwrap(),
        ))
    }

    /// Decrypts AES-256-GCM ciphertext
    pub fn decrypt(
        &self,
        ciphertext: &[u8],
        nonce: &[u8; 12],
        tag: &[u8; 16],
    ) -> Result<Vec<u8>, aes_gcm::Error>
    {
        let nonce_ref = Nonce::from_slice(nonce);

        let mut combined = Vec::with_capacity(
            ciphertext.len() + 16
        );

        combined.extend_from_slice(ciphertext);
        combined.extend_from_slice(tag);

        let plaintext = self.cipher.decrypt(
            nonce_ref,
            combined.as_ref(),
        )?;

        Ok(plaintext)
    }
}

/// Zero-copy in-place API
impl AesGcmContext
{
    /// Encrypts buffer in-place
    ///
    /// Returns:
    /// (nonce, authentication_tag)
    pub fn encrypt_in_place(
        &self,
        buffer: &mut Vec<u8>,
        aad: &[u8],
    ) -> Result<([u8; 12], [u8; 16]), aes_gcm::Error>
    {
        let nonce = Self::generate_nonce();

        let nonce_ref = Nonce::from_slice(&nonce);

        let tag = self.cipher.encrypt_in_place_detached(
            nonce_ref,
            aad,
            buffer,
        )?;

        Ok((
            nonce,
            tag.into(),
        ))
    }

    /// Decrypts buffer in-place
    pub fn decrypt_in_place(
        &self,
        buffer: &mut Vec<u8>,
        nonce: &[u8; 12],
        aad: &[u8],
        tag: &[u8; 16],
    ) -> Result<(), aes_gcm::Error>
    {
        let nonce_ref = Nonce::from_slice(nonce);

        self.cipher.decrypt_in_place_detached(
            nonce_ref,
            aad,
            buffer,
            tag.into(),
        )?;

        Ok(())
    }
}

/// XChaCha20-Poly1305 context
/// Recommended for distributed systems
pub struct XChaCha20Context
{
    cipher: XChaCha20Poly1305,
}

impl XChaCha20Context
{
    /// Creates a new XChaCha20 context
    pub fn new(key: [u8; 32]) -> Self
    {
        let cipher = XChaCha20Poly1305::new_from_slice(&key)
            .expect("XChaCha20 key must be 32 bytes");

        Self
        {
            cipher,
        }
    }

    /// Generates secure random key
    pub fn generate_key() -> [u8; 32]
    {
        let mut key = [0u8; 32];

        OsRng.fill_bytes(&mut key);

        key
    }

    /// Internal nonce generator
    #[inline]
    fn generate_nonce() -> [u8; 24]
    {
        match generate_nonce(NonceType::XChaCha20)
        {
            NonceResult::Bytes(bytes) =>
            {
                let mut nonce = [0u8; 24];

                nonce.copy_from_slice(&bytes);

                nonce
            }

            _ =>
            {
                panic!("Invalid XChaCha20 nonce type");
            }
        }
    }
}

/// Standard allocation API
impl XChaCha20Context
{
    /// Encrypts plaintext using XChaCha20-Poly1305
    ///
    /// Returns:
    /// (ciphertext, nonce, tag)
    pub fn encrypt(
        &self,
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, [u8; 24], [u8; 16]), chacha20poly1305::Error>
    {
        let nonce = Self::generate_nonce();

        let nonce_ref = XNonce::from_slice(&nonce);

        let payload = ChaChaPayload
        {
            msg: plaintext,
            aad: &[],
        };

        let ciphertext_with_tag = self.cipher.encrypt(
            nonce_ref,
            payload,
        )?;

        let split_index = ciphertext_with_tag.len() - 16;

        let (ciphertext, tag) = ciphertext_with_tag.split_at(split_index);

        Ok((
            ciphertext.to_vec(),
            nonce,
            tag.try_into().unwrap(),
        ))
    }

    /// Decrypts XChaCha20 ciphertext
    pub fn decrypt(
        &self,
        ciphertext: &[u8],
        nonce: &[u8; 24],
        tag: &[u8; 16],
    ) -> Result<Vec<u8>, chacha20poly1305::Error>
    {
        let nonce_ref = XNonce::from_slice(nonce);

        let mut combined = Vec::with_capacity(
            ciphertext.len() + 16
        );

        combined.extend_from_slice(ciphertext);
        combined.extend_from_slice(tag);

        let plaintext = self.cipher.decrypt(
            nonce_ref,
            combined.as_ref(),
        )?;

        Ok(plaintext)
    }
}

/// Zero-copy in-place API
#[allow(dead_code)]
impl XChaCha20Context
{
    /// Encrypts buffer in-place
    ///
    /// Returns:
    /// (nonce, authentication_tag)
    pub fn encrypt_in_place(
        &self,
        buffer: &mut Vec<u8>,
        aad: &[u8],
    ) -> Result<([u8; 24], [u8; 16]), chacha20poly1305::Error>
    {
        let nonce = Self::generate_nonce();

        let nonce_ref = XNonce::from_slice(&nonce);

        let tag = self.cipher.encrypt_in_place_detached(
            nonce_ref,
            aad,
            buffer,
        )?;

        Ok((
            nonce,
            tag.into(),
        ))
    }

    /// Decrypts buffer in-place
    pub fn decrypt_in_place(
        &self,
        buffer: &mut Vec<u8>,
        nonce: &[u8; 24],
        aad: &[u8],
        tag: &[u8; 16],
    ) -> Result<(), chacha20poly1305::Error>
    {
        let nonce_ref = XNonce::from_slice(nonce);

        self.cipher.decrypt_in_place_detached(
            nonce_ref,
            aad,
            buffer,
            tag.into(),
        )?;

        Ok(())
    }
}

/// Streaming AES-GCM API
#[allow(dead_code)]
pub struct AesGcmStream
{
    context: Arc<AesGcmContext>,
}

#[allow(dead_code)]
impl AesGcmStream
{
    pub fn new(context: Arc<AesGcmContext>) -> Self
    {
        Self
        {
            context,
        }
    }

    pub fn encrypt_stream(
        &self,
        data: &[u8],
        chunk_size: usize,
    ) -> Result<Vec<(Vec<u8>, [u8; 12], [u8; 16])>, aes_gcm::Error>
    {
        let mut output = Vec::new();

        for chunk in data.chunks(chunk_size)
        {
            output.push(
                self.context.encrypt(chunk)?
            );
        }

        Ok(output)
    }

    pub fn decrypt_stream(
        &self,
        chunks: Vec<(Vec<u8>, [u8; 12], [u8; 16])>,
    ) -> Result<Vec<u8>, aes_gcm::Error>
    {
        let mut output = Vec::new();

        for (ciphertext, nonce, tag) in chunks
        {
            let mut plaintext = self.context.decrypt(
                &ciphertext,
                &nonce,
                &tag,
            )?;

            output.append(&mut plaintext);
        }

        Ok(output)
    }
}

/// Streaming XChaCha20 API
#[allow(dead_code)]
pub struct XChaCha20Stream
{
    context: Arc<XChaCha20Context>,
}

#[allow(dead_code)]
impl XChaCha20Stream
{
    pub fn new(context: Arc<XChaCha20Context>) -> Self
    {
        Self
        {
            context,
        }
    }

    pub fn encrypt_stream(
        &self,
        data: &[u8],
        chunk_size: usize,
    ) -> Result<Vec<(Vec<u8>, [u8; 24], [u8; 16])>, chacha20poly1305::Error>
    {
        let mut output = Vec::new();

        for chunk in data.chunks(chunk_size)
        {
            output.push(
                self.context.encrypt(chunk)?
            );
        }

        Ok(output)
    }

    pub fn decrypt_stream(
        &self,
        chunks: Vec<(Vec<u8>, [u8; 24], [u8; 16])>,
    ) -> Result<Vec<u8>, chacha20poly1305::Error>
    {
        let mut output = Vec::new();

        for (ciphertext, nonce, tag) in chunks
        {
            let mut plaintext = self.context.decrypt(
                &ciphertext,
                &nonce,
                &tag,
            )?;

            output.append(&mut plaintext);
        }

        Ok(output)
    }
}

/// Supported crypto backends
#[allow(dead_code)]
pub enum CryptoBackend
{
    AesGcm(AesGcmContext),
    XChaCha20(XChaCha20Context),
}

#[allow(dead_code)]
impl CryptoBackend
{
    /// Encrypts using selected backend
    pub fn encrypt(
        &self,
        data: &[u8],
    ) -> Vec<u8>
    {
        match self
        {
            CryptoBackend::AesGcm(context) =>
            {
                let (ciphertext, _, _) = context.encrypt(data)
                    .expect("AES-GCM encryption failed");

                ciphertext
            }

            CryptoBackend::XChaCha20(context) =>
            {
                let (ciphertext, _, _) = context.encrypt(data)
                    .expect("XChaCha20 encryption failed");

                ciphertext
            }
        }
    }
}