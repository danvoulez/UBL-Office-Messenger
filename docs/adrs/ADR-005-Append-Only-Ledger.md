# ADR-005 — Append-Only Ledger (Imutabilidade como Fundamento)

**Status:** Aprovado  
**Data:** 30-dez-2025  
**Owner:** Dan (LAB 512)

---

## 1) Contexto

Bancos de dados tradicionais permitem UPDATE e DELETE, criando problemas:
- Histórico perdido (quem mudou o quê, quando?)
- Disputas sem evidência
- Auditoria impossível ou custosa
- LLMs podem "alucinar" sem trail verificável

Para um sistema de "verdade verificável", precisamos de:
- Imutabilidade: o que foi escrito, foi escrito
- Auditoria: trail completo de quem fez o quê
- Criptografia: assinaturas verificáveis
- Containers: isolamento lógico de domínios

## 2) Decisão

### Append-Only Ledger com Containers

```
┌─────────────────────────────────────────────────────────────┐
│                       UBL LEDGER                            │
│                                                             │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐│
│  │  C.Jobs   │  │C.Messenger│  │ C.Tenant  │  │  C.Pacts  ││
│  │           │  │           │  │           │  │           ││
│  │ seq: 142  │  │ seq: 891  │  │ seq: 23   │  │ seq: 7    ││
│  │ hash: a3f │  │ hash: 7c2 │  │ hash: e91 │  │ hash: 4b8 ││
│  └───────────┘  └───────────┘  └───────────┘  └───────────┘│
│                                                             │
│  Cada container: seq monotônico + hash encadeado            │
└─────────────────────────────────────────────────────────────┘
```

### Link Structure (Commit Envelope)

```json
{
  "container_id": "C.Jobs",
  "seq": 143,
  "prev_hash": "a3f...",
  "atom": {
    "type": "job.completed",
    "job_id": "job_abc",
    "result": { ... },
    "timestamp": "2025-12-30T12:00:00Z"
  },
  "atom_hash": "blake3(canonical(atom))",
  "sig": "ed25519(actor_privkey, atom_hash)",
  "actor_id": "sid_xyz",
  "intent_class": "Observation"
}
```

### Canonicalização (JSON✯Atomic v1.0)

Para garantir hash determinístico:
1. Keys ordenadas alfabeticamente
2. Sem whitespace extra
3. Números sem trailing zeros
4. Strings UTF-8 escapadas consistentemente

### Intent Classes (Física do Sistema)

| Classe | Δ | Descrição | Exemplo |
|--------|---|-----------|---------|
| Observation | = 0 | Read-only, sem mudança | Consulta |
| Conservation | ≤ 0 | Mantém ou reduz valor | Transfer |
| Entropy | < 0 | Destrói valor (irreversível) | Delete |
| Evolution | > 0 | Cria valor (requer pact) | Mint |

## 3) Operações

### ✅ Permitido
```sql
INSERT INTO ledger (container_id, seq, ...) VALUES (...);
```

### ❌ Proibido
```sql
UPDATE ledger SET ... WHERE ...;  -- NUNCA
DELETE FROM ledger WHERE ...;      -- NUNCA
```

### Correções
Erros são corrigidos com novos eventos, não edições:

```json
// Evento original
{ "type": "job.completed", "job_id": "job_abc", "status": "success" }

// Correção (novo evento, não UPDATE)
{ "type": "job.status_corrected", "job_id": "job_abc", 
  "original_seq": 143, "corrected_status": "partial",
  "reason": "Artifact missing" }
```

## 4) Projeções (Views Materializadas)

Ledger é write-optimized. Para reads, usamos projeções:

```
┌──────────────┐     ┌──────────────────┐     ┌──────────────┐
│    LEDGER    │────▶│   PROJECTIONS    │────▶│   QUERIES    │
│ (append-only)│     │ (materialized)   │     │  (fast read) │
└──────────────┘     └──────────────────┘     └──────────────┘
```

```sql
-- Projeção de jobs (rebuilda do ledger)
CREATE TABLE projection_jobs AS
SELECT 
  (atom->>'job_id') as job_id,
  (atom->>'status') as status,
  ...
FROM ledger 
WHERE container_id = 'C.Jobs'
ORDER BY seq;
```

### Rebuild Guarantee

Se a projeção corromper, reconstruímos do ledger:

```bash
# Rebuild completo
TRUNCATE projection_jobs;
INSERT INTO projection_jobs SELECT ... FROM ledger WHERE container_id = 'C.Jobs';
```

## 5) Verificação Criptográfica

### Hash Chain

```
Link[n].prev_hash = Link[n-1].hash
Link[n].hash = blake3(Link[n].atom_hash + Link[n].prev_hash + Link[n].seq)
```

Qualquer alteração quebra a chain.

### Signature Verification

```rust
pub fn verify_link(link: &Link, pubkey: &PublicKey) -> bool {
    let expected_hash = blake3::hash(&canonical(link.atom));
    if link.atom_hash != expected_hash { return false; }
    
    pubkey.verify(&link.atom_hash, &link.sig)
}
```

## 6) Consequências

### Positivas
- ✅ Auditoria completa e verificável
- ✅ Disputas resolvíveis (evidência imutável)
- ✅ LLMs accountable (assinaram o que fizeram)
- ✅ Rebuild garantido (projeções são derivadas)
- ✅ Compliance (GDPR right-to-audit)

### Negativas
- ⚠️ Storage cresce monotonicamente
- ⚠️ Não há DELETE (GDPR right-to-erasure?)
- ⚠️ Projeções podem ficar out-of-sync

### Mitigações
- Storage barato (S3 para archive)
- GDPR: dados pessoais em container separado, tombstone events
- SSE para sync de projeções em real-time

## 7) Containers Definidos

| Container | Propósito | Eventos principais |
|-----------|-----------|-------------------|
| C.Jobs | Job lifecycle | job.created, job.approved, job.completed |
| C.Messenger | Messages | msg.sent, msg.read, msg.deleted |
| C.Tenant | Multi-tenancy | tenant.created, member.added |
| C.Entities | Office entities | entity.created, session.completed |
| C.Pacts | Multi-sig | pact.created, pact.signed |
| C.Policy | Rules | policy.updated |

## 8) Referências

- [SPEC-UBL-LEDGER](../../ubl/specs/ubl-ledger/SPEC-UBL-LEDGER.md)
- [SPEC-UBL-LINK](../../ubl/specs/ubl-link/SPEC-UBL-LINK.md)
- [SPEC-UBL-ATOM](../../ubl/specs/ubl-atom/SPEC-UBL-ATOM.md)
- Event Sourcing (Martin Fowler)
- Blockchain (inspiração, sem consensus distribuído)

---

*"Truth is not what you say. Truth is what you can prove."*
