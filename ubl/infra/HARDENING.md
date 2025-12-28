# UBL 2.0 Hardening & Verification

**Status:** PRODUCTION READY  
**Version:** 2.0.0+postgres+webauthn  
**Date:** 2025-12-26  

---

## ✅ Conformidade Crítica Verificada

### 1. atom_hash ≡ JSON✯Atomic v1.0

- ✅ **NO domain tag UBL** no cálculo de `atom_hash`
- ✅ `atom_hash = BLAKE3(canonical_bytes)` exatamente
- ✅ Idêntico ao hash que JSON✯Atomic v1.0 produz
- ✅ Teste guardrail: `atom_hash_binding.rs` (5 testes)

**Verificação:**
```bash
cargo test --test atom_hash_binding
grep -n "hasher.update(canonical_bytes)" crates/ubl-kernel/src/lib.rs
```

---

### 2. MembraneError - 8 Nomes Canônicos

- ✅ `InvalidVersion` (V1)
- ✅ `InvalidSignature` (V2)
- ✅ `InvalidTarget` (V3) - era `ContainerMismatch`
- ✅ `RealityDrift` (V4)
- ✅ `SequenceMismatch` (V5)
- ✅ `PhysicsViolation { reason }` (V6) - unifica Conservation/Observation
- ✅ `PactViolation` (V7)
- ✅ `UnauthorizedEvolution` (V8)

**Verificação:**
```bash
grep -A 20 "pub enum MembraneError" crates/ubl-membrane/src/lib.rs
cargo test --test physics_invariants
```

---

### 3. signing_bytes - Ordem Canônica

**SPEC-UBL-LINK v1.0 §5:**
```
signing_bytes := version || container_id || expected_sequence || 
                 previous_hash || atom_hash || intent_class || physics_delta
```

**CRÍTICO: NÃO inclui:**
- ❌ `pact`
- ❌ `author_pubkey`
- ❌ `signature`

**Verificação:**
```bash
cargo test --test signing_bytes_canon
grep -A 25 "fn signing_bytes" crates/ubl-link/src/lib.rs
```

---

## 🛡️ Guardrails (Testes de Regressão)

### Testes Criados

| Arquivo | Testes | Propósito |
|---------|--------|-----------|
| `crates/ubl-link/tests/signing_bytes_canon.rs` | 3 | Prevenir inclusão de pact/author/signature |
| `crates/ubl-kernel/tests/atom_hash_binding.rs` | 5 | Garantir atom_hash sem domain tag |
| `crates/ubl-membrane/tests/physics_invariants.rs` | 8 | Enforçar invariantes físicas |

**Total: 16 testes guardrails**

### Rodar Todos os Guardrails

```bash
cargo test --test signing_bytes_canon \
           --test atom_hash_binding \
           --test physics_invariants
```

---

## 🔒 Pre-commit Hook

**Localização:** `.git/hooks/pre-commit`

**Proteções:**
1. ❌ Proíbe modificação de specs FROZEN
2. ❌ Proíbe uso de `"ubl:atom"` em código Rust
3. ✅ Verifica `signing_bytes` não inclui `author_pubkey`
4. ✅ Roda testes guardrails
5. ✅ Roda suite completa de testes

**Ativar:**
```bash
chmod +x .git/hooks/pre-commit
```

**Testar:**
```bash
.git/hooks/pre-commit
```

---

## 📊 Estatísticas de Testes

```bash
cargo test --workspace
```

**Resultados esperados:**
- `ubl-atom`: 5 testes
- `ubl-kernel`: 7 testes (incluindo atom_hash_binding)
- `ubl-link`: 5 testes (incluindo signing_bytes_canon)
- `ubl-ledger`: 3 testes
- `ubl-membrane`: 17 testes (9 unit + 8 physics_invariants)
- `ubl-server`: 1 teste
- `conformance_tests`: 16 testes

**Total: ~54 testes**

---

## 🔥 Smoke Test End-to-End

**Script:** `scripts/smoke-test.sh`

