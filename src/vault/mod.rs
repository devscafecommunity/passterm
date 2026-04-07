use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod crypto;
pub mod storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub id: String,
    pub folder: Option<String>,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub entries: HashMap<String, VaultEntry>,
    pub folders: Vec<String>,
}

impl Default for Vault {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            folders: Vec::new(),
        }
    }
}

impl Vault {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_entry(
        &mut self,
        id: String,
        folder: Option<String>,
        variables: HashMap<String, String>,
    ) {
        let key = if let Some(ref f) = folder {
            format!("{}/{}", f, id)
        } else {
            id.clone()
        };
        self.entries.insert(
            key,
            VaultEntry {
                id,
                folder,
                variables,
            },
        );
    }

    pub fn get_entry(&self, id: &str) -> Option<&VaultEntry> {
        self.entries.get(id)
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<VaultEntry> {
        self.entries.remove(id)
    }

    pub fn add_folder(&mut self, name: String) {
        if !self.folders.contains(&name) {
            self.folders.push(name);
        }
    }

    pub fn remove_folder(&mut self, name: &str) {
        self.folders.retain(|f| f != name);
        for entry in self.entries.values_mut() {
            if entry.folder.as_deref() == Some(name) {
                entry.folder = None;
            }
        }
    }

    pub fn get_entries_in_folder(&self, folder: Option<&str>) -> Vec<&VaultEntry> {
        self.entries
            .values()
            .filter(|e| e.folder.as_deref() == folder)
            .collect()
    }
}
