![C.Messenger • * Verde (Público)](https://img.shields.io/badge/C.Messenger-*%20Verde%20(Público)-brightgreen)

# 🟩 C.Messenger — Você está aqui

**Path:** `containers/C.Messenger`  
**Role/Cor:** Verde (Público)  
**Zona:** LAB 256 (API) + 8GB (dev)  

## Credenciais necessárias
- **Passkey (ubl-id)**: usuário logado
- **LLM agent (escopo limitado)**: opcional para assinar como usuário


## Função
Placa de orientação do container. Leia o README principal para rotas e Done if.

## Entradas permitidas (Inbound)
- requests do cliente/serviços confiáveis
- SSE do ledger

## Saídas permitidas (Outbound)
- kernel (signing_bytes/validate/commit)
- outros containers via LINKS (nunca por import)

## Dados que passam por aqui
- Drafts, Links, Eventos do ledger

## Done if…
- README principal passa nos critérios de Done if e testes verdes.

## Dicas
- Nunca importe outro container; somente `@kernel/*` e tipos do OpenAPI.
## Mapa da Fronteira
```
[local] --draft--> [boundary] --signing_bytes--> [ubl-link] --(signature)--> [membrane]
                                                            \--Accept--> [ledger] --tail--> [projections]
```
