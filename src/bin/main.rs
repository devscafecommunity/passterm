use clap::{Parser, Subcommand};
use passterm::vault::storage;
use std::collections::HashMap;
use std::io::{self, Write};
use std::process::Command as ProcCommand;

#[derive(Parser, Debug)]
#[command(name = "passterm")]
#[command(about = "Terminal Password Manager")]
struct Args {
    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Subcommand, Debug)]
enum CliCommand {
    Init,
    Add { id: String },
    List,
    Get { id: String },
    Env { id: String, cmd: Vec<String> },
    Delete { id: String },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Some(CliCommand::Init) => cmd_init(),
        Some(CliCommand::Add { id }) => cmd_add(id),
        Some(CliCommand::List) => cmd_list(),
        Some(CliCommand::Get { id }) => cmd_get(id),
        Some(CliCommand::Env { id, cmd }) => cmd_env(id, cmd),
        Some(CliCommand::Delete { id }) => cmd_delete(id),
        None => {
            println!("passterm v{}", env!("CARGO_PKG_VERSION"));
            println!("Run 'passterm init' to create a vault");
        }
    }
}

fn read_password(prompt: &str) -> String {
    print!("{}: ", prompt);
    io::stdout().flush().unwrap();
    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();
    password.trim().to_string()
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

fn cmd_init() {
    let password = read_password("Enter master password");
    if password.is_empty() {
        eprintln!("Password cannot be empty");
        return;
    }

    let confirm = read_password("Confirm password");
    if password != confirm {
        eprintln!("Passwords do not match");
        return;
    }

    match storage::create_vault(&password) {
        Ok(_) => println!("Vault created successfully"),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn cmd_add(id: String) {
    let password = read_password("Master password");

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

fn cmd_list() {
    let password = read_password("Master password");

    let vault = match storage::load_vault(&password) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to unlock vault: {}", e);
            return;
        }
    };

    if vault.entries.is_empty() {
        println!("No entries found");
        return;
    }

    for entry in vault.entries.keys() {
        println!("{}", entry);
    }
}

fn cmd_get(id: String) {
    let password = read_password("Master password");

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

    println!("Secrets for '{}':", id);
    for (key, value) in &entry.variables {
        println!("  {}={}", key, value);
    }
}

fn cmd_env(id: String, cmd: Vec<String>) {
    if cmd.is_empty() {
        eprintln!("Usage: passterm env <id> <command>");
        return;
    }

    let password = read_password("Master password");

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

    let mut command = ProcCommand::new(&cmd[0]);
    command.args(&cmd[1..]);

    for (key, value) in &entry.variables {
        command.env(key, value);
    }

    let status = command.status().unwrap_or_default();
    std::process::exit(status.code().unwrap_or(1));
}

fn cmd_delete(id: String) {
    let password = read_password("Master password");

    let mut vault = match storage::load_vault(&password) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to unlock vault: {}", e);
            return;
        }
    };

    if vault.remove_entry(&id).is_none() {
        eprintln!("Entry '{}' not found", id);
        return;
    }

    if let Err(e) = storage::save_vault(&vault, &password) {
        eprintln!("Failed to save vault: {}", e);
        return;
    }

    println!("Entry '{}' deleted", id);
}
