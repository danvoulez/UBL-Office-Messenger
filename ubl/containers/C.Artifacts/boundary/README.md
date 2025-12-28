![boundary • * Verde (Público)](https://img.shields.io/badge/boundary-*%20Verde%20(Público)-brightgreen)

# C.Artifacts/boundary — Você está aqui

**Path:** `containers/C.Artifacts/boundary`  
**Role/Cor:** Verde (Público)  
**Zona:** LAB 256 (API) + MinIO  

## Credenciais necessárias
- **Passkey (ubl-id)**: usuário logado
- **LLM agent (escopo limitado)**: opcional para assinar como usuário


## Função
Ponte container ↔ kernel (TDLN → LINK → MEMBRANE)

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

## Dicas
- Mantenha arquivos pequenos e claros; nada de 'mega controladores'.

---
_Navegação:_ [Resumo](../../SUMMARY.md  ) · [Guia](GUIDE.md)