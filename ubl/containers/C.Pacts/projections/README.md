![projections • * Azul (Admin)](https://img.shields.io/badge/projections-*%20Azul%20(Admin)-blue)

# C.Pacts/projections — Você está aqui

**Path:** `containers/C.Pacts/projections`  
**Role/Cor:** Azul (Admin)  
**Zona:** LAB 256 (Service)  

## Credenciais necessárias
- **Passkey (ubl-id)**: Admin com **step-up**
- **Quórum PACT** quando exigido (L5/Evolution)


## Função
Estados derivados somente-leitura

## Entradas permitidas (Inbound)
- Tail do ledger + fold


## Saídas permitidas (Outbound)
- Servem a UI/SDK


## Dados que passam por aqui
- Views reconstituíveis


## Done if…
- Sem acesso direto a DB.
- Logs mínimos, sem PII desnecessária.

## Dicas
- Mantenha arquivos pequenos e claros; nada de 'mega controladores'.

---
_Navegação:_ [Resumo](../../SUMMARY.md  ) · [Guia](GUIDE.md)