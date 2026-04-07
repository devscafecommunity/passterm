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
    println!("passterm v{}", env!("CARGO_PKG_VERSION"));
}
