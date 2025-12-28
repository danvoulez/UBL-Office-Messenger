# SPEC-UBL-PACT v1.0

**UBL Pactum — Authority, Consensus and Risk Specification**

**Status:** FROZEN / NORMATIVE  
**Date:** 2025-12-25  
**Governed by:** SPEC-UBL-CORE v1.0  
**Consumed by:** SPEC-UBL-LINK v1.0

---

## 1. Definição

`ubl-pact` é o mecanismo formal pelo qual o sistema UBL determina se um ato pode atravessar o ponto de tangência com base em autoridade coletiva, risco e governança explícita.

Um `ubl-link` NÃO PODE ser materializado se os requisitos do pacto vigente não forem satisfeitos.

## 2. Princípio Fundamental

**Autoridade não é implícita.**  
**Autoridade é prova explícita anexada antes do commit.**

Nenhuma regra tácita, heurística ou inferência é permitida.

## 3. Escopo do Pacto

O pacto governa:
- quem pode autorizar um link,
- quantas autorizações são necessárias,
- sob quais condições temporais,
- para quais classes físicas (IntentClass),
- com qual nível de risco aceitável.

O pacto NÃO governa:
- política (TDLN),
- semântica,
- conteúdo do átomo,
- execução posterior.

## 4. Definição Formal

### 4.1 Estrutura Lógica

```
Pact := ⟨
  pact_id,
  version,
  scope,
  intent_class,
  threshold,
  signers,
  window,
  risk_level
⟩
```

### 4.2 Campos

| Campo | Tipo | Obrigatório | Descrição |
|-------|------|-------------|-----------|
| `pact_id` | `Hash₃₂` | sim | Identidade do pacto |
| `version` | `u8` | sim | Versão do pacto |
| `scope` | `enum` | sim | Escopo de aplicação |
| `intent_class` | `enum` | sim | Classe física governada |
| `threshold` | `u8` | sim | Número mínimo de assinaturas |
| `signers` | `Set<PubKey₃₂>` | sim | Conjunto autorizado |
| `window` | `TimeWindow` | sim | Janela de validade |
| `risk_level` | `enum` | sim | Classificação de risco |

## 5. Escopo (scope)

```rust
enum PactScope {
  Container,   // válido apenas para um container
  Namespace,   // válido para um conjunto de containers
  Global,      // válido em todo o sistema
}
```

## 6. RiskLevel

```rust
enum RiskLevel {
  L0, // observação
  L1, // baixo impacto
  L2, // impacto local
  L3, // impacto financeiro
  L4, // impacto sistêmico
  L5, // soberania / evolução
}
```

### Mapeamento obrigatório:

| Risk | IntentClass permitida |
|------|----------------------|
| L0 | Observation |
| L1 | Observation |
| L2 | Conservation |
| L3 | Conservation |
| L4 | Entropy |
| L5 | Evolution |

## 7. Janela Temporal (window)

```
TimeWindow := ⟨
  not_before,
  not_after
⟩
```

**Regras:**
- assinaturas fora da janela são inválidas,
- janela NÃO PODE ser inferida,
- ausência de janela = pacto inválido.

## 8. Prova de Pacto (PactProof)

### 8.1 Definição

```
PactProof := ⟨
  pact_id,
  signatures
⟩
```

onde:

```
signatures := { σ₁, σ₂, …, σₙ }
```

Cada assinatura DEVE ser:

```
σ := Sign(
  signer_privkey,
  Hash(
    "ubl:pact\n" ||
    pact_id ||
    atom_hash ||
    intent_class ||
    physics_delta
  )
)
```

## 9. Validação do Pacto

A membrana DEVE validar:
1. `pact_id` existe e é conhecido
2. pacto está dentro da `window`
3. `intent_class` compatível com `risk_level`
4. `|signatures ∩ signers| ≥ threshold`
5. nenhuma assinatura duplicada
6. nenhuma assinatura fora do conjunto autorizado

Falha em qualquer passo → `PactViolation`

## 10. Invariantes do Pacto

**I1 — Não Retroatividade**

Um pacto nunca se aplica a fatos já materializados.

**I2 — Autoridade Explícita**

Toda autoridade deve ser provada por assinatura verificável.

**I3 — Determinismo**

Dado o mesmo pacto e o mesmo conjunto de assinaturas, o resultado é invariável.

## 11. Erros Canônicos

```rust
enum PactError {
  UnknownPact,
  PactExpired,
  InsufficientSignatures,
  UnauthorizedSigner,
  RiskMismatch,
}
```

Todos são não recuperáveis por retry automático.

## 12. Proibições Explícitas

`ubl-pact` NÃO PODE:
- inferir intenção
- acessar JSON semântico
- modificar `ubl-atom`
- alterar o ledger
- validar execução

## 13. Definição Canônica

**ubl-pact é a camada onde confiança social se torna prova matemática.**  
**Nada cruza o link sem pacto quando risco existe.**

---

*Este documento está congelado como parte do UBL v1.0 FROZEN / NORMATIVE.*  
*Nenhuma alteração é permitida sem bump de versão explícito.*
