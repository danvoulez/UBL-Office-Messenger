![local • * Preto (Execução)](https://img.shields.io/badge/local-*%20Preto%20(Execução)-000000)

# C.Runner/local — Você está aqui

**Path:** `containers/C.Runner/local`  
**Role/Cor:** Preto (Execução)  
**Zona:** LAB 512 (Sandbox)  

## Credenciais necessárias
- **Passkey (ubl-id)**: usuário logado
- **LLM agent (escopo limitado)**: opcional para assinar como usuário


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