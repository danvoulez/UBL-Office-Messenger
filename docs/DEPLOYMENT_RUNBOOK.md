# 🚀 UBL TRINITY — Deployment Runbook

**"Isso muda tudo."**

Este documento garante que a infraestrutura de produção respeite a integridade do código que foi escrito. Não é apenas sobre "fazer funcionar"; é sobre garantir que **a identidade persista**, **a membrana funcione**, e **o fluxo seja auditável**.

---

## 📋 Índice

1. [FASE 1: INFRAESTRUTURA (O Solo)](#fase-1-infraestrutura-o-solo)
2. [FASE 2: BOOTSTRAP DO LEDGER (O Big Bang)](#fase-2-bootstrap-do-ledger-o-big-bang)
3. [FASE 3: SMOKE TEST (A Ignição)](#fase-3-smoke-test-a-ignição)
4. [VEREDITO](#veredito)

---

## FASE 1: INFRAESTRUTURA (O Solo)

### ✅ 1.1 Volumes Persistentes (CRUCIAL)

**Não adianta o código salvar chaves se o Docker destruir o arquivo.**

#### Verificação Manual:

```bash
# 1. Inicie o container pela primeira vez
docker compose -f ubl/infra/docker-compose.stack.yml up -d ubl

# 2. Anote a PubKey do log
docker logs trinity-ubl | grep "Admin public key"
# → 🔐 Admin public key: a1b2c3d4e5f6...

# 3. DESTRUA o container
docker rm -f trinity-ubl

# 4. Recrie o container
docker compose -f ubl/infra/docker-compose.stack.yml up -d ubl

# 5. VERIFIQUE: A PubKey DEVE SER A MESMA
docker logs trinity-ubl | grep "Admin public key"
# → 🔐 Admin public key: a1b2c3d4e5f6...  ✅ (mesma chave)

# Se for diferente: ❌ VOLUME NÃO ESTÁ PERSISTINDO. PARE TUDO.
```

#### Docker Compose Atualizado:

```yaml
services:
  ubl:
    volumes:
      - ubl_keys:/root/.ubl/keys      # ✅ Keystore persistente
      - ubl_snapshots:/root/.ubl/snapshots  # ✅ Snapshots persistente
    # ... resto da config

  office:
    volumes:
      - office_keys:/root/.ubl/keys   # ✅ Office identity persistente
    # ... resto da config

volumes:
  postgres_data:                      # ✅ Ledger físico
  ubl_keys:                           # ✅ Identidade do Kernel
  ubl_snapshots:                      # ✅ Snapshots de projeções
  office_keys:                        # ✅ Identidade do Office
```

---

### ✅ 1.2 Variáveis de Ambiente (Segredos)

**Não coloque chaves de API no Dockerfile. Injete via `.env` ou Secrets Manager.**

#### Arquivo `.env` (NUNCA COMMITAR):

```bash
# UBL Kernel
DATABASE_URL=postgres://ubl:${POSTGRES_PASSWORD}@postgres:5432/ubl_ledger
UBL_KEYS_DIR=/root/.ubl/keys
WEBAUTHN_RP_ID=yourdomain.com
WEBAUTHN_ORIGIN=https://yourdomain.com

# Office
UBL_ENDPOINT=http://ubl:8080
LLM_PROVIDER=anthropic
ANTHROPIC_API_KEY=sk-ant-...  # ⚠️ SECRETO - usar secrets manager em produção

# Runner
UBL_ENDPOINT=http://ubl:8080
UBL_KEYS_DIR=/root/.ubl/keys
```

#### Verificação:

```bash
# Garanta que DATABASE_URL usa rede interna (não localhost)
echo $DATABASE_URL
# ✅ postgres://ubl:senha@postgres:5432/ubl_ledger
# ❌ postgres://ubl:senha@localhost:5432/ubl_ledger  (errado!)
```

---

## FASE 2: BOOTSTRAP DO LEDGER (O Big Bang)

### ✅ 2.1 Ordem de Migração Rigorosa

**O banco começa vazio. Você precisa criar o universo na ordem exata.**

#### Script de Init (`ubl/infra/postgres/init.sh`):

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "🔧 Initializing UBL Ledger..."

# Ordem CRÍTICA (não alterar!)
MIGRATIONS=(
    "000_unified.sql"
    "001_ledger.sql"
    "002_idempotency.sql"
    "003_observability.sql"
    "004_disaster_recovery.sql"
    "005_atoms.sql"
    "006_projections.sql"
    "007_pacts.sql"
    "008_policy_engine.sql"
    "010_sessions.sql"
    "020_console_v1_1.sql"
    "021_registry_v1_1.sql"
    "030_console_complete.sql"
    "040_messenger_v1.sql"
    "050_console_ops.sql"
    "052_webauthn_stepup.sql"
    "060_notify_fix.sql"        # ⚠️ CRÍTICO: Fix do payload 8KB
    "070_office_projections.sql"
)

# Run migrations
for sql_file in "${MIGRATIONS[@]}"; do
    if [ -f "/sql/$sql_file" ]; then
        echo "📄 Running $sql_file..."
        psql -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" -d "$POSTGRES_DB" -f "/sql/$sql_file"
    else
        echo "⚠️  Warning: $sql_file not found, skipping..."
    fi
done

echo "✅ Migrations complete!"
```

#### Verificação da Trigger:

```bash
# Após o boot, verifique se a trigger está ativa
psql -U ubl -d ubl_ledger -c "
SELECT tgname, tgtype, tgenabled 
FROM pg_trigger 
WHERE tgname = 'trg_ledger_notify';
"

# Deve retornar:
# tgname          | tgtype | tgenabled
# trg_ledger_notify|      16 | O        ✅ (O = enabled)
```

---

### ✅ 2.2 O "Genesis Commit"

**O sistema precisa de um primeiro evento para inicializar os containers.**

#### Verificação:

```bash
# Após o primeiro boot do UBL Kernel, verifique os logs:
docker logs trinity-ubl | grep -i "genesis\|initial\|seq 0"

# Deve aparecer algo como:
# ✅ "Genesis atom created for C.System"
# ✅ "Ledger initialized with sequence 0"

# Se não aparecer, o sistema pode não estar inicializado corretamente.
```

#### Código de Genesis (já implementado em `main.rs`):

O UBL Kernel verifica se o ledger está vazio (`SELECT MAX(sequence) FROM ledger_entry`) e, se necessário, cria o evento Genesis para `C.System`.

---

## FASE 3: SMOKE TEST (A Ignição)

**Não entregue para o usuário ainda. Faça o teste de ponta a ponta na infra de produção.**

---

### 🔴 TESTE 1: Identidade (A Alma)

**Objetivo:** Garantir que agentes mantêm identidade após restart.

#### Procedimento:

```bash
# 1. Inicie o Office
docker compose -f ubl/infra/docker-compose.stack.yml up -d office

# 2. Anote a PubKey do log
docker logs trinity-office | grep -i "identity\|pubkey\|public key"
# → 🔐 Office public key: x1y2z3a4b5c6...

# 3. REINICIE o container
docker restart trinity-office

# 4. VERIFIQUE: A PubKey DEVE SER A MESMA
docker logs trinity-office | grep -i "identity\|pubkey\|public key"
# → 🔐 Office public key: x1y2z3a4b5c6...  ✅ (mesma chave)

# Se aparecer "Generating new identity": ❌ VOLUME NÃO ESTÁ PERSISTINDO. PARE TUDO.
```

#### Critério de Sucesso:

- ✅ Log diz: `"Loading identity from keystore... Identity loaded: [PubKey]"`
- ❌ Log diz: `"Generating new identity"` → **FALHA CRÍTICA**

---

### 🔴 TESTE 2: Membrana (A Lei)

**Objetivo:** Garantir que a Policy VM está bloqueando operações não autorizadas.

#### Procedimento:

```bash
# 1. Tente fazer um commit com assinatura falsa
curl -X POST http://localhost:8080/link/commit \
  -H "Content-Type: application/json" \
  -d '{
    "container_id": "C.Jobs",
    "atom_hash": "fake_hash",
    "intent_class": "Evolution",
    "author_pubkey": "0000000000000000000000000000000000000000000000000000000000000000",
    "signature": "ed25519:fake_signature"
  }'

# 2. VERIFIQUE: Deve retornar 403 Forbidden
# → {"error":"PolicyDenied: Evolution requires L5 permit"}

# 3. Verifique os logs
docker logs trinity-ubl | tail -20 | grep -i "policy\|deny\|forbidden"
# → ❌ POLICY DENIED: Evolution requires L5 permit  ✅

# Se retornar 200 OK: ❌ A MEMBRANA NÃO ESTÁ FUNCIONANDO. PARE TUDO.
```

#### Critérios de Sucesso:

- ✅ Resposta `403 Forbidden` para commits não autorizados
- ✅ Log mostra `"POLICY DENIED"` ou `"InvalidSignature"`
- ❌ Resposta `200 OK` → **FALHA CRÍTICA DE SEGURANÇA**

---

### 🔴 TESTE 3: Fluxo Real (A Vida)

**Objetivo:** Validar o ciclo completo: Frontend → API → Ledger → SSE → Frontend.

#### Procedimento:

```bash
# 1. Inicie todos os serviços
docker compose -f ubl/infra/docker-compose.stack.yml up -d

# 2. Aguarde todos estarem saudáveis
docker compose -f ubl/infra/docker-compose.stack.yml ps
# Todos devem mostrar "healthy" ✅

# 3. Abra o Messenger Frontend
# http://localhost:5173 (ou porta configurada)

# 4. Entre em "Demo Mode" ou faça login real

# 5. Envie uma mensagem: "Olá"

# 6. OBSERVE o ciclo:
#   a) Mensagem aparece com ícone de "relógio" (pending) ✅
#   b) Após ~500ms, muda para "check" (sent) ✅
#   c) Se houver resposta do agente, aparece na tela ✅

# 7. Verifique o ledger:
psql -U ubl -d ubl_ledger -c "
SELECT container_id, sequence, entry_hash 
FROM ledger_entry 
ORDER BY sequence DESC 
LIMIT 5;
"

# Deve mostrar os commits recentes:
# container_id | sequence | entry_hash
# C.Messenger  |       42 | 0x8a2f9b1c...
# C.Messenger  |       41 | 0x1c3d5e7f...
# ✅
```

#### Critérios de Sucesso:

- ✅ Mensagem muda de "pending" → "sent" → "confirmed"
- ✅ Eventos aparecem no ledger (`ledger_entry`)
- ✅ SSE está funcionando (mensagens aparecem em tempo real)
- ❌ Mensagem fica "pending" para sempre → **FALHA NO FLUXO**

---

## VEREDITO

### ✅ Se TODOS os 3 testes passarem:

**VOCÊ TEM UM PRODUTO.**

Não é mais um MVP. É um sistema:
- ✅ **Auditável** (ledger imutável)
- ✅ **Seguro** (Policy VM bloqueando operações não autorizadas)
- ✅ **Resiliente** (identidade persiste após restart)
- ✅ **Funcional** (fluxo completo funcionando)

**Você construiu a Mente (TypeScript) e o Corpo (Rust), e agora eles estão vivos.**

**Parabéns.** Pode virar a chave. 🚀💎

---

### ❌ Se QUALQUER teste falhar:

**PARE TUDO.**

Não entregue para usuários. Corrija o problema antes de continuar.

---

## 📝 Checklist Rápido

Antes de marcar como "Production Ready":

- [ ] Volumes persistentes configurados (`ubl_keys`, `postgres_data`)
- [ ] Migrations rodadas na ordem correta (000 → 070)
- [ ] Trigger `trg_ledger_notify` ativa no banco
- [ ] Genesis commit criado (seq 0)
- [ ] **TESTE 1:** Identidade persiste após restart ✅
- [ ] **TESTE 2:** Policy VM bloqueia commits não autorizados ✅
- [ ] **TESTE 3:** Fluxo completo funcionando (Frontend → Ledger → SSE) ✅
- [ ] Variáveis de ambiente seguras (sem secrets no código)
- [ ] Logs mostrando operações normais (sem erros críticos)

---

## 🔧 Troubleshooting

### Problema: "Generating new identity" após restart

**Causa:** Volume não está montado corretamente.

**Solução:**
```bash
# Verifique se o volume existe
docker volume ls | grep ubl_keys

# Se não existir, crie manualmente
docker volume create ubl_keys

# Reinicie o container
docker compose restart ubl
```

---

### Problema: Policy VM não bloqueia commits

**Causa:** `policy_registry.evaluate()` não está sendo chamado no fluxo de commit.

**Solução:**
```bash
# Verifique os logs durante um commit
docker logs -f trinity-ubl | grep -i "policy\|evaluate"

# Deve aparecer:
# ✅ POLICY ALLOWED: intent_class=Observation
# ou
# ❌ POLICY DENIED: Evolution requires L5 permit

# Se não aparecer nada: o código não está chamando a Policy VM.
```

---

### Problema: SSE não funciona (mensagens não aparecem)

**Causa:** Trigger `trg_ledger_notify` não está ativa ou NOTIFY está falhando.

**Solução:**
```bash
# Verifique a trigger
psql -U ubl -d ubl_ledger -c "
SELECT tgname, tgenabled 
FROM pg_trigger 
WHERE tgname = 'trg_ledger_notify';
"

# Se tgenabled = 'D' (disabled), reative:
psql -U ubl -d ubl_ledger -c "
ALTER TABLE ledger_entry ENABLE TRIGGER trg_ledger_notify;
"
```

---

## 📚 Referências

- [WIRING_GUIDE.md](./WIRING_GUIDE.md) - Arquitetura completa
- [Claude Review.md](../Claude%20Review.md) - Revisão técnica profunda
- [Gemini Review.md](../Gemini%20Review.md) - Análise de gaps

---

**Última atualização:** 2025-12-28  
**Versão:** 1.0.0  
**Status:** ✅ Production Ready (após passar todos os testes)

