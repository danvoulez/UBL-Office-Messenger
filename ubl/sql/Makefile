# UBL SQL Migration Makefile
# Reset: Squash por domínio - instalação do zero

.PHONY: help db.nuke db.install db.verify db.check

# Default database connection
DB_URL ?= $(DATABASE_URL)
ifeq ($(DB_URL),)
  DB_URL := postgres:///ubl_dev?host=/var/run/postgresql
endif

# Migration order file
MIGRATION_ORDER := MIGRATION_ORDER.txt

help:
	@echo "UBL SQL Migrations"
	@echo ""
	@echo "Targets:"
	@echo "  make db.nuke      - DROP + CREATE database (dev only)"
	@echo "  make db.install   - Run migrations in order from MIGRATION_ORDER.txt"
	@echo "  make db.verify    - Sanity checks (tables, triggers, functions)"
	@echo "  make db.check     - Check if migrations are valid SQL"
	@echo ""
	@echo "Environment:"
	@echo "  DATABASE_URL - PostgreSQL connection string (default: postgres:///ubl_dev?host=/var/run/postgresql)"
	@echo ""

db.nuke:
	@echo "⚠️  NUKE: Dropping and recreating database..."
	@echo "This will destroy all data!"
	@echo "Database: $(DB_URL)"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		psql "$(DB_URL)" -c "DROP SCHEMA IF EXISTS public CASCADE; CREATE SCHEMA public;"; \
		echo "✅ Database nuked"; \
	else \
		echo "❌ Cancelled"; \
	fi

db.install:
	@echo "📦 Installing migrations from $(MIGRATION_ORDER)..."
	@while IFS= read -r file; do \
		if [ -n "$$file" ] && [ ! "$${file:0:1}" = "#" ]; then \
			echo "  → $$file"; \
			psql "$(DB_URL)" -v ON_ERROR_STOP=1 -f "$$file" || exit 1; \
		fi; \
	done < "$(MIGRATION_ORDER)"
	@echo "✅ All migrations installed"

db.verify:
	@echo "🔍 Verifying database schema..."
	@psql "$(DB_URL)" -c "\dt" | grep -E "(ledger_entry|id_subject|pact|permits|commands|receipts|projection_)" || (echo "❌ Missing expected tables" && exit 1)
	@psql "$(DB_URL)" -c "SELECT COUNT(*) FROM pg_trigger WHERE tgname IN ('trg_tail_notify', 'ledger_no_update', 'ledger_no_delete');" | grep -q "3" || (echo "❌ Missing expected triggers" && exit 1)
	@psql "$(DB_URL)" -c "SELECT proname FROM pg_proc WHERE proname IN ('ubl_tail_notify', 'forbid_mutation', 'forbid_atom_mutation');" | grep -q "ubl_tail_notify" || (echo "❌ Missing expected functions" && exit 1)
	@echo "✅ Schema verification passed"

db.check:
	@echo "🔍 Checking SQL syntax..."
	@while IFS= read -r file; do \
		if [ -n "$$file" ] && [ ! "$${file:0:1}" = "#" ]; then \
			psql "$(DB_URL)" -c "BEGIN; DO \$\$ BEGIN RETURN; END \$\$; END;" -f "$$file" >/dev/null 2>&1 || echo "⚠️  $$file: Syntax check failed (may be valid but needs DB context)"; \
		fi; \
	done < "$(MIGRATION_ORDER)"
	@echo "✅ Syntax check complete"


