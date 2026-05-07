use crate::core::capability::NodeInspection;
use crate::core::node::NodeSummary;

pub fn print_nodes(nodes: &[NodeSummary]) {
    let mut rows = vec![vec![
        "NAME".to_string(),
        "PROVIDER".to_string(),
        "HOST".to_string(),
        "USER".to_string(),
        "PORT".to_string(),
        "LABELS".to_string(),
    ]];

    for node in nodes {
        let labels = node
            .labels
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(",");
        rows.push(vec![
            node.name.clone(),
            node.provider.clone(),
            node.host.clone(),
            node.user.clone(),
            node.port.to_string(),
            labels,
        ]);
    }

    print_rows(&rows);
}

pub fn print_inspection(inspection: &NodeInspection) {
    let mut rows = vec![vec!["FIELD".to_string(), "VALUE".to_string()]];
    add_field(&mut rows, "name", inspection.name.clone());
    add_field(
        &mut rows,
        "hostname",
        inspection.hostname.clone().unwrap_or_default(),
    );
    add_field(&mut rows, "os", inspection.os.clone().unwrap_or_default());
    add_field(
        &mut rows,
        "arch",
        inspection.arch.clone().unwrap_or_default(),
    );
    add_field(
        &mut rows,
        "cpu_cores",
        inspection
            .cpu_cores
            .map(|value| value.to_string())
            .unwrap_or_default(),
    );
    add_field(
        &mut rows,
        "memory_total_mb",
        inspection
            .memory_total_mb
            .map(|value| value.to_string())
            .unwrap_or_default(),
    );
    add_field(
        &mut rows,
        "disk_total_mb",
        inspection
            .disk_total_mb
            .map(|value| value.to_string())
            .unwrap_or_default(),
    );
    add_field(
        &mut rows,
        "uptime",
        inspection.uptime.clone().unwrap_or_default(),
    );
    print_rows(&rows);
}

fn add_field(rows: &mut Vec<Vec<String>>, name: &str, value: String) {
    rows.push(vec![name.to_string(), value]);
}

fn print_rows(rows: &[Vec<String>]) {
    if rows.is_empty() {
        return;
    }

    let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
    let mut widths = vec![0; columns];
    for row in rows {
        for (index, value) in row.iter().enumerate() {
            widths[index] = widths[index].max(value.len());
        }
    }

    for row in rows {
        for (index, value) in row.iter().enumerate() {
            if index > 0 {
                print!("  ");
            }
            print!("{value:<width$}", width = widths[index]);
        }
        println!();
    }
}
