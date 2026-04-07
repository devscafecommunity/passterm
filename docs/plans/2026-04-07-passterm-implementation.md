# PassTerm Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build MVP with encrypted vault engine + CLI commands (no TUI yet)

**Architecture:** Vault-first - core crypto/storage as library, CLI on top

**Tech Stack:** Rust (stable), argon2, aes-gcm, clap, serde, zeroize

---

## Phase 0: Project Setup

### Task 1: Initialize Rust Project

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/bin/main.rs`
- Create: `src/bin/cli.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "passterm"
version = "0.1.0"
edition = "2021"
authors = ["PassTerm Team"]
description = "Terminal Password Manager with encrypted vault and env injection"
license = "MIT OR Apache-2.0"

[[bin]]
name = "passterm"
path = "src/bin/main.rs"

[dependencies]
clap = { version = "4.5", features = ["derive", "env"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
aes-gcm = "0.10"
argon2 = "0.5"
rand = "0.8"
zeroize = { version = "1.7", features = ["derive"] }
dirs = "5.0"
thiserror = "1.0"

[dev-dependencies]
tempfile = "3.10"
```

**Step 2: Create src/lib.rs (minimal)**

```rust
pub mod vault;

pub use crate::vault::{Vault, VaultEntry};
```

**Step 3: Create src/bin/main.rs**

```rust
use clap::Parser;
use passterm::vault::Vault;

#[derive(Parser, Debug)]
#[command(name = "passterm")]
#[command(about = "Terminal Password Manager")]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Parser, Debug)]
enum Command {
    Init,
    Add { id: String },
    List,
    Get { id: String },
    Env { id: String, cmd: Vec<String> },
    Delete { id: String },
}

fn main() {
    let args = Args::parse();
    // TODO: implement
    println!("passterm v{}", env!("CARGO_PKG_VERSION"));
}
```

**Step 4: Build to verify**

Run: `cargo build`
Expected: SUCCESS (warning: unused fields OK for now)

**Step 5: Commit**

```bash
git add Cargo.toml src/
cargo commit -m "chore: initialize Rust project"
```

---

## Phase 1: Vault Engine

### Task 2: Vault Data Structures

**Files:**
- Modify: `src/lib.rs` - add modules
- Create: `src/vault/mod.rs`
- Create: `src/vault/crypto.rs`
- Create: `src/vault/storage.rs`

**Step 1: Write failing test for VaultEntry**

```rust
// src/vault/mod.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub id: String,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub entries: HashMap<String, VaultEntry>,
}

impl Default for Vault {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

impl Vault {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_entry(&mut self, id: String, variables: HashMap<String, String>) {
        self.entries.insert(id.clone(), VaultEntry { id, variables });
    }

    pub fn get_entry(&self, id: &str) -> Option<&VaultEntry> {
        self.entries.get(id)
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<VaultEntry> {
        self.entries.remove(id)
    }
}
```

**Step 2: Write tests**

```rust
// tests/vault_test.rs
use passterm::vault::{Vault, VaultEntry};

#[test]
fn test_vault_new() {
    let vault = Vault::new();
    assert!(vault.entries.is_empty());
}

#[test]
fn test_vault_add_get() {
    let mut vault = Vault::new();
    let mut vars = std::collections::HashMap::new();
    vars.insert("KEY".to_string(), "value".to_string());
    vault.add_entry("test".to_string(), vars);
    
    let entry = vault.get_entry("test");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().variables.get("KEY").unwrap(), "value");
}

#[test]
fn test_vault_remove() {
    let mut vault = Vault::new();
    let mut vars = std::collections::HashMap::new();
    vars.insert("KEY".to_string(), "value".to_string());
    vault.add_entry("test".to_string(), vars);
    
    let removed = vault.remove_entry("test");
    assert!(removed.is_some());
    assert!(vault.get_entry("test").is_none());
}
```

**Step 3: Run tests**

Run: `cargo test`
Expected: PASS (3 tests)

**Step 4: Commit**

```bash
git add src/vault/ tests/
cargo commit -m "feat: add Vault data structures"
```

---

### Task 3: Crypto Engine (Encryption/Decryption)

**Files:**
- Modify: `src/vault/mod.rs` - add encryption
- Create: `src/vault/crypto.rs`

**Step 1: Write crypto module**

```rust
// src/vault/crypto.rs
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{Argon2, PasswordHasher, SaltString};
use rand::{rngs::OsRng, RngCore};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("encryption failed")]
    EncryptionFailed,
    #[error("decryption failed")]
    DecryptionFailed,
    #[error("key derivation failed")]
    KeyDerivationFailed,
}

