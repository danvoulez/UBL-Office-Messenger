# RUNBOOK 04 — Trinity Services (Native Deploy)

**Alvo:** LAB 8GB, LAB 256, LAB 512 (todos macOS, sem Docker)  
**Última atualização:** 2025-12-30

---

## Arquitetura Final

```
┌─────────────────────────────────────────────────────────────────┐
│                         INTERNET                                 │
│                    (Cloudflare CDN)                              │
└────────────────────────┬────────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────────┐
│  LAB 256 — PRODUCTION SERVER                                     │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                │
│  │ PostgreSQL  │ │ ubl-server  │ │   office    │                │
│  │   :5432     │ │   :8080     │ │   :8081     │                │
│  └─────────────┘ └─────────────┘ └─────────────┘                │
│  + Messenger Frontend (static → Cloudflare)                      │
└────────────────────────┬────────────────────────────────────────┘
                         │ internal network (lab)
┌────────────────────────▼────────────────────────────────────────┐
│  LAB 512 — SANDBOXED COMPUTE                                     │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                │
│  │ ubl-runner  │ │  S3/MinIO   │ │ LLM Infer   │                │
│  │  (jobs)     │ │  (storage)  │ │  (heavy)    │                │
│  └─────────────┘ └─────────────┘ └─────────────┘                │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  LAB 8GB + iPhone 16 — ADMIN                                     │
│  SSH control │ IDE │ Monitoring │ Mobile Messenger               │
└─────────────────────────────────────────────────────────────────┘
```

| LAB | Hostname | Role | Services |
|-----|----------|------|----------|
| **256** | `lab-256` | Production | PostgreSQL, `ubl-server` (8080), `office` (8081), Frontend (Cloudflare) |
| **512** | `lab-512` | Sandboxed | `ubl-runner`, S3/MinIO, LLM inference |
| **8GB** | `lab-8gb` | Admin | SSH control, IDE, daily use, iPhone sync |

---

## Parte 0 — Pré-voo (LAB 8GB)

### 0.1 Rotulagem (2 min)

Em cada máquina, defina o nome:
```bash
sudo scutil --set ComputerName "lab-8gb"   # ou lab-256, lab-512
sudo scutil --set HostName "lab-8gb"
sudo scutil --set LocalHostName "lab-8gb"
```

**Proof:** `scutil --get ComputerName` → retorna o nome certo.

---

### 0.2 Contas macOS (5 min)

No **LAB 8GB** crie duas contas:
- **`dan`** (Admin, uso diário)
- **`labops`** (Admin restrito, automação)

Ative **FileVault** e desative login automático.

**Proof:**
```bash
fdesetup status   # → "FileVault is On."
```

---

### 0.3 SSH de controle (3 min)

No **LAB 8GB (dan)**:
```bash
ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519 -N ""
pbcopy < ~/.ssh/id_ed25519.pub
```

No **LAB 256 e 512**, adicione em `~/.ssh/authorized_keys`.

**Proof:**
```bash
ssh lab-256.local 'hostname'   # → lab-256
ssh lab-512.local 'hostname'   # → lab-512
```

---

### 0.4 Toolchain (10 min)

```bash
# Xcode CLT
xcode-select --install

# Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
eval "$(/opt/homebrew/bin/brew shellenv)"

# Pacotes
brew install git jq wget curl coreutils gnu-sed gnupg tmux tree watch rsync
brew install node@20 pnpm postgresql@16 pkg-config openssl cmake
brew install rustup-init && rustup-init -y
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc
```

**Proof:** `git --version`, `node -v`, `rustc -V` → todos respondem.

---

### 0.5 Pastas de trabalho

```bash
mkdir -p ~/Work/ubl ~/Work/tmp ~/.config/ubl ~/.logs
git config --global user.name "Dan"
git config --global user.email "dan@ubl.agency"
```

---

### 0.6 Passkey (WebAuthn)

Teste em `webauthn.io` criando/validando uma credencial com Touch ID.

---

## Parte 1 — Deploy no LAB 256

### 1.1 PostgreSQL

```bash
brew services start postgresql@16
createdb ubl_ledger
```

### 1.2 Migrar banco

