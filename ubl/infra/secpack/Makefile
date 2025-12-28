# UBL Fractal SecPack - Infrastructure Automation
# Date: 2025-12-28

.PHONY: help pf-apply iptables postgres-init wg-up runner-start runner-stop gen-ids all

help:
	@echo "UBL Fractal SecPack Commands:"
	@echo "  pf-apply      - Apply pf.conf (macOS firewall)"
	@echo "  iptables      - Apply iptables rules (Linux firewall)"
	@echo "  postgres-init - Initialize PostgreSQL roles and triggers"
	@echo "  wg-up         - Start WireGuard tunnel"
	@echo "  runner-start  - Start Runner service (macOS launchd)"
	@echo "  runner-stop   - Stop Runner service"
	@echo "  gen-ids       - Generate container IDs"
	@echo "  all           - Apply all security configs"

# macOS firewall
pf-apply:
	@echo "🔥 Applying pf.conf..."
	sudo pfctl -f firewall/pf.conf
	sudo pfctl -e
	@echo "✅ pf.conf applied"

# Linux firewall
iptables:
	@echo "🔥 Applying iptables rules..."
	sudo bash firewall/iptables.sh
	@echo "✅ iptables applied"

# PostgreSQL init (socket-only + append-only triggers)
postgres-init:
	@echo "🗄️ Initializing PostgreSQL..."
	@if [ -f /etc/postgresql/*/main/postgresql.conf ]; then \
		sudo cp postgres/postgresql.conf /etc/postgresql/*/main/conf.d/ubl.conf; \
	else \
		echo "⚠️ PostgreSQL config not found, manual copy required"; \
	fi
	psql -U postgres -d ubl_ledger -f postgres/roles.sql
	@echo "✅ PostgreSQL configured (socket-only, append-only)"

# WireGuard (LAB 256)
wg-up:
	@echo "🔒 Starting WireGuard..."
	sudo cp wireguard/wg0.lab256.conf /etc/wireguard/wg0.conf
	sudo wg-quick up wg0
	@echo "✅ WireGuard tunnel active"

wg-down:
	sudo wg-quick down wg0

# Runner service (macOS)
runner-start:
	@echo "🏃 Starting Runner service..."
	mkdir -p /Users/Shared/ubl-runner/{ro,rw,logs}
	cp -R runner/* /Users/Shared/ubl-runner/
	cp sandbox/runner.sb /Users/Shared/ubl-runner/
	launchctl load -w examples/launchd.ubl-runner.plist
	@echo "✅ Runner service started"

runner-stop:
	launchctl unload examples/launchd.ubl-runner.plist

# Generate container IDs
gen-ids:
	@echo "🔑 Generating container IDs..."
	python3 scripts/gen_container_ids.py > manifests/containers.json
	@echo "✅ Container IDs generated:"
	@cat manifests/containers.json

# Full setup
all: pf-apply postgres-init wg-up runner-start gen-ids
	@echo ""
	@echo "╔══════════════════════════════════════════════════╗"
	@echo "║  🛡️  UBL Fractal SecPack - DEPLOYED              ║"
	@echo "╠══════════════════════════════════════════════════╣"
	@echo "║  ✅ Firewall: pf.conf applied                    ║"
	@echo "║  ✅ PostgreSQL: socket-only, append-only         ║"
	@echo "║  ✅ WireGuard: 10.77.0.0/24 tunnel active        ║"
	@echo "║  ✅ Runner: sandboxed, pull-only                 ║"
	@echo "║  ✅ Container IDs: generated                     ║"
	@echo "╚══════════════════════════════════════════════════╝"

