ADR-UBL-Console-001 (v1.1) — Console único, Runner-Factory e Poder por Office

Status: Aprovado
Data: 27-dez-2025
Owner: Dan (LAB 512)
V1.0: mantido • Emenda v1.1: multi-tenant + reforços de segurança + integração com Registry (ADR-002)

1) Contexto
	•	O UBL é único e soberano (policies, identidade, ledger).
	•	Operação do ambiente (LAB 512/256/8GB) e dos apps UBL deve ser governada por Office → Permit → Runner → Receipt, com auditoria total.
	•	O Console UBL é um PWA com dois modos: Operator (verde) L0–L2 e Admin (azul) L3–L5 com step-up WebAuthn.
	•	Execução acontece no Runner-Factory (LAB 512, outbound-only) e, opcionalmente, no Pocket Runner (iPhone, L0–L2).
	•	v1.1 traz multi-tenant end-to-end e escopos mais fortes de Permit para suportar o Office Git Registry (ADR-002).

2) Decisão
	•	Um único Console (PWA): Operator (verde) L0–L2; Admin (azul) L3–L5 com step-up.
	•	Poder nasce do Office: cada ação emite Permit efêmero (single-use, TTL curto) assinado no device.
	•	Runner-Factory no LAB 512 (pull/outbound-only) executa jobs nomeados por allowlist; Receipt assinado volta e é registrado.
	•	Multi-tenant obrigatório: tenant_id presente em Permit, Command, Receipt e eventos/projeções.
	•	Escopos fortes no Permit: tenant_id, jobType, target, subject_hash, policy_hash, approval_ref* (L3+).
	•	Política de risco L0–L5 com TTL por nível e binding anti-replay.

3) Escopo

Dentro: operação de LAB 512/256/8GB; execução de jobs infra/app; emissão e consumo de Permits; auditoria/receipts.
Fora (v1): issues/wiki públicas; CI genérica (entra depois via jobs dedicados).

4) Nós & papéis
	•	LAB 512 — Factory Runner: serviços pesados, inferência, execuções L3–L5; outbound-only.
	•	LAB 256 — Worker (default).
	•	LAB 8GB — Worker (lowram) + dev local.
	•	iPhone — Console UBL (Operator+Admin) + Pocket Runner (opcional, L0–L2).

Offices oficiais (v1):
	•	office:infra.deployer (ceiling L4; alguns L5 com quorum)
	•	office:cluster.operator (L3–L4)
	•	office:workstation.maintenance (L2–L3)

Cada Office define: allowed_actions[], scope{targets}, risk_ceiling, approval_policy, ttl_default.

5) Política de risco e aprovação
	•	L0–L2 (consultas/toggles reversíveis): sem aprovação; TTL 1–2 min.
	•	L3 (efeito moderado): card com aprovação assíncrona; TTL 5 min.
	•	L4 (custo/infra crítica): step-up WebAuthn + texto do plano; TTL 2–5 min pós step-up.
	•	L5 (soberania/esquema): step-up + quorum (≥2) e, quando exigido, attestation A2.

6) Allowlist de jobs (seed v1)
	•	service.restart(name=minio|ray|nginx)
	•	infra.ray.scale_worker(target=LAB256, count=±N)
	•	infra.ray.set_profile(target=LAB8GB, profile=lowram)
	•	backup.minio.gc(bucket=*)
	•	docker.prune_safe()
	•	system.pkg.update(target=LAB256, manager=brew, pkg=…)
	•	system.security.patch(target=LAB256|LAB8GB)
	•	observer.health.scan(scope=cluster)
	•	logs.snapshot.hash_and_ship()
	•	ledger.verify.receipts(window=24h)
(+ família git.registry.* definida no ADR-002)

7) Guardrails obrigatórios
	•	Runner sem portas abertas; tudo pull/outbound.
	•	Console não guarda segredos; apenas fabrica/assina Permits efêmeros.
	•	UI segura: borda azul persistente em Admin, “Active Office: X”, contador de TTL, hold-to-approve e diff do plano.
	•	Target explícito: LAB_512|LAB_256|LAB_8GB (sem wildcard).
	•	Anti-replay/binding: Permit vinculado a (jobType,target,subject_hash,policy_hash) + jti + exp.
	•	CSP+SRI: bundle azul com políticas restritivas.

8) Interfaces canônicas (v1.1)

Permit (JSON)

{
  "aud": "runner:LAB_512",
  "jti": "uuid",
  "exp": 1735320000000,
  "sig": "base64",
  "scopes": {
    "tenant_id": "T.UBL",
    "jobType": "service.restart|...|git.registry.push",
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
  "jobType": "service.restart",
  "params": { "name": "minio", "target": "LAB_512" },
  "subject_hash": "blake3",
  "policy_hash": "blake3(policy.wasm)",
  "permit": { /* acima */ },
  "target": "LAB_512",
  "office_id": "office::default"
}

Receipt (JSON)

{
  "tenant_id": "T.UBL",
  "jobId": "job-uuid",
  "status": "OK|ERROR|denied|expired",
  "finished_at": 1735320000001,
  "logs_hash": "blake3",
  "ret": {"summary":"minio restarted"},
  "usage": {"wall_ms": 1234},
  "error": ""
}

9) Endpoints (LAB 256)
	•	POST /v1/policy/permit → {tenant_id, actor_id, intent, context, jobType, params, target, approval_ref?}
	•	POST /v1/commands/issue → CommandEnvelope v1.1
	•	GET  /v1/query/commands?tenant_id=…&target=LAB_512&pending=1&limit=…
	•	POST /v1/exec.finish → Receipt v1.1

10) Migração & Cutover
	•	Compatibilidade: aceitar v1.0 apenas para tenant_id=T.UBL com warning.
	•	Corte definitivo: rejeitar envelopes sem tenant_id|subject_hash|policy_hash.
	•	Data recomendada de corte: 2026-01-03 12:00 UTC (ajustável).

11) Proof of Done
	•	3 eventos office.activate (um por Office).
	•	1 execução L3 real (service.restart minio @ LAB_512) com exec.start/finish + Receipt verificável no Console.
	•	Tentativa com Permit expirado → denied conforme esperado.

12) Consequências
	•	+Segurança, +auditoria, +clareza operacional.
	•	Ligeiro overhead (aprovações/step-up), compensado por fail-closed e rastreabilidade.

⸻

