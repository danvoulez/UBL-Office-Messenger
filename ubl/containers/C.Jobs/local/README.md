![local • * Azul (Work Tracking)](https://img.shields.io/badge/local-*%20Azul%20(Work%20Tracking)-blue)

# C.Jobs/local — Você está aqui

**Path:** `containers/C.Jobs/local`  
**Role/Cor:** Azul (Work Tracking)  
**Zona:** LAB 256 (API) + 8GB (dev)  

## Credenciais necessárias
- **Passkey (ubl-id)**: usuário logado
- **LLM agent (escopo limitado)**: opcional para assinar como usuário

## Função
Usuário/UX & validações leves

HTTP handlers para queries de jobs e aprovações.

## Entradas permitidas (Inbound)
- HTTP da UI/app
- Inputs de formulário

## Saídas permitidas (Outbound)
- Chamada interna para `boundary/` (para commits)
- Chamada para `projections/` (para queries)
- NUNCA falar com DB

## Dados que passam por aqui
- Drafts de intents (memória efêmera)
- Queries para projections

## Done if…
- Sem acesso direto a DB.
- Logs mínimos, sem PII desnecessária.
- Queries usam projections, não DB direto

## Dicas
- Mantenha arquivos pequenos e claros; nada de 'mega controladores'.
- Validações leves apenas (validação pesada no boundary)

