use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Read, Write};

#[derive(Debug, Clone)]
struct ProcessInfo {
    name: String,
    memory_mb: f64,
    #[allow(dead_code)]
    ppid: u32,
    cmdline: String,
    total_memory_mb: f64,
}

struct TreeNode {
    pid: u32,
    level: usize,
    has_children: bool,
}

struct AppState {
    processes: HashMap<u32, Vec<u32>>,
    process_info: HashMap<u32, ProcessInfo>,
    expanded: HashSet<u32>,
    visible_nodes: Vec<TreeNode>,
    selected_index: usize,
}

fn read_process_stat(pid: u32) -> Option<(String, u32)> {
    let stat_path = format!("/proc/{}/stat", pid);
    let stat_content = fs::read_to_string(stat_path).ok()?;

    let start = stat_content.find('(')?;
    let end = stat_content.rfind(')')?;
    let name = stat_content[start + 1..end].to_string();

    let after_name = &stat_content[end + 2..];
    let fields: Vec<&str> = after_name.split_whitespace().collect();
    let ppid = fields.get(1)?.parse::<u32>().ok()?;

    Some((name, ppid))
}

fn read_process_memory(pid: u32) -> Option<f64> {
    let statm_path = format!("/proc/{}/statm", pid);
    let statm_content = fs::read_to_string(statm_path).ok()?;

    let fields: Vec<&str> = statm_content.split_whitespace().collect();
    let rss_pages = fields.get(1)?.parse::<u64>().ok()?;

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

impl AppState {
    fn new() -> Self {
        let mut processes: HashMap<u32, Vec<u32>> = HashMap::new();
        let mut process_info: HashMap<u32, ProcessInfo> = HashMap::new();

        // Collect all processes
        for pid in get_all_pids() {
            let (name, ppid) = match read_process_stat(pid) {
                Some(data) => data,
                None => continue,
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

        // Calculate total memory
        let mut visited = HashSet::new();
        calculate_total_memory(1, &processes, &mut process_info, &mut visited);

        sort_processes(&mut processes, &process_info);

        // Start with root expanded
        let mut expanded = HashSet::new();
        expanded.insert(1);

        let mut state = AppState {
            processes,
            process_info,
            expanded,
            visible_nodes: Vec::new(),
            selected_index: 0,
        };

        state.rebuild_visible_nodes();
        state
    }

    fn rebuild_visible_nodes(&mut self) {
        self.visible_nodes.clear();
        self.build_visible_nodes_recursive(1, 0, &mut HashSet::new());
    }

    fn build_visible_nodes_recursive(&mut self, pid: u32, level: usize, visited: &mut HashSet<u32>) {
        if visited.contains(&pid) || !self.process_info.contains_key(&pid) {
            return;
        }

        visited.insert(pid);

        let has_children = self.processes.get(&pid).map(|c| !c.is_empty()).unwrap_or(false);

        self.visible_nodes.push(TreeNode {
            pid,
            level,
            has_children,
        });

        // Only show children if this node is expanded
        if self.expanded.contains(&pid) {
            if let Some(children) = self.processes.get(&pid) {
                // Clone to avoid borrow checker issues
                let children_copy: Vec<u32> = children.clone();
                for &child_pid in &children_copy {
                    self.build_visible_nodes_recursive(child_pid, level + 1, visited);
                }
            }
        }
    }

    fn toggle_selected(&mut self) {
        if let Some(node) = self.visible_nodes.get(self.selected_index) {
            if node.has_children {
                let pid = node.pid;
                if self.expanded.contains(&pid) {
                    self.expanded.remove(&pid);
                } else {
                    self.expanded.insert(pid);
                }
                self.rebuild_visible_nodes();
            }
        }
    }

    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    fn move_down(&mut self) {
        if self.selected_index < self.visible_nodes.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    fn render(&self) {
        // Clear screen
        print!("\x1b[2J\x1b[H");

        println!("MemTree - Interactive Process Memory Viewer");
        println!("Arrow keys: navigate | Enter/Space: expand/collapse | q: quit\n");

        for (index, node) in self.visible_nodes.iter().enumerate() {
            if let Some(info) = self.process_info.get(&node.pid) {
                let indent = "  ".repeat(node.level);
                let is_selected = index == self.selected_index;
                let selector = if is_selected { "> " } else { "  " };

                let expand_indicator = if node.has_children {
                    if self.expanded.contains(&node.pid) {
                        "[-] "
                    } else {
                        "[+] "
                    }
                } else {
                    "    "
                };

                // Truncate cmdline if too long
                let max_cmd_len = 60;
                let cmd_display = if info.cmdline.len() > max_cmd_len {
                    format!("{}...", &info.cmdline[..max_cmd_len])
                } else {
                    info.cmdline.clone()
                };

                let line = format!(
                    "{}{}{}{} MB - {} (PID: {}, Memory: {:.2} MB, CMD: {})",
                    selector,
                    indent,
                    expand_indicator,
                    info.total_memory_mb,
                    info.name,
                    node.pid,
                    info.memory_mb,
                    cmd_display
                );

                if is_selected {
                    println!("\x1b[7m{}\x1b[0m", line); // Invert colors for selected line
                } else {
                    println!("{}", line);
                }
            }
        }

        io::stdout().flush().unwrap();
    }
}

// Terminal raw mode handling using direct libc FFI
#[cfg(unix)]
mod termios {
    use std::os::raw::{c_int, c_uint, c_uchar};

    pub const NCCS: usize = 32;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct Termios {
        pub c_iflag: c_uint,
        pub c_oflag: c_uint,
        pub c_cflag: c_uint,
        pub c_lflag: c_uint,
        pub c_line: c_uchar,
        pub c_cc: [c_uchar; NCCS],
        pub c_ispeed: c_uint,
        pub c_ospeed: c_uint,
    }

    pub const ECHO: c_uint = 0o000010;
    pub const ICANON: c_uint = 0o000002;
    pub const VMIN: usize = 6;
    pub const VTIME: usize = 5;
    pub const TCSANOW: c_int = 0;

    extern "C" {
        pub fn tcgetattr(fd: c_int, termios_p: *mut Termios) -> c_int;
        pub fn tcsetattr(fd: c_int, optional_actions: c_int, termios_p: *const Termios) -> c_int;
    }
}

#[cfg(unix)]
static mut ORIGINAL_TERMIOS: Option<termios::Termios> = None;

#[cfg(unix)]
fn enable_raw_mode() -> io::Result<()> {
    use std::os::unix::io::AsRawFd;

    unsafe {
        let mut termios: termios::Termios = std::mem::zeroed();

        if termios::tcgetattr(io::stdin().as_raw_fd(), &mut termios) != 0 {
            return Err(io::Error::last_os_error());
        }

        // Save original
        ORIGINAL_TERMIOS = Some(termios.clone());

        // Disable canonical mode and echo
        termios.c_lflag &= !(termios::ICANON | termios::ECHO);
        termios.c_cc[termios::VMIN] = 1;
        termios.c_cc[termios::VTIME] = 0;

        if termios::tcsetattr(io::stdin().as_raw_fd(), termios::TCSANOW, &termios) != 0 {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(())
}

#[cfg(unix)]
fn disable_raw_mode() -> io::Result<()> {
    use std::os::unix::io::AsRawFd;

    unsafe {
        if let Some(original) = ORIGINAL_TERMIOS {
            if termios::tcsetattr(io::stdin().as_raw_fd(), termios::TCSANOW, &original) != 0 {
                return Err(io::Error::last_os_error());
            }
        }
    }

    Ok(())
}

fn run_interactive() -> io::Result<()> {
    let mut state = AppState::new();

    enable_raw_mode()?;

    // Hide cursor
    print!("\x1b[?25l");
    io::stdout().flush()?;

    state.render();

    let stdin = io::stdin();
    let mut buffer = [0u8; 3];

    loop {
        let bytes_read = stdin.lock().read(&mut buffer)?;

        if bytes_read == 0 {
            continue;
        }

        match buffer[0] {
            b'q' | b'Q' => break,
            b'\r' | b'\n' | b' ' => {
                state.toggle_selected();
                state.render();
            }
            27 => { // ESC sequence (arrow keys)
                if bytes_read == 3 && buffer[1] == b'[' {
                    match buffer[2] {
                        b'A' => { // Up arrow
                            state.move_up();
                            state.render();
                        }
                        b'B' => { // Down arrow
                            state.move_down();
                            state.render();
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    // Show cursor
    print!("\x1b[?25h");
    // Clear screen
    print!("\x1b[2J\x1b[H");
    io::stdout().flush()?;

    disable_raw_mode()?;

    Ok(())
}

fn main() {
    if let Err(e) = run_interactive() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
