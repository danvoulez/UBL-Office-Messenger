![outbox • * Azul (Work Tracking)](https://img.shields.io/badge/outbox-*%20Azul%20(Work%20Tracking)-blue)

# C.Jobs/outbox — Você está aqui

**Path:** `containers/C.Jobs/outbox`  
**Role/Cor:** Azul (Work Tracking)  
**Zona:** LAB 256 (API) + 8GB (dev)  

## Credenciais necessárias
- **Passkey (ubl-id)**: usuário logado
- **LLM agent (escopo limitado)**: opcional para assinar como usuário

## Função
Intents locais (pré-TDLN), efêmeros

Criação de drafts de jobs antes de commit ao ledger.

## Entradas permitidas (Inbound)
- UI/UX
- Messenger backend
- Office backend

## Saídas permitidas (Outbound)
- Somente para `boundary/`

## Dados que passam por aqui
- Draft JSON (não verificável)

## Done if…
- Sem acesso direto a DB.
- Logs mínimos, sem PII desnecessária.
- Drafts são efêmeros (não persistidos)

## Dicas
- Mantenha arquivos pequenos e claros; nada de 'mega controladores'.
- Drafts são apenas preparação para TDLN

