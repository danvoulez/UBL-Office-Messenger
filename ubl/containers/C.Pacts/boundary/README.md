![boundary • * Azul (Admin)](https://img.shields.io/badge/boundary-*%20Azul%20(Admin)-blue)

# C.Pacts/boundary — Você está aqui

**Path:** `containers/C.Pacts/boundary`  
**Role/Cor:** Azul (Admin)  
**Zona:** LAB 256 (Service)  

## Credenciais necessárias
- **Passkey (ubl-id)**: Admin com **step-up**
- **Quórum PACT** quando exigido (L5/Evolution)


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