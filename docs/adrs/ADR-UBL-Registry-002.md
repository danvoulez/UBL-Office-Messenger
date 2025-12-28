ADR-UBL-Registry-002 — Office Git Registry (multi-tenant: card → permit → command → runner → receipt)

Status: Aprovado
Data: 27-dez-2025
Owner: Dan (LAB 512)
Depende de: ADR-UBL-Console-001 (v1.1)

1) Contexto
	•	Precisamos de um “GitHub entre aspas” oficial, governado pelo UBL, onde projetos viram oficiais via card e execução auditável.
	•	Requisitos: multi-tenant, allowlist de jobs, Runner outbound-only, S3/MinIO como storage de repositórios e metadados.

2) Decisão
	•	Um Registry oficial por UBL, multi-tenant, onde cada projeto é um repo Git bare em S3/MinIO (repos/<tenant>/<project>.git).
	•	Todo fluxo é card → Permit → Command → Runner → Receipt.
	•	Scopes de Permit incluem tenant_id, jobType, target, subject_hash, policy_hash, approval_ref* (L3+).
	•	Branch rules por nível: feature/* e review/* livres (L0–L2); main/release/* protegidas (L4/L5).

3) Escopo

Dentro: registro de projetos, push oficial, releases, catálogo, feed de atividades.
Fora (v1): issues, wiki e CI pública (virão como jobs dedicados).

4) Nós & papéis
	•	LAB 256: UBL + APIs (/v1/policy/permit, /v1/commands, /v1/query, /v1/exec.finish).
	•	LAB 512: Runner-Factory + MinIO (S3) + segredos (pull-only).
	•	LAB 8GB: Messenger/Admin (UI), rascunhos locais (não oficial).

5) Estrutura S3 (namespacing por tenant)

s3://repos/<tenant_id>/<project_id>.git
s3://registry-metadata/<tenant_id>/projects/<project_id>/{project.json,index.json}
s3://registry-metadata/<tenant_id>/releases/<project_id>/<tag>/manifest.json
s3://artifacts/<tenant_id>/<project_id>/<commit>/...

6) Jobs permitidos (allowlist)
	•	git.registry.init — cria bare repo + project.json.
	•	git.registry.push — aplica patch/bundle, valida branch/policy, atualiza ref e emite atividade.
	•	git.registry.tag_release — cria tag vX.Y.Z, publica manifest da release.
	•	git.registry.merge_protected (v1.1) — merge em main/release/* com Pact/attestation.

Runner: allowlist por tenant com fs_scope dedicado e network_whitelist mínima (MinIO).

7) Eventos & Projeções (Container C.Registry::<tenant_id>)

Eventos:
	•	project.registered { project_id, name, owners[], visibility }
	•	project.repo.bound { project_id, repo_url }
	•	project.activity { project_id, action, actor, ref, commit }
	•	project.release.tagged { project_id, tag, commit, notes_hash }

Projeções:
	•	/query/registry/projects?tenant_id=… → { id, name, owners, visibility, repo_url, last_activity }
	•	/query/registry/project/:id?tenant_id=… → detalhe + refs + últimas releases

8) Interfaces (v1.1)

Permit (JSON)

{
  "aud": "runner:LAB_512",
  "jti": "uuid",
  "exp": 1735320000000,
  "sig": "base64",
  "scopes": {
    "tenant_id": "T.UBL",
    "jobType": "git.registry.init|git.registry.push|git.registry.tag_release|git.registry.merge_protected",
    "target": "LAB_512",
    "subject_hash": "blake3(params)",
    "policy_hash": "blake3(policy.wasm)",
    "approval_ref": "k-<card_id>"
  }
}

CommandEnvelope (JSON)

{
  "jti": "uuid",
  "tenant_id": "T.UBL",
  "jobId": "job-uuid",
  "jobType": "git.registry.push",
  "params": {
    "project_id": "P.demo",
    "branch": "feature/hello",
    "patch": "<git-bundle-or-diff-base64>",
    "commit_summary": "feat: hello",
    "subject_hash": "<blake3_of_patch>"
  },
  "subject_hash": "blake3_of_patch",
  "policy_hash": "blake3(policy.wasm)",
  "permit": { /* acima */ },
  "target": "LAB_512",
  "office_id": "office::default"
}

Receipt (JSON)

{
  "tenant_id": "T.UBL",
  "jobId": "job-uuid",
  "status": "success|error|denied|expired",
  "finished_at": 1735320000001,
  "logs_hash": "blake3",
  "artifacts": ["{\"commit\":\"abc\",\"branch\":\"feature/hello\"}"],
  "usage": { "wall_ms": 1234 },
  "error": ""
}

9) Policies (níveis & branches)
	•	L0–L2: repo_push em feature/* e review/* (card opcional).
	•	L3: register_project (card obrigatório).
	•	L4: merge_protected(main) & tag_release (card + step-up).
	•	L5: críticos (quorum + attestation quando exigido).

10) Endpoints (LAB 256)
	•	POST /v1/policy/permit
	•	POST /v1/commands/issue
	•	GET  /v1/query/commands?tenant_id=…&target=LAB_512&pending=1
	•	POST /v1/exec.finish
	•	GET  /v1/query/registry/projects?tenant_id=…
	•	GET  /v1/query/registry/project/:id?tenant_id=…

11) Rollout & DoD
	1.	Schemas com tenant_id em commands/receipts/registry_*.
	2.	Allowlist git.registry.* por tenant + whitelist MinIO.
	3.	Cards no Messenger (register_project / repo_push / tag_release / merge_protected).
	4.	Policies publicadas por tenant (branch rules e níveis).
	5.	Projeções /query/registry/* ativas.

Proof of Done
	•	Registrar P.demo em T.UBL via card → approve → repo s3://repos/T.UBL/P.demo.git + project.registered.
	•	repo_push em feature/x → approve → project.activity + commit refletido.
	•	tag_release v1.2.0 (L4) → manifest publicado e exibido no catálogo.

12) Segurança & Consequências
	•	Runner pull-only, sem exposição; network mínima whitelisted.
	•	Fail-closed com TTL curto; jti single-use; binding a (jobType,target,subject_hash,policy_hash).
	•	UI Admin com contador de TTL, diff do plano e hold-to-approve.
	•	Ganha-se governança fractal multi-tenant reutilizando a mesma infra.

⸻

se quiser, eu já te mando também os arquivos seed correspondentes (offices.yaml, risk_policy.yaml, jobs.allowlist.json, schemas.sql) no mesmo estilo destes ADRs.