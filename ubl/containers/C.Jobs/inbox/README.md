![inbox • * Azul (Work Tracking)](https://img.shields.io/badge/inbox-*%20Azul%20(Work%20Tracking)-blue)

# C.Jobs/inbox — Você está aqui

**Path:** `containers/C.Jobs/inbox`  
**Role/Cor:** Azul (Work Tracking)  
**Zona:** LAB 256 (API) + 8GB (dev)  

## Credenciais necessárias
- **Passkey (ubl-id)**: usuário logado
- **LLM agent (escopo limitado)**: opcional para assinar como usuário

## Função
Eventos JÁ no ledger para este container

Processa eventos do SSE tail e atualiza projections.

## Entradas permitidas (Inbound)
- SSE tail do ledger

## Saídas permitidas (Outbound)
- Atualiza projeções locais

## Dados que passam por aqui
- Átomos aceitos + metadados de commit (job.*, approval.*)

## Done if…
- Sem acesso direto a DB.
- Logs mínimos, sem PII desnecessária.
- Eventos são processados e projections atualizados

## Dicas
- Mantenha arquivos pequenos e claros; nada de 'mega controladores'.
- Processar eventos em ordem (sequence)

