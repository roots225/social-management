use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rand::RngCore;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Erreur de clé invalide : {0}")]
    InvalidKey(String),

    #[error("Erreur de chiffrement : {0}")]
    EncryptionFailed(String),

    #[error("Erreur de déchiffrement : {0}")]
    DecryptionFailed(String),

    #[error("Format de base64 invalide : {0}")]
    Base64Decode(#[from] base64::DecodeError),
}

#[derive(Clone)]
pub struct CryptoService {
    cipher: Arc<Aes256Gcm>,
}

impl CryptoService {
    /// Initialise le service de chiffrement avec une clé de 32 octets.
    /// Accepte soit une chaîne hexadécimale de 64 caractères, soit une chaîne brute de 32 caractères.
    pub fn new(key_str: &str) -> Result<Self, CryptoError> {
        let key_bytes: Vec<u8> = if key_str.len() == 64 && key_str.chars().all(|c| c.is_ascii_hexdigit()) {
            hex::decode(key_str).map_err(|e| CryptoError::InvalidKey(e.to_string()))?
        } else if key_str.len() == 32 {
            key_str.as_bytes().to_vec()
        } else {
            return Err(CryptoError::InvalidKey(
                "La MASTER_KEY doit contenir exactement 32 octets (chaîne de 32 caractères ou 64 caractères hexadécimaux)".into()
            ));
        };

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        Ok(Self {
            cipher: Arc::new(cipher),
        })
    }

    /// Chiffre un texte clair en AES-256-GCM avec un Nonce de 96 bits (12 octets) aléatoire.
    /// Retourne la chaîne encodée en base64 : [nonce 12 octets] + [ciphertext + tag 16 octets].
    pub fn encrypt(&self, plaintext: &str) -> Result<String, CryptoError> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        let mut payload = Vec::with_capacity(12 + ciphertext.len());
        payload.extend_from_slice(&nonce_bytes);
        payload.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(payload))
    }

    /// Déchiffre une chaîne encodée en base64 produite par `encrypt`.
    pub fn decrypt(&self, encrypted_b64: &str) -> Result<String, CryptoError> {
        let payload = BASE64.decode(encrypted_b64)?;
        if payload.len() < 12 + 16 {
            return Err(CryptoError::DecryptionFailed(
                "Charge utile chiffrée trop courte".into(),
            ));
        }

        let (nonce_bytes, ciphertext) = payload.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let decrypted_bytes = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

        String::from_utf8(decrypted_bytes)
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
    }
}

/// Masque une chaîne secrète pour l'exposition API (ex: "****abcd")
pub fn mask_secret(secret: &str) -> String {
    if secret.is_empty() {
        return String::new();
    }
    if secret.len() <= 4 {
        return "****".to_string();
    }
    let suffix = &secret[secret.len().saturating_sub(4)..];
    format!("****{}", suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = "0123456789abcdef0123456789abcdef";
        let service = CryptoService::new(key).unwrap();

        let original = "my-super-secret-twitter-token-12345";
        let encrypted = service.encrypt(original).unwrap();
        assert_ne!(original, encrypted);

        let decrypted = service.decrypt(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_mask_secret() {
        assert_eq!(mask_secret("abcdefghij"), "****ghij");
        assert_eq!(mask_secret("123"), "****");
    }
}