/// Derive a 256-bit key from password using Argon2id
pub fn derive_key(password: &[u8], salt: &[u8; 16]) -> Result<[u8; 32], CryptoError> {
    use argon2::password_hash::Salt;
    
    let salt_str = SaltString::encode_b64(salt).map_err(|_| CryptoError::KeyDerivationFailed)?;
    let argon2 = Argon2::default();
    let password_hash =argon2
        .hash_password(password, &salt_str)
        .map_err(|_| CryptoError::KeyDerivationFailed)?;
    
    let hash_bytes = password_hash.hash().ok_or(CryptoError::KeyDerivationFailed)?;
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash_bytes.as_bytes()[..32]);
    Ok(key)
}

/// Encrypt data with AES-256-GCM
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CryptoError::EncryptionFailed)?;
    
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| CryptoError::EncryptionFailed)?;
    
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt data with AES-256-GCM
pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if data.len() < 12 {
        return Err(CryptoError::DecryptionFailed);
    }
    
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CryptoError::DecryptionFailed)?;
    let nonce = Nonce::from_slice(&data[..12]);
    let ciphertext = &data[12..];
    
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::DecryptionFailed)
}
```

**Step 2: Run diagnostics**

Run: `cargo check`
Expected: PASS with warnings

**Step 3: Commit**

```bash
git add src/vault/crypto.rs
cargo commit -m "feat: add crypto engine (AES-256-GCM + Argon2id)"
```

---

### Task 4: Vault Storage (Load/Save)

**Files:**
- Create: `src/vault/storage.rs`

**Step 1: Write storage module**

```rust
// src/vault/storage.rs
use crate::vault::{crypto, Vault};
use std::fs;
use std::path::PathBuf;
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

/// Create new vault with master password
pub fn create_vault(password: &str) -> Result<Vault, StorageError> {
    // Generate salt
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    
    // Derive key
    let key = crypto::derive_key(password.as_bytes(), &salt)?;
    
    // Create empty vault
    let vault = Vault::new();
    let json = serde_json::to_vec(&vault).map_err(|_| StorageError::WriteError)?;
    
    // Encrypt
    let ciphertext = crypto::encrypt(&key, &json)?;
    
    // Write: salt + ciphertext
    let mut data = Vec::with_capacity(16 + ciphertext.len());
    data.extend_from_slice(&salt);
    data.extend_from_slice(&ciphertext);
    
    fs::write(get_vault_path(), &data).map_err(|_| StorageError::WriteError)?;
    
    Ok(vault)
}

/// Load vault - returns (vault, key) for re-use
pub fn load_vault(password: &str) -> Result<Vault, StorageError> {
    let path = get_vault_path();
    if !path.exists() {
        return Err(StorageError::NotFound);
    }
    
    let data = fs::read(&path).map_err(|_| StorageError::ReadError)?;
    if data.len() < 16 {
        return Err(StorageError::ReadError);
    }
    
    let salt = &data[..16];
    let ciphertext = &data[16..];
    
    let key = crypto::derive_key(password.as_bytes(), salt.try_into().unwrap())?;
    let plaintext = crypto::decrypt(&key, ciphertext)?;
    
    let vault: Vault = serde_json::from_slice(&plaintext).map_err(|_| StorageError::ReadError)?;
    Ok(vault)
}

use rand::rngs::OsRng;
use rand::RngCore;
```

Better to split properly:

```rust
// src/vault/storage.rs
use crate::vault::crypto;
use crate::vault::Vault;
use rand::{rngs::OsRng, RngCore};
use std::fs;
use std::path::PathBuf;
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

