use crate::core::capability::NodeInspection;
use crate::core::node::NodeSummary;
use anyhow::Result;

pub trait Provider {
    fn name(&self) -> &'static str;
    fn list_nodes(&self) -> Result<Vec<NodeSummary>>;
    fn ping(&self, name: &str) -> Result<()>;
    fn inspect(&self, name: &str) -> Result<NodeInspection>;
}
