# UBL 2.0 — Makefile
DB_URL ?= postgres://ubl_dev@localhost:5432/ubl_dev
.PHONY: sql-init sql-reset tail doctor
sql-init:
	psql "$(DB_URL)" -v ON_ERROR_STOP=1 -f sql/001_identity.sql
	psql "$(DB_URL)" -v ON_ERROR_STOP=1 -f sql/002_ledger.sql
sql-reset:
	psql "$(DB_URL)" -v ON_ERROR_STOP=1 -c "DROP TABLE IF EXISTS id_api_token,id_session,id_challenge,id_webauthn,id_asc,id_subject CASCADE;"
	psql "$(DB_URL)" -v ON_ERROR_STOP=1 -c "DROP TABLE IF EXISTS ledger_entry CASCADE;"
	$(MAKE) sql-init
tail:
	@echo 'curl -N http://localhost:8080/ledger/$$CONTAINER_ID/tail'
doctor:
	@psql "$(DB_URL)" -c "select now();" 
check-ledger:
	@psql "$(DB_URL)" -c "\d ledger_entry"
check-id:
	@psql "$(DB_URL)" -c "\d id_subject" && psql "$(DB_URL)" -c "\d id_webauthn"

api-preview:
	@echo "OpenAPI: open openapi/ubl.openapi.yaml in your viewer (or redoc-cli serve)"
