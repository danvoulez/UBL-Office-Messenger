# SPEC-UBL-LINK v1.0

**UBL Tangency / Commit Interface**

**Status:** FROZEN / NORMATIVE  
**Date:** 2025-12-25  
**Governed by:** SPEC-UBL-CORE v1.0

---

## 1. Definição

`ubl-link` é o único protocolo válido para materialização de efeitos entre containers no sistema UBL.

Nenhuma modificação de estado, execução, side-effect ou projeção externa é válida sem um `ubl-link` aceito.

## 2. Papel Sistêmico

O `ubl-link` representa o ponto de tangência entre:
- **Mente** (semântica local, intenção, linguagem arbitrária)
- **Corpo** (física, causalidade, conservação, história)

O `ubl-link` não transporta semântica.  
Ele transporta prova de tradução.

## 3. Estrutura Canônica (Envelope)

### 3.1 Estrutura Lógica

```
Link := ⟨
  version,
  container_id,
  expected_sequence,
  previous_hash,
  atom_hash,
  intent_class,
  physics_delta,
  pact,
  author_pubkey,
  signature
⟩
```

### 3.2 Definição de Campos

| Campo | Tipo | Obrigatório | Descrição |
|-------|------|-------------|-----------|
| `version` | `u8` | sim | Versão do protocolo (0x01) |
| `container_id` | `Hash₃₂` | sim | Identidade física do container alvo |
| `expected_sequence` | `u64` | sim | Controle causal otimista |
| `previous_hash` | `Hash₃₂` | sim | Último hash aceito no ledger |
| `atom_hash` | `Hash₃₂` | sim | Hash do ubl-atom |
| `intent_class` | `enum` | sim | Classe física da intenção |
| `physics_delta` | `i128` | sim | Delta físico (conservação/entropia) |
| `pact` | `PactProof` | opcional | Prova de consenso coletivo |
| `author_pubkey` | `PubKey₃₂` | sim | Autor primário |
| `signature` | `Sig₆₄` | sim | Assinatura Ed25519 |

## 4. IntentClass (Classes Físicas)

```rust
enum IntentClass {
  Observation = 0x00,
  Conservation = 0x01,
  Entropy = 0x02,
  Evolution = 0x03,
}
```

### 4.1 Restrições Obrigatórias

| Classe | Restrição Física |
|--------|------------------|
| `Observation` | `physics_delta == 0` |
| `Conservation` | `Σ(delta) == 0` (pareamento obrigatório) |
| `Entropy` | `delta ≠ 0` autorizado por pacto |
| `Evolution` | altera explicitamente Φ |

Violação resulta em rejeição determinística.

## 5. Conteúdo Assinado

A assinatura DEVE cobrir exatamente:

```
signing_bytes :=
  version ||
  container_id ||
  expected_sequence ||
  previous_hash ||
  atom_hash ||
  intent_class ||
  physics_delta
```

- Ordem fixa
- Big-endian
- Nenhum campo opcional incluído

## 6. Validação na Membrana

A função:

```
validate(Link) → Accept | Reject(error)
```

DEVE executar as verificações nesta ordem:

1. Versão
2. Integridade da Assinatura
3. Causalidade (previous_hash)
4. Sequência
5. Classe Física
6. Conservação / Entropia
7. Pacto (se presente)

## 7. Erros Canônicos

### 7.1 Enumeração

```rust
enum TangencyError {
  InvalidVersion,
  InvalidSignature,
  RealityDrift,
  SequenceMismatch,
  PhysicsViolation,
  PactViolation,
  UnauthorizedEvolution,
}
```

### 7.2 Semântica dos Erros

| Erro | Significado | Retry |
|------|-------------|-------|
| `InvalidVersion` | Cliente incompatível | ❌ |
| `InvalidSignature` | Fraude ou bug crítico | ❌ |
| `RealityDrift` | Estado local obsoleto | ✅ |
| `SequenceMismatch` | Replay ou race | ❌ |
| `PhysicsViolation` | Violação de conservação | ❌ |
| `PactViolation` | Assinaturas insuficientes | ❌ |
| `UnauthorizedEvolution` | Tentativa ilegal de mutação Φ | ❌ |

## 8. Aceitação e Commit

Se aceito:
- O Link DEVE ser anexado ao `ubl-ledger`
- Um novo `final_hash` DEVE ser derivado
- Um `MaterializationReceipt` DEVE ser emitido

## 9. Receipt de Materialização

```rust
struct MaterializationReceipt {
  container_id: Hash32,
  sequence: u64,
  final_hash: Hash32,
  timestamp_unix_ns: u128,
  merkle_root: Hash32,
}
```

### 9.1 Invariante

Nenhum estado local pode ser atualizado sem um receipt válido.

## 10. Proibições Explícitas

O `ubl-link` NÃO PODE:
- transportar JSON
- transportar semântica
- transportar código
- ser parcialmente validado
- ser reinterpretado

## 11. Axiomas do ubl-link

- Um link é indivisível.
- Um link é definitivo.
- Um link é verificável offline.
- Um link não carrega intenção — apenas prova.
- Um link é o único portal entre mundos.

## 12. Definição Canônica

**ubl-link é a unidade mínima de realidade no UBL.**  
**Tudo que existe fora de um link é potencial.**

---

*Este documento está congelado como parte do UBL v1.0 FROZEN / NORMATIVE.*  
*Nenhuma alteração é permitida sem bump de versão explícito.*
