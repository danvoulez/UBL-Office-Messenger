# SPEC-UBL-MEMBRANE v1.0

**UBL Physical Validation Layer**

**Status:** FROZEN / NORMATIVE  
**Date:** 2025-12-25  
**Governed by:** SPEC-UBL-CORE v1.0  
**Applies to:** ubl-link, ubl-ledger  
**Independent of:** semântica, política, execução

---

## 1. Definição

A `ubl-membrane` é a camada física do UBL responsável por decidir, de forma determinística, síncrona e definitiva, se um `ubl-link` pode atravessar a fronteira entre potencial e realidade.

Formalmente:

```
Membrane : Link → {Accept, Reject(error)}
```

A decisão da membrana é final e irreversível.

## 2. Princípio Fundamental

**A membrana não entende intenção.**  
**Ela aplica leis físicas.**

Ela não interpreta significado, não executa código, não consulta agentes, não prevê consequências.

## 3. Escopo da Membrana

A membrana governa exclusivamente:
- Integridade criptográfica
- Causalidade temporal
- Conservação física
- Autoridade explícita
- Evolução declarada

Ela não governa:
- política (TDLN),
- semântica,
- execução,
- projeções de estado.

## 4. Entrada Canônica

A membrana DEVE receber exatamente um `ubl-link` válido segundo SPEC-UBL-LINK.

Nenhum outro input é permitido.

## 5. Ordem Obrigatória de Validação

A membrana DEVE executar as validações estritamente nesta ordem.  
Falha em qualquer etapa interrompe o processo.

### V1 — Versão do Protocolo

```
link.version == SUPPORTED_VERSION
```

Falha → `InvalidVersion`

### V2 — Integridade da Assinatura

```
verify(
  link.signature,
  link.author_pubkey,
  signing_bytes(link)
)
```

Falha → `InvalidSignature`

### V3 — Identidade do Container

```
link.container_id == ledger.container_id
```

Falha → `InvalidTarget`

### V4 — Causalidade (Reality Drift)

```
link.previous_hash == ledger.last_hash
```

Falha → `RealityDrift`

Sem retry automático.  
O chamador deve reconstruir o estado.

### V5 — Sequência Causal

```
link.expected_sequence == ledger.sequence + 1
```

Falha → `SequenceMismatch`

### V6 — Classe Física

Verificar coerência entre:

```
link.intent_class ↔ link.physics_delta
```

**Regras mínimas:**

| Classe | Regra |
|--------|-------|
| `Observation` | `delta == 0` |
| `Conservation` | `delta ≠ 0` |
| `Entropy` | `delta ≠ 0` |
| `Evolution` | `delta == 0` |

Falha → `PhysicsViolation`

### V7 — Conservação / Entropia

**Conservation**

Para `intent_class == Conservation`:
- a soma algébrica dos deltas pareados DEVE ser zero
- o saldo atual DEVE suportar o delta negativo

Falha → `PhysicsViolation`

**Entropy**

Para `intent_class == Entropy`:
- pacto DEVE estar presente
- pacto DEVE autorizar criação/destruição

Falha → `PactViolation`

### V8 — Evolução da Física

Para `intent_class == Evolution`:
- pacto OBRIGATÓRIO
- pacto DEVE ter `risk_level == L5`
- nova física DEVE ser explicitamente declarada

Falha → `UnauthorizedEvolution`

### V9 — Validação do Pacto (se presente)

Delegado integralmente a SPEC-UBL-PACT.

Falha → `PactViolation`

## 6. Decisão Final

Se todas as validações forem satisfeitas:

```
return Accept
```

Caso contrário:

```
return Reject(error)
```

Nenhum estado intermediário é permitido.

## 7. Efeitos da Aceitação

Quando a membrana retorna `Accept`:
- O `ubl-link` DEVE ser anexado ao `ubl-ledger`
- Um novo hash causal DEVE ser gerado
- Um `MaterializationReceipt` DEVE ser emitido

A membrana NÃO executa esses passos — ela apenas autoriza.

## 8. Erros Canônicos da Membrana

```rust
enum MembraneError {
  InvalidVersion,
  InvalidSignature,
  InvalidTarget,
  RealityDrift,
  SequenceMismatch,
  PhysicsViolation,
  PactViolation,
  UnauthorizedEvolution,
}
```

## 9. Invariantes da Membrana

- Determinismo absoluto
- Ordem fixa de validação
- Zero semântica
- Zero side effects
- Decisão síncrona
- Reprodutibilidade offline

## 10. Proibições Absolutas

A membrana NÃO PODE:
- ler `ubl-atom`
- interpretar JSON
- acessar política TDLN
- executar código
- observar execução
- inferir intenção
- corrigir dados

## 11. Propriedade de Segurança Central

**Se dois nós executarem a mesma membrana sobre o mesmo link e o mesmo ledger, o resultado será idêntico.**

Essa propriedade é mais importante que performance.

## 12. Definição Canônica

**A membrana é o limite físico da realidade no UBL.**  
**Nada cruza sem obedecer às leis.**

---

*Este documento está congelado como parte do UBL v1.0 FROZEN / NORMATIVE.*  
*Nenhuma alteração é permitida sem bump de versão explícito.*
