# UBL-ATOM-BINDING v1.0

**Title:** UBL Atom Binding to JSON✯Atomic v1.0  
**Status:** NORMATIVE  
**Change-Control:** STRICT (no retroactive changes)  
**Hash:** BLAKE3 | **Signature:** Ed25519 | **Canonical Format:** Json✯Atomic  
**Freeze-Date:** 2025-12-25  
**Governed by:** SPEC-UBL-CORE v1.0, SPEC-UBL-ATOM v1.0

---

## 1. Definição

`ubl-atom` é **idêntico** aos bytes canônicos produzidos por **JSON✯Atomic v1.0**.

Formalmente:

```
ubl-atom ≡ JSON✯Atomic(input)
```

Não há transformação adicional, não há wrapper, não há metadata.

---

## 2. Propriedades Obrigatórias

### 2.1 Determinismo Absoluto

```
∀ x, y : JSON
  x ≡ y  ⟺  JSON✯Atomic(x) = JSON✯Atomic(y)
```

Dois JSONs semanticamente equivalentes **DEVEM** produzir bytes idênticos.

### 2.2 Estabilidade entre Linguagens

```
JSON✯Atomic_TypeScript(x) = JSON✯Atomic_Rust(x) = JSON✯Atomic_Python(x)
```

A canonicalização **DEVE** ser independente de linguagem.

### 2.3 Rejeição de Valores Não-Finitos

```
JSON✯Atomic(NaN)       → ERROR
JSON✯Atomic(Infinity)  → ERROR
JSON✯Atomic(-Infinity) → ERROR
```

Valores não-finitos **DEVEM** ser rejeitados antes da canonicalização.

---

## 3. Hash do Átomo

O hash de um `ubl-atom` é **idêntico** ao hash oficial do JSON✯Atomic v1.0:

```
atom_hash := JSON✯Atomic.hash(bytes_canônicos)
```

**Importante:**
- **NÃO** há domain tag UBL no cálculo do `atom_hash`
- O `atom_hash` é **exatamente** o hash que a ferramenta JSON✯Atomic v1.0 produz
- Domain tags UBL são usados apenas para ledger/link/pact/receipt, **nunca** para o átomo
- O hash é calculado **sobre os bytes canônicos**, não sobre o JSON original

---

## 4. Pinagem de Versão

Este binding está fixado em:

```
JSON✯Atomic-Version: 1.0
```

Qualquer mudança na especificação JSON✯Atomic **REQUER** nova versão deste binding.

---

## 5. Proibições Explícitas

`ubl-atom` **NÃO PODE**:

- ❌ Conter identidade de container
- ❌ Conter assinatura
- ❌ Conter sequência
- ❌ Conter política
- ❌ Conter código executável
- ❌ Conter timestamps implícitos
- ❌ Ser re-hashado após canonicalização
- ❌ Ser reserializado após canonicalização
- ❌ Ser interpretado pela Membrane/Kernel

---

## 6. Verificação (CLI)

### 6.1 Canonicalização

```bash
# Usando json-atomic (referência)
json-atomic canonicalize input.json > canonical.json
```

### 6.2 Hash

```bash
# Calcular atom_hash oficial
json-atomic hash canonical.json
# Output: <hash_hex>
```

### 6.3 Verificação Cross-Language

```bash
# TypeScript
npx tsx -e "import {canonicalize} from './ubl-atom'; console.log(canonicalize({z:1,a:2}))"

# Rust
cargo run -p ubl-atom --example canonicalize -- '{"z":1,"a":2}'

# Python
python3 -c "from ubl_atom import canonicalize; print(canonicalize({'z':1,'a':2}))"

# Todos DEVEM produzir: {"a":2,"z":1}
```

---

## 7. Invariantes

### I1 — Identidade por Hash

```
atom_hash(x) = atom_hash(y)  ⟹  x ≡ y
```

Dois fatos distintos **NÃO PODEM** compartilhar o mesmo `atom_hash`.

### I2 — Zero Semântica no Kernel

```
ubl-kernel.hash(bytes)  ≠  ubl-kernel.interpret(bytes)
```

O kernel **NÃO PODE** interpretar, validar ou modificar o átomo.

### I3 — Append-Only no Link

```
ubl-link.atom_hash  ⟹  ubl-atom existe e é imutável
```

O `atom_hash` no `ubl-link` **DEVE** referenciar um átomo que nunca muda.

---

## 8. Testes de Conformidade (Obrigatórios)

Implementações **DEVEM** fornecer:

1. **Vetores de teste cross-language** (TS, Rust, Python)
2. **Testes de equivalência semântica**
3. **Testes de rejeição** (NaN, Infinity, ordering)
4. **Golden hashes versionados**

Exemplo de vetor de teste:

```json
{
  "input": {"z": 1, "a": 2, "nested": {"y": 3, "x": 4}},
  "canonical": "{\"a\":2,\"nested\":{\"x\":4,\"y\":3},\"z\":1}",
  "atom_hash": "7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069"
}
```

---

## 9. Referências Normativas

- **JSON✯Atomic v1.0** — Especificação de canonicalização
- **SPEC-UBL-CORE v1.0** — Ontologia fundamental
- **SPEC-UBL-ATOM v1.0** — Definição formal do átomo
- **SPEC-UBL-LINK v1.0** — Uso do `atom_hash`

---

## 10. Definição Canônica

> `ubl-atom` é a matéria digital mínima do UBL.  
> Tudo que é real no sistema é, no fundo, um hash de um átomo.  
> O átomo não carrega semântica — apenas bytes verificáveis.

---

**Assinatura Digital:** (pendente)  
**Hash deste documento:** (pendente)
