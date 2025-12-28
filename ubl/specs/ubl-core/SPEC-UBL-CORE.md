# SPEC-UBL-CORE v1.0

**Universal Business Ledger — Core Specification**

**Status:** FROZEN / NORMATIVE  
**Version:** 1.0  
**Date:** 2025-12-25  
**Escopo:** Ontologia, entidades fundamentais e axiomas invariantes do UBL  
**Precedência:** Este documento precede e governa todas as outras specs (ubl-atom, ubl-link, ubl-pact, ubl-policy)

---

## 1. Definição Formal do Sistema

O **Universal Business Ledger (UBL)** é um sistema distribuído composto por Containers Universais independentes, conectados exclusivamente por traduções determinísticas verificáveis (TDLN), no qual:

- Nenhum container compartilha estado interno com outro.
- Toda interação entre containers ocorre apenas via commit verificável.
- A verdade do sistema é definida por causalidade + prova criptográfica.
- Semântica nunca atravessa fronteiras; apenas provas atravessam.

---

## 2. Container Universal

### 2.1 Definição

Um Container Universal é definido pelo quíntuplo:

```
C := ⟨id, L, S, H, Φ⟩
```

onde:

- **id ∈ Hash₃₂** — Identidade física e estável do container.
- **L — Linguagem Local** — Sistema semântico interno arbitrário. Pode ser humano, probabilístico, simbólico ou assistido por IA.
- **S — Estado Local** — Estado derivável exclusivamente da história H.
- **H — História** — Sequência causal imutável de commits aceitos.
- **Φ — Física** — Conjunto mínimo de invariantes globais verificáveis.

### 2.2 Invariantes do Container

- **S NÃO PODE ser modificado diretamente.** Apenas projeções de H são válidas.
- **L NÃO É compartilhável nem verificável externamente.**
- **Todo efeito observável fora do container DEVE estar ancorado em H.**

---

## 3. Linguagem Local e Semântica

### 3.1 Linguagem Local (L)

Cada container define uma função interna:

```
Lᵢ : Intent → Meaning
```

Características:
- Não determinística (permitido)
- Evolutiva (permitido)
- Incompleta ou ambígua (permitido)

UBL não impõe restrições à linguagem local.

### 3.2 Consequência Fundamental

**Semântica não é verificável.**  
Logo, semântica não cruza fronteiras.

---

## 4. ubl-atom — Matéria Digital Canônica

### 4.1 Definição

`ubl-atom` é a única representação universal de dados no UBL.

Formalmente:
```
A := canonicalize(JSON) → Bytes
```

Propriedades obrigatórias:
- Canonicalização determinística
- Ordem total de campos
- Rejeição de valores não finitos
- Estabilidade de bytes entre linguagens

### 4.2 Axioma do Átomo

- Dois significados diferentes PODEM gerar o mesmo átomo.
- Dois átomos iguais NUNCA representam fatos diferentes.

---

## 5. TDLN — Deterministic Translation of Language to Notation

### 5.1 Definição

TDLN é a função que traduz significado local em fato verificável:

```
TDLN : L → ⟨A, h, π⟩
```

onde:
- **A ∈ ubl-atom**
- **h = Hash(A)**
- **π = conjunto de provas** (assinaturas, pactos, políticas)

### 5.2 Propriedade de Isolamento

O verificador de TDLN:
- NÃO interpreta A
- NÃO conhece L
- APENAS valida h e π

---

## 6. ubl-link — Interface Única de Materialização

### 6.1 Definição

`ubl-link` é o único protocolo válido para cruzar a fronteira entre containers.

```
Link := ⟨id_C, h, σ, Δ, κ⟩
```

onde:
- **id_C** — container alvo
- **h** — hash do ubl-atom
- **σ** — prova de autoria/autoridade
- **Δ** — delta físico
- **κ** — classe física da intenção

### 6.2 Classes Físicas (κ)

UBL reconhece exclusivamente:

| Classe | Restrição |
|--------|-----------|
| **Observation** | Δ = 0 |
| **Conservation** | ∑Δ = 0 (pareamento obrigatório) |
| **Entropy** | Δ ≠ 0 permitido mediante autorização |
| **Evolution** | altera Φ explicitamente |

---

## 7. ubl-pact — Autoridade Coletiva

### 7.1 Definição

`ubl-pact` define regras de autoridade antes da materialização.

```
Pact := {σ₁, σ₂, ..., σₙ}
```

Um ubl-link SÓ É VÁLIDO se satisfizer o pacto vigente.

### 7.2 Invariante

Nenhum pacto pode ser aplicado retroativamente.

---

## 8. ubl-membrane — Validação Física

### 8.1 Definição

A membrana é a função:

```
Membrane : Link → {Accept, Reject}
```

Ela valida exclusivamente:
- Integridade criptográfica
- Causalidade
- Invariantes físicas (Φ)

### 8.2 Proibição Absoluta

A membrana NÃO PODE:
- interpretar semântica,
- acessar JSON,
- inferir intenção.

---

## 9. ubl-ledger — História Imutável

### 9.1 Definição

O ledger é uma sequência:

```
H := [e₁, e₂, ..., eₙ]
```

onde cada `e_i` é um ubl-link aceito.

### 9.2 Propriedades

- Append-only
- Imutável
- Verificável offline
- Ordenado causalmente

---

## 10. Execução e Materialização

Alguns commits exigem execução externa:

```
Link → Execution → Receipt
```

O receipt:
- é um novo fato,
- nunca altera o passado,
- entra no ledger como evento independente.

---

## 11. Axiomas Fundamentais do UBL

1. Semântica é local.
2. Estado não é compartilhado.
3. Commit é o único efeito real.
4. Hash identifica o fato.
5. Ledger define a verdade.
6. Execução não reescreve história.
7. Tradução precede materialização.
8. Física é cega.
9. Autoridade é explícita.
10. Evolução é declarada.

---

## 12. Definição Canônica

**UBL é um sistema de Containers Universais conectados exclusivamente por TDLN, onde significado local é traduzido em fato verificável sem compartilhamento de estado.**

---

**🔒 Este documento está FROZEN. Qualquer alteração requer nova versão (v1.1, v2.0).**
