# ips
A tool to scan IPs on Linux
## Installation
Note: Cargo must be installed before installing ips
```
git clone https://github.com/leuriato/ips
cd ips
./install.sh
```
## Usage
Scan ips on the interfaces: ```ips```

Scan specific ip : ```ips <ip>```

Scan ips on a network : ```ips <nw_ip>/<mask>```

Scan ips on a range : ```ips <ip_start>-<ip_end>```

Note: only IPv4 are supported
