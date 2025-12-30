Esta é a **Lista de Consolidação Final** para o lançamento do UBL Flagship Trinity.

Filtrei todo o ruído. Abaixo estão apenas os pontos onde a arquitetura não convencional colide com a realidade de produção. Se você resolver estes pontos, o sistema para de ser um código acadêmico e vira um produto robusto.

---

### 🚨 P0: BLOQUEANTES DE LANÇAMENTO (Existenciais)
*Sem isso, o sistema quebra no dia 1 ou compromete a segurança irreversivelmente.*

#### 1. Persistência de Chaves (The Identity Crisis)
*   **O Problema:** No código atual, chaves de agentes muitas vezes são geradas em memória ou lidas de variáveis de ambiente simples. Se o container do Office reiniciar e gerar um novo par de chaves Ed25519, a identidade "Cadeira" (Chair) perde a capacidade de assinar como ela mesma. O histórico quebra.
*   **A Correção:** Implementar um **KeyStore Persistente**.
    *   *Mínimo:* Arquivo `agents.keystore` criptografado no volume persistente do Docker.
    *   *Ideal:* Integração com HashiCorp Vault ou AWS KMS.
    *   *Regra:* O Agente Office deve sempre carregar a *mesma* chave privada ao bootar.

#### 2. O Limite do Payload do Postgres (The Silent Crash)
*   **O Problema:** Você usa `pg_notify` para o SSE. O Postgres tem um limite hard de **8000 bytes** no payload do notify. Se um `ubl-atom` (o JSON canônico) for maior que 8KB (ex: um Job Card complexo com histórico), o `NOTIFY` falha e o SSE não recebe nada. O frontend para de atualizar.
*   **A Correção:** Alterar a trigger SQL e o `sse.rs`.
    *   O `NOTIFY` deve enviar apenas: `{"container_id": "...", "sequence": 123}`.
    *   O `ubl-server` recebe o sinal, faz um `SELECT` rápido pelo ID para pegar o payload completo (que pode ter MBs) e então empurra para o SSE.

#### 3. Fiação da Policy VM (The Law Enforcement)
*   **O Problema:** Os componentes existem (`ubl-membrane` e `ubl-policy-vm`), mas a análise de código sugere que a chamada da VM pode estar desconectada ou permissiva demais no endpoint `/link/commit`.
*   **A Correção:** Teste de Fogo.
    *   Crie um teste de integração que tenta submeter uma transação `Evolution` (mudança de regras) **sem** a credencial correta.
    *   Se passar, a segurança é nula. O código deve falhar explicitamente se a VM não retornar `Allow`.

---

### 🟡 P1: EXPERIÊNCIA E PERFORMANCE (Retenção de Usuário)
*Sem isso, o produto funciona, mas parece "amador" ou "lento".*

#### 4. Snapshots de Projeção (The Cold Start)
*   **O Problema:** O Messenger é "UBL-Native" (não tem banco próprio). Ao reiniciar, ele precisa reconstruir o estado lendo o Ledger. Com 10.000 eventos, isso leva segundos. Com 1 milhão, leva minutos (downtime inaceitável).
*   **A Correção:** Implementar Snapshots periódicos no Messenger Backend.
    *   A cada X eventos (ou tempo), salve o estado atual da `projection_jobs` em disco/Redis.
    *   No boot: Carrega Snapshot -> Lê Ledger apenas do `last_sequence` para frente.

#### 5. Retry Loop para JSON de LLM (The Brain Fart)
*   **O Problema:** O Office confia que o LLM vai gerar o JSON do `JobCard` perfeitamente. LLMs erram vírgulas, aspas ou nomes de campos, especialmente sob carga.
*   **A Correção:** Adicionar um **Validation Loop** no Office.
    *   Se o JSON falhar no parse ou schema: Devolva o erro para o LLM ("Você gerou JSON inválido, corrija: [erro]") e tente novamente (máx 3x). Não deixe o erro explodir para o usuário final.

#### 6. Optimistic UI no Messenger (The Lag)
*   **O Problema:** O ciclo completo (User -> UBL -> Office -> UBL -> SSE -> User) tem latência física. O usuário clica "Aprovar" e espera 2 segundos até o card atualizar.
*   **A Correção:** No Frontend React, ao clicar, mude o estado visual para "Aprovado (sincronizando...)" imediatamente. Se o SSE voltar com erro, reverta e mostre toast de erro. Isso faz o app parecer instantâneo.

---

### 🟢 P2: ROBUSTEZ OPERACIONAL (Sono Tranquilo)

#### 7. Graceful Degradation do SSE
*   **O Problema:** Conexões SSE caem (mobile, wifi instável). Se o cliente reconectar e perder 3 eventos, o estado fica corrompido.
*   **A Correção:** Garanta que o cliente React envie o cabeçalho `Last-Event-ID` ao reconectar, e que o `ubl-server` saiba re-enviar os eventos perdidos a partir daquele ID.

---

### RESUMO DO PLANO DE AÇÃO

1.  **Hoje:** Resolver **P0 #2 (Postgres Notify)**. É uma mudança de código pequena, mas crítica para estabilidade.
2.  **Amanhã:** Implementar **P0 #1 (KeyStore)**. Garanta que seus agentes tenham "alma" imortal.
3.  **Segunda-feira:** Rodar o teste de fogo da **P0 #3 (Policy VM)**.
4.  **Terça-feira:** Implementar **P1 #4 (Snapshots)** no Messenger.

Com essa lista ticada, o UBL deixa de ser um "experimento fascinante" e torna-se uma **plataforma de software auditável pronta para o mercado.**