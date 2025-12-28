#!/usr/bin/env bash
set -euo pipefail

WG_IF="wg0"
API_PORT="8080"
SSH_PORT="22"

iptables -F
iptables -X
iptables -t nat -F
iptables -t mangle -F

iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT ACCEPT

iptables -A INPUT -i lo -j ACCEPT
iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT

iptables -A INPUT -i "$WG_IF" -p tcp --dport "$API_PORT" -j ACCEPT
iptables -A INPUT -i "$WG_IF" -p tcp --dport "$SSH_PORT" -j ACCEPT

iptables -A INPUT -j LOG --log-prefix "DROP_INPUT: " --log-level 4
iptables -A INPUT -j DROP

echo "iptables applied for LAB 256"
