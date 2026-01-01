# 🏗️ PLANO DE REFATORAÇÃO: Sistema de Autenticação UBL

**Data:** 2026-01-01  
**Status:** EM PROGRESSO (Phases 1-5 completas)  
**Autor:** GitHub Copilot + Dan Voulez  
**Escopo:** Messenger + Office + UBL Kernel (auth seamless)  
**Objetivo:** Autenticação seamless entre Messenger, Office e UBL Kernel

## 📊 STATUS DE IMPLEMENTAÇÃO

| Phase | Descrição | Status |
|-------|-----------|--------|
| 1 | identity/ module (config, error, mod) | ✅ Completo |
| 2 | ASC Validate endpoint no UBL Kernel | ✅ Completo |
| 3 | Office usa UBL client (não banco direto) | ✅ Completo |
| 4 | Messenger auth consolidado | ✅ Completo |
| 5 | UBL Services (ChallengeManager) | ✅ Completo |
| 6 | Session unified | ⏳ Pendente |
| 7 | Cleanup | ⏳ Pendente |
| 8 | E2E Integration | ⏳ Pendente |

---

## 🎯 VISÃO GERAL DOS 3 SISTEMAS

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              ARQUITETURA ATUAL                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐       │
│  │   MESSENGER     │     │     OFFICE      │     │   UBL KERNEL    │       │
│  │   (Frontend)    │     │   (Backend)     │     │   (Identity)    │       │
│  │   :3000         │     │   :8081         │     │   :8080         │       │
│  └────────┬────────┘     └────────┬────────┘     └────────┬────────┘       │
│           │                       │                       │                 │
│           │  WebAuthn             │  ASC/SID              │  Sessions       │
│           │  ────────────────────────────────────────────►│  Challenges     │
│           │                       │                       │  Credentials    │
│           │                       │  ────────────────────►│                 │
│           │                       │  Bearer + X-UBL-ASC   │                 │
│           │                       │                       │                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Sistema 1: MESSENGER (Frontend React)

| Componente | Arquivo | Função |
|------------|---------|--------|
| useAuth hook | `hooks/useAuth.ts` | WebAuthn register/login + PRF |
| AuthContext | `context/AuthContext.tsx` | Estado global de auth |
| apiClient | `services/apiClient.ts` | HTTP com Bearer token |

**Como autentica:**
```typescript
// 1. Guarda token em localStorage
localStorage.setItem('ubl_session_token', session_token);

// 2. Usa em todas as requests
headers: { Authorization: `Bearer ${sessionToken}` }

// 3. Valida com /id/whoami
fetch(`${API_BASE}/id/whoami`, { headers: { Authorization: ... } })
```

**Proxy em dev:**
```typescript
// vite.config.ts - Todas as chamadas vão para UBL Kernel
proxy: {
  '/api': { target: 'http://localhost:8080' },
  '/ws':  { target: 'ws://localhost:8080', ws: true }
}
```

---

### Sistema 2: OFFICE (Backend Rust)

| Componente | Arquivo | Função |
|------------|---------|--------|
| ASC Validator | `src/asc.rs` | Valida Authorization + X-UBL-ASC |
| UBL Client | `src/ubl_client/` | HTTP client para UBL Kernel |
| Session | `src/session/` | Sessions de LLM (não auth!) |

**Como autentica:**
```rust
// Espera headers:
//   Authorization: Bearer <sid>
//   X-UBL-ASC: <asc_id>

// Valida contra id_asc no banco
validate_asc_with_db(pool, asc_id, sid, container, intent, delta)
```

**Problema:** Office valida ASC diretamente no banco, não passa por UBL Kernel!

---

### Sistema 3: UBL KERNEL (Backend Rust - Identity Provider)

| Componente | Arquivo | Função |
|------------|---------|--------|
| id_routes | 1298 linhas | TODOS os endpoints de auth |
| id_db | 969 linhas | TODAS as operações de banco |
| auth/session | OK | Session types |
| auth/session_db | OK | Session CRUD |

**Endpoints:**
```
POST /id/register/begin           → WebAuthn registration
POST /id/register/finish          
POST /id/login/begin              → WebAuthn login (username-first)
POST /id/login/finish             
POST /id/login/discoverable/begin → WebAuthn login (passkey button)
POST /id/login/discoverable/finish
POST /id/stepup/begin             → Step-up auth
POST /id/stepup/finish
GET  /id/whoami                   → Validate session
POST /id/agents                   → Create LLM/App agent
POST /id/agents/{sid}/asc         → Issue ASC
```

---

## 📊 MAPEAMENTO DE FLUXOS

### Fluxo 1: Registro de Pessoa (Messenger → UBL)

```
┌──────────────┐   POST /id/register/begin    ┌──────────────┐
│   Messenger  │ ───────────────────────────► │  UBL Kernel  │
│   (useAuth)  │ ◄─────────────────────────── │              │
│              │   { challenge_id, options }  │              │
│              │                              │              │
│  WebAuthn    │   POST /id/register/finish   │              │
│  Browser API │ ───────────────────────────► │              │
│              │ ◄─────────────────────────── │              │
│              │   { sid, session_token }     │              │
│              │                              │              │
│  localStorage│                              │  id_subject  │
│  .setItem()  │                              │  id_cred     │
└──────────────┘                              └──────────────┘
```

### Fluxo 2: Login com Passkey Button (Discoverable)

```
┌──────────────┐   POST /id/login/discoverable/begin   ┌──────────────┐
│   Messenger  │ ─────────────────────────────────────►│  UBL Kernel  │
│              │ ◄─────────────────────────────────────│              │
│              │   { challenge_id, public_key }        │              │
│              │                                       │              │
│  WebAuthn    │   POST /id/login/discoverable/finish  │              │
│  Browser API │ ─────────────────────────────────────►│              │
│              │ ◄─────────────────────────────────────│              │
│              │   { sid, session_token }              │  id_session  │
└──────────────┘                                       └──────────────┘
```

### Fluxo 3: LLM Agent + ASC (Office → UBL)

```
┌──────────────┐                                       ┌──────────────┐
│    OFFICE    │   POST /id/agents                     │  UBL Kernel  │
│              │ ─────────────────────────────────────►│              │
│              │ ◄─────────────────────────────────────│              │
│              │   { sid, kind, display_name }         │              │
│              │                                       │              │
│              │   POST /id/agents/{sid}/asc           │              │
│              │ ─────────────────────────────────────►│              │
│              │ ◄─────────────────────────────────────│              │
│              │   { asc_id, scopes, signature }       │  id_asc      │
└──────────────┘                                       └──────────────┘
```

