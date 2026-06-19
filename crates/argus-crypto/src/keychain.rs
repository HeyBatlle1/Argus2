//! Hardware keychain integration

use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeychainError {
    #[error("Keychain not available")]
    NotAvailable,
    #[error("Item not found")]
    NotFound,
    #[error("Platform error: {0}")]
    Platform(String),
}

pub struct KeychainProvider {
    service_name: String,
}

impl KeychainProvider {
    pub fn new(service_name: &str) -> Self {
        Self { service_name: service_name.to_string() }
    }

    pub fn store_master_key(&self, key: &[u8]) -> Result<(), KeychainError> {
        let entry = keyring::Entry::new(&self.service_name, "master_key")
            .map_err(|e| KeychainError::Platform(e.to_string()))?;
        let encoded: String = key.iter().map(|b| format!("{:02x}", b)).collect();
        entry.set_password(&encoded).map_err(|e| KeychainError::Platform(e.to_string()))
    }

    pub fn retrieve_master_key(&self) -> Result<Vec<u8>, KeychainError> {
        let entry = keyring::Entry::new(&self.service_name, "master_key")
            .map_err(|e| KeychainError::Platform(e.to_string()))?;
        let encoded = entry.get_password().map_err(|e| match e {
            keyring::Error::NoEntry => KeychainError::NotFound,
            _ => KeychainError::Platform(e.to_string()),
        })?;
        if encoded.len() % 2 != 0 {
            return Err(KeychainError::Platform("Corrupted keychain entry: odd-length hex string".to_string()));
        }
        (0..encoded.len()).step_by(2)
            .map(|i| u8::from_str_radix(&encoded[i..i+2], 16).map_err(|e| KeychainError::Platform(e.to_string())))
            .collect()
    }
}