**Fluxo:**
1. Canonicaliza JSON
2. Calcula `atom_hash` (sem domain tag)
3. Constrói `LinkCommit`
4. Submete ao `ubl-server`
5. Valida `MaterializationReceipt`

**Executar:**
```bash
# Terminal 1: Iniciar servidor
cd crates/ubl-server
cargo run --release

# Terminal 2: Rodar smoke test
./scripts/smoke-test.sh
```

---

## 📝 Checklist Pré-Tag

- [x] Build release passa
- [x] Todos os testes passam (54 testes)
- [x] Guardrails criados e passando (16 testes)
- [x] Pre-commit hook configurado
- [x] DOCS_MANIFEST.sha256.txt gerado
- [x] Smoke test criado
- [x] atom_hash sem domain tag UBL
- [x] MembraneError com 8 nomes canônicos
- [x] signing_bytes exclui pact/author/signature
- [x] SPEC_MANIFEST.json atualizado
- [x] UBL-ATOM-BINDING.md correto

---

## 🏷️ Criar Tag

```bash
# Verificar estado
cargo build --release
cargo test --workspace
.git/hooks/pre-commit

# Commit final
git add .
git commit -m "Hardening: binding atom_hash, canonical errors, signing_bytes guardrails"

# Criar tag
git tag -a v2.0.0 -m "UBL 2.0 Body aligned to SPEC-UBL v1.0 + JSON✯Atomic binding"

# Push (quando pronto)
git push origin main --tags
```

---

## 🔍 Comandos de Verificação Rápida

```bash
# 1. Verificar enums canônicos
grep "enum MembraneError" crates/ubl-membrane/src/lib.rs -A 20

# 2. Verificar atom_hash sem domain tag
grep "fn hash_atom" crates/ubl-kernel/src/lib.rs -A 10

# 3. Verificar signing_bytes
grep "fn signing_bytes" crates/ubl-link/src/lib.rs -A 30

# 4. Contar testes
cargo test --workspace 2>&1 | grep "test result"

# 5. Verificar zero uso de "ubl:atom" em código
grep -r "ubl:atom" crates/ --include="*.rs" | grep -v "//"

# 6. Build otimizado
cargo build --release

# 7. Rodar guardrails
cargo test --test signing_bytes_canon --test atom_hash_binding --test physics_invariants
```

---

## 📚 Documentação Normativa

**Binding:**
- `specs/ubl-atom/UBL-ATOM-BINDING.md` - JSON✯Atomic v1.0 binding

**Specs FROZEN (não modificar):**
- Localização: `/Users/voulezvous/UBL WAR/UBL-backend/SPEC-UBL-CORE v1.ini`

**Manifesto:**
- `SPEC_MANIFEST.json` - Inventário completo
- `DOCS_MANIFEST.sha256.txt` - Hashes de documentação

---

## ⚠️ Invariantes Críticos

1. **atom_hash = JSON✯Atomic.hash(bytes)** - SEM domain tag UBL
2. **signing_bytes termina em physics_delta** - NÃO inclui pact/author/signature
3. **MembraneError tem exatamente 8 variantes** - nomes canônicos da spec
4. **physics_delta é i128** - não i64
5. **Pact é opcional** - `Option<PactProof>`
6. **IntentClass usa PascalCase** - não snake_case
7. **Ledger é append-only** - estado derivado de H
8. **Membrane é semanticamente cega** - valida física, não semântica

---

## 🎯 Próximos Passos (Pós-Tag)

1. **Runner & Receipts** - Implementar `ubl-runner` e `ExecutionReceipt`
2. **Pact Validation** - Implementar verificação de threshold/risk
3. **Mind ↔ Body Integration** - Cliente TypeScript para `ubl-cortex`
4. **Specs Formais** - Criar SPEC-UBL-PACT.md, SPEC-UBL-POLICY.md
5. **Agreements + ABAC** - Integrar com TDLN → Link

---

**UBL 2.0 está pronto para tag v2.0.0** 🚀
