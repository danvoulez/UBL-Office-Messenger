bora virar a página, Dan. Aqui está o “texto de transição” — do que deu ruim → para o Console v1.1 e o Registry (ADR-002) — já organizado pra orientar a correção sem drama e sem dispersão.

⸻

Transição: da bagunça para o v1.1 + ADR-002

Onde estourou (verdade nua e crua)
	•	Governança duplicada: Office tinha regras próprias e o UBL também. Resultado: dois “juízes” que não se falam.
	•	LLM sem ferramentas reais: tinha auditor de tools… mas sem tools. Zero capacidade de executar trabalho concreto.
	•	Sandbox ausente: execução sem isolamento claro por job/target. Risco operativo desnecessário.
	•	UBL não era “dono de tudo”: apps tentavam legislar sozinhos, ignorando o mandato do UBL único.
	•	Fluxos informais: push/merge/release sem card → permit → command → runner → receipt.
	•	Multi-tenant faltando: sem tenant_id de ponta a ponta, difícil oficializar catálogo por cliente/tenant.

Decisões que corrigem a rota (âncora contratual)
	•	ADR-UBL-Console-001 v1.1: um Console/PWA (verde/azul), poder nasce do Office → Permit efêmero (TTL curto), Runner-Factory (LAB 512, outbound-only), allowlist por job, Receipt verificável, multi-tenant end-to-end.
	•	ADR-UBL-Registry-002: “GitHub entre aspas” do UBL. Repos bare em S3/MinIO namespaced por tenant, e tudo passa por card → permit → command → runner → receipt. Branch rules e níveis (L0–L5) cravados em policy.

Mudanças obrigatórias (mapa de conserto)
	1.	Uma só governança: remover/silenciar office/governance/* e centralizar no UBL via POST /v1/policy/permit.
	2.	Contratos canônicos v1.1: adotar tenant_id, policy_hash, subject_hash, approval_ref* (L3+) em Permit e Command; Receipt multi-tenant.
	3.	Runner sob controle: LAB 512 pull-only, allowlist por tenant, sandbox-exec (perfil) + whitelist de rede (MinIO).
	4.	Cards obrigatórios: Messenger emite register_project, repo_push, tag_release, merge_protected.
	5.	Registry oficial: repos em s3://repos/<tenant>/<project>.git + metadados e releases sob registry-metadata/ e artifacts/.
	6.	Políticas que valem na prática: níveis L0–L5 + branch rules; TTL por nível; jti single-use; binding a (jobType,target,subject_hash,policy_hash).
	7.	Projeções úteis: /query/registry/* e /query/commands por tenant_id e target.

Cutover enxuto (datas concretas)
	•	Hoje (D0 – 27/12): comitar ADR-001 v1.1 e ADR-002 + configs (offices.yaml, risk_policy.yaml, allowlist.json, schemas).
	•	D1: aplicar migração SQL (console_v1_1.sql e registry_v1_1.sql).
	•	D2: publicar allowlist no Runner (LAB 512) + sandbox profile + ACL MinIO.
	•	D3: ligar endpoints v1.1 (/v1/policy/permit, /v1/commands/issue, /v1/exec.finish, /v1/query/registry/*).
	•	D4: ativar Cards no Messenger; testar register_project e repo_push (feature/)*.
	•	D5: ligar branch rules e níveis; tag_release com step-up (azul).
	•	D+7 (03/01/2026 12:00 UTC): corte: rejeitar envelopes sem tenant_id|subject_hash|policy_hash.

Prova de Done (não é “parece que foi”, é foi)
	1.	Projeto oficial: Card register_project → approve → s3://repos/T.UBL/P.demo.git + evento project.registered.
	2.	Push governado: Card repo_push → approve → project.activity + ref atualizada.
	3.	Release protegida: Card tag_release v1.2.0 (L4) → step-up → manifest publicado.
	4.	Segurança funcionando: tentar reutilizar Permit expirado → denied; tentar main sem approval → denied.

Riscos e mitigação (já endereçados na 1.1/002)
	•	Replay/abuso: jti single-use + TTL curto + binding de escopo → fail-closed.
	•	Acesso de rede indevido: Runner sem portas; whitelist estrita (MinIO).
	•	Desalinhamento tenant: tudo carrega tenant_id; storage e projeções namespaced.
	•	Builds do “azul”: CSP/SRI e UI com contador de TTL + hold-to-approve.

O que fica para o trilho seguinte (sem travar agora)
	•	Policy → WASM (TDLN fiel) para portabilidade browser/edge — mantemos bytecode atual, planejamos compilador WASM como etapa posterior.
	•	Tooling extra (CI, issues, wiki) como novos jobs governados; não antes do cutover.

⸻

Checklist operacional (uma tela)
	•	Comitar ADR-001 v1.1 + ADR-002 + arquivos de config (já no chat).
	•	Rodar migrações SQL (console + registry).
	•	Publicar allowlist do Runner (LAB 512) + sandbox.sb + ACL MinIO.
	•	Subir/validar endpoints v1.1 no LAB 256.
	•	Ligar templates de Cards no Messenger.
	•	Executar 3 cenários de DoD (acima).
	•	Programar cutover para 03/01/2026 12:00 UTC.

se quiser, já sigo com os handlers Axum prontos dessas rotas, no mesmo padrão dos contratos v1.1 — mas essa transição aqui já te dá a linha de chegada e o caminho sem atalhos.