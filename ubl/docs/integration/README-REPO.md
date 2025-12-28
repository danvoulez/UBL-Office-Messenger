# UBL Repo/Git Integration (Static Container)

## Endpoints (server)
- `POST /repo/presign` → retorna URLs de upload (MinIO 'mc' fallback)
- `POST /repo/commit-ref` → cria átomo `git/ref` (Δ=0) e executa `link/commit`

## Fluxo (CLI)
```bash
# Build SDK/CLI
(cd clients/ts/sdk && npm i && npm run build)
(cd clients/cli && npm i && npm run build && npm link)

# Presumindo SID (session token) válido
export SID="...session..."

# Push diretório como static (ff)
ubl repo push --tenant acme --repo billing --ref refs/heads/main --dir ./site --sid "$SID"
```

## MinIO
```bash
export ENDPOINT=http://10.8.0.2:9000
export ACCESS=...
export SECRET=...
bash templates/minio/alias-setup.sh
export MINIO_ALIAS=ubl
export MINIO_BUCKET_REPOS=vault-repos
```

## Observações
- Δ=0 sempre (Observation). Force push exige PACT mais forte a nível de política (campo `mode: "force"`). A membrana não lê semântica; a **política TDLN** é quem diferencia.
- O endpoint `/repo/presign` usa o `mc` como fallback pragmático. Troque por AWS SDK (S3) quando desejar.