### Fluxo 4: Validar Session (Qualquer → UBL)

```
┌──────────────┐   GET /id/whoami                      ┌──────────────┐
│  Messenger   │   Authorization: Bearer <token>       │  UBL Kernel  │
│  ou Office   │ ─────────────────────────────────────►│              │
│              │ ◄─────────────────────────────────────│              │
│              │   { authenticated, sid, kind, ... }   │  id_session  │
└──────────────┘                                       └──────────────┘
```

---

## 🔴 PROBLEMAS IDENTIFICADOS

### Backend (UBL Kernel)

| # | Problema | Impacto | Localização |
|---|----------|---------|-------------|
| 1 | Monolito `id_routes.rs` (1298 linhas) | Manutenção impossível | UBL |
| 2 | 2 structs `Session` incompatíveis | Bugs de tipo | id_db vs auth/session |
| 3 | `WEBAUTHN_ORIGIN` repetido 12+ vezes | DRY violation | id_routes.rs |
| 4 | 4 funções `create_*_challenge` quase iguais | Duplicação | id_db.rs |
| 5 | `webauthn_store.rs` duplica step-up | Código morto | UBL |
| 6 | Tabela `id_webauthn_credentials` não usada | Schema sujo | DB |
| 7 | SID é `Uuid` em alguns lugares, `String` em outros | Type mismatch | Vários |

### Frontend (Messenger)

| # | Problema | Impacto | Localização |
|---|----------|---------|-------------|
| 8 | Dois hooks de auth (useAuth + AuthContext) | Confusão | hooks + context |
| 9 | `ubl_session` vs `ubl_session_token` inconsistente | Bugs | localStorage |

### Inter-sistema (Office ↔ UBL)

| # | Problema | Impacto | Localização |
|---|----------|---------|-------------|
| 10 | Office valida ASC direto no banco | Bypass do Kernel | office/asc.rs |
| 11 | Não há validação de session entre sistemas | Security gap | - |

---

## 📊 DIAGNÓSTICO DETALHADO

### Inventário de Arquivos (UBL Kernel)

| Arquivo | Linhas | Responsabilidades | Status |
|---------|--------|-------------------|--------|
| `id_routes.rs` | 1298 | 18 handlers + tipos + helpers | ⛔ MONOLÍTICO |
| `id_db.rs` | 969 | 20+ funções DB | ⛔ FAT REPOSITORY |
| `auth.rs` | 180 | ASC validation + re-exports | ⚠️ OK |
| `auth/session.rs` | 110 | Session struct + builders | ✅ BOM |
| `auth/session_db.rs` | 95 | Session CRUD | ✅ BOM |
| `auth/require_stepup.rs` | 85 | Middleware step-up | ✅ BOM |
| `webauthn_store.rs` | 220 | Step-up duplicado | ⛔ REMOVER |
| `id_session_token.rs` | 150 | JWT token issuing | ✅ BOM |

### Tabelas do Banco (001_identity.sql)

| Tabela | Usado Por | Status |
|--------|-----------|--------|
| `id_subject` | UBL, Office | ✅ OK |
| `id_credential` | UBL | ✅ OK |
| `id_webauthn_credentials` | NINGUÉM | ⛔ REMOVER |
| `id_challenge` | UBL | ✅ OK |
| `id_stepup_challenges` | webauthn_store.rs | ⛔ REMOVER |
| `id_session` | UBL, Messenger | ✅ OK |
| `id_asc` | UBL, Office | ✅ OK |
| `id_key_revocation` | UBL | ✅ OK |

---

## 🔴 PROBLEMAS CRÍTICOS

### 1. Monolito de 1300 linhas (`id_routes.rs`)

```
┌─────────────────────────────────────────────────────────────┐
│                      id_routes.rs (1298 linhas)             │
├─────────────────────────────────────────────────────────────┤
│ • WebAuthn Registration (begin/finish)                      │
│ • WebAuthn Login (begin/finish)                             │
│ • Discoverable Login (begin/finish)                         │
│ • Step-Up Auth (begin/finish)                               │
│ • Agent CRUD (create/export/rotate)                         │
│ • ASC Management (issue/list/revoke)                        │
│ • ICT Sessions (begin/finish)                               │
│ • Whoami                                                    │
│ • 20+ Request/Response structs                              │
│ • Helper functions (set_cookie, parse_cdj, assert_origin)   │
└─────────────────────────────────────────────────────────────┘
```

**Sintomas:**
- Difícil de navegar
- Impossível testar handlers isoladamente
- Mudanças arriscadas (impacto cascata)
- Code review doloroso

### 2. Código Duplicado

| Código | Ocorrências | Localização |
|--------|-------------|-------------|
| `std::env::var("WEBAUTHN_ORIGIN")...` | 12+ | id_routes.rs |
| `create_*_challenge()` | 4 funções | id_db.rs (quase idênticas) |
| `.map_err(\|e\| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))` | 50+ | id_routes.rs |
| Sign count validation | 3 | login_finish, discoverable_finish, stepup_finish |
| Session creation | 2 | login_finish, stepup_finish |

### 3. Duas Structs Session

```rust
// id_db.rs linha 60
pub struct Session {
    pub sid: String,
    pub session_id: Uuid,
    pub flavor: String,
    pub scope: serde_json::Value,
    pub not_before: OffsetDateTime,
    pub not_after: OffsetDateTime,
}

// auth/session.rs linha 22
pub struct Session {
    pub token: String,
    pub sid: Uuid,  // ⚠️ Uuid vs String!
    pub tenant_id: Option<String>,
    pub flavor: SessionFlavor,
    pub scope: serde_json::Value,
    pub context: SessionContext,  // Zona Schengen
    pub exp_unix: i64,
}
```

**Problema:** `sid` é `String` em um e `Uuid` no outro. Mas nosso SID é `"ubl:sid:..."` (String), não UUID!

### 4. Tabela Duplicada

`id_webauthn_credentials` foi criada mas **nunca é usada**. Todo o código usa `id_credential`.

### 5. webauthn_store.rs Reimplementa Step-Up

`webauthn_store.rs` (220 linhas) reimplementa step-up authentication com:
- Sua própria tabela (`id_stepup_challenges`)
- Sua própria lógica de verificação
- Duplicação completa do que já existe em `id_routes.rs`

---

## 🟢 ARQUITETURA PROPOSTA

```
src/
├── identity/                      # 🆕 Novo módulo
│   ├── mod.rs                     # Re-exports públicos
│   │
│   ├── config.rs                  # (40 linhas)
│   │   └── WebAuthnConfig         # Origin, RP, TTLs centralizados
│   │
│   ├── error.rs                   # (80 linhas)
│   │   └── IdentityError          # thiserror enum + IntoResponse
│   │
│   ├── types.rs                   # (60 linhas)
│   │   ├── Subject
│   │   ├── Credential
│   │   └── Challenge
│   │
│   ├── webauthn/                  # WebAuthn puro
│   │   ├── mod.rs
│   │   ├── register.rs            # (120 linhas) RegistrationService
│   │   ├── login.rs               # (120 linhas) LoginService
│   │   └── discoverable.rs        # (100 linhas) DiscoverableService
│   │
│   ├── challenge/                 # Challenge management
│   │   ├── mod.rs
│   │   └── manager.rs             # (80 linhas) ChallengeManager - UNIFICADO
│   │
│   ├── agents/                    # LLM/App agents
│   │   ├── mod.rs
│   │   ├── service.rs             # (80 linhas) create/export/rotate
│   │   └── asc.rs                 # (100 linhas) issue/validate ASC
│   │
│   ├── session/                   # 🔄 Absorve auth/session*
│   │   ├── mod.rs
│   │   ├── types.rs               # Session, SessionFlavor, SessionContext
│   │   ├── db.rs                  # CRUD operations
│   │   └── middleware.rs          # Extract session from request
│   │
│   ├── stepup/                    # Step-up unificado
│   │   ├── mod.rs
│   │   ├── service.rs             # (100 linhas) begin/finish
│   │   └── middleware.rs          # require_stepup
│   │
│   └── routes.rs                  # (150 linhas) HTTP handlers SLIM
│
├── auth.rs                        # 🔄 Re-export identity/* + legacy compat
└── id_routes.rs                   # 🗑️ DEPRECATED → routes em identity/
```

---

## 📋 FASES DE IMPLEMENTAÇÃO

### Fase 1: Fundação (2h)
**Risco: BAIXO | Impacto: ALTO**

```rust
// src/identity/config.rs
pub struct WebAuthnConfig {
    pub origin: String,
    pub rp_id: String,
    pub rp_name: String,
    pub challenge_ttl_secs: i64,
    pub stepup_ttl_secs: i64,
}

impl WebAuthnConfig {
    pub fn from_env() -> Self {
        Self {
            origin: env::var("WEBAUTHN_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:8080".into()),
            rp_id: env::var("WEBAUTHN_RP_ID")
                .unwrap_or_else(|_| "localhost".into()),
            rp_name: env::var("WEBAUTHN_RP_NAME")
                .unwrap_or_else(|_| "UBL".into()),
            challenge_ttl_secs: 300,
            stepup_ttl_secs: 120,
        }
    }
}
```

```rust
// src/identity/error.rs
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("Challenge not found or already used")]
    ChallengeInvalid,
    
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("Credential not found")]
    CredentialNotFound,
    
    #[error("Sign count rollback (replay attack?)")]
    CounterRollback { old: u32, new: u32 },
    
    #[error("Origin mismatch")]
    OriginMismatch,
    
    #[error("WebAuthn error: {0}")]
    WebAuthn(String),
    
    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for IdentityError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Self::ChallengeInvalid => (StatusCode::BAD_REQUEST, "CHALLENGE_INVALID"),
            Self::UserNotFound(_) => (StatusCode::NOT_FOUND, "USER_NOT_FOUND"),
            Self::CredentialNotFound => (StatusCode::NOT_FOUND, "CREDENTIAL_NOT_FOUND"),
            Self::CounterRollback { .. } => (StatusCode::UNAUTHORIZED, "COUNTER_ROLLBACK"),
            Self::OriginMismatch => (StatusCode::UNAUTHORIZED, "ORIGIN_MISMATCH"),
            Self::WebAuthn(_) => (StatusCode::UNAUTHORIZED, "WEBAUTHN_ERROR"),
            Self::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DB_ERROR"),
        };
        
        (status, Json(json!({ "error": code, "message": self.to_string() }))).into_response()
    }
}
```

**Benefícios:**
- Remove 15+ `env::var()` duplicados
- Remove 50+ `.map_err(|e| (StatusCode::..., e.to_string()))`

---

### Fase 2: Unificar Challenges (1h)
**Risco: BAIXO | Impacto: MÉDIO**

```rust
// src/identity/challenge/manager.rs
pub struct ChallengeManager {
    pool: PgPool,
    config: WebAuthnConfig,
}

pub enum ChallengeKind {
    Register,
    Login,
    Stepup,
}

impl ChallengeManager {
    pub async fn create(
        &self,
        kind: ChallengeKind,
        sid: Option<&str>,
        data: &[u8],
    ) -> Result<Uuid, IdentityError> {
        let ttl = match kind {
            ChallengeKind::Stepup => self.config.stepup_ttl_secs,
            _ => self.config.challenge_ttl_secs,
        };
        
        let expires_at = OffsetDateTime::now_utc() + Duration::seconds(ttl);
        let kind_str = match kind {
            ChallengeKind::Register => "register",
            ChallengeKind::Login => "login",
            ChallengeKind::Stepup => "stepup",
        };
        
        let row = sqlx::query_as::<_, (Uuid,)>(
            "INSERT INTO id_challenge (kind, sid, challenge, origin, expires_at)
             VALUES ($1, $2, $3, $4, $5) RETURNING id"
        )
        .bind(kind_str)
        .bind(sid)
        .bind(data)
        .bind(&self.config.origin)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(row.0)
    }
    
    pub async fn consume(&self, id: Uuid) -> Result<Challenge, IdentityError> {
        sqlx::query_as::<_, Challenge>(
            "UPDATE id_challenge SET used = true
             WHERE id = $1 AND used = false AND origin = $2 AND expires_at > NOW()
             RETURNING *"
        )
        .bind(id)
        .bind(&self.config.origin)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(IdentityError::ChallengeInvalid)
    }
}
```

**Benefícios:**
- Remove 4 funções `create_*_challenge` quase idênticas
- Anti-replay atômico garantido em um lugar só

---

### Fase 3: Services com Lógica de Negócio (3h)
**Risco: MÉDIO | Impacto: ALTO**

