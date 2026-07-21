//! Stub of upstream `xai-grok-shell::mcp_doctor`.

use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct DoctorServerStatus {
    pub name: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DoctorReport {
    pub ok: bool,
    pub messages: Vec<String>,
    pub servers: Vec<DoctorServerStatus>,
    pub all_server_names: Vec<String>,
    pub failing_count: usize,
}

pub fn run(_cwd: &std::path::Path) -> DoctorReport {
    DoctorReport::default()
}

pub async fn run_doctor(cwd: &std::path::Path, _name: Option<&str>) -> DoctorReport {
    run(cwd)
}

pub fn print_report(report: &DoctorReport) {
    for msg in &report.messages {
        println!("{msg}");
    }
    for s in &report.servers {
        println!("{}: {}", s.name, if s.ok { "ok" } else { &s.message });
    }
    if report.ok {
        println!("MCP doctor: ok");
    } else {
        println!("MCP doctor: issues found");
    }
}
