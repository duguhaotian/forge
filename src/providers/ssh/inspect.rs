use crate::core::capability::NodeInspection;
use crate::providers::ssh::client::SshClient;
use anyhow::Result;

pub fn inspect_node(name: &str, client: &SshClient) -> Result<NodeInspection> {
    let script = r#"printf 'hostname=%s\n' "$(hostname 2>/dev/null)"
printf 'os=%s\n' "$(. /etc/os-release 2>/dev/null && printf '%s' "$PRETTY_NAME")"
printf 'arch=%s\n' "$(uname -m 2>/dev/null)"
printf 'cpu_cores=%s\n' "$(getconf _NPROCESSORS_ONLN 2>/dev/null || nproc 2>/dev/null)"
printf 'memory_total_mb=%s\n' "$(awk '/MemTotal/ {printf "%d", $2 / 1024}' /proc/meminfo 2>/dev/null)"
printf 'disk_total_mb=%s\n' "$(df -Pm / 2>/dev/null | awk 'NR==2 {print $2}')"
printf 'uptime=%s\n' "$(uptime -p 2>/dev/null || uptime 2>/dev/null)""#;

    let output = client.run(script)?;
    let mut inspection = NodeInspection {
        name: name.to_string(),
        ..NodeInspection::default()
    };

    for line in output.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        match key {
            "hostname" => inspection.hostname = Some(value.to_string()),
            "os" => inspection.os = Some(value.to_string()),
            "arch" => inspection.arch = Some(value.to_string()),
            "cpu_cores" => inspection.cpu_cores = value.parse().ok(),
            "memory_total_mb" => inspection.memory_total_mb = value.parse().ok(),
            "disk_total_mb" => inspection.disk_total_mb = value.parse().ok(),
            "uptime" => inspection.uptime = Some(value.to_string()),
            _ => {}
        }
    }

    Ok(inspection)
}
