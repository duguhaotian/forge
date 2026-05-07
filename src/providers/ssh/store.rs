use crate::providers::ssh::config::{ImportMode, ImportReport, SshNode, SshNodeFile};
use anyhow::{anyhow, bail, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SshNodeStore {
    path: PathBuf,
}

impl SshNodeStore {
    pub fn default() -> Result<Self> {
        let home = home_dir()?;
        Ok(Self {
            path: home.join(".forge/providers/ssh/nodes.yaml"),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<Vec<SshNode>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let file = SshNodeFile::read(&self.path)?;
        file.validate()?;
        Ok(file.nodes)
    }

    pub fn save(&self, nodes: &[SshNode]) -> Result<()> {
        let file = SshNodeFile {
            nodes: nodes.to_vec(),
        };
        file.validate()?;
        file.write_yaml(&self.path)
    }

    pub fn add(&self, node: SshNode, replace: bool) -> Result<()> {
        node.validate()?;
        let mut nodes = self.load()?;
        if let Some(index) = nodes.iter().position(|existing| existing.name == node.name) {
            if !replace {
                bail!("SSH node already exists: {}", node.name);
            }
            nodes[index] = node;
        } else {
            nodes.push(node);
        }
        self.save(&nodes)
    }

    pub fn remove(&self, name: &str) -> Result<()> {
        let mut nodes = self.load()?;
        let before = nodes.len();
        nodes.retain(|node| node.name != name);
        if before == nodes.len() {
            bail!("SSH node not found: {name}");
        }
        self.save(&nodes)
    }

    pub fn get(&self, name: &str) -> Result<SshNode> {
        self.load()?
            .into_iter()
            .find(|node| node.name == name)
            .ok_or_else(|| anyhow!("SSH node not found: {name}"))
    }

    pub fn import_file(&self, path: &Path, mode: ImportMode) -> Result<ImportReport> {
        let incoming_file = SshNodeFile::read(path)?;
        incoming_file.validate()?;

        let existing_nodes = self.load()?;
        let mut nodes_by_name = existing_nodes
            .into_iter()
            .map(|node| (node.name.clone(), node))
            .collect::<BTreeMap<_, _>>();

        let mut report = ImportReport {
            imported: 0,
            replaced: 0,
            skipped: 0,
        };

        for node in incoming_file.nodes {
            if nodes_by_name.contains_key(&node.name) {
                match mode {
                    ImportMode::Replace => {
                        nodes_by_name.insert(node.name.clone(), node);
                        report.replaced += 1;
                    }
                    ImportMode::Merge | ImportMode::DryRun => {
                        bail!("SSH node already exists: {}", node.name);
                    }
                }
            } else {
                nodes_by_name.insert(node.name.clone(), node);
                report.imported += 1;
            }
        }

        if mode != ImportMode::DryRun {
            let nodes = nodes_by_name.into_values().collect::<Vec<_>>();
            self.save(&nodes)?;
        }

        Ok(report)
    }
}

fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("failed to locate home directory from HOME"))
}
