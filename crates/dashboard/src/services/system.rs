//! System metrics — CPU, memory, disk usage via /proc.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemInfo {
    pub cpu_percent: f32,
    pub mem_percent: f32,
    pub used_mem: u64,
    pub total_mem: u64,
}

impl SystemInfo {
    pub fn refresh(&mut self) {
        let (cpu, mem_used, mem_total) = read_proc_stats();
        self.cpu_percent = cpu;
        self.used_mem = mem_used;
        self.total_mem = mem_total;
        if mem_total > 0 {
            self.mem_percent = mem_used as f32 / mem_total as f32 * 100.0;
        }
    }

    pub fn disk_info(&self) -> String {
        let out = std::process::Command::new("df").args(["-h", "/"]).output();
        match out {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let lines: Vec<&str> = stdout.lines().collect();
                if lines.len() >= 2 {
                    let parts: Vec<&str> = lines[1].split_whitespace().collect();
                    if parts.len() >= 6 {
                        return format!("{} / {}", parts[2], parts[1]);
                    }
                }
                "error reading disk".into()
            }
            _ => "unavailable".into(),
        }
    }
}

fn read_proc_stats() -> (f32, u64, u64) {
    let cpu_percent = read_cpu_percent();
    let (used, total) = read_mem_info();
    (cpu_percent, used, total)
}

fn read_cpu_percent() -> f32 {
    thread_local! {
        static CPU_STATE: std::cell::RefCell<(u64, u64)> = const { std::cell::RefCell::new((0, 0)) };
    }

    let raw = std::fs::read_to_string("/proc/stat").unwrap_or_default();
    let line = raw.lines().next().unwrap_or("");
    let parts: Vec<u64> = line
        .split_whitespace()
        .filter_map(|p| p.parse().ok())
        .collect();

    if parts.len() < 5 {
        return 0.0;
    }

    let idle = parts[3];
    let total: u64 = parts.iter().sum();

    CPU_STATE.with(|state| {
        let (prev_total, prev_idle) = {
            let s = state.borrow();
            (s.0, s.1)
        };

        *state.borrow_mut() = (total, idle);

        compute_cpu_diff(total, idle, prev_total, prev_idle)
    })
}

fn compute_cpu_diff(total: u64, idle: u64, prev_total: u64, prev_idle: u64) -> f32 {
    if prev_total == 0 && prev_idle == 0 {
        return 0.0;
    }

    let total_diff = total as i64 - prev_total as i64;
    let idle_diff = idle as i64 - prev_idle as i64;

    if total_diff <= 0 {
        return 0.0;
    }

    let usage = (1.0 - (idle_diff as f32 / total_diff as f32)) * 100.0;
    usage.clamp(0.0, 100.0)
}

fn read_mem_info() -> (u64, u64) {
    let raw = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut mem_total = 0u64;
    let mut mem_available = 0u64;

    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            mem_total = rest.trim().trim_end_matches(" kB").parse().unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            mem_available = rest.trim().trim_end_matches(" kB").parse().unwrap_or(0);
        }
    }

    let used = if mem_total > mem_available && mem_total > 0 {
        mem_total - mem_available
    } else {
        0
    };
    (used, mem_total)
}
