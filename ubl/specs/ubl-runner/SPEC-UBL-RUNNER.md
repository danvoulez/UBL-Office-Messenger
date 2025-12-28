# SPEC-UBL-RUNNER v1.0

**UBL Isolated Execution & Receipt Specification**

**Status:** FROZEN / NORMATIVE  
**Date:** 2025-12-25  
**Governed by:** SPEC-UBL-CORE v1.0  
**Consumed by:** ubl-membrane, ubl-ledger  
**Independent of:** semântica, política, UI

---

## 1. Definição

O `ubl-runner` é o componente responsável por materializar efeitos externos solicitados por fatos já aceitos no UBL, produzindo recibos verificáveis que retornam ao ledger como novos fatos.

Formalmente:

```
Runner : AcceptedLink → ⟨Execution, Receipt⟩
```

O runner NÃO decide se algo pode acontecer.  
Ele apenas executa o que já foi autorizado.

## 2. Princípio Fundamental

**Execução nunca cria realidade.**  
**Execução apenas responde a fatos já materializados.**

A autoridade do runner é derivada, nunca soberana.

## 3. Escopo do Runner

O runner governa exclusivamente:
- Execução isolada de computação externa
- Captura determinística de resultados
- Emissão de recibos verificáveis

O runner NÃO governa:
- política (TDLN),
- validação física (membrana),
- causalidade,
- semântica.

## 4. Entrada Canônica

O runner DEVE receber apenas:
- um `ubl-link` já aceito e anexado ao ledger
- metadados explícitos de execução (quando aplicável)

O runner NÃO PODE executar links rejeitados, pendentes ou não ancorados.

## 5. Modelo de Execução

### 5.1 Execução Isolada

Toda execução DEVE ocorrer em ambiente isolado:
- sandbox
- WASM
- VM
- container
- enclave (opcional)

**Isolamento NÃO É opcional.**

### 5.2 Determinismo Parcial

A execução PODE ser não determinística (IO, tempo, rede), porém:
- a descrição da execução
- os artefatos produzidos
- os hashes dos resultados

DEVEM ser determinísticos.

## 6. Tipos de Execução

Implementações PODEM suportar:
- execução de código (scripts, binaries)
- deploys
- chamadas externas (APIs)
- operações físicas (IoT)

O tipo DEVE ser declarado explicitamente no link ou no receipt.

## 7. Receipt — Unidade de Prova de Execução

### 7.1 Definição

Cada execução DEVE produzir exatamente um receipt:

```
ExecutionReceipt := ⟨
  container_id,
  trigger_link_hash,
  execution_id,
  status,
  artifacts,
  stdout_hash?,
  stderr_hash?,
  started_at,
  finished_at
⟩
```

### 7.2 Campos

| Campo | Descrição |
|-------|-----------|
| `container_id` | Container associado |
| `trigger_link_hash` | Link que causou a execução |
| `execution_id` | Identidade única |
| `status` | Success \| Failure |
| `artifacts` | Lista de artefatos produzidos |
| `stdout_hash` | Hash opcional |
| `stderr_hash` | Hash opcional |
| `started_at` | Timestamp |
| `finished_at` | Timestamp |

## 8. Artefatos

### 8.1 Definição

Um artefato é definido como:

```
Artifact := ⟨
  artifact_id,
  type,
  size,
  content_hash,
  metadata?
⟩
```

Artefatos NÃO entram diretamente no ledger.  
Apenas seus hashes entram.

## 9. Ancoragem do Receipt

### 9.1 Regra Obrigatória

Todo receipt DEVE:
1. Ser reduzido a um `ubl-atom`
2. Gerar um novo `ubl-link`
3. Passar novamente pela membrana
4. Ser anexado ao ledger como novo fato

**Execução não altera fatos anteriores.**

## 10. Falhas de Execução

### 10.1 Falha Controlada

Se a execução falhar:
- o receipt DEVE ser emitido
- `status = Failure`
- nenhuma correção automática ocorre

**Falha é fato, não exceção.**

### 10.2 Falha do Runner

Se o runner falhar antes de emitir receipt:
- o sistema DEVE permitir retry
- retries NÃO PODEM duplicar efeitos

**Idempotência é obrigatória.**

## 11. Invariantes do Runner

- Execução só após commit
- Um link → no máximo uma execução
- Uma execução → exatamente um receipt
- Receipt é fato imutável
- Runner nunca decide autoridade
- Runner nunca altera passado

## 12. Erros Canônicos do Runner

```rust
enum RunnerError {
  InvalidTrigger,
  ExecutionFailed,
  ArtifactViolation,
  ReceiptCommitFailed,
}
```

## 13. Proibições Absolutas

O runner NÃO PODE:
- rejeitar um link aceito
- executar sem isolamento
- modificar ledger diretamente
- corrigir erros por mutação
- interpretar política
- inferir intenção

## 14. Propriedade de Segurança Central

**Mesmo se todos os runners falharem, o ledger permanece correto.**

Execução é periférica.  
História é central.

## 15. Definição Canônica

**O runner é o braço do sistema.**  
**Ele age, mas não decide.**

---

*Este documento está congelado como parte do UBL v1.0 FROZEN / NORMATIVE.*  
*Nenhuma alteração é permitida sem bump de versão explícito.*
