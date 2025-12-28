![projections • * Azul (Work Tracking)](https://img.shields.io/badge/projections-*%20Azul%20(Work%20Tracking)-blue)

# C.Jobs/projections — Você está aqui

**Path:** `containers/C.Jobs/projections`  
**Role/Cor:** Azul (Work Tracking)  
**Zona:** LAB 256 (API) + 8GB (dev)  

## Credenciais necessárias
- **Passkey (ubl-id)**: usuário logado
- **LLM agent (escopo limitado)**: opcional para assinar como usuário

## Função
Estados derivados somente-leitura

Deriva estado de jobs e aprovações a partir dos eventos do ledger.

## Entradas permitidas (Inbound)
- Tail do ledger + fold

## Saídas permitidas (Outbound)
- Servem a UI/SDK

## Dados que passam por aqui
- Views reconstituíveis (jobs, aprovações, status)

## Queries Suportadas

- Listar jobs (filtro por status, assignee, conversation_id)
- Obter detalhes de job
- Listar aprovações pendentes
- Obter eventos de job
- Obter receipt criptográfico

## Done if…
- Sem acesso direto a DB.
- Logs mínimos, sem PII desnecessária.
- Estado é sempre derivável do ledger

## Dicas
- Mantenha arquivos pequenos e claros; nada de 'mega controladores'.
- Projections são read-only, sempre derivadas de eventos

