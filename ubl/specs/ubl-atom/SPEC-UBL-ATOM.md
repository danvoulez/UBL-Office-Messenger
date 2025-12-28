# SPEC-UBL-ATOM v1.0

**UBL Canonical Atomic Data Format**

**Status:** FROZEN / NORMATIVE  
**Date:** 2025-12-25  
**Governed by:** SPEC-UBL-CORE v1.0  
**Consumed by:** SPEC-UBL-LINK v1.0

---

## 1. Definição

`ubl-atom` é o único formato de dados canônico permitido no UBL.

Todo significado que pretende tornar-se fato DEVE ser reduzido a um `ubl-atom` antes de qualquer assinatura, pacto ou materialização.

## 2. Objetivo

Garantir que:
- Dois significados semanticamente equivalentes gerem bytes idênticos
- Um mesmo conjunto de bytes represente um único fato
- O hash de um átomo seja estável, verificável e universal

## 3. Domínio de Aplicação

`ubl-atom` é usado para:
- geração de `atom_hash` no `ubl-link`
- ancoragem de significado local
- prova de equivalência semântica
- auditoria offline
- reexecução determinística de projeções

## 4. Definição Formal

### 4.1 Espaço de Valores Permitidos

Um `ubl-atom` DEVE ser derivado de um JSON que satisfaça:

**Tipos permitidos:**
- `null`
- `boolean`
- `string` (UTF-8)
- `number` (inteiro ou decimal finito)
- `array`
- `object`

**Tipos proibidos:**
- `NaN`
- `Infinity`
- `-Infinity`
- `undefined`
- `function`
- `symbol`
- referências cíclicas

Violação DEVE resultar em erro.

## 5. Canonicalização

### 5.1 Função Canônica

```
canonicalize : JSON → Bytes
```

A função DEVE aplicar exatamente as seguintes regras, nesta ordem:

### 5.2 Regras de Canonicalização

**R1 — Ordenação de Objetos**
- Todas as chaves de objetos DEVEM ser ordenadas lexicograficamente (UTF-8 byte order).
- A ordenação É SENSÍVEL A CASE.
- Exemplo: `{ "b": 1, "a": 2 }` → `{ "a": 2, "b": 1 }`

**R2 — Preservação de Arrays**
- Arrays NÃO DEVEM ser reordenados.
- A ordem é semanticamente significativa.

**R3 — Normalização Numérica**
- Apenas números finitos são permitidos.
- Inteiros NÃO DEVEM ser convertidos em floats.
- Decimais DEVEM ser serializados sem notação científica.
- Exemplo proibido: `1e3`
- Exemplo válido: `1000`

**R4 — Normalização de Strings**
- Strings DEVEM estar em UTF-8 normalizado (NFC).
- Nenhuma transformação semântica é permitida.

**R5 — Serialização Estrita**
Serialização DEVE ser feita em JSON compacto:
- sem espaços
- sem quebras de linha
- sem trailing commas

## 6. Resultado Canônico

O resultado final de `canonicalize` é um vetor de bytes:

```
A := UTF8(JSON.stringify(canonical_object))
```

## 7. Hash Canônico

### 7.1 Definição

O hash de um `ubl-atom` é definido como:

```
atom_hash := BLAKE3( domain_tag || A )
```

onde:
- `domain_tag = "ubl:atom\n"` (fixo)
- `A` = bytes canônicos do átomo

### 7.2 Propriedades Obrigatórias

- Determinístico
- Estável entre linguagens
- Independente de plataforma
- Verificável offline

## 8. Invariantes do ubl-atom

**I1 — Determinismo**

```
canonicalize(x) == canonicalize(y) ⇔ x ≡ y
```

**I2 — Identidade por Hash**

Dois fatos distintos NÃO PODEM compartilhar o mesmo `atom_hash`.

**I3 — Zero Semântica no Kernel**

O kernel NÃO PODE interpretar, validar ou modificar A.

## 9. Erros Canônicos

```rust
enum AtomError {
  InvalidType,
  NonFiniteNumber,
  InvalidEncoding,
  CanonicalizationFailure,
}
```

Qualquer erro DEVE impedir:
- assinatura
- pactuação
- geração de `ubl-link`

## 10. Testes de Conformidade (Obrigatórios)

Implementações DEVEM fornecer:
- Vetores de teste cross-language (TS, Rust, Python)
- Testes de equivalência semântica
- Testes de rejeição (NaN, ordering, floats)
- Golden hashes versionados

## 11. Proibições Explícitas

`ubl-atom` NÃO PODE:
- conter identidade de container
- conter assinatura
- conter sequência
- conter política
- conter código executável
- conter timestamps implícitos

## 12. Definição Canônica

**ubl-atom é a matéria digital mínima do UBL.**  
**Tudo que é real no sistema é, no fundo, um hash de um átomo.**

---

*Este documento está congelado como parte do UBL v1.0 FROZEN / NORMATIVE.*  
*Nenhuma alteração é permitida sem bump de versão explícito.*
