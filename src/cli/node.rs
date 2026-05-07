use crate::core::provider::Provider;
use crate::output::table::{print_inspection, print_nodes};
use crate::providers::ssh::{ImportMode, SshAuth, SshNode, SshNodeStore, SshProvider};
use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct NodeCommand {
    #[command(subcommand)]
    command: NodeSubcommand,
}

#[derive(Debug, Subcommand)]
enum NodeSubcommand {
    Add(NodeAddCommand),
    Import(NodeImportCommand),
    List(NodeListCommand),
    Show(NodeShowCommand),
    Remove(NodeRemoveCommand),
    Ping(NodePingCommand),
    Inspect(NodeInspectCommand),
    Exec(NodeExecCommand),
}

#[derive(Debug, Args)]
struct NodeAddCommand {
    #[command(subcommand)]
    provider: NodeAddProvider,
}

#[derive(Debug, Subcommand)]
enum NodeAddProvider {
    Ssh(SshAddArgs),
}

#[derive(Debug, Args)]
struct SshAddArgs {
    name: String,
    #[arg(long)]
    host: String,
    #[arg(long, default_value_t = 22)]
    port: u16,
    #[arg(long)]
    user: String,
    #[arg(long, value_enum)]
    auth: AuthKind,
    #[arg(long)]
    password_ref: Option<String>,
    #[arg(long)]
    password: Option<String>,
    #[arg(long)]
    key: Option<PathBuf>,
    #[arg(long)]
    cert: Option<PathBuf>,
    #[arg(long = "label", value_parser = parse_label)]
    labels: Vec<(String, String)>,
    #[arg(long)]
    replace: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum AuthKind {
    Password,
    KeyPair,
    Certificate,
}

#[derive(Debug, Args)]
struct NodeImportCommand {
    #[command(subcommand)]
    provider: NodeImportProvider,
}

#[derive(Debug, Subcommand)]
enum NodeImportProvider {
    Ssh(SshImportArgs),
}

#[derive(Debug, Args)]
struct SshImportArgs {
    file: PathBuf,
    #[arg(long, conflicts_with = "replace")]
    dry_run: bool,
    #[arg(long)]
    replace: bool,
}

#[derive(Debug, Args)]
struct NodeListCommand {
    #[command(subcommand)]
    provider: Option<NodeListProvider>,
}

#[derive(Debug, Subcommand)]
enum NodeListProvider {
    Ssh,
}

#[derive(Debug, Args)]
struct NodeShowCommand {
    #[command(subcommand)]
    provider: NodeShowProvider,
}

#[derive(Debug, Subcommand)]
enum NodeShowProvider {
    Ssh(NodeNameArgs),
}

#[derive(Debug, Args)]
struct NodeRemoveCommand {
    #[command(subcommand)]
    provider: NodeRemoveProvider,
}

#[derive(Debug, Subcommand)]
enum NodeRemoveProvider {
    Ssh(NodeNameArgs),
}

#[derive(Debug, Args)]
struct NodePingCommand {
    #[command(subcommand)]
    provider: NodePingProvider,
}

#[derive(Debug, Subcommand)]
enum NodePingProvider {
    Ssh(NodeNameArgs),
}

#[derive(Debug, Args)]
struct NodeInspectCommand {
    #[command(subcommand)]
    provider: NodeInspectProvider,
}

#[derive(Debug, Subcommand)]
enum NodeInspectProvider {
    Ssh(NodeNameArgs),
}

#[derive(Debug, Args)]
struct NodeExecCommand {
    #[command(subcommand)]
    provider: NodeExecProvider,
}

#[derive(Debug, Subcommand)]
enum NodeExecProvider {
    Ssh(SshExecArgs),
}

#[derive(Debug, Args)]
struct NodeNameArgs {
    name: String,
}

#[derive(Debug, Args)]
struct SshExecArgs {
    name: String,
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

pub fn handle(command: NodeCommand) -> Result<()> {
    match command.command {
        NodeSubcommand::Add(command) => handle_add(command),
        NodeSubcommand::Import(command) => handle_import(command),
        NodeSubcommand::List(command) => handle_list(command),
        NodeSubcommand::Show(command) => handle_show(command),
        NodeSubcommand::Remove(command) => handle_remove(command),
        NodeSubcommand::Ping(command) => handle_ping(command),
        NodeSubcommand::Inspect(command) => handle_inspect(command),
        NodeSubcommand::Exec(command) => handle_exec(command),
    }
}

fn handle_add(command: NodeAddCommand) -> Result<()> {
    match command.provider {
        NodeAddProvider::Ssh(args) => {
            let node = SshNode {
                name: args.name,
                host: args.host,
                port: args.port,
                user: args.user,
                auth: build_auth(
                    args.auth,
                    args.password_ref,
                    args.password,
                    args.key,
                    args.cert,
                )?,
                labels: args.labels.into_iter().collect::<BTreeMap<_, _>>(),
            };
            let node_name = node.name.clone();
            SshNodeStore::default()?.add(node, args.replace)?;
            println!("added SSH node {node_name}");
            Ok(())
        }
    }
}

fn handle_import(command: NodeImportCommand) -> Result<()> {
    match command.provider {
        NodeImportProvider::Ssh(args) => {
            let mode = if args.dry_run {
                ImportMode::DryRun
            } else if args.replace {
                ImportMode::Replace
            } else {
                ImportMode::Merge
            };
            let report = SshNodeStore::default()?.import_file(&args.file, mode)?;
            if args.dry_run {
                println!(
                    "dry run succeeded: {} new SSH nodes validated",
                    report.imported
                );
            } else {
                println!(
                    "imported {} SSH nodes, replaced {}, skipped {}",
                    report.imported, report.replaced, report.skipped
                );
            }
            Ok(())
        }
    }
}

fn handle_list(command: NodeListCommand) -> Result<()> {
    match command.provider.unwrap_or(NodeListProvider::Ssh) {
        NodeListProvider::Ssh => {
            let provider = ssh_provider()?;
            print_nodes(&provider.list_nodes()?);
            Ok(())
        }
    }
}

fn handle_show(command: NodeShowCommand) -> Result<()> {
    match command.provider {
        NodeShowProvider::Ssh(args) => {
            let mut node = SshNodeStore::default()?.get(&args.name)?;
            redact_auth(&mut node.auth);
            println!("{}", serde_yaml::to_string(&node)?);
            Ok(())
        }
    }
}

fn handle_remove(command: NodeRemoveCommand) -> Result<()> {
    match command.provider {
        NodeRemoveProvider::Ssh(args) => {
            SshNodeStore::default()?.remove(&args.name)?;
            println!("removed SSH node {}", args.name);
            Ok(())
        }
    }
}

fn handle_ping(command: NodePingCommand) -> Result<()> {
    match command.provider {
        NodePingProvider::Ssh(args) => {
            ssh_provider()?.ping(&args.name)?;
            println!("SSH node {} is reachable", args.name);
            Ok(())
        }
    }
}

fn handle_inspect(command: NodeInspectCommand) -> Result<()> {
    match command.provider {
        NodeInspectProvider::Ssh(args) => {
            let inspection = ssh_provider()?.inspect(&args.name)?;
            print_inspection(&inspection);
            Ok(())
        }
    }
}

fn handle_exec(command: NodeExecCommand) -> Result<()> {
    match command.provider {
        NodeExecProvider::Ssh(args) => {
            let exit_code = ssh_provider()?.exec(&args.name, &args.command)?;
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
            Ok(())
        }
    }
}

fn ssh_provider() -> Result<SshProvider> {
    Ok(SshProvider::new(SshNodeStore::default()?))
}

fn build_auth(
    auth: AuthKind,
    password_ref: Option<String>,
    password: Option<String>,
    key: Option<PathBuf>,
    cert: Option<PathBuf>,
) -> Result<SshAuth> {
    match auth {
        AuthKind::Password => Ok(SshAuth::Password {
            password_ref: password_ref.map(crate::providers::ssh::config::SecretRef::String),
            password,
        }),
        AuthKind::KeyPair => Ok(SshAuth::KeyPair {
            key_path: key.ok_or_else(|| anyhow::anyhow!("--key is required for key-pair auth"))?,
            passphrase_ref: None,
            passphrase: None,
        }),
        AuthKind::Certificate => Ok(SshAuth::Certificate {
            key_path: key
                .ok_or_else(|| anyhow::anyhow!("--key is required for certificate auth"))?,
            cert_path: cert
                .ok_or_else(|| anyhow::anyhow!("--cert is required for certificate auth"))?,
            passphrase_ref: None,
            passphrase: None,
        }),
    }
}

fn parse_label(value: &str) -> Result<(String, String), String> {
    let Some((key, value)) = value.split_once('=') else {
        return Err("labels must use key=value format".to_string());
    };
    if key.trim().is_empty() {
        return Err("label key cannot be empty".to_string());
    }
    Ok((key.to_string(), value.to_string()))
}

fn redact_auth(auth: &mut SshAuth) {
    if let SshAuth::Password { password, .. } = auth {
        if password.is_some() {
            *password = Some("<redacted>".to_string());
        }
    }
}
