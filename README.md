# MemTree
Tool that allows to view the memory usage of the processes in a tree-like structure. It is useful to identify the memory usage of the processes and the memory usage of the children processes.

## Installation
```
sudo curl -L https://github.com/zerint/memtree/releases/latest/download/memtree -o /usr/local/sbin/memtree
sudo chmod +x /usr/local/sbin/memtree
```

## Usage
```
memtree | less -S
```

## Example
```
15584.82 MB - systemd (PID: 1, Memory: 15.17 MB, CMD: /sbin/init splash)
  5277.44 MB - bash (PID: 138371, Memory: 3.50 MB, CMD: /bin/bash)
    5273.94 MB - firefox (PID: 138374, Memory: 666.84 MB, CMD: /snap/firefox/5239/usr/lib/
      615.24 MB - Isolated Web Co (PID: 139593, Memory: 615.24 MB, CMD: /snap/firefox/5239>
      559.3 MB - Isolated Web Co (PID: 139155, Memory: 559.30 MB, CMD: /snap/firefox/5239/>
      359.79 MB - WebExtensions (PID: 138649, Memory: 359.79 MB, CMD: /snap/firefox/5239/u>
      283.6 MB - Isolated Web Co (PID: 825400, Memory: 283.60 MB, CMD: /snap/firefox/5239/>
      ...
```
