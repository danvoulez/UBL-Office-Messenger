![local • * Azul (Admin)](https://img.shields.io/badge/local-*%20Azul%20(Admin)-blue)

# C.Pacts/local — Você está aqui

**Path:** `containers/C.Pacts/local`  
**Role/Cor:** Azul (Admin)  
**Zona:** LAB 256 (Service)  

## Credenciais necessárias
- **Passkey (ubl-id)**: Admin com **step-up**
- **Quórum PACT** quando exigido (L5/Evolution)


## Função
Usuário/UX & validações leves

## Entradas permitidas (Inbound)
- HTTP da UI/app
- Inputs de formulário


## Saídas permitidas (Outbound)
- Chamada interna para `boundary/`
- NUNCA falar com DB


## Dados que passam por aqui
- Drafts de intents (memória efêmera)


## Done if…
- Sem acesso direto a DB.
- Logs mínimos, sem PII desnecessária.

## Dicas
- Mantenha arquivos pequenos e claros; nada de 'mega controladores'.

---
_Navegação:_ [Resumo](../../SUMMARY.md  ) · [Guia](GUIDE.md)