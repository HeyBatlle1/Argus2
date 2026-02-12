//! Secure vault - encrypted secret storage
//!
//! Master key is wrapped in `Zeroizing<>` so it's automatically
//! scrubbed from memory on drop. No lingering key material.

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use ring::rand::SecureRandom;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use zeroize::Zeroizing;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Vault is locked")]
    Locked,
    #[error("Secret not found: {0}")]
    NotFound(String),
    #[error("Encryption error")]
    Encryption,
    #[error("Decryption error")]
    Decryption,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Keychain error: {0}")]
    Keychain(String),
}

pub struct SecureVault {
    master_key: Option<Zeroizing<[u8; 32]>>,
    vault_path: PathBuf,
    secrets: HashMap<String, Vec<u8>>,
}

impl SecureVault {
    pub fn new(vault_path: PathBuf) -> Self {
        Self {
            master_key: None,
            vault_path,
            secrets: HashMap::new(),
        }
    }

    pub fn init(vault_path: PathBuf) -> Result<Self, VaultError> {
        let mut key = Zeroizing::new([0u8; 32]);
        ring::rand::SystemRandom::new()
            .fill(key.as_mut())
            .map_err(|_| VaultError::Encryption)?;

        // Store in keychain
        let keychain = crate::keychain::KeychainProvider::new("argus");
        keychain.store_master_key(&*key).map_err(|e| VaultError::Keychain(e.to_string()))?;

        fs::create_dir_all(vault_path.parent().unwrap_or(&vault_path))?;

        let vault = Self {
            master_key: Some(key),
            vault_path,
            secrets: HashMap::new(),
        };
        vault.save()?;
        Ok(vault)
    }

    pub fn unlock(&mut self) -> Result<(), VaultError> {
        let keychain = crate::keychain::KeychainProvider::new("argus");
        let key_vec = keychain.retrieve_master_key().map_err(|e| VaultError::Keychain(e.to_string()))?;
        let mut key = Zeroizing::new([0u8; 32]);
        key.copy_from_slice(&key_vec);
        self.master_key = Some(key);
        self.load()?;
        Ok(())
    }

    pub fn store(&mut self, name: &str, secret: &str) -> Result<(), VaultError> {
        let key = self.master_key.as_ref().ok_or(VaultError::Locked)?;
        let cipher = ChaCha20Poly1305::new_from_slice(&**key).map_err(|_| VaultError::Encryption)?;
        let mut nonce_bytes = [0u8; 12];
        ring::rand::SystemRandom::new().fill(&mut nonce_bytes).map_err(|_| VaultError::Encryption)?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, secret.as_bytes()).map_err(|_| VaultError::Encryption)?;
        
        let mut stored = nonce_bytes.to_vec();
        stored.extend(ciphertext);
        self.secrets.insert(name.to_string(), stored);
        self.save()?;
        Ok(())
    }

    pub fn retrieve(&self, name: &str) -> Result<String, VaultError> {
        let key = self.master_key.as_ref().ok_or(VaultError::Locked)?;
        let stored = self.secrets.get(name).ok_or_else(|| VaultError::NotFound(name.to_string()))?;
        let (nonce_bytes, ciphertext) = stored.split_at(12);
        let cipher = ChaCha20Poly1305::new_from_slice(&**key).map_err(|_| VaultError::Decryption)?;
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|_| VaultError::Decryption)?;
        String::from_utf8(plaintext).map_err(|_| VaultError::Decryption)
    }

    pub fn list_keys(&self) -> Vec<String> {
        self.secrets.keys().cloned().collect()
    }

    pub fn delete(&mut self, name: &str) -> Result<(), VaultError> {
        self.secrets.remove(name).ok_or_else(|| VaultError::NotFound(name.to_string()))?;
        self.save()
    }

    fn save(&self) -> Result<(), VaultError> {
        let data = serde_json::to_vec(&self.secrets).map_err(|_| VaultError::Encryption)?;
        fs::write(&self.vault_path, data)?;
        Ok(())
    }

    fn load(&mut self) -> Result<(), VaultError> {
        if self.vault_path.exists() {
            let data = fs::read(&self.vault_path)?;
            self.secrets = serde_json::from_slice(&data).map_err(|_| VaultError::Decryption)?;
        }
        Ok(())
    }
}