```rust
// src/identity/webauthn/register.rs
pub struct RegistrationService {
    webauthn: Webauthn,
    challenges: ChallengeManager,
    pool: PgPool,
    config: WebAuthnConfig,
}

impl RegistrationService {
    pub async fn begin(
        &self,
        username: &str,
        display_name: &str,
    ) -> Result<(Uuid, CreationChallengeResponse), IdentityError> {
        // 1. Check if user exists
        if get_subject_by_username(&self.pool, username).await?.is_some() {
            return Err(IdentityError::UserAlreadyExists(username.to_string()));
        }
        
        // 2. Start WebAuthn registration
        let (ccr, reg_state) = self.webauthn
            .start_passkey_registration(Uuid::new_v4(), username, display_name, None)
            .map_err(|e| IdentityError::WebAuthn(format!("{:?}", e)))?;
        
        // 3. Store challenge with username
        let state_bytes = serde_json::to_vec(&reg_state)?;
        let challenge_data = json!({ "username": username, "state": state_bytes });
        let challenge_id = self.challenges
            .create(ChallengeKind::Register, None, &serde_json::to_vec(&challenge_data)?)
            .await?;
        
        Ok((challenge_id, ccr))
    }
    
    pub async fn finish(
        &self,
        challenge_id: Uuid,
        attestation: &PublicKeyCredential,
    ) -> Result<String, IdentityError> {
        // 1. Consume challenge atomically
        let challenge = self.challenges.consume(challenge_id).await?;
        
        // 2. Extract username from challenge
        let data: serde_json::Value = serde_json::from_slice(&challenge.challenge)?;
        let username = data["username"].as_str()
            .ok_or(IdentityError::Internal("No username in challenge"))?;
        
        // 3. Finish WebAuthn registration
        let state: PasskeyRegistration = serde_json::from_slice(&data["state"])?;
        let passkey = self.webauthn
            .finish_passkey_registration(attestation, &state)
            .map_err(|e| IdentityError::WebAuthn(format!("{:?}", e)))?;
        
        // 4. Create subject
        let sid = create_person(&self.pool, username, username).await?;
        
        // 5. Store credential
        create_credential(&self.pool, &sid, "passkey", &passkey).await?;
        
        Ok(sid)
    }
}
```

**Agora o handler fica SLIM:**

```rust
// src/identity/routes.rs
pub async fn route_register_begin(
    State(state): State<IdState>,
    Json(req): Json<RegisterBeginReq>,
) -> Result<Json<RegisterBeginResp>, IdentityError> {
    let (challenge_id, options) = state.registration
        .begin(&req.username, &req.display_name.unwrap_or(req.username.clone()))
        .await?;
    
    Ok(Json(RegisterBeginResp {
        challenge_id: challenge_id.to_string(),
        options,
    }))
}
```

**De 70 linhas → 10 linhas!**

---

### Fase 4: Unificar Session (1h)
**Risco: MÉDIO | Impacto: MÉDIO**

```rust
// src/identity/session/types.rs

/// Canonical Session - UMA definição
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    /// Session token (primary key in DB)
    pub token: String,
    
    /// Subject ID ("ubl:sid:...")
    pub sid: String,  // ⚠️ String, não Uuid!
    
    /// Current tenant (Zona Schengen)
    pub tenant_id: Option<String>,
    
    /// Session type
    pub flavor: SessionFlavor,
    
    /// Extended context
    pub context: SessionContext,
    
    /// Expiration (unix timestamp)
    pub exp_unix: i64,
}

impl Session {
    /// Parse SID to Uuid if needed (for legacy code)
    pub fn sid_uuid(&self) -> Option<Uuid> {
        // "ubl:sid:abc123..." → try parse "abc123..." as hex to bytes, but...
        // Actually our SIDs are blake3 hashes, not UUIDs!
        // Return None - callers should use sid string directly
        None
    }
}
```

---

### Fase 5: Remover Código Morto (30min)
**Risco: BAIXO | Impacto: BAIXO**

| Remover | Justificativa |
|---------|---------------|
| `webauthn_store.rs` | Merge com identity/stepup/ |
| `id_db::Session` | Usar identity/session/types.rs |
| `id_webauthn_credentials` (tabela) | Nunca usada, temos `id_credential` |
| `create_register_challenge()` | Usar ChallengeManager |
| `create_login_challenge()` | Usar ChallengeManager |
| `create_discoverable_challenge()` | Usar ChallengeManager |
| `create_stepup_challenge()` | Usar ChallengeManager |

---

### Fase 6: Migrar Rotas (2h)
**Risco: ALTO | Impacto: ALTO**

1. Criar `identity/routes.rs` com handlers slim
2. Atualizar `main.rs` para usar novo router
3. Marcar `id_routes.rs` como deprecated
4. Remover após validação

---

## 📈 MÉTRICAS DE SUCESSO

| Métrica | Antes | Depois | Melhoria |
|---------|-------|--------|----------|
| Linhas em id_routes.rs | 1298 | 0 (identity/routes.rs: ~150) | -88% |
| Funções duplicadas | 4 | 0 | -100% |
| Chamadas env::var() espalhadas | 15+ | 1 | -93% |
| Structs Session | 2 | 1 | -50% |
| .map_err boilerplate | 50+ | 0 | -100% |
| Tabelas não usadas | 1 | 0 | -100% |
| Arquivos duplicados | 1 (webauthn_store) | 0 | -100% |
| Testabilidade | Baixa | Alta | ⬆️ |

---

## 🚀 ORDEM RECOMENDADA

```
Semana 1:
├── Dia 1: Fase 1 (config + error) ✅ Baixo risco, alto valor
├── Dia 2: Fase 2 (ChallengeManager) ✅ Remove duplicação
└── Dia 3: Fase 5 (cleanup code morto) ✅ Menor footprint

Semana 2:
├── Dia 1-2: Fase 3 (Services) 🔶 Mais trabalho
├── Dia 3: Fase 4 (Session unificada) 🔶 Mudança de tipos
└── Dia 4-5: Fase 6 (Migrar rotas) 🔴 Validação cuidadosa

Semana 3:
└── Testes e validação de todos os fluxos
```

---

## ⚠️ RISCOS E MITIGAÇÕES

| Risco | Probabilidade | Impacto | Mitigação |
|-------|---------------|---------|-----------|
| Quebrar login existente | Média | Alto | Feature flags + rollback rápido |
| Tipo SID incompatível | Alta | Médio | Usar String em toda parte |
| Migração incompleta | Média | Médio | Manter id_routes.rs temporariamente |
| Performance regression | Baixa | Baixo | Benchmark antes/depois |

---

## 📝 DECISÕES PENDENTES

