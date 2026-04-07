/// Vault entry for storing password credentials
#[derive(Debug, Clone)]
pub struct VaultEntry {
    pub id: String,
    pub password: String,
}

/// Password vault management
pub struct Vault;

impl Vault {
    pub fn new() -> Self {
        Vault
    }
}
