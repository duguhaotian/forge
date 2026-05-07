use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeInspection {
    pub name: String,
    pub hostname: Option<String>,
    pub os: Option<String>,
    pub arch: Option<String>,
    pub cpu_cores: Option<u32>,
    pub memory_total_mb: Option<u64>,
    pub disk_total_mb: Option<u64>,
    pub uptime: Option<String>,
}
