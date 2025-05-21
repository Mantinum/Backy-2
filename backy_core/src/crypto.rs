// Crypto module: AES-256-GCM encryption + Argon2 key derivation

use aes_gcm::{Aes256Gcm, KeyInit, aead::{Aead, OsRng, generic_array::GenericArray}};
use rand::RngCore;
use argon2::Argon2;
use password_hash::SaltString;

/// Encrypt data with password:
/// - derive 32-byte key via Argon2id
/// - generate random 12-byte nonce
/// - return (salt, nonce, ciphertext)
/// Errors are returned as String.
pub fn encrypt(data: &[u8], password: &str) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), String> {
    let salt = SaltString::generate(&mut OsRng);
    let salt_bytes = salt.as_bytes().to_vec();
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt.as_bytes(), &mut key)
        .map_err(|e| e.to_string())?;
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher
        .encrypt(GenericArray::from_slice(&nonce), data)
        .map_err(|e| e.to_string())?;
    Ok((salt_bytes, nonce.to_vec(), ciphertext))
}

/// Decrypt data with password, salt, nonce.
/// Errors are returned as String.
pub fn decrypt(
    salt: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
    password: &str,
) -> Result<Vec<u8>, String> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| e.to_string())?;
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
    let plaintext = cipher
        .decrypt(GenericArray::from_slice(nonce), ciphertext)
        .map_err(|e| e.to_string())?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let password = "secret";
        let data = b"Hello, Tauri!";
        let (salt, nonce, ct) = encrypt(data, password).expect("encrypt failed");
        let pt = decrypt(&salt, &nonce, &ct, password).expect("decrypt failed");
        assert_eq!(pt, data);
    }
}
