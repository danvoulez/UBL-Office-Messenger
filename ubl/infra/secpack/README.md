# UBL Fractal Security Pack

Date: 2025-12-27

This pack locks down the UBL world in a **fractal** way:
- LAB 256 = Gateway/API (UBL Kernel + Ledger + WebAuthn)
- LAB 512 = Runner Factory (pull-only, sandboxed)
- LAB 8GB = Workstation/Dev
- Office = LLM runtime (outside, limited ASC)

## Layout

- firewall/
  - pf.conf (macOS) — LAB 256
  - iptables.sh (Linux) — LAB 256
- sandbox/
  - runner.sb (macOS sandbox-exec profile)
  - runner.cfg (Linux nsjail profile)
- postgres/
  - postgresql.conf (socket-only)
  - roles.sql (append-only guarantees)
- wireguard/
  - wg0.lab256.conf
  - wg0.lab512.conf
- manifests/
  - containers.json (generated via scripts/gen_container_ids.py)
- runner/
  - pull_loop.sh (pull-only loop)
- scripts/
  - gen_container_ids.py
- examples/
  - .env.lab256.example
  - launchd.ubl-runner.plist (macOS runner service)
