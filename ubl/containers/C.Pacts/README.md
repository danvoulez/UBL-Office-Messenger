![C.Pacts • * Azul (Admin)](https://img.shields.io/badge/C.Pacts-*%20Azul%20(Admin)-blue)

# 🟦 C.Pacts — Você está aqui

**Path:** `containers/C.Pacts`  
**Role/Cor:** Azul (Admin)  
**Zona:** LAB 256 (Service)  

## Credenciais necessárias
- **Passkey (ubl-id)**: Admin com **step-up**
- **Quórum PACT** quando exigido (L5/Evolution)


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
[pacts/create/rotate/revoke] --registry--> (session) --PactProof(atom_hash)--> boundary of caller
caller boundary --ubl-link+pact--> [membrane] --Accept--> [ledger]
```
