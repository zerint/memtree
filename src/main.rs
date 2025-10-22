use std::collections::{HashMap, HashSet};
use std::fs;

#[derive(Debug, Clone)]
struct ProcessInfo {
    name: String,
    memory_mb: f64,
    ppid: u32,
    cmdline: String,
    total_memory_mb: f64,
}

fn read_process_stat(pid: u32) -> Option<(String, u32)> {
    let stat_path = format!("/proc/{}/stat", pid);
    let stat_content = fs::read_to_string(stat_path).ok()?;

    // Parse stat file: pid (name) state ppid ...
    // Name can contain spaces and is enclosed in parentheses
    let start = stat_content.find('(')?;
    let end = stat_content.rfind(')')?;
    let name = stat_content[start + 1..end].to_string();

    // Get ppid (4th field after the closing parenthesis)
    let after_name = &stat_content[end + 2..]; // Skip ") "
    let fields: Vec<&str> = after_name.split_whitespace().collect();
    let ppid = fields.get(1)?.parse::<u32>().ok()?;

    Some((name, ppid))
}

fn read_process_memory(pid: u32) -> Option<f64> {
    // Read from statm: size resident shared text lib data dt
    let statm_path = format!("/proc/{}/statm", pid);
    let statm_content = fs::read_to_string(statm_path).ok()?;

    let fields: Vec<&str> = statm_content.split_whitespace().collect();
    let rss_pages = fields.get(1)?.parse::<u64>().ok()?;

    // Convert pages to MB (page size is typically 4096 bytes)
    let page_size = 4096u64;
    let memory_bytes = rss_pages * page_size;
    let memory_mb = (memory_bytes as f64) / (1024.0 * 1024.0);
    let memory_mb_rounded = (memory_mb * 100.0).round() / 100.0;

    Some(memory_mb_rounded)
}

fn read_process_cmdline(pid: u32) -> String {
    let cmdline_path = format!("/proc/{}/cmdline", pid);

    match fs::read_to_string(cmdline_path) {
        Ok(content) => {
            if content.is_empty() {
                "[No Command]".to_string()
            } else {
                // cmdline uses null bytes as separators
                content.replace('\0', " ").trim().to_string()
            }
        }
        Err(_) => "[No Command]".to_string(),
    }
}

fn get_all_pids() -> Vec<u32> {
    let mut pids = Vec::new();

    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if let Ok(pid) = file_name.parse::<u32>() {
                    pids.push(pid);
                }
            }
        }
    }

    pids
}

fn get_process_tree() {
    // Maps: ppid -> list of child pids
    let mut processes: HashMap<u32, Vec<u32>> = HashMap::new();
    // Maps: pid -> ProcessInfo
    let mut process_info: HashMap<u32, ProcessInfo> = HashMap::new();

    // Collect all processes
    for pid in get_all_pids() {
        // Try to read process information
        let (name, ppid) = match read_process_stat(pid) {
            Some(data) => data,
            None => continue, // Process might have exited or we don't have permission
        };

        let memory_mb = match read_process_memory(pid) {
            Some(mem) => mem,
            None => continue,
        };

        let cmdline = read_process_cmdline(pid);

        let info = ProcessInfo {
            name,
            memory_mb,
            ppid,
            cmdline,
            total_memory_mb: memory_mb,
        };

        process_info.insert(pid, info);
        processes.entry(ppid).or_insert_with(Vec::new).push(pid);
    }

    // Calculate total memory recursively
    fn calculate_total_memory(
        pid: u32,
        processes: &HashMap<u32, Vec<u32>>,
        process_info: &mut HashMap<u32, ProcessInfo>,
        visited: &mut HashSet<u32>,
    ) -> f64 {
        if visited.contains(&pid) || !process_info.contains_key(&pid) {
            return 0.0;
        }

        visited.insert(pid);
        let mut total = process_info[&pid].memory_mb;

        if let Some(children) = processes.get(&pid) {
            for &child_pid in children {
                total += calculate_total_memory(child_pid, processes, process_info, visited);
            }
        }

        if let Some(info) = process_info.get_mut(&pid) {
            info.total_memory_mb = (total * 100.0).round() / 100.0;
        }

        total
    }

    // Sort processes by total memory
    fn sort_processes(
        processes: &mut HashMap<u32, Vec<u32>>,
        process_info: &HashMap<u32, ProcessInfo>,
    ) {
        for children in processes.values_mut() {
            children.sort_by(|a, b| {
                let a_mem = process_info.get(a).map(|p| p.total_memory_mb).unwrap_or(0.0);
                let b_mem = process_info.get(b).map(|p| p.total_memory_mb).unwrap_or(0.0);
                b_mem.partial_cmp(&a_mem).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }

    // Print tree recursively
    fn print_tree(
        pid: u32,
        level: usize,
        processes: &HashMap<u32, Vec<u32>>,
        process_info: &HashMap<u32, ProcessInfo>,
        visited: &mut HashSet<u32>,
    ) {
        if visited.contains(&pid) {
            return;
        }
        visited.insert(pid);

        if let Some(info) = process_info.get(&pid) {
            let indent = "  ".repeat(level);
            println!(
                "{}{} MB - {} (PID: {}, Memory: {:.2} MB, CMD: {})",
                indent, info.total_memory_mb, info.name, pid, info.memory_mb, info.cmdline
            );

            if let Some(children) = processes.get(&pid) {
                for &child_pid in children {
                    print_tree(child_pid, level + 1, processes, process_info, visited);
                }
            }
        }
    }

    // Start from PID 1 (systemd/init)
    let mut visited = HashSet::new();
    calculate_total_memory(1, &processes, &mut process_info, &mut visited);

    sort_processes(&mut processes, &process_info);

    let mut visited = HashSet::new();
    print_tree(1, 0, &processes, &process_info, &mut visited);
}

fn main() {
    get_process_tree();
}
