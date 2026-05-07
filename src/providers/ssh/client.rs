use crate::providers::ssh::config::{SecretRef, SshAuth, SshNode};
use anyhow::{anyhow, bail, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SshClient {
    node: SshNode,
}

impl SshClient {
    pub fn new(node: SshNode) -> Self {
        Self { node }
    }

    pub fn run(&self, remote_command: &str) -> Result<String> {
        let mut command = self.base_command()?;
        command.arg(remote_command);
        let output = command.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            bail!("SSH command failed on {}: {stderr}", self.node.name);
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn run_args(&self, args: &[String]) -> Result<i32> {
        if args.is_empty() {
            bail!("remote command is required");
        }
        let remote_command = shell_join(args);
        let mut command = self.base_command()?;
        command.arg(remote_command);
        let status = command.status()?;
        Ok(status.code().unwrap_or(1))
    }

    fn base_command(&self) -> Result<Command> {
        match &self.node.auth {
            SshAuth::Password {
                password_ref,
                password,
            } => self.password_command(password_ref.as_ref(), password.as_deref()),
            SshAuth::KeyPair { key_path, .. } => {
                let mut command = self.ssh_command();
                command.arg("-i").arg(expand_tilde(key_path));
                Ok(command)
            }
            SshAuth::Certificate {
                key_path,
                cert_path,
                ..
            } => {
                let mut command = self.ssh_command();
                command.arg("-i").arg(expand_tilde(key_path));
                command.arg("-o").arg(format!(
                    "CertificateFile={}",
                    expand_tilde(cert_path).display()
                ));
                Ok(command)
            }
            SshAuth::Agent { identities_only } => {
                let mut command = self.ssh_command();
                if *identities_only {
                    command.arg("-o").arg("IdentitiesOnly=yes");
                }
                Ok(command)
            }
            SshAuth::OpenSshConfig {
                host_alias,
                config_path,
            } => {
                let mut command = Command::new("ssh");
                if let Some(config_path) = config_path {
                    command.arg("-F").arg(expand_tilde(config_path));
                }
                command.arg(host_alias.as_deref().unwrap_or(&self.node.host));
                Ok(command)
            }
            other => bail!(
                "SSH auth type '{}' is modeled but not implemented yet",
                other.type_name()
            ),
        }
    }

    fn ssh_command(&self) -> Command {
        let mut command = Command::new("ssh");
        command.arg("-p").arg(self.node.port.to_string());
        command.arg("-o").arg("ConnectTimeout=10");
        command.arg("-o").arg("StrictHostKeyChecking=accept-new");
        command.arg(format!("{}@{}", self.node.user, self.node.host));
        command
    }

    fn password_command(
        &self,
        password_ref: Option<&SecretRef>,
        plain_password: Option<&str>,
    ) -> Result<Command> {
        if !command_exists("sshpass") {
            bail!("password auth requires sshpass to be installed");
        }

        let password = resolve_secret(password_ref, plain_password)?;
        let mut command = Command::new("sshpass");
        command.arg("-e");
        command.env("SSHPASS", password);
        command.arg("ssh");
        command.arg("-p").arg(self.node.port.to_string());
        command.arg("-o").arg("ConnectTimeout=10");
        command.arg("-o").arg("StrictHostKeyChecking=accept-new");
        command.arg("-o").arg("PreferredAuthentications=password");
        command.arg("-o").arg("PubkeyAuthentication=no");
        command.arg(format!("{}@{}", self.node.user, self.node.host));
        Ok(command)
    }
}

fn resolve_secret(
    password_ref: Option<&SecretRef>,
    plain_password: Option<&str>,
) -> Result<String> {
    if let Some(password) = plain_password {
        return Ok(password.to_string());
    }

    let Some(secret_ref) = password_ref else {
        bail!("password auth requires password_ref or password");
    };

    let value = secret_ref.value();
    if let Some(env_name) = value.strip_prefix("env:") {
        return std::env::var(env_name)
            .map_err(|_| anyhow!("environment variable is not set: {env_name}"));
    }
    if value == "prompt" {
        eprint!("SSH password: ");
        let mut password = String::new();
        std::io::stdin().read_line(&mut password)?;
        return Ok(password.trim_end_matches(['\r', '\n']).to_string());
    }
    if let Some(password) = value.strip_prefix("plain:") {
        return Ok(password.to_string());
    }
    Ok(value.to_string())
}

fn expand_tilde(path: &Path) -> PathBuf {
    let path_string = path.to_string_lossy();
    if path_string == "~" {
        return home_dir().unwrap_or_else(|| path.to_path_buf());
    }
    if let Some(rest) = path_string.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }
    path.to_path_buf()
}

fn command_exists(name: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|path| path.join(name).is_file())
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn shell_join(args: &[String]) -> String {
    args.iter()
        .map(|arg| shell_quote(arg))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || "_+-./:=,@%".contains(ch))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}