1. **SID Format:** Manter String (`"ubl:sid:..."`) ou migrar para Uuid?
   - **Recomendação:** Manter String - é mais expressivo e já funciona

2. **Session Storage:** Manter dual (token + session_id) ou simplificar?
   - **Recomendação:** Manter token como PK, remover session_id UUID

3. **Tenant Context:** Onde vive a lógica de tenant?
   - **Recomendação:** Em identity/session/context.rs

4. **webauthn_store.rs:** Merge ou remover?
   - **Recomendação:** Remover, usar stepup de id_routes (mais completo)

---

## 🧪 ESTRATÉGIA DE TESTES

### Testes por Fase

| Fase | Tipo de Teste | O que Validar |
|------|---------------|---------------|
| 1 (config/error) | Unit | `WebAuthnConfig::from_env()` com diferentes envs |
| 2 (ChallengeManager) | Integration | Create + consume + expiração + anti-replay |
| 3 (Services) | Integration | Fluxo completo register/login/discoverable |
| 4 (Session) | Unit + Integration | Serialização, DB round-trip |
| 5 (cleanup) | Smoke | Nada quebrou após remoção |
| 6 (rotas) | E2E | Todos os endpoints respondem igual |

### Checklist de Validação (executar após cada fase)

```bash
# 1. UBL Kernel compila
cd ubl/kernel/rust && SQLX_OFFLINE=true cargo build --release

# 2. Endpoints respondem
curl http://localhost:8080/health
curl http://localhost:8080/id/whoami

# 3. Fluxo completo (usar Messenger)
# - Registrar novo usuário
# - Login com passkey
# - Verificar /id/whoami retorna dados

# 4. Office conecta
curl http://localhost:8081/health
# - Criar agent
# - Emitir ASC
# - Validar ASC
```

### Testes Automatizados a Criar

| Arquivo | Testes |
|---------|--------|
| `identity/config_test.rs` | `test_from_env_defaults`, `test_from_env_custom` |
| `identity/challenge/manager_test.rs` | `test_create_consume`, `test_expired_rejected`, `test_replay_rejected` |
| `identity/webauthn/register_test.rs` | `test_begin_creates_challenge`, `test_finish_creates_subject` |
| `identity/webauthn/login_test.rs` | `test_login_flow`, `test_wrong_credential_rejected` |

---

## 🔄 ROLLBACK PLAN

### Estratégia: Feature Flags + Dual-Running

```rust
// Em main.rs - manter AMBOS os routers durante migração
let use_new_identity = env::var("USE_NEW_IDENTITY").unwrap_or("false".into()) == "true";

if use_new_identity {
    app = app.merge(identity::routes::router(state.clone()));
} else {
    app = app.merge(id_routes::router(state.clone()));
}
```

### Níveis de Rollback

| Nível | Trigger | Ação | Tempo |
|-------|---------|------|-------|
| 1 - Soft | Erro em 1 endpoint | `USE_NEW_IDENTITY=false` + restart | 30s |
| 2 - Medium | Múltiplos erros | Git revert do último commit | 2min |
| 3 - Hard | Sistema inoperante | Deploy da última release estável | 5min |

### Checklist Pré-Deploy

- [ ] Backup do banco (pg_dump id_*)
- [ ] Tag da versão atual: `git tag pre-identity-refactor`
- [ ] Testar rollback em staging primeiro
- [ ] Monitorar logs por 15min após deploy

### Sinais de Problema

| Sinal | Ação |
|-------|------|
| 401 em /id/whoami aumenta | Rollback nível 1 |
| 500 errors em /id/login/finish | Rollback nível 2 |
| Messenger não consegue logar | Rollback nível 3 |
| Office não valida ASC | Rollback nível 3 |

---

## 📡 MAPA DE APIs (Antes → Depois)

### Endpoints que NÃO MUDAM (contrato mantido)

| Endpoint | Request | Response |
|----------|---------|----------|
| `POST /id/register/begin` | `{ username, display_name? }` | `{ challenge_id, options }` |
| `POST /id/register/finish` | `{ challenge_id, attestation }` | `{ sid, session_token }` |
| `POST /id/login/begin` | `{ username }` | `{ challenge_id, options }` |
| `POST /id/login/finish` | `{ challenge_id, assertion }` | `{ sid, session_token }` |
| `POST /id/login/discoverable/begin` | `{}` | `{ challenge_id, options }` |
| `POST /id/login/discoverable/finish` | `{ assertion }` | `{ sid, session_token }` |
| `GET /id/whoami` | Header: `Authorization: Bearer <token>` | `{ authenticated, sid, ... }` |
| `POST /id/agents` | `{ kind, display_name }` | `{ sid }` |
| `POST /id/agents/{sid}/asc` | `{ container, intent, ... }` | `{ asc_id, signature }` |

### Endpoint NOVO (Fase 6)

| Endpoint | Request | Response | Justificativa |
|----------|---------|----------|---------------|
| `GET /id/asc/{id}/validate` | Header: `Authorization: Bearer <token>` | `{ valid, scopes, ... }` | Office precisa validar via Kernel |

### Headers que NÃO MUDAM

| Header | Usado Por | Valor |
|--------|-----------|-------|
| `Authorization` | Todos | `Bearer <session_token>` |
| `X-UBL-ASC` | Office | `<asc_id>` |
| `X-Tenant-ID` | Office | `<tenant_id>` |

### Mudanças Internas (invisíveis para clientes)

| Antes | Depois |
|-------|--------|
| `id_routes.rs` handler gigante | `identity/routes.rs` handler slim |
| 4 funções `create_*_challenge` | 1 `ChallengeManager.create()` |
| 12+ `env::var("WEBAUTHN_ORIGIN")` | 1 `WebAuthnConfig.origin` |
| 2 structs `Session` | 1 struct canônica |

---

## ✅ PRÓXIMOS PASSOS

1. [ ] Aprovar este plano
2. [ ] Criar branch `refactor/identity-module`
3. [ ] Implementar Fase 1 (config + error)
4. [ ] Implementar Fase 2 (ChallengeManager)
5. [ ] Implementar Fase 5 (cleanup)
6. [ ] Validar em staging
7. [ ] Continuar com Fases 3, 4, 6

---

## 🌐 MAPEAMENTO DOS 3 SISTEMAS

### Messenger (Frontend :3000)
- **Auth:** `useAuth.ts` + `AuthContext.tsx` → WebAuthn + PRF
- **Storage:** `localStorage.ubl_session_token`
- **API:** Bearer token via `apiClient.ts`
- **Proxy:** Vite → `http://localhost:8080`

