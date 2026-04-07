use passterm::vault::Vault;
use std::collections::HashMap;

#[test]
fn test_vault_new() {
    let vault = Vault::new();
    assert!(vault.entries.is_empty());
}

#[test]
fn test_vault_add_get() {
    let mut vault = Vault::new();
    let mut vars = HashMap::new();
    vars.insert("KEY".to_string(), "value".to_string());
    vault.add_entry("test".to_string(), vars);

    let entry = vault.get_entry("test");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().variables.get("KEY").unwrap(), "value");
}

#[test]
fn test_vault_remove() {
    let mut vault = Vault::new();
    let mut vars = HashMap::new();
    vars.insert("KEY".to_string(), "value".to_string());
    vault.add_entry("test".to_string(), vars);

    let removed = vault.remove_entry("test");
    assert!(removed.is_some());
    assert!(vault.get_entry("test").is_none());
}
