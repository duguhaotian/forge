use crate::cli::node::NodeCommand;
use crate::core::provider::Provider;
use crate::providers::ssh::SshNodeStore;
use crate::providers::ssh::SshProvider;
use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};

pub mod node;

#[derive(Debug, Parser)]
#[command(name = "forge")]
#[command(version)]
#[command(about = "Agent-oriented compute broker and workspace runtime")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    Node(NodeCommand),
    Provider(ProviderCommand),
    Lease(PlaceholderCommand),
    Workspace(PlaceholderCommand),
}

#[derive(Debug, Args)]
struct ProviderCommand {
    #[command(subcommand)]
    command: ProviderSubcommand,
}

#[derive(Debug, Subcommand)]
enum ProviderSubcommand {
    List,
}

#[derive(Debug, Args)]
struct PlaceholderCommand {
    #[command(subcommand)]
    command: Option<PlaceholderSubcommand>,
}

#[derive(Debug, Subcommand)]
enum PlaceholderSubcommand {
    List,
    Create,
    Release,
    Remove,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init => init(),
        Command::Node(command) => node::handle(command),
        Command::Provider(command) => handle_provider(command),
        Command::Lease(_) => not_implemented("lease"),
        Command::Workspace(_) => not_implemented("workspace"),
    }
}

fn init() -> Result<()> {
    let store = SshNodeStore::default()?;
    if let Some(parent) = store.path().parent() {
        std::fs::create_dir_all(parent)?;
    }

    let forge_dir = store
        .path()
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .ok_or_else(|| anyhow!("SSH store path should live under ~/.forge"))?;
    std::fs::create_dir_all(forge_dir)?;

    let config_path = forge_dir.join("config.yaml");
    if !config_path.exists() {
        std::fs::write(
            &config_path,
            "version: 1\nproviders:\n  ssh:\n    enabled: true\n",
        )?;
    }

    if !store.path().exists() {
        store.save(&[])?;
    }

    println!("initialized Forge at {}", forge_dir.display());
    Ok(())
}

fn handle_provider(command: ProviderCommand) -> Result<()> {
    match command.command {
        ProviderSubcommand::List => {
            let ssh = SshProvider::new(SshNodeStore::default()?);
            println!("{}", ssh.name());
            Ok(())
        }
    }
}

fn not_implemented(name: &str) -> Result<()> {
    println!("{name} commands are not implemented yet");
    Ok(())
}
