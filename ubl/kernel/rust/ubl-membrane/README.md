![ubl-membrane • * Kernel (neutro)](https://img.shields.io/badge/ubl-membrane-*%20Kernel%20(neutro)-lightgrey)

# ubl-membrane — Você está aqui

**Path:** `kernel/rust/ubl-membrane`  
**Role/Cor:** Kernel (neutro)  
**Zona:** LAB 256 (build)  

## Credenciais necessárias
- Build standard; sem credenciais em tempo de compilação.


## Função
Valida fisicamente o link: Accept | MembraneError

## Entradas permitidas (Inbound)
- Funções chamadas pelos containers via API/SDK

## Saídas permitidas (Outbound)
- Postgres (apenas via ubl-ledger) quando aplicável

## Dados que passam por aqui
- Bytes canônicos, hashes, erros canônicos

## Dicas
- Snapshot tests: qualquer byte fora de lugar tem que quebrar os testes.

---
_Navegação:_ [Resumo](../../SUMMARY.md  ) · [Guia](GUIDE.md)