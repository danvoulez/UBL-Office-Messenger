# RUNBOOK 02 — Rede & Isolamento (LAB vs Casa)

**Objetivo:** criar SSIDs `santoandre` (casa) e `lab` (isolado), manter **internet ok** para ambos, **impedir** LAB→Casa; e preservar AirPlay/Chromecast na casa.

> Preferência: configurar direto no roteador Mercusys. Se não houver firewall/VLAN, usar **pf** no LAB 256 como isolamento temporário.

---

## A) No roteador (preferível)

1. **WAN**: modem ISP → porta WAN do Mercusys.
2. **LAN**: LAB 256 por **Ethernet** (mais estável).
3. **SSIDs**:
   - `santoandre` — WPA2/3, **sem captive**, client isolation **OFF**, 2.4+5GHz
   - `lab` — WPA2/3, client isolation **ON**, 5GHz preferencial
4. **Firewall/VLAN** (se existir no modelo):
   - **Bloquear** tráfego de `lab` → `santoandre`
   - **Permitir** `lab` → Internet (WAN)
   - **Permitir** `santoandre` ↔ `santoandre`
5. **DHCP reservas** (opcional):
   - tv-sala → 10.77.0.10
   - tv-quarto → 10.77.0.11
   - lab256 → 10.88.0.2
   - lab512 → 10.88.0.3
   - lab8gb → 10.88.0.4

**Teste:**  
- De um host `lab`, ping em 10.77.0.10 (TV): **deve falhar**  
- AirPlay/Chromecast da `santoandre` para TVs: **deve funcionar**

---

## B) Isolamento por `pf` no LAB 256 (fallback)

> Use CIDRs conforme a sua rede. Exemplo: `lab = 10.88.0.0/16`, `santoandre = 10.77.0.0/16`

```bash
sudo tee /etc/pf.anchors/ubl_isolation.conf >/dev/null <<'EOF'
block drop on en0 from 10.88.0.0/16 to 10.77.0.0/16
EOF

# incluir anchor no pf.conf, se ainda não estiver
sudo bash -lc 'grep -q ubl_isolation /etc/pf.conf ||   (echo "anchor \"ubl_isolation\"" | sudo tee -a /etc/pf.conf;    echo "load anchor \"ubl_isolation\" from \"/etc/pf.anchors/ubl_isolation.conf\"" | sudo tee -a /etc/pf.conf)'

# aplicar e habilitar
sudo pfctl -f /etc/pf.conf
sudo pfctl -e

# listar regras ativas
sudo pfctl -sr
```

**Teste:** do LAB 256, `ping 10.77.0.10` → **bloqueado**. Internet ok.

---

## C) Conectar os LABs

- LAB 256: Ethernet (preferido) ou `lab` (Wi‑Fi)  
- LAB 512: `lab` (Wi‑Fi) por enquanto  
- LAB 8GB: `lab` (Wi‑Fi) e manter perfil salvo de `santoandre` (não prioritário)

**macOS comandos úteis:**
```bash
networksetup -listallhardwareports
networksetup -setairportpower en0 on
networksetup -setairportnetwork en0 "lab" "<SENHA_LAB>"
networksetup -addpreferredwirelessnetworkatindex en0 "santoandre" 1 WPA2 "<SENHA_SANTO>"
```

---

## D) Done if…

- SSIDs `santoandre` e `lab` ativos.
- LABs **não** alcançam IPs da `santoandre`.
- TVs espelham normal (AirPlay/Chromecast).
