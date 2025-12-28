![C.Runner • * Preto (Execução)](https://img.shields.io/badge/C.Runner-*%20Preto%20(Execução)-000000)

# ⬛️ C.Runner — Você está aqui

**Path:** `containers/C.Runner`  
**Role/Cor:** Preto (Execução)  
**Zona:** LAB 512 (Sandbox)  

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
[ledger Accept] --trigger--> [queue] --sandbox--> (artifacts) --Receipt--> [ubl-link] --Accept--> [ledger]
```
