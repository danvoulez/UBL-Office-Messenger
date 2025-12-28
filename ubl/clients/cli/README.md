# UBL CLI — v0.1.0

Status: draft • Generated: 2025-12-26T06:11:44.651874Z

Uma CLI **LLM-friendly** e **rigorosa** para operar o UBL com baixo atrito humano.

## Comandos (MVP)

### Config
- `ubl config init` — cria `~/.ubl/config.json` (server, auth).
- `ubl config get [key]` — lê config.
- `ubl config set <key> <value>` — grava config.

### Identidade (PR28)
- `ubl id agent:create --kind <llm|app> --name <DISPLAY> [--pub <hex>]` → cria sujeito; retorna `sid`.
- `ubl id whoami` — consulta sujeito/sessões atuais.

### Átomo / Link
- `ubl atom canonicalize <file.json>` — imprime JSON✯Atomic canônico.
- `ubl atom hash <file.json>` — imprime BLAKE3 do canônico (sem domain tag).
- `ubl link sign --version 1 --container <hex32> --sequence <u64> --prev <hex32> --atom <hex32> --class <0..3> --delta <i128> --priv <hex>` — imprime assinatura Ed25519 e `signing_bytes` em hex.

### S3 (opcional)
- `ubl s3 draft-push --src dist/ --dst minio512/ubl-drafts/<proj>/<comp>/<sha>/` — passthrough para `mc cp`.

## Done if…
1. `ubl atom hash sample.json` produz hash igual ao obtido no Rust.
2. `ubl link sign ...` gera **signing_bytes** idêntico ao vetor de teste.
3. `ubl id agent:create ...` retorna `sid` válido.
4. `ubl id whoami` funciona quando autenticado.

## Testes
- `npm test` — valida `i128` big-endian, ordenação canônica, e snapshot de `signing_bytes`.


## Novidades (2025-12-26T06:12:44.135282Z)

### Commits
- `ubl commit make --version --container --sequence --prev --atom --class --delta --priv` → imprime envelope JSON pronto.
- `ubl commit send --file link.json` → POST `/commit` no servidor configurado.

### Tail (Ledger SSE)
- `ubl tail` — conecta em `/tail` (SSE) e imprime `data:`.

### UBL ID — ASC & Rotação & ICTE
- `ubl id asc:issue --sid <sid> --containers c1,c2 --classes 0,1 --max-delta <i128> --ttl <sec>`
- `ubl id rotate --sid <sid> --pub <hex>`
- `ubl id ict:begin --scope <container> --ttl <sec>` / `ubl id ict:finish --token <id>`



## Extra (Pro) — 2025-12-26T06:13:46.135553Z

### Verificação
- `ubl commit verify --file link.json` — recomputa `signing_bytes` e **verifica** a assinatura Ed25519.

### Publicação
- `ubl publish --project <p> --component <c> --version <x.y.z> --manifest manifests/RELEASE.json \\
    --dist dist/ --container <hex32> --sequence <u64> --prev <hex32> --priv <ed25519>`
  1) canonicaliza manifesto → `atom_hash`
  2) `POST /commit` (Observation Δ=0) no LAB 256
  3) `mc cp` dist/ + manifesto → `minio://<alias>/<bucket_official>/<p>/<c>/<ver>/`

### Tail avançado
- `ubl tail --jsonl out.ndjson --container <hex32>` — persiste eventos e filtra por container.

### UBL ID (admin)
- `ubl id asc:list --sid <sid>`
- `ubl id asc:revoke --sid <sid> --asc <asc_id>`

### Diagnóstico
- `ubl doctor` — checa server `/healthz`, `/tail`, e presença do `mc`.


## Ultra switches — 2025-12-26T06:15:11.833864Z
- `commit verify --strict` → valida **shape** (hex32/hex64), **classe/delta** coerentes, além da assinatura.
- `publish --dry-run` → imprime plano (hash, signing_bytes, destino S3) sem side-effects.
- `id asc:issue --from-file asc.json` → emite ASC a partir de payload declarativo.
- `tail --since-seq N` → reconecta a partir de um cursor.
- `doctor --db` → verifica saúde do Postgres via `/healthz/db` (se existir).


## MAX add-ons — 2025-12-26T06:16:15.014994Z
- `commit make --pact-file pact.json` → inclui `pact` no envelope (fora dos signing_bytes).
- `publish --verify` → verifica assinatura local **antes** do `/commit`.
- `tail --pretty` → imprime `"[seq] C<class>@Δ<delta> atom=<hash8> cid=<cid8>"` se possível.
- `id export --sid <sid> [--out file]` → backup JSON do agente.

### Exemplos
```bash
# commit com pacto
./dist/index.js commit make ... --pact-file pact.json --out link.json

# publish com verificação
./dist/index.js publish ... --verify

# tail bonito
./dist/index.js tail --pretty --container <hex32>

# exportar identidade
./dist/index.js id export --sid <sid> --out agent.json
```


## MAX+ refinamentos — 2025-12-26T06:17:18.934887Z
- `commit verify --strict --pretty` → diagnóstico colorido (strict, signing_bytes, signature).
- `publish --verify --dry-run` → combina verificação local com ensaio sem efeitos.
- `id export --redact` → ofusca campos sensíveis (ex.: `private_key`, tokens).
- `tail --pretty` já incluso (exibe `seq/class/delta/atom[0:8]/cid[0:8]`).

### Exemplos rápidos
```bash
./dist/index.js commit verify --file link.json --strict --pretty
./dist/index.js publish ... --verify --dry-run
./dist/index.js id export --sid <sid> --redact --out public_agent.json
./dist/index.js tail --pretty --since-seq 5000
```


## ULTIMATE add-ons — 2025-12-26T06:18:00.833089Z
- `doctor --runner` → verifica `runsc` (gVisor), `nsjail` e presença de `iptables`.
- `commit make --strict` → bloqueia envelopes inválidos (hex32/64, coerência classe↔delta).
- `publish --plan` → imprime plano detalhado (destino, arquivos, bytes, SHA-256) — pode combinar com `--dry-run`.

### Exemplos
```bash
# Runner probes
./dist/index.js doctor --runner

# Commit estrito
./dist/index.js commit make ... --strict --out link.json

# Plano de publicação (com verify e sem efeitos)
./dist/index.js publish ... --verify --plan --dry-run
```


## HYPER add-ons — 2025-12-26T06:19:03.124420Z
- `runner exec` → payload declarativo com `egress_whitelist` e limites (seconds/mem/tmpfs). Suporta `--dry-run`.
- `pack` → gera `UBL-RELEASE-MANIFEST/1.0` com lista de arquivos, bytes e SHA-256.
- `doctor --perf` → mede latência `/healthz`, **opcionalmente** `/commit` (via env `UBL_PERF_COMMIT=1`) e throughput do `/tail` (env `SEC=5`).

### Exemplos
```bash
# Runner (ensaio)
./dist/index.js runner exec --container <hex32> --type cmd --cmd "python" --args "-V" --egress "api.example.com" --dry-run

# Manifesto de release
./dist/index.js pack --project arena --component ui --version 0.3.0 --dist ./build --out manifests/RELEASE.json

# Perf (apenas healthz/tail)
./dist/index.js doctor --perf
# Perf commit (cautela! requer vars)
UBL_PERF_COMMIT=1 CID=<hex32> PRIV=<ed25519-priv-hex> PREV=<hex32> SEQ0=1 ITER=5 SEC=5 ./dist/index.js doctor --perf
```