```bash
cd ~/Work/ubl
DB=ubl_ledger

# Nova estrutura de SQL
psql "$DB" -f ubl/sql/00_base/000_core.sql
psql "$DB" -f ubl/sql/00_base/001_identity.sql
psql "$DB" -f ubl/sql/00_base/002_policy.sql
psql "$DB" -f ubl/sql/00_base/003_triggers.sql
psql "$DB" -f ubl/sql/10_projections/100_console.sql
psql "$DB" -f ubl/sql/10_projections/101_messenger.sql
psql "$DB" -f ubl/sql/10_projections/102_office.sql
```

### 1.3 Build binaries

```bash
# UBL Kernel
cd ubl/kernel/rust && cargo build --release

# Office
cd ../../../apps/office && cargo build --release

# Messenger Frontend
cd ../messenger/frontend && npm install && npm run build
```

### 1.4 Variáveis de ambiente

Crie `~/.config/ubl/env.sh`:
```bash
# UBL Kernel
export DATABASE_URL="postgres://$(whoami)@localhost/ubl_ledger"
export PORT=8080
export WEBAUTHN_RP_ID=localhost
export WEBAUTHN_ORIGIN="http://localhost:8080"

# Office
export OFFICE__UBL__ENDPOINT="http://localhost:8080"
export OFFICE__UBL__CONTAINER_ID="C.Office"
export OFFICE__SERVER__HOST="0.0.0.0"
export OFFICE__SERVER__PORT="8081"
```

### 1.5 Subir serviços (tmux)

```bash
source ~/.config/ubl/env.sh

# UBL Kernel
tmux new -s ublk -d 'cd ~/Work/ubl/ubl/kernel/rust && ./target/release/ubl-server 2>&1 | tee -a ~/.logs/ubl-server.log'

# Office
tmux new -s office -d 'cd ~/Work/ubl/apps/office && ./target/release/office 2>&1 | tee -a ~/.logs/office.log'

# Messenger (static serve)
tmux new -s messenger -d 'cd ~/Work/ubl/apps/messenger/frontend && npx serve dist -l 3000 2>&1 | tee -a ~/.logs/messenger.log'
```

---

## Parte 2 — Deploy no LAB 512

### 2.1 UBL Runner

```bash
cd ~/Work/ubl/ubl/kernel/rust
cargo build --release --bin ubl-runner

# Ambiente
export UBL_ENDPOINT="http://lab-256.local:8080"
export UBL_KEYS_DIR="$HOME/.ubl/keys"

tmux new -s runner -d './target/release/ubl-runner 2>&1 | tee -a ~/.logs/ubl-runner.log'
```

### 2.2 S3/MinIO (opcional)

```bash
brew install minio/stable/minio
mkdir -p ~/Work/s3data
minio server ~/Work/s3data --console-address ":9001"
```

---

## Parte 3 — Smoke Tests

```bash
# Saúde
curl -sf http://lab-256.local:8080/health   # → 200
curl -sf http://lab-256.local:8081/health   # → 200

# Stream do ledger
curl -N http://lab-256.local:8080/ledger/C.Messenger/tail | head -n 5

# Frontend: http://lab-256.local:3000
```

---

## Parte 4 — Operação Diária

### Ver logs

```bash
tmux attach -t ublk       # sair: Ctrl-b d
tmux attach -t office
tmux attach -t messenger
```

### Reiniciar serviço

```bash
tmux kill-session -t office
source ~/.config/ubl/env.sh
tmux new -s office -d 'cd ~/Work/ubl/apps/office && ./target/release/office'
```

### Status rápido

```bash
curl -s http://localhost:8080/health | jq .
curl -s http://localhost:8081/health | jq .
```

---

## Checklist Final

- [ ] PostgreSQL rodando no LAB 256
- [ ] Migrations aplicadas (41 tabelas)
- [ ] `ubl-server` respondendo em :8080
- [ ] `office` respondendo em :8081
- [ ] Frontend servido em :3000 (ou Cloudflare)
- [ ] `ubl-runner` no LAB 512 conectado
- [ ] SSH do LAB 8GB funcionando para 256 e 512
- [ ] WebAuthn testado e funcionando

---

## Rollback

```bash
# Parar tudo
tmux kill-server

# Restaurar banco
dropdb ubl_ledger && createdb ubl_ledger
# Re-rodar migrations
```

---

**Status:** ✅ Production Ready (após passar smoke tests)