### Office (Backend :8081)
- **Auth:** `asc.rs` → Authorization + X-UBL-ASC headers
- **Client:** `ubl_client/` → HTTP para UBL Kernel
- **Problema:** Valida ASC direto no banco, bypassa Kernel

### UBL Kernel (Backend :8080) - Identity Provider
- **Endpoints:** /id/register, /id/login, /id/whoami, /id/agents, /id/asc
- **Problema:** Monolito de 1298 linhas em id_routes.rs

### Fluxo Seamless Esperado
```
Messenger → WebAuthn → UBL Kernel → Session Token
                ↓
Office → Bearer + ASC → UBL Kernel → Validate → OK
```

---

## 🔴 PROBLEMAS ADICIONAIS IDENTIFICADOS

### Frontend (Messenger)
| # | Problema | Fix |
|---|----------|-----|
| F1 | `ubl_session` vs `ubl_session_token` | Padronizar para `ubl_session_token` |
| F2 | useAuth e AuthContext duplicam lógica | Consolidar em um |

### Inter-Sistema
| # | Problema | Fix |
|---|----------|-----|
| I1 | Office valida ASC direto no DB | Chamar UBL Kernel `/id/asc/validate` |
| I2 | Não há endpoint de validação de ASC | Criar `/id/asc/{id}/validate` |

---

## 🌐 ALTERAÇÕES HARMÔNICAS NOS 3 SISTEMAS

### Visão Geral das Mudanças

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                     REFATORAÇÃO AUTH: 3 SISTEMAS                                │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐         │
│  │   MESSENGER     │      │     OFFICE      │      │   UBL KERNEL    │         │
│  │   (Frontend)    │      │   (Backend)     │      │   (Identity)    │         │
│  │   :3000         │      │   :8081         │      │   :8080         │         │
│  ├─────────────────┤      ├─────────────────┤      ├─────────────────┤         │
│  │                 │      │                 │      │                 │         │
│  │ MUDANÇAS:       │      │ MUDANÇAS:       │      │ MUDANÇAS:       │         │
│  │                 │      │                 │      │                 │         │
│  │ • Consolidar    │      │ • Chamar        │      │ • identity/     │         │
│  │   useAuth       │      │   /id/asc/      │      │   module        │         │
│  │ • Padronizar    │      │   validate      │      │ • ChallengeM.   │         │
│  │   storage key   │      │ • Remover       │      │ • Services      │         │
│  │ • AuthProvider  │      │   DB direto     │      │ • Session       │         │
│  │   único         │      │ • UblClient     │      │   unificada     │         │
│  │                 │      │   melhorado     │      │                 │         │
│  └────────┬────────┘      └────────┬────────┘      └────────┬────────┘         │
│           │                        │                        │                   │
│           │◄───────────────────────┼───────────────────────►│                   │
│           │     Contratos Mantidos │  Novo: /id/asc/validate│                   │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 📦 ALTERAÇÕES POR SISTEMA

### 1. MESSENGER (Frontend React)

**Problema atual:** Dois hooks de auth, storage key inconsistente

**Arquitetura Proposta:**

```
apps/messenger/src/
├── auth/                          # 🆕 Novo módulo consolidado
│   ├── index.ts                   # Re-exports
│   ├── AuthProvider.tsx           # Context + Provider ÚNICO
│   ├── useAuth.ts                 # Hook que usa o context
│   ├── storage.ts                 # Abstração de localStorage
│   ├── api.ts                     # Chamadas /id/* para UBL
│   └── types.ts                   # AuthState, User, etc.
│
├── hooks/
│   └── useAuth.ts                 # 🗑️ DEPRECATED → auth/useAuth.ts
│
├── context/
│   └── AuthContext.tsx            # 🗑️ DEPRECATED → auth/AuthProvider.tsx
```

**Código:**

```typescript
// auth/storage.ts - ÚNICA fonte de verdade para storage
const STORAGE_KEY = 'ubl_session_token';  // Padronizado!

export const authStorage = {
  getToken: (): string | null => {
    return localStorage.getItem(STORAGE_KEY);
  },
  
  setToken: (token: string): void => {
    localStorage.setItem(STORAGE_KEY, token);
  },
  
  clearToken: (): void => {
    localStorage.removeItem(STORAGE_KEY);
    // Também limpa chave antiga se existir
    localStorage.removeItem('ubl_session');
  },
  
  // Migração automática da chave antiga
  migrate: (): void => {
    const oldToken = localStorage.getItem('ubl_session');
    if (oldToken && !localStorage.getItem(STORAGE_KEY)) {
      localStorage.setItem(STORAGE_KEY, oldToken);
      localStorage.removeItem('ubl_session');
    }
  }
};
```

```typescript
// auth/AuthProvider.tsx - Provider ÚNICO
interface AuthState {
  isAuthenticated: boolean;
  isLoading: boolean;
  user: User | null;
  sid: string | null;
  error: string | null;
}

interface AuthContextValue extends AuthState {
  login: () => Promise<void>;
  loginWithUsername: (username: string) => Promise<void>;
  register: (username: string, displayName?: string) => Promise<void>;
  logout: () => void;
  refreshSession: () => Promise<void>;
}

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [state, setState] = useState<AuthState>({
    isAuthenticated: false,
    isLoading: true,
    user: null,
    sid: null,
    error: null,
  });

  // Migração automática na inicialização
  useEffect(() => {
    authStorage.migrate();
    refreshSession();
  }, []);

  const refreshSession = async () => {
    const token = authStorage.getToken();
    if (!token) {
      setState(s => ({ ...s, isLoading: false }));
      return;
    }

    try {
      const response = await fetch('/api/id/whoami', {
        headers: { Authorization: `Bearer ${token}` }
      });
      
      if (response.ok) {
        const data = await response.json();
        setState({
          isAuthenticated: true,
          isLoading: false,
          user: data.user,
          sid: data.sid,
          error: null,
        });
      } else {
        authStorage.clearToken();
        setState(s => ({ ...s, isAuthenticated: false, isLoading: false }));
      }
    } catch (error) {
      setState(s => ({ ...s, isLoading: false, error: 'Network error' }));
    }
  };

  // ... resto da implementação (login, register, logout)
  
  return (
    <AuthContext.Provider value={{ ...state, login, loginWithUsername, register, logout, refreshSession }}>
      {children}
    </AuthContext.Provider>
  );
};
```

```typescript
// auth/useAuth.ts - Hook simples que usa o context
export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
}
```

