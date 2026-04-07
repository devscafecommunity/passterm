use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        self.entries
            .insert(id.clone(), VaultEntry { id, variables });
    }

    pub fn get_entry(&self, id: &str) -> Option<&VaultEntry> {
        self.entries.get(id)
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<VaultEntry> {
        self.entries.remove(id)
    }
}
