use crate::core::capability::NodeInspection;
use crate::core::node::NodeSummary;
use crate::core::provider::Provider;
use crate::providers::ssh::client::SshClient;
use crate::providers::ssh::inspect::inspect_node;
use crate::providers::ssh::store::SshNodeStore;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct SshProvider {
    store: SshNodeStore,
}

impl SshProvider {
    pub fn new(store: SshNodeStore) -> Self {
        Self { store }
    }

    pub fn exec(&self, name: &str, args: &[String]) -> Result<i32> {
        let node = self.store.get(name)?;
        SshClient::new(node).run_args(args)
    }
}

impl Provider for SshProvider {
    fn name(&self) -> &'static str {
        "ssh"
    }

    fn list_nodes(&self) -> Result<Vec<NodeSummary>> {
        Ok(self
            .store
            .load()?
            .into_iter()
            .map(|node| NodeSummary {
                name: node.name,
                provider: "ssh".to_string(),
                host: node.host,
                user: node.user,
                port: node.port,
                labels: node.labels,
            })
            .collect())
    }

    fn ping(&self, name: &str) -> Result<()> {
        let node = self.store.get(name)?;
        SshClient::new(node).run("true")?;
        Ok(())
    }

    fn inspect(&self, name: &str) -> Result<NodeInspection> {
        let node = self.store.get(name)?;
        inspect_node(name, &SshClient::new(node))
    }
}
