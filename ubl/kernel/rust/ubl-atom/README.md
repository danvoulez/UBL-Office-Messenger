![ubl-atom • * Kernel (neutro)](https://img.shields.io/badge/ubl-atom-*%20Kernel%20(neutro)-lightgrey)

# ubl-atom — Você está aqui

**Path:** `kernel/rust/ubl-atom`  
**Role/Cor:** Kernel (neutro)  
**Zona:** LAB 256 (build)  

## Credenciais necessárias
- Build standard; sem credenciais em tempo de compilação.


## Função
Brilho do JSON✯Atomic: canonicalize() + atom_hash()

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