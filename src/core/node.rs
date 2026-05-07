use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSummary {
    pub name: String,
    pub provider: String,
    pub host: String,
    pub user: String,
    pub port: u16,
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
}