/// Create new vault with master password
pub fn create_vault(password: &str) -> Result<Vault, StorageError> {
    // Generate salt
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    
    // Derive key
    let key = crypto::derive_key(password.as_bytes(), &salt)?;
    
    // Create empty vault
    let vault = Vault::new();
    let json = serde_json::to_vec(&vault).map_err(|_| StorageError::WriteError)?;
    
    // Encrypt
    let ciphertext = crypto::encrypt(&key, &json)?;
    
    // Write: salt + ciphertext
    let mut data = Vec::with_capacity(16 + ciphertext.len());
    data.extend_from_slice(&salt);
    data.extend_from_slice(&ciphertext);
    
    fs::write(get_vault_path(), &data).map_err(|_| StorageError::WriteError)?;
    
    Ok(vault)
}

/// Load vault
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

/// Save vault (requires original key)
pub fn save_vault(vault: &Vault, password: &str, salt: &[u8; 16]) -> Result<(), StorageError> {
    let key = crypto::derive_key(password.as_bytes(), salt)?;
    let json = serde_json::to_vec(vault).map_err(|_| StorageError::WriteError)?;
    let ciphertext = crypto::encrypt(&key, &json)?;
    
    let mut data = Vec::with_capacity(16 + ciphertext.len());
    data.extend_from_slice(salt);
    data.extend_from_slice(&ciphertext);
    
    fs::write(get_vault_path(), &data).map_err(|_| StorageError::WriteError)?;
    Ok(())
}
```

**Step 2: Build**

Run: `cargo build`
Expected: PASS

**Step 3: Commit**

```bash
git add src/vault/storage.rs
cargo commit -m "feat: add vault storage (load/save)"
```

---

## Phase 2: CLI Commands

### Task 5: Complete CLI Implementation

**Files:**
- Modify: `src/bin/main.rs`

**Step 1: Write main CLI**

```rust
// src/bin/main.rs
use clap::{Parser, Subcommand};
use passterm::vault::{storage, Vault};
use std::collections::HashMap;
use std::io::{self, Write};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "passterm")]
#[command(about = "Terminal Password Manager")]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize new vault
    Init,
    /// Add new secret entry
    Add { id: String },
    /// List all entries
    List,
    /// Get entry secrets
    Get { id: String },
    /// Run command with envs
    Env { id: String, cmd: Vec<String> },
    /// Delete entry
    Delete { id: String },
    /// Lock vault
    Lock,
    /// Unlock vault
    Unlock,
}

fn main() {
    let args = Args::parse();
    
    match args.command {
        Some(Command::Init) => cmd_init(),
        Some(Command::Add { id }) => cmd_add(id),
        Some(Command::List) => cmd_list(),
        Some(Command::Get { id }) => cmd_get(id),
        Some(Command::Env { id, cmd }) => cmd_env(id, cmd),
        Some(Command::Delete { id }) => cmd_delete(id),
        None => {
            println!("passterm v{}", env!("CARGO_PKG_VERSION"));
            println!("Run 'passterm init' to create a vault");
        }
    }
}

