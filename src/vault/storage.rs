use crate::vault::crypto;
use crate::vault::Vault;
use rand::{rngs::OsRng, RngCore};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("vault not found")]
    NotFound,
    #[error("failed to read vault")]
    ReadError,
    #[error("failed to write vault")]
    WriteError,
    #[error("crypto error: {0}")]
    Crypto(#[from] crypto::CryptoError),
}

pub fn get_vault_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("passterm");
    fs::create_dir_all(&config_dir).ok();
    config_dir.join("vault")
}

pub fn create_vault(password: &str) -> Result<Vault, StorageError> {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let key = crypto::derive_key(password.as_bytes(), &salt)?;

    let vault = Vault::new();
    let json = serde_json::to_vec(&vault).map_err(|_| StorageError::WriteError)?;

    let ciphertext = crypto::encrypt(&key, &json)?;

    let mut data = Vec::with_capacity(16 + ciphertext.len());
    data.extend_from_slice(&salt);
    data.extend_from_slice(&ciphertext);

    fs::write(get_vault_path(), &data).map_err(|_| StorageError::WriteError)?;

    Ok(vault)
}

pub fn load_vault(password: &str) -> Result<Vault, StorageError> {
    let path = get_vault_path();
    if !path.exists() {
        return Err(StorageError::NotFound);
    }

    let data = fs::read(&path).map_err(|_| StorageError::ReadError)?;
    if data.len() < 16 {
        return Err(StorageError::ReadError);
    }

    let salt: [u8; 16] = data[..16].try_into().unwrap();
    let ciphertext = &data[16..];

    let key = crypto::derive_key(password.as_bytes(), &salt)?;
    let plaintext = crypto::decrypt(&key, ciphertext)?;

    let vault: Vault = serde_json::from_slice(&plaintext).map_err(|_| StorageError::ReadError)?;
    Ok(vault)
}

pub fn save_vault(vault: &Vault, password: &str) -> Result<(), StorageError> {
    let path = get_vault_path();
    if !path.exists() {
        return Err(StorageError::NotFound);
    }

    let data = fs::read(&path).map_err(|_| StorageError::ReadError)?;
    if data.len() < 16 {
        return Err(StorageError::ReadError);
    }

    let salt: [u8; 16] = data[..16].try_into().unwrap();

    let key = crypto::derive_key(password.as_bytes(), &salt)?;
    let json = serde_json::to_vec(vault).map_err(|_| StorageError::WriteError)?;
    let ciphertext = crypto::encrypt(&key, &json)?;

    let mut data = Vec::with_capacity(16 + ciphertext.len());
    data.extend_from_slice(&salt);
    data.extend_from_slice(&ciphertext);

    fs::write(get_vault_path(), &data).map_err(|_| StorageError::WriteError)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_create_load_vault() {
        let password = "test_password_123";
        let vault = create_vault(password).expect("create_vault failed");
        assert!(vault.entries.is_empty());

        let loaded = load_vault(password).expect("load_vault failed");
        assert!(loaded.entries.is_empty());
    }
}

pub fn parse_env_file(path: &Path) -> Result<HashMap<String, String>, StorageError> {
    let content = fs::read_to_string(path).map_err(|_| StorageError::ReadError)?;
    let mut vars = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            vars.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    Ok(vars)
}
