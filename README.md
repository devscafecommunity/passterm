# PassTerm

Terminal password manager with encrypted vault and environment variable injection.

## Features

- End-to-end encrypted vault (AES-256-GCM + Argon2id)
- Secure environment variable injection for commands
- Interactive CLI with secret management

## Installation

```bash
cargo install --git https://github.com/devscafecommunity/passterm.git
```

Or build from source:

```bash
cargo build --release
./target/release/passterm
```

## Usage

### Initialize vault

```bash
passterm init
```

Enter and confirm your master password.

### Add secrets

```bash
passterm add <env-id>
```

Example:
```bash
passterm add vercel-prod
Key: VERCEL_API_TOKEN
Value: xxxxx
Key: (empty to finish)
```

### List all environments

```bash
passterm list
```

### Get secrets

```bash
passterm get <env-id>
```

### Run command with environment variables

```bash
passterm env <env-id> <command>
```

Example:
```bash
passterm env vercel-prod npm run deploy
```

This injects all stored variables into the command process without writing them to disk or shell history.

### Delete environment

```bash
passterm delete <env-id>
```

## Security

- AES-256-GCM authenticated encryption
- Argon2id key derivation (resistant to GPU/ASIC attacks)
- Secrets stored encrypted at rest
- Memory cleared on process exit

## License

MIT OR Apache-2.0