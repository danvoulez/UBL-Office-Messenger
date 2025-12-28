CREATE ROLE ubl_kernel LOGIN;
CREATE ROLE ubl_readonly;

REVOKE ALL ON TABLE ledger_entry FROM PUBLIC;
REVOKE ALL ON TABLE ledger_atom  FROM PUBLIC;

GRANT INSERT, SELECT ON ledger_entry TO ubl_kernel;
GRANT INSERT, SELECT ON ledger_atom  TO ubl_kernel;

GRANT SELECT ON ledger_entry, ledger_atom TO ubl_readonly;

CREATE OR REPLACE FUNCTION deny_ud_on_ledger() RETURNS trigger AS $$
BEGIN
  RAISE EXCEPTION 'Ledger is append-only.';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS ledger_entry_ud ON ledger_entry;
CREATE TRIGGER ledger_entry_ud BEFORE UPDATE OR DELETE ON ledger_entry
  FOR EACH STATEMENT EXECUTE FUNCTION deny_ud_on_ledger();

DROP TRIGGER IF EXISTS ledger_atom_ud ON ledger_atom;
CREATE TRIGGER ledger_atom_ud BEFORE UPDATE OR DELETE ON ledger_atom
  FOR EACH STATEMENT EXECUTE FUNCTION deny_ud_on_ledger();
