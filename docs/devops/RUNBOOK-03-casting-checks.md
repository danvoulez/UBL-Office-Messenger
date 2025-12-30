# RUNBOOK 03 — TVs & Casting (AirPlay/Chromecast)

**Alvo:** 2 TVs Samsung (65" e 43") + 3 Android TV boxes  
**Rede:** `santoandre` (sem captive) — mesma rede do celular que espelha.

---

## 1) TVs Samsung (AirPlay)

- Conectar as TVs em **`santoandre`**.
- Ativar **AirPlay**: Configurações → Geral → AirPlay → **Ativado**.
- Se falhar/instável: Suporte → Autodiagnóstico → **Redefinir Rede** (e reconectar).

**Ver serviços mDNS (AirPlay) via macOS:**
```bash
dns-sd -B _airplay._tcp
# resolver uma TV específica (ex.: "TV 65"")
dns-sd -L "TV 65"" _airplay._tcp local
```

---

## 2) Android TV boxes (Chromecast)

- Conectar cada box em **`santoandre`**.
- Abrir Google Home no celular → “Transmitir tela” → selecionar o box.
- Atualizar Google Play Services se necessário.

**SSDP scan (opcional)**
```bash
brew install gnu-netcat || true
echo -ne "M-SEARCH * HTTP/1.1
HOST:239.255.255.250:1900
MAN:"ssdp:discover"
MX:1
ST:ssdp:all

" | nc -u -w2 239.255.255.250 1900
```

---

## 3) Cartão de bolso (para família/clientes)

- Rede: **santoandre**
- Senha: **<defina>**
- iPhone: botão **AirPlay** → escolha **TV 65"** ou **TV 43"**  
- Android: app **Home** → **Transmitir tela** → escolha o dispositivo

---

## 4) Diagnóstico rápido

- **TV não aparece:** desligar/ligar Wi‑Fi do celular; conferir que ambos estão em `santoandre`.
- **Android não transmite:** reiniciar box; confirmar mesma rede.
- **mDNS invisível:** `dns-sd -B _airplay._tcp` precisa listar as TVs.

---

## 5) Done if…

- `dns-sd` lista as TVs.  
- iPhone/Android espelham em até **10 segundos**.
