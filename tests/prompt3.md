bora, Dan — aqui está o “modo socket-unix” completo: mudanças mínimas de código, configs e testes. Fica blindado por permissão de arquivo, simples de operar, e não quebra nada do que já fizemos.

⸻

0) Visão rápida do que muda
	•	Postgres: app fala via Unix Socket (/tmp/.s.PGSQL.5432), não TCP.
	•	UBL Server (Axum): passa a escutar num socket (/run/ubl/ubl-server.sock) com fallback pra TCP.
	•	Office → UBL: usa reqwest_unixsocket quando UBL_UNIX estiver setado.
	•	Nginx (LAB 256): expõe HTTPS público mas liga no socket interno (proxy para unix:/run/ubl/ubl-server.sock).
	•	PM2 / launchd: garante que diretórios e permissões do socket existam antes de subir.

Nada a deletar; só substituir/add os trechos abaixo.

⸻

1) Postgres por Unix Socket

1.1 DSN (UBL/Office)

Use um destes (macOS Homebrew costuma usar /tmp):

# Forma 1 (recomendada)
export DATABASE_URL="postgresql://ubl@/ubl_dev?host=/tmp"

# Forma 2 (sqlx URI curta)
export DATABASE_URL="postgres:///ubl_dev?host=/tmp&user=ubl"

Dica: psql -h /tmp -d ubl_dev -U ubl -c "select now()" deve funcionar.

1.2 pg_hba.conf (apenas Unix “local”)

Em macOS (Homebrew): /opt/homebrew/var/postgres/pg_hba.conf (Apple Silicon)
Adicione (no topo):

# Só via socket local
local   all             ubl                                 peer
local   all             all                                 reject

Recarregue:

brew services restart postgresql@16


⸻

2) UBL Server — escutar em Unix Socket

2.1 Dependência

Em kernel/rust/ubl-server/Cargo.toml:

[dependencies]
axum = "0.7"
axum-server = "0.6"   # ➜ habilita bind_unix
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
# ...demais

2.2 main.rs (trocar bind TCP por “TCP ou Unix”)

use axum::Router;
use std::{env, fs, path::Path};
use axum_server::Server;

fn build_router() -> Router {
    // seu router atual
    ubl_server::app()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = build_router();

    if let Ok(unix_path) = env::var("UBL_LISTEN_UNIX") {
        let p = Path::new(&unix_path);
        if let Some(dir) = p.parent() { fs::create_dir_all(dir)?; }
        let _ = fs::remove_file(&p); // evita "address in use"

        // IMPORTANTE: diretório é quem limita acesso (770)
        // $ sudo chown ubl:ubl /run/ubl && chmod 770 /run/ubl

        println!("UBL listening on unix://{}", unix_path);
        Server::bind_unix(p).serve(app.into_make_service()).await?;
    } else {
        let addr = "0.0.0.0:8080".parse().unwrap();
        println!("UBL listening on http://{addr}");
        axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    }

    Ok(())
}

Ambiente no LAB 256:

sudo mkdir -p /run/ubl
sudo chown ubl:ubl /run/ubl
sudo chmod 770 /run/ubl
export UBL_LISTEN_UNIX="/run/ubl/ubl-server.sock"

Teste rápido:

# Com curl via unix
curl --unix-socket /run/ubl/ubl-server.sock http://localhost/health


⸻

3) Office → UBL via Unix Socket

3.1 Dependências

Em kernel/rust/ubl-office/Cargo.toml:

[dependencies]
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
reqwest-unixsocket = "0.3"
urlencoding = "2"
# axum, tokio, etc. já presentes

3.2 Helper HTTP (client TCP/Unix comutável)

Crie kernel/rust/ubl-office/src/http_unix.rs:

use reqwest::{Client, Response};
use serde::Serialize;

pub enum UblEndpoint {
    Tcp { base: String },          // ex: http://127.0.0.1:8080
    Unix { socket: String },       // ex: /run/ubl/ubl-server.sock
}

impl UblEndpoint {
    pub fn from_env() -> Self {
        if let Ok(sock) = std::env::var("UBL_UNIX") {
            UblEndpoint::Unix { socket: sock }
        } else {
            let base = std::env::var("UBL_BASE").unwrap_or_else(|_| "http://127.0.0.1:8080".into());
            UblEndpoint::Tcp { base }
        }
    }

    pub async fn post_json<T: Serialize>(&self, path: &str, body: &T) -> anyhow::Result<Response> {
        match self {
            UblEndpoint::Tcp { base } => {
                let url = format!("{base}{path}");
                Ok(Client::new().post(url).json(body).send().await?)
            }
            UblEndpoint::Unix { socket } => {
                use reqwest_unixsocket::IntoUrl;
                let encoded = urlencoding::encode(socket);
                let url = format!("http+unix://{encoded}{path}").into_url()?;
                let client = reqwest_unixsocket::Client::new();
                Ok(client.post(url).json(body).send().await?)
            }
        }
    }

