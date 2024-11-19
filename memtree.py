import psutil
from collections import defaultdict

def get_process_tree():
    processes = defaultdict(list)
    process_info = {}

    for proc in psutil.process_iter(['pid', 'name', 'memory_info', 'ppid', 'cmdline']):
        try:
            info = proc.info
            memory_mb = info['memory_info'].rss / (1024 * 1024)
            cmdline = ' '.join(info['cmdline']) if info['cmdline'] else '[No Command]'
            process_info[info['pid']] = {
                'name': info['name'],
                'memory_mb': round(memory_mb, 2),
                'ppid': info.get('ppid', 0),
                'cmdline': cmdline,
                'total_memory_mb': memory_mb
            }
            processes[info.get('ppid', 0)].append(info['pid'])
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            pass

    def calculate_total_memory(pid, visited=None):
        if visited is None:
            visited = set()

        if pid in visited or pid not in process_info:
            return 0

        visited.add(pid)
        total = process_info[pid]['memory_mb']

        for child_pid in processes[pid]:
            total += calculate_total_memory(child_pid, visited)

        process_info[pid]['total_memory_mb'] = round(total, 2)
        return total

    def sort_processes():
        for ppid in processes:
            processes[ppid].sort(
                key=lambda pid: process_info[pid]['total_memory_mb'] if pid in process_info else 0,
                reverse=True
            )

    def print_tree(pid, level=0, visited=None):
        if visited is None:
            visited = set()

        if pid in visited:
            return
        visited.add(pid)

        if pid in process_info:
            info = process_info[pid]
            indent = "  " * level
            print(f"{indent}{info['total_memory_mb']} MB - {info['name']} (PID: {pid}, Memory: {info['memory_mb']:.2f} MB, CMD: {info['cmdline']})")

            for child_pid in processes[pid]:
                print_tree(child_pid, level + 1, visited)

    calculate_total_memory(1)
    sort_processes()
    print_tree(1)

if __name__ == '__main__':
    get_process_tree()
