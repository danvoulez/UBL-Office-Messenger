![boundary • * Azul (Work Tracking)](https://img.shields.io/badge/boundary-*%20Azul%20(Work%20Tracking)-blue)

# C.Jobs/boundary — Você está aqui

**Path:** `containers/C.Jobs/boundary`  
**Role/Cor:** Azul (Work Tracking)  
**Zona:** LAB 256 (API) + 8GB (dev)  

## Credenciais necessárias
- **Passkey (ubl-id)**: usuário logado
- **LLM agent (escopo limitado)**: opcional para assinar como usuário

## Função
Ponte container ↔ kernel (TDLN → LINK → MEMBRANE)

Converte drafts de jobs em ubl-links e commita ao ledger.

## Entradas permitidas (Inbound)
- Drafts vindos de `local/`
- PactProof de `pacts/` quando necessário

## Saídas permitidas (Outbound)
- `signing_bytes` → assinatura → `validate` → `commit`

## Dados que passam por aqui
- `LinkCommit` assinado, `MembraneError`/`Accept`

## Done if…
- Sem acesso direto a DB.
- Logs mínimos, sem PII desnecessária.
- Todos drafts são convertidos corretamente para ubl-links

## Dicas
- Mantenha arquivos pequenos e claros; nada de 'mega controladores'.
- Job drafts devem ser validados antes de commit

