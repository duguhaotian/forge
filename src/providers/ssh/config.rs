use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshNodeFile {
    #[serde(default)]
    pub nodes: Vec<SshNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshNode {
    pub name: String,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub user: String,
    pub auth: SshAuth,
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SshAuth {
    Password {
        #[serde(default)]
        password_ref: Option<SecretRef>,
        #[serde(default)]
        password: Option<String>,
    },
    KeyPair {
        key_path: PathBuf,
        #[serde(default)]
        passphrase_ref: Option<SecretRef>,
        #[serde(default)]
        passphrase: Option<String>,
    },
    Certificate {
        key_path: PathBuf,
        cert_path: PathBuf,
        #[serde(default)]
        passphrase_ref: Option<SecretRef>,
        #[serde(default)]
        passphrase: Option<String>,
    },
    Agent {
        #[serde(default)]
        identities_only: bool,
    },
    KeyboardInteractive,
    GssApi {
        #[serde(default)]
        delegate_credentials: bool,
    },
    HostBased,
    None,
    OpenSshConfig {
        #[serde(default)]
        host_alias: Option<String>,
        #[serde(default)]
        config_path: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SecretRef {
    String(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    Merge,
    Replace,
    DryRun,
}

#[derive(Debug, Clone)]
pub struct ImportReport {
    pub imported: usize,
    pub replaced: usize,
    pub skipped: usize,
}

impl SshNodeFile {
    pub fn read(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|error| anyhow!("failed to read {}: {error}", path.display()))?;
        match path.extension().and_then(|extension| extension.to_str()) {
            Some("json") => Ok(serde_json::from_str(&content)?),
            Some("yaml") | Some("yml") => Ok(serde_yaml::from_str(&content)?),
            Some(extension) => bail!("unsupported SSH node file extension: {extension}"),
            None => bail!("SSH node file must use .yaml, .yml, or .json"),
        }
    }

    pub fn write_yaml(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        let mut names = std::collections::BTreeSet::new();
        for node in &self.nodes {
            node.validate()?;
            if !names.insert(node.name.as_str()) {
                bail!("duplicate SSH node name in import file: {}", node.name);
            }
        }
        Ok(())
    }
}

impl SshNode {
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            bail!("SSH node name is required");
        }
        if self.host.trim().is_empty() {
            bail!("SSH node host is required for {}", self.name);
        }
        if self.user.trim().is_empty() {
            bail!("SSH node user is required for {}", self.name);
        }
        self.auth.validate(&self.name)
    }
}

impl SshAuth {
    pub fn validate(&self, node_name: &str) -> Result<()> {
        match self {
            SshAuth::Password {
                password_ref,
                password,
            } => {
                if password_ref.is_none() && password.is_none() {
                    bail!("password auth requires password_ref or password for {node_name}");
                }
            }
            SshAuth::KeyPair { key_path, .. } => {
                if key_path.as_os_str().is_empty() {
                    bail!("key_pair auth requires key_path for {node_name}");
                }
            }
            SshAuth::Certificate {
                key_path,
                cert_path,
                ..
            } => {
                if key_path.as_os_str().is_empty() {
                    bail!("certificate auth requires key_path for {node_name}");
                }
                if cert_path.as_os_str().is_empty() {
                    bail!("certificate auth requires cert_path for {node_name}");
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            SshAuth::Password { .. } => "password",
            SshAuth::KeyPair { .. } => "key_pair",
            SshAuth::Certificate { .. } => "certificate",
            SshAuth::Agent { .. } => "agent",
            SshAuth::KeyboardInteractive => "keyboard_interactive",
            SshAuth::GssApi { .. } => "gssapi",
            SshAuth::HostBased => "hostbased",
            SshAuth::None => "none",
            SshAuth::OpenSshConfig { .. } => "openssh_config",
        }
    }
}

impl SecretRef {
    pub fn value(&self) -> &str {
        match self {
            SecretRef::String(value) => value,
        }
    }
}

pub fn default_port() -> u16 {
    22
}
