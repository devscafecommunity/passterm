# PassTerm Design Document

**Date**: 2026-04-07
**Status**: Approved

## Overview

Terminal password manager with encrypted vault and environment variable injection for developer workflows.

## Architecture Layers

```
┌─────────────────────────────────────┐
│           TUI (Ratatui)             │  ← Layer 3 (optional)
├─────────────────────────────────────┤
│       CLI ( clap + commands )        │  ← Layer 2
├─────────────────────────────────────┤
│    Vault Engine (crypto + storage)     │  ← Layer 1 (core)
└─────────────────────────────────────┘
```

## Layer 1: Vault Engine

### Encryption

- **Algorithm**: AES-256-GCM (authenticated encryption)
- **Key Derivation**: Argon2id (64MB RAM, 3 iterations, 4 parallel threads)
- **Memory Safety**: zeroize crate for secure memory clearing

### Vault File Format** (binary)

| Offset | Size | Field |
|--------|------|-------|
| 0 | 16 bytes | Salt (for Argon2id) |
| 16 | 12 bytes | Nonce (for AES-GCM) |
| 28+ | N bytes | Ciphertext + Auth Tag |

### Encrypted Data Structure (JSON)

```json
{
  "entries": {
    "vercel-prod": {
      "VERCEL_API_TOKEN": "xxx",
      "DATABASE_URL": "postgres://..."
    },
    "db-prod": {
      "POSTGRES_PASSWORD": "xxx"
    }
  }
}
```

### Storage Locations

- Vault: `~/.config/passterm/vault`
- Lock file: `~/.config/passterm/vault.lock`

## Layer 2: CLI Commands

| Command | Description |
|---------|-------------|
| `passterm init` | Create vault with master password |
| `passterm add <id>` | Add new secret entry |
| `passterm list` | List all entry IDs |
| `passterm get <id>` | Show secrets for entry (with confirmation) |
| `passterm env <id> <cmd>` | Execute command with injected envs |
| `passterm delete <id>` | Delete entry |
| `passterm edit <id>` | Edit entry |
| `passterm lock` | Lock vault (clear memory) |
| `passterm unlock` | Unlock vault |

## Layer 3: TUI (Future)

- Ratatui-based terminal UI
- Arrow key navigation
- Fuzzy search (fzf-style)
- Mouse support for selection

## Sync Model

Git-based vault export/import:
- `passterm export` → Print encrypted vault to stdout (pipe to git)
- `passterm import` → Import from stdin

## Security Requirements

1. **No plaintext on disk** - All secrets always encrypted
2. **Memory clearing** - Use zeroize to wipe secrets from RAM
3. **No swap exposure** - Use mlock where possible
4. **No env logging** - Command output does not inherit secrets

## Platform

- Linux (primary)
- macOS (compatible)
- Windows: Out of scope for MVP

## Implementation Priority

1. Vault engine (crypto + storage)
2. CLI commands
3. TUI (optional enhancement)