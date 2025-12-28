# SPEC-UBL-LEDGER v1.0

**UBL Immutable Ledger Specification**

**Status:** FROZEN / NORMATIVE  
**Date:** 2025-12-25  
**Governed by:** SPEC-UBL-CORE v1.0  
**Consumed by:** ubl-membrane, ubl-runner, ubl-cortex  
**Independent of:** storage engine, database, filesystem

---

## 1. Definição

O `ubl-ledger` é a memória imutável e causal do sistema UBL.

Ele é a única fonte de verdade sobre o que ocorreu em um container.  
Todo estado observável DEVE ser derivável exclusivamente do ledger.

Formalmente:

```
Ledger := ⟨C, H, I⟩
```

onde:
- `C` = container associado
- `H` = história imutável de eventos
- `I` = índices derivados (opcionais)

## 2. Princípio Fundamental

**O ledger não guarda estado.**  
**O ledger guarda fatos.**

Estado é sempre uma projeção.  
Fatos são irreversíveis.

## 3. Unidade Fundamental: Ledger Entry

### 3.1 Definição

Cada entrada do ledger é definida como:

```
LedgerEntry := ⟨
  container_id,
  sequence,
  link_hash,
  previous_hash,
  timestamp,
  merkle_path?,
  metadata?
⟩
```

### 3.2 Campos Obrigatórios

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `container_id` | `Hash₃₂` | Identidade do container |
| `sequence` | `u64` | Ordem causal estrita |
| `link_hash` | `Hash₃₂` | Hash do ubl-link aceito |
| `previous_hash` | `Hash₃₂` | Hash da entrada anterior |
| `timestamp` | `u128` | Tempo físico do commit |

Campos opcionais NÃO participam da causalidade.

## 4. Cadeia Causal

### 4.1 Regra de Encadeamento

Para qualquer container C:

```
H = [e₁, e₂, ..., eₙ]
```

onde:

```
e₁.previous_hash == 0x00…00
eᵢ.previous_hash == hash(eᵢ₋₁)
eᵢ.sequence == i
```

Violação em qualquer ponto DEVE invalidar o ledger.

### 4.2 Consequência

O ledger define uma linha do tempo única, total e não ramificável por container.

**Forks não existem dentro de um container.**

## 5. Hash da Entrada

### 5.1 Definição

O hash de uma entrada é definido como:

```
entry_hash := BLAKE3(
  "ubl:ledger\n" ||
  container_id ||
  sequence ||
  link_hash ||
  previous_hash ||
  timestamp
)
```

### 5.2 Propriedades

- Determinístico
- Ordenável
- Offline-verificável
- Independente de storage

## 6. Imutabilidade

### 6.1 Proibição Absoluta

Uma vez anexada, uma `LedgerEntry`:
- NÃO PODE ser modificada
- NÃO PODE ser removida
- NÃO PODE ser sobrescrita

### 6.2 Correção de Erros

Erros NUNCA são corrigidos por mutação.  
Correções ocorrem apenas por novas entradas compensatórias.

## 7. Inserção (Append)

### 7.1 Regra de Ouro

Uma nova entrada SÓ PODE ser anexada se:
- O `ubl-link` foi aceito pela membrana
- `sequence == last.sequence + 1`
- `previous_hash == last.entry_hash`

### 7.2 Atomicidade

O append DEVE ser atômico:

```
validate → append → commit
```

Nenhum estado intermediário é permitido.

## 8. Índices e Projeções

### 8.1 Índices (I)

Índices são derivados, não canônicos.

**Exemplos:**
- lookup por sequence
- lookup por intervalo temporal
- lookup por link_hash

### 8.2 Projeções de Estado

Projeções são funções puras:

```
State := fold(H)
```

Projeções:
- PODEM falhar
- PODEM ser reexecutadas
- NUNCA alteram o ledger

## 9. Merkle Anchoring (Opcional)

### 9.1 Blocos

Implementações PODEM agrupar entradas em blocos:

```
Block := MerkleTree(entries)
```

### 9.2 Prova de Inclusão

Se usado, o ledger DEVE fornecer:
- `merkle_root`
- `merkle_path`

## 10. Verificação Offline

Qualquer parte DEVE poder verificar:
- Integridade da cadeia
- Sequência
- Hashes
- Provas de inclusão (se aplicável)

Sem acesso a:
- rede
- semântica
- política

## 11. Erros Canônicos do Ledger

```rust
enum LedgerError {
  BrokenChain,
  SequenceViolation,
  InvalidHash,
  AppendOutOfOrder,
}
```

## 12. Proibições Explícitas

O ledger NÃO PODE:
- interpretar `ubl-link`
- validar política
- executar código
- corrigir estado
- inferir intenção
- compactar história semanticamente

## 13. Invariantes do Ledger

- Append-only
- Ordem causal total
- Imutabilidade absoluta
- Derivação determinística de estado
- Verificação offline possível

## 14. Definição Canônica

**O ubl-ledger é a memória factual do sistema.**  
**O que não está no ledger nunca aconteceu.**

---

*Este documento está congelado como parte do UBL v1.0 FROZEN / NORMATIVE.*  
*Nenhuma alteração é permitida sem bump de versão explícito.*
