# Wiring notes (/metrics, identity atoms, JWT tokens)

1) main.rs
```rust
mod metrics;
mod id_ledger;
mod id_session_token;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    metrics::init();

    let app = Router::new()
        .merge(metrics::router())          // <-- early, guaranteed non-empty
        .merge(id_routes::router())
        .merge(id_session_token::router()) // POST /id/session/token
        // .route_layer(from_fn_with_state(state.clone(), require_stepup)) // for admin routes
        .with_state(state);

    // ...
}
```

2) In id_routes.rs, after successful operations:
```rust
metrics::WEBAUTHN_OPS_TOTAL.with_label_values(&["register","begin"]).inc();
metrics::WEBAUTHN_OPS_TOTAL.with_label_values(&["register","finish"]).inc();
metrics::ID_DECISION_TOTAL.with_label_values(&["login","accept"]).inc();
metrics::RATE_LIMIT_REJECTIONS_TOTAL.with_label_values(&["login"]).inc();
metrics::PROGRESSIVE_LOCKOUT_TOTAL.with_label_values(&[&fail_count.to_string()]).inc();

// Emit ledger events
let _ = crate::id_ledger::emit_identity_event(&state, "person_registered", json!({ "username": username })).await;
```

3) Middleware (routes admin):
```rust
use axum::middleware::from_fn_with_state;
let admin = Router::new()
   .route("/id/agents/:sid/rotate", post(rotate))
   .route("/id/agents/:sid/asc/:asc_id", delete(revoke));
let admin = admin.route_layer(from_fn_with_state(state.clone(), crate::middleware_require_stepup::require_stepup));
let app = app.nest("/admin", admin);
```

4) Build:

- Add the dependencies from `Cargo.additions.toml` to your Cargo.toml.
- Run SQL `011_api_tokens.sql` migration.
- Set env for JWT in prod:
```
export JWT_ED25519_PEM='-----BEGIN PRIVATE KEY-----
...ed25519 pkcs8...
-----END PRIVATE KEY-----'
export JWT_KID='ubl-ed25519-v1'
```
```

5) Quick tests:
```bash
# /metrics non-empty
curl -s http://localhost:8080/metrics | head

# Register/Login and see counters
curl -s -X POST http://localhost:8080/id/register/begin -H 'Content-Type: application/json' -d '{"username":"alice"}' >/dev/null
curl -s http://localhost:8080/metrics | grep ubl_webauthn_operations_total

# Issue JWT
curl -s -X POST http://localhost:8080/id/session/token -H 'Content-Type: application/json' -d '{"aud":"ubl://cli","scope":["read"]}'
```