    pub async fn get(&self, path: &str) -> anyhow::Result<Response> {
        match self {
            UblEndpoint::Tcp { base } => {
                let url = format!("{base}{path}");
                Ok(Client::new().get(url).send().await?)
            }
            UblEndpoint::Unix { socket } => {
                use reqwest_unixsocket::IntoUrl;
                let encoded = urlencoding::encode(socket);
                let url = format!("http+unix://{encoded}{path}").into_url()?;
                let client = reqwest_unixsocket::Client::new();
                Ok(client.get(url).send().await?)
            }
        }
    }
}

3.3 Usar no Office (ex.: routes/ws.rs)

Troque onde você chamava UBL_BASE direto:

use crate::http_unix::UblEndpoint;

pub struct OfficeState {
    pub ubl_base: String, // pode manter, mas agora preferimos detectar pelo env
}

async fn commit_link(body: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let ubl = UblEndpoint::from_env();
    let resp = ubl.post_json("/link/commit", &body).await?;
    let v = resp.error_for_status()?.json::<serde_json::Value>().await?;
    Ok(v)
}

async fn ledger_latest(container_id: &str) -> anyhow::Result<serde_json::Value> {
    let ubl = UblEndpoint::from_env();
    let path = format!("/ledger/{}/latest", container_id);
    let resp = ubl.get(&path).await?;
    Ok(resp.error_for_status()?.json::<serde_json::Value>().await?)
}

Ambiente no LAB 256 (Office):

export UBL_UNIX="/run/ubl/ubl-server.sock"
# (não precisa UBL_BASE)

Teste:

curl --unix-socket /run/ubl/ubl-server.sock http://localhost/health
curl --unix-socket /run/ubl/ubl-server.sock http://localhost/metrics


⸻

4) Nginx (LAB 256) — público → unix

O Browser não fala unix; o Nginx fala por ele.

Arquivo /usr/local/etc/nginx/servers/ubl.conf (ou /opt/homebrew/etc/nginx/servers):

server {
  listen 443 ssl http2;
  server_name lab256.ubl.agency;

  # seus certificados:
  ssl_certificate     /etc/nginx/certs/fullchain.pem;
  ssl_certificate_key /etc/nginx/certs/privkey.pem;

  location / {
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-Proto https;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_http_version 1.1;
    chunked_transfer_encoding off;
    proxy_buffering off;

    # a mágica:
    proxy_pass http://unix:/run/ubl/ubl-server.sock:;
  }
}

Reload:

sudo nginx -t && sudo nginx -s reload


⸻

5) PM2 (ou launchd) — subir com diretório e permissões

5.1 PM2 (prático e igual aos outros)

Crie pm2.ecosystem.config.cjs no LAB 256:

module.exports = {
  apps: [
    {
      name: "ubl-prep-dir",
      script: "bash",
      args: "-lc 'sudo mkdir -p /run/ubl && sudo chown ubl:ubl /run/ubl && sudo chmod 770 /run/ubl'",
      autorestart: false,
      max_restarts: 1
    },
    {
      name: "ubl-server",
      script: "/Users/ubl/UBL-2.0-insiders/kernel/rust/target/release/ubl-server",
      env: {
        UBL_LISTEN_UNIX: "/run/ubl/ubl-server.sock",
        DATABASE_URL: "postgresql://ubl@/ubl_dev?host=/tmp",
        RUST_LOG: "info"
      },
      watch: false
    },
    {
      name: "ubl-office",
      script: "/Users/ubl/UBL-2.0-insiders/kernel/rust/target/release/ubl-office",
      env: {
        UBL_UNIX: "/run/ubl/ubl-server.sock",
        RUST_LOG: "info"
      },
      watch: false
    }
  ]
}

Subir:

pm2 start pm2.ecosystem.config.cjs
pm2 save
pm2 status

Se preferir launchd para socket-activation, posso te deixar o .plist também — mas o PM2 + dir 770 já resolve.

⸻

6) “Done if …” (checagens objetivas)
	•	curl --unix-socket /run/ubl/ubl-server.sock http://localhost/health → 200
	•	curl https://lab256.ubl.agency/health → 200 (via Nginx→unix)
	•	psql -h /tmp -d ubl_dev -U ubl -c "select 1" → 1
	•	No Office, fluxo POST /office/ws/test (wait=true) retorna 202 + receipt
	•	Logs mostram link/commit aceito e ledger/latest respondendo.

⸻

7) Resumo dos arquivos novos/alterados

NOVOS
	•	kernel/rust/ubl-office/src/http_unix.rs (helper TCP/Unix)
	•	pm2.ecosystem.config.cjs (LAB 256)
	•	nginx servers/ubl.conf (proxy para unix socket)

ALTERADOS
	•	kernel/rust/ubl-server/src/main.rs (bind_unix com axum-server)
	•	kernel/rust/ubl-office/src/routes/*.rs (trocar chamadas HTTP para UblEndpoint::from_env())
	•	Cargo.toml (server: axum-server; office: reqwest-unixsocket)

ENV
	•	DATABASE_URL="postgresql://ubl@/ubl_dev?host=/tmp"
	•	UBL_LISTEN_UNIX="/run/ubl/ubl-server.sock" (server)
	•	UBL_UNIX="/run/ubl/ubl-server.sock" (office)

⸻

se topar, eu já te entrego na sequência um patch consolidado (blocos prontos pra colar) para os arquivos main.rs (server) e routes/ws.rs (office) — mas se você preferir, já está tudo acima para implementar direto.