**Migração:**
1. Criar `auth/` module
2. Atualizar imports em todos os componentes
3. Remover `hooks/useAuth.ts` e `context/AuthContext.tsx`
4. Testar login/logout/register

---

### 2. OFFICE (Backend Rust)

**Problema atual:** Valida ASC direto no banco, bypassa UBL Kernel

**Arquitetura Proposta:**

```
apps/office/src/
├── auth/                          # 🆕 Novo módulo
│   ├── mod.rs                     # Re-exports
│   ├── middleware.rs              # Middleware de auth
│   ├── asc_validator.rs           # Valida ASC via UBL Kernel
│   └── session_validator.rs       # Valida session via /id/whoami
│
├── ubl_client/
│   ├── mod.rs
│   ├── identity.rs                # 🆕 Métodos de identity
│   │   ├── validate_session()
│   │   ├── validate_asc()
│   │   └── create_agent()
│   └── ...
│
├── asc.rs                         # 🔄 Delega para auth/asc_validator.rs
```

**Código:**

```rust
// ubl_client/identity.rs - Novo arquivo
impl UblClient {
    /// Validate session token via UBL Kernel
    pub async fn validate_session(&self, token: &str) -> Result<WhoamiResponse> {
        let response = self.client
            .get(format!("{}/id/whoami", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(OfficeError::Unauthorized("Invalid session".into()))
        }
    }
    
    /// Validate ASC via UBL Kernel (NOVO ENDPOINT!)
    pub async fn validate_asc(&self, asc_id: &str, token: &str) -> Result<AscValidation> {
        let response = self.client
            .get(format!("{}/id/asc/{}/validate", self.base_url, asc_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(OfficeError::Unauthorized("Invalid ASC".into()))
        }
    }
    
    /// Create agent via UBL Kernel
    pub async fn create_agent(
        &self, 
        token: &str,
        kind: &str, 
        display_name: &str
    ) -> Result<AgentResponse> {
        let response = self.client
            .post(format!("{}/id/agents", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({ "kind": kind, "display_name": display_name }))
            .send()
            .await?;
        
        response.json().await.map_err(Into::into)
    }
}
```

```rust
// auth/asc_validator.rs - Substitui lógica direta no banco
pub struct AscValidator {
    ubl_client: Arc<UblClient>,
}

impl AscValidator {
    pub async fn validate(
        &self,
        asc_id: &str,
        bearer_token: &str,
        expected_container: &str,
        expected_intent: &str,
    ) -> Result<AscValidation> {
        // 1. Validar via UBL Kernel (NÃO direto no banco!)
        let validation = self.ubl_client
            .validate_asc(asc_id, bearer_token)
            .await?;
        
        // 2. Verificar container e intent
        if validation.container != expected_container {
            return Err(OfficeError::Unauthorized("ASC container mismatch".into()));
        }
        
        if !validation.allowed_intents.contains(&expected_intent.to_string()) {
            return Err(OfficeError::Unauthorized("ASC intent not allowed".into()));
        }
        
        Ok(validation)
    }
}
```

```rust
// auth/middleware.rs - Middleware de autenticação
pub async fn require_auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, OfficeError> {
    // 1. Extrair Bearer token
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(OfficeError::Unauthorized("Missing token".into()))?;
    
    // 2. Validar via UBL Kernel
    let session = state.ubl_client
        .validate_session(token)
        .await?;
    
    // 3. Injetar no request
    request.extensions_mut().insert(session);
    
    Ok(next.run(request).await)
}

pub async fn require_asc(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, OfficeError> {
    // 1. Extrair ASC
    let asc_id = headers
        .get("X-UBL-ASC")
        .and_then(|v| v.to_str().ok())
        .ok_or(OfficeError::Unauthorized("Missing ASC".into()))?;
    
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(OfficeError::Unauthorized("Missing token".into()))?;
    
    // 2. Validar via UBL Kernel
    let validation = state.asc_validator
        .validate(asc_id, token, &state.container_id, "llm_request")
        .await?;
    
    // 3. Injetar no request
    request.extensions_mut().insert(validation);
    
    Ok(next.run(request).await)
}
```

**Migração:**
1. Criar `auth/` module
2. Adicionar `identity.rs` ao `ubl_client/`
3. Atualizar rotas para usar novos middlewares
4. Remover código que acessa banco diretamente para ASC

---

### 3. UBL KERNEL (Backend Rust)

**Mudanças já detalhadas nas fases 1-6, mais:**

**Novo Endpoint:**

```rust
// identity/routes.rs - Adicionar

/// Validate an ASC (for Office to call)
pub async fn route_asc_validate(
    State(state): State<IdState>,
    Path(asc_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<AscValidation>, IdentityError> {
    // 1. Validar session do caller
    let token = extract_bearer(&headers)?;
    let session = state.session_service.validate(&token).await?;
    
    // 2. Buscar ASC
    let asc = state.asc_service
        .get(&asc_id)
        .await?
        .ok_or(IdentityError::AscNotFound)?;
    
    // 3. Verificar que caller pode validar este ASC
    // (caller deve ser o owner do ASC ou um admin)
    if asc.owner_sid != session.sid && !session.is_admin() {
        return Err(IdentityError::Unauthorized);
    }
    
    // 4. Retornar validação
    Ok(Json(AscValidation {
        valid: !asc.revoked && asc.expires_at > Utc::now(),
        asc_id: asc.id,
        owner_sid: asc.owner_sid,
        container: asc.container,
        allowed_intents: asc.allowed_intents,
        scopes: asc.scopes,
        expires_at: asc.expires_at,
    }))
}

// Adicionar rota
.route("/id/asc/:id/validate", get(route_asc_validate))
```

---

## 📡 CONTRATOS ENTRE SISTEMAS

### Messenger → UBL Kernel

| Endpoint | Quando Usa | Contrato |
|----------|------------|----------|
| `POST /id/register/begin` | Registro | `{ username }` → `{ challenge_id, options }` |
| `POST /id/register/finish` | Registro | `{ challenge_id, attestation }` → `{ sid, session_token }` |
| `POST /id/login/discoverable/begin` | Login passkey | `{}` → `{ challenge_id, options }` |
| `POST /id/login/discoverable/finish` | Login passkey | `{ assertion }` → `{ sid, session_token }` |
| `GET /id/whoami` | Validar sessão | `Bearer <token>` → `{ sid, user, ... }` |

### Office → UBL Kernel

