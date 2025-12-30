# ADR-004 — Zona Schengen (Authorization Cascade)

**Status:** Aprovado  
**Data:** 30-dez-2025  
**Owner:** Dan (LAB 512)

---

## 1) Contexto

Sistemas tradicionais verificam permissões em cada endpoint, resultando em:
- Código de autorização duplicado
- Inconsistências entre endpoints
- Verificações "deep" custosas (N queries por request)
- Dificuldade de propagar contexto (tenant, role, workspace)

Inspiração: União Europeia. Passaporte verificado na fronteira (entry point), livre circulação interna.

## 2) Decisão

### Zona Schengen: Verificação na Fronteira, Confiança Interna

```
                    ┌─────────────────────────────────────────┐
                    │           ZONA SCHENGEN                 │
                    │                                         │
  [Request] ──────▶ │ 🛂 FRONTEIRA (routes.rs)                │
                    │    ├── Extrai session cookie            │
                    │    ├── Valida assinatura                │
                    │    ├── Verifica expiração               │
                    │    ├── Carrega UserInfo completo        │
                    │    └── Injeta no request                │
                    │                                         │
                    │    ▼                                    │
                    │                                         │
                    │ 🏛️ INTERIOR (services, handlers)        │
                    │    └── Confia em user: UserInfo         │
                    │        (sem re-verificação)             │
                    │                                         │
                    └─────────────────────────────────────────┘
```

### SessionContext (Passaporte Enriquecido)

```rust
pub struct SessionContext {
    pub tenant_id: Option<String>,      // Organização atual
    pub role: Option<String>,           // owner|admin|member
    pub mode: Option<String>,           // operator|admin
    pub workspace_id: Option<String>,   // Workspace ativo
    pub impersonating: Option<String>,  // Se admin está impersonando
}
```

### Níveis de Verificação

| Nível | Onde | O que verifica |
|-------|------|----------------|
| L0 | Fronteira | Session válida, não expirada |
| L1 | Fronteira | Tenant membership |
| L2 | Handler | Role (owner/admin/member) |
| L3 | Handler | Step-up WebAuthn recente |
| L4 | Policy | Pact/multi-sig |
| L5 | Policy | Quorum + attestation |

### Step-Up para Operações Sensíveis

```
Regular Session (flavor: regular)
    │
    │  [Operação L3+]
    ▼
Step-Up Challenge (WebAuthn)
    │
    │  [Passkey touch]
    ▼
Step-Up Session (flavor: stepup, expires: 5min)
    │
    └── Operação autorizada
```

## 3) Implementação

### Fronteira (Gateway)

```rust
// routes.rs - ÚNICO ponto de verificação
pub async fn gateway_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    // ...
) -> Result<Response, Error> {
    // 1. Extrai e valida sessão
    let session = extract_session(&cookies, &state.pool).await?;
    
    // 2. Carrega contexto completo (1 query)
    let user = load_user_info(&state.pool, &session).await?;
    
    // 3. Injeta no request (handlers confiam)
    // ... resto do handler
}
```

### Interior (Handlers)

```rust
// Dentro da Zona Schengen - CONFIA no UserInfo
pub async fn create_job(user: UserInfo, payload: CreateJob) -> Result<Job> {
    // ✅ Usa user.tenant_id diretamente
    // ✅ Não re-verifica sessão
    // ✅ Não faz query de permissão
    
    Job::create(user.tenant_id, user.sid, payload).await
}
```

## 4) Consequências

### Positivas
- ✅ Uma verificação por request (performance)
- ✅ Código de autorização centralizado
- ✅ Contexto propagado automaticamente
- ✅ Step-up apenas quando necessário
- ✅ Handlers simples e focados

### Negativas
- ⚠️ Fronteira é single point of failure
- ⚠️ Mudanças de permissão só aplicam em nova sessão

### Mitigações
- Sessions curtas (30min regular, 5min step-up)
- Fronteira bem testada e auditada
- Logout força re-autenticação

## 5) Padrão de Código

### ❌ Errado (verificação em cada handler)
```rust
pub async fn handler(cookies: Cookies, pool: Pool) {
    let session = get_session(&cookies).await?;      // Duplicado
    let user = get_user(&pool, &session).await?;     // Duplicado
    let perms = get_permissions(&pool, &user).await?; // Duplicado
    // ...
}
```

### ✅ Certo (Zona Schengen)
```rust
pub async fn handler(user: UserInfo) {
    // UserInfo já validado na fronteira
    // Confia e executa
}
```

## 6) Referências

- [SPEC_UBL_SCHENGEN.md](../SPEC_UBL_SCHENGEN.md)
- [SCHENGEN_IMPLEMENTATION_CHECKLIST.md](../SCHENGEN_IMPLEMENTATION_CHECKLIST.md)
- União Europeia (inspiração do nome)

---

*Verificar uma vez, confiar internamente, step-up quando necessário.*
