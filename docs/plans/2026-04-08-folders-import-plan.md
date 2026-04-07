# PassTerm Folders & Import Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add folder organization and .env file import to PassTerm

**Architecture:** Add folder support to VaultEntry, enhance TUI with folder navigation, add import/export commands

**Tech Stack:** Rust, ratatui, crossterm

---

## Task 1: Update Vault Data Structures

**Files:**
- Modify: `src/vault/mod.rs`

**Step 1: Update VaultEntry structure**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub id: String,
    pub folder: Option<String>,  // None = root
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub entries: HashMap<String, VaultEntry>,
    pub folders: Vec<String>,  // List of folder names
}
```

**Step 2: Add folder methods to Vault**

```rust
impl Vault {
    pub fn add_folder(&mut self, name: String) {
        if !self.folders.contains(&name) {
            self.folders.push(name);
        }
    }

    pub fn remove_folder(&mut self, name: &str) {
        self.folders.retain(|f| f != name);
        // Move entries in folder to root
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
```

**Step 3: Run build to verify**

Run: `cargo build`
Expected: PASS

**Step 4: Commit**

```bash
git add src/vault/mod.rs
git commit -m "feat: add folder support to vault"
```

---

## Task 2: Update TUI Data Structures

**Files:**
- Modify: `src/tui.rs`

**Step 1: Add folder state to App**

```rust
pub struct App {
    pub current_folder: Option<String>,
    pub show_folders: bool,
    pub selected_folder: usize,
    // ... existing fields
}
```

**Step 2: Update unlock to load folders**

```rust
pub fn unlock(&mut self) -> bool {
    match storage::load_vault(&self.password) {
        Ok(v) => {
            self.entries = v.entries.keys().cloned().collect();
            self.folders = v.folders.clone();
            self.vault = Some(v);
            true
        }
        Err(_) => false,
    }
}
```

**Step 3: Run build**

Run: `cargo build`
Expected: PASS

**Step 4: Commit**

```bash
git add src/tui.rs
git commit -m "feat: add folder state to TUI"
```

---

## Task 3: Add TUI Folder Navigation

**Files:**
- Modify: `src/tui.rs` - handle_key and ui functions

**Step 1: Add Tab key handling for folders**

```rust
KeyCode::Tab => {
    if app.vault.is_some() {
        app.show_folders = !app.show_folders;
        if app.show_folders {
            app.selected = 0;
        }
    }
}
```

**Step 2: Update UI to show folders**

```rust
fn ui(f: &mut Frame, app: &mut App) {
    // If showing folders, render folder list first
    if app.show_folders && !app.folders.is_empty() {
        let folder_items: Vec<ListItem> = app
            .folders
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let marker = if i == app.selected_folder { ">" } else { " " };
                ListItem::new(format!("{} 📁 {}", marker, f))
            })
            .collect();
        // render folder list
    }
    // ... existing entry list
}
```

**Step 3: Add folder creation key**

```rust
KeyCode::Char('f') if app.vault.is_some() && app.show_folders => {
    // Enable folder name input mode
}
```

**Step 4: Build and test**

Run: `cargo build`
Expected: PASS

**Step 5: Commit**

```bash
git commit -m "feat: add folder navigation to TUI"
```

---

## Task 4: Add .env Import

**Files:**
- Modify: `src/vault/storage.rs`
- Modify: `src/tui.rs`

**Step 1: Add .env parsing to storage**

```rust
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
```

**Step 2: Add import to TUI**

```rust
KeyCode::Char('i') if app.vault.is_some() && !app.adding_entry => {
    // Show file picker or prompt for path
}
```

**Step 3: Run build**

Run: `cargo build`
Expected: PASS

**Step 4: Commit**

```bash
git commit -m "feat: add .env import"
```

---

## Task 5: Final Integration & Testing

**Step 1: Test full flow**

```bash
# Build release
cargo build --release

# Test TUI
./target/release/passterm

# Create folder f + name
# Add entry a + name
# Import env i + path
# Navigate Tab between folders/entries
```

**Step 2: Commit final**

```bash
git commit -m "feat: complete folders and import"
git push origin main
```

---

## Implementation Complete

**Summary:**
- Task 1: Vault data structures with folders ✅
- Task 2: TUI state updates ✅
- Task 3: Folder navigation in TUI ✅
- Task 4: .env import ✅

**Plan complete.**