| Endpoint | Quando Usa | Contrato |
|----------|------------|----------|
| `GET /id/whoami` | Validar sessão do user | `Bearer <token>` → `{ sid, user, ... }` |
| `GET /id/asc/:id/validate` | 🆕 Validar ASC | `Bearer <token>` → `{ valid, scopes, ... }` |
| `POST /id/agents` | Criar agent LLM | `{ kind, display_name }` → `{ sid }` |
| `POST /id/agents/:sid/asc` | Emitir ASC | `{ container, intent, ... }` → `{ asc_id, signature }` |

### Messenger → Office (via WebSocket)

| Event | Quando | Headers Requeridos |
|-------|--------|-------------------|
| Qualquer request | Sempre | `Authorization: Bearer <session_token>` |
| Job execution | LLM | `Authorization: Bearer <token>` + `X-UBL-ASC: <asc_id>` |

---

## 🗂️ ARQUIVOS A CRIAR/MODIFICAR

### Messenger (TypeScript)

| Arquivo | Ação | Descrição |
|---------|------|-----------|
| `src/auth/index.ts` | Criar | Re-exports |
| `src/auth/AuthProvider.tsx` | Criar | Context + Provider único |
| `src/auth/useAuth.ts` | Criar | Hook que usa context |
| `src/auth/storage.ts` | Criar | Abstração localStorage |
| `src/auth/api.ts` | Criar | Chamadas /id/* |
| `src/auth/types.ts` | Criar | Tipos TypeScript |
| `src/hooks/useAuth.ts` | Remover | Deprecated |
| `src/context/AuthContext.tsx` | Remover | Deprecated |
| `src/App.tsx` | Modificar | Usar novo AuthProvider |

### Office (Rust)

| Arquivo | Ação | Descrição |
|---------|------|-----------|
| `src/auth/mod.rs` | Criar | Re-exports |
| `src/auth/middleware.rs` | Criar | require_auth, require_asc |
| `src/auth/asc_validator.rs` | Criar | Valida via UBL |
| `src/auth/session_validator.rs` | Criar | Valida via /whoami |
| `src/ubl_client/identity.rs` | Criar | validate_session, validate_asc |
| `src/asc.rs` | Modificar | Delegar para auth/ |
| `src/api/mod.rs` | Modificar | Usar novos middlewares |

### UBL Kernel (Rust)

| Arquivo | Ação | Descrição |
|---------|------|-----------|
| `src/identity/mod.rs` | Criar | Re-exports |
| `src/identity/config.rs` | Criar | WebAuthnConfig |
| `src/identity/error.rs` | Criar | IdentityError |
| `src/identity/types.rs` | Criar | Subject, Credential, etc |
| `src/identity/challenge/manager.rs` | Criar | ChallengeManager |
| `src/identity/webauthn/*.rs` | Criar | Services |
| `src/identity/session/types.rs` | Criar | Session canônica |
| `src/identity/agents/asc.rs` | Criar | 🆕 validate endpoint |
| `src/identity/routes.rs` | Criar | Handlers slim |
| `src/id_routes.rs` | Deprecar | Manter temporariamente |

---

## 📋 FASES HARMÔNICAS DE IMPLEMENTAÇÃO

### Fase 1: Fundação UBL (2h)
**Sistema: UBL Kernel | Risco: BAIXO**

- [x] identity/config.rs
- [x] identity/error.rs
- [ ] identity/mod.rs

### Fase 2: Endpoint ASC Validate (1h)
**Sistema: UBL Kernel | Risco: BAIXO**

- [ ] Criar `GET /id/asc/:id/validate`
- [ ] Testes do endpoint

### Fase 3: Office usa UBL (2h)
**Sistema: Office | Risco: MÉDIO**

- [ ] ubl_client/identity.rs
- [ ] auth/asc_validator.rs
- [ ] auth/middleware.rs
- [ ] Remover acesso direto ao banco para ASC

### Fase 4: Messenger Consolidado (2h)
**Sistema: Messenger | Risco: MÉDIO**

- [ ] auth/storage.ts (com migração)
- [ ] auth/AuthProvider.tsx
- [ ] auth/useAuth.ts
- [ ] Remover hooks e context antigos

### Fase 5: UBL Services (3h)
**Sistema: UBL Kernel | Risco: MÉDIO**

- [ ] identity/challenge/manager.rs
- [ ] identity/webauthn/register.rs
- [ ] identity/webauthn/login.rs
- [ ] identity/webauthn/discoverable.rs

### Fase 6: UBL Session Unificada (1h)
**Sistema: UBL Kernel | Risco: MÉDIO**

- [ ] identity/session/types.rs (Session canônica)
- [ ] Migrar usos

### Fase 7: Cleanup (1h)
**Sistema: Todos | Risco: BAIXO**

- [ ] Remover webauthn_store.rs
- [ ] Remover código deprecated
- [ ] Remover tabelas não usadas

### Fase 8: Integração E2E (2h)
**Sistema: Todos | Risco: ALTO**

- [ ] Testar Messenger → UBL Kernel
- [ ] Testar Office → UBL Kernel
- [ ] Testar fluxo completo

---

## 📊 CRONOGRAMA HARMÔNICO

```
Semana 1: Fundação
├── Dia 1: Fase 1 (UBL config/error) + Fase 2 (ASC validate)
├── Dia 2: Fase 3 (Office usa UBL)
└── Dia 3: Fase 4 (Messenger consolidado)

Semana 2: Core
├── Dia 1-2: Fase 5 (UBL Services)
├── Dia 3: Fase 6 (Session unificada)
└── Dia 4: Fase 7 (Cleanup)

Semana 3: Validação
├── Dia 1-2: Fase 8 (Integração E2E)
└── Dia 3-5: Testes, edge cases, documentação
```

---

## ✅ CHECKLIST DE VALIDAÇÃO POR SISTEMA

### Messenger ✓
- [ ] Login com passkey funciona
- [ ] Registro funciona
- [ ] Token persiste após refresh
- [ ] Logout limpa storage
- [ ] Migração da key antiga funciona

### Office ✓
- [ ] Recebe requests com Bearer token
- [ ] Valida ASC via UBL (não banco)
- [ ] Cria agents via UBL
- [ ] Emite ASC via UBL

### UBL Kernel ✓
- [ ] Todos endpoints de /id/* respondem
- [ ] /id/asc/:id/validate funciona
- [ ] Challenge anti-replay funciona
- [ ] Sessions expiram corretamente

### Integração ✓
- [ ] Messenger → login → UBL → token → Messenger funciona
- [ ] Messenger → Office (com token) → UBL → validate → OK
- [ ] Office → create agent → UBL → issue ASC → Office funciona
