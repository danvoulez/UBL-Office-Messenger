# RUNBOOK 01 — Headless LAB Bootstrap (macOS)

**Alvo:** LAB 8GB, LAB 256, LAB 512 (todos macOS)  
**Você executa:** do **LAB 8GB**, com SSH por chave nos demais.  
**Objetivo:** deixar os 3 nós prontos para operar **sem mouse/teclado/monitor**.

---

## 0) Pré‑requisitos (uma vez)

- Usuário **dan** (admin) existe em todos os Macs (ou ajuste os comandos).
- Sua **chave pública** já copiada para o LAB 8GB (`~/.ssh/id_ed25519.pub`).  
- Descobrir IPs/hosts dos outros nós (temporário):
  - `LAB256_HOST=192.168.1.26`
  - `LAB512_HOST=192.168.1.51`

Coloque estes valores no shell:
```bash
export LAB256_HOST=192.168.1.26
export LAB512_HOST=192.168.1.51
export SSH_USER=dan
export AUTHORIZED_KEY="$(cat ~/.ssh/id_ed25519.pub)"
export SSID_LAB="lab"
export PASS_LAB="<SENHA_LAB>"
export SSID_SANTO="santoandre"
export PASS_SANTO="<SENHA_SANTO>"
```

---

## 1) O que este runbook faz

1. Padroniza **hostname** (lab8gb / lab256 / lab512).
2. Desativa **sono/hibernação** (headless).
3. Habilita **SSH** e **hardening básico** (chave pública; senha OFF opcional).
4. Garante **usuário** e **authorized_keys**.
5. Instala utilitários via **Homebrew** (jq, git, etc.).
6. Cria perfis **Wi‑Fi** (`lab` e `santoandre`) e conecta o necessário.
7. Roda **sanity checks** em cada nó.

---

## 2) Executar (simples)

> Requer: `/mnt/data/bootstrap_headless.sh` (script deste pacote).

```bash
chmod +x /mnt/data/bootstrap_headless.sh
/mnt/data/bootstrap_headless.sh
```

O script é **idempotente** e roda local (LAB 8GB) e remoto (LAB 256/512) via SSH.

---

## 3) Sanity checks (depois do script)

- SSH sem senha nos três nós:
  ```bash
  ssh -o PubkeyAuthentication=yes ${SSH_USER}@${LAB256_HOST} 'echo ok-256'
  ssh -o PubkeyAuthentication=yes ${SSH_USER}@${LAB512_HOST} 'echo ok-512'
  ```
- Sono/hibernação:
  ```bash
  pmset -g | egrep -i 'sleep|powernap'
  ```
- Hostnames:
  ```bash
  scutil --get HostName; hostname
  ```
- Rede:
  ```bash
  ipconfig getifaddr en0 || ipconfig getifaddr en1
  ping -c 2 1.1.1.1
  ```

---

## 4) Rollback curto (se algo der ruim)

- Reativar autenticação por senha no SSH:
  ```bash
  sudo sed -i.bak 's/^PasswordAuthentication no/PasswordAuthentication yes/' /etc/ssh/sshd_config
  sudo launchctl kickstart -k system/com.openssh.sshd
  ```
- Restaurar nomes antigos: `scutil --set HostName <antigo>` etc.
- Reativar sleep (não recomendado para headless):
  ```bash
  sudo pmset -a sleep 10 displaysleep 10 disksleep 10 powernap 1
  ```

---

## 5) Troubleshooting rápido

- **Sem SSH**: verifique `systemsetup -getremotelogin` (deve ser `On`).  
- **Wi‑Fi device**: `networksetup -listallhardwareports` (procure `Device: en0/en1`).  
- **Permissões SSH**: `chmod 700 ~/.ssh && chmod 600 ~/.ssh/authorized_keys`.  
- **Bateria de portáteis**: use `sudo pmset -b ...` além de `-a` para cobrir modo bateria.