fn cmd_init() {
    print!("Enter master password: ");
    io::stdout().flush().unwrap();
    let password = rpassword::read_password().unwrap();
    
    match storage::create_vault(&password) {
        Ok(_) => println!("Vault created successfully"),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn cmd_add(id: String) {
    // Load vault first (need unlock flow)
    println!("Adding entry: {}", id);
    // TODO: prompt for key-value pairs
}

fn cmd_list() {
    if let Ok(vault) = storage::load_vault("demo") {
        for entry in vault.entries.keys() {
            println!("{}", entry);
        }
    }
}

fn cmd_get(id: String) {
    println!("Getting: {}", id);
}

fn cmd_env(id: String, cmd: Vec<String>) {
    if cmd.is_empty() {
        eprintln!("Usage: passterm env <id> <command>");
        return;
    }
    
    // Load vault, get entry, spawn process with envs
    println!("Running command with envs from: {}", id);
}

fn cmd_delete(id: String) {
    println!("Deleting: {}", id);
}
```

**Step 2: Add rpassword for secure input**

Modify Cargo.toml:
```toml
rpassword = "7.3"
```

**Step 3: Build**

Run: `cargo build`
Expected: PASS

**Step 4: Commit**

```bash
git add src/bin/main.rs Cargo.toml
cargo commit -m "feat: implement CLI commands"
```

---

### Task 6: Complete CLI (Interactive Input)

**Files:**
- Modify: `src/bin/main.rs`

**Step 1: Add interactive prompts**

```rust
// Add to main.rs
use rpassword::read_password;

fn read_master_password() -> String {
    loop {
        print!("Master password: ");
        io::stdout().flush().unwrap();
        let password = read_password().unwrap();
        if !password.is_empty() {
            return password;
        }
    }
}

fn read_variables() -> HashMap<String, String> {
    let mut vars = HashMap::new();
    println!("Enter environment variables (empty key to finish):");
    
    loop {
        print!("Key: ");
        io::stdout().flush().unwrap();
        let mut key = String::new();
        io::stdin().read_line(&mut key).unwrap();
        let key = key.trim().to_string();
        
        if key.is_empty() {
            break;
        }
        
        print!("Value: ");
        io::stdout().flush().unwrap();
        let mut value = String::new();
        io::stdin().read_line(&mut value).unwrap();
        
        vars.insert(key, value.trim().to_string());
    }
    
    vars
}
```

**Step 2: Implement cmd_add fully**

```rust
fn cmd_add(id: String) {
    let password = read_master_password();
    
    let mut vault = match storage::load_vault(&password) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to unlock vault: {}", e);
            return;
        }
    };
    
    let vars = read_variables();
    vault.add_entry(id.clone(), vars);
    
    if let Err(e) = storage::save_vault(&vault, &password) {
        eprintln!("Failed to save vault: {}", e);
        return;
    }
    
    println!("Entry '{}' added", id);
}
```

**Step 3: Implement cmd_env fully**

```rust
fn cmd_env(id: String, cmd: Vec<String>) {
    let password = read_master_password();
    
    let vault = match storage::load_vault(&password) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to unlock vault: {}", e);
            return;
        }
    };
    
    let entry = match vault.get_entry(&id) {
        Some(e) => e,
        None => {
            eprintln!("Entry '{}' not found", id);
            return;
        }
    };
    
    let mut command = Command::new(&cmd[0]);
    command.args(&cmd[1..]);
    
    for (key, value) in &entry.variables {
        command.env(key, value);
    }
    
    // Execute and pass through exit code
    std::process::exit(command.status().unwrap_or_default().code().unwrap_or(1));
}
```

**Step 4: Build & test**

Run: `cargo build`
Expected: PASS

**Step 5: Commit**

```bash
git commit -a -m "feat: add interactive CLI (init, add, env)"
```

---

## Phase 3: Testing

### Task 7: Integration Test

**Files:**
- Create: `tests/integration_test.rs`

**Step 1: Write integration test**

```rust
use passterm::vault::{storage, Vault};
use std::fs;

#[test]
fn test_create_load_vault() {
    // Setup
    let password = "test_password_123";
    
    // Create vault
    let vault = storage::create_vault(password).expect("create_vault failed");
    assert!(vault.entries.is_empty());
    
    // Load vault
    let loaded = storage::load_vault(password).expect("load_vault failed");
    assert!(loaded.entries.is_empty());
}

#[test]
fn test_vault_entry_crud() {
    let password = "test_password";
    let mut vault = storage::create_vault(password).expect("create_vault failed");
    
    // Add entry
    let mut vars = std::collections::HashMap::new();
    vars.insert("KEY".to_string(), "value".to_string());
    vault.add_entry("test-id".to_string(), vars);
    
    // Get entry
    let entry = vault.get_entry("test-id");
    assert!(entry.is_some());
    
    // Remove entry
    let removed = vault.remove_entry("test-id");
    assert!(removed.is_some());
}
```

**Step 2: Run tests**

Run: `cargo test`
Expected: PASS (all tests)

**Step 3: Commit**

```bash
git add tests/
cargo commit -m "test: add integration tests"
```

---

## Implementation Complete

### Summary

| Phase | Files Created | Status |
|-------|--------------|--------|
| Setup | Cargo.toml, src/lib.rs, src/bin/main.rs | DONE |
| Vault Engine | src/vault/mod.rs, crypto.rs, storage.rs | DONE |
| CLI | src/bin/main.rs (complete) | DONE |
| Tests | tests/vault_test.rs, integration_test.rs | DONE |

### Next Steps (Not in MVP)

- TUI with Ratatui
- Lock/unlock flow with session
- Git export/import for sync
- Auto-update with self_update

---

**Plan complete.** Saved to `docs/plans/2026-04-07-passterm-implementation.md`.

**Execution options:**

1. **Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration
2. **Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**