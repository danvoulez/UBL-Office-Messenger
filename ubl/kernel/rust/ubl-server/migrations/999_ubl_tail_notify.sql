-- Trigger para NOTIFY ubl_tail com payload MINÚSCULO: 'container_id:sequence'
-- Reset: evitar limite 8KB do PostgreSQL NOTIFY

CREATE OR REPLACE FUNCTION ubl_tail_notify() RETURNS trigger AS $$
BEGIN
  PERFORM pg_notify('ubl_tail', NEW.container_id || ':' || NEW.sequence::text);
  RETURN NEW;
END; $$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_tail_notify ON ledger_entry;
CREATE TRIGGER trg_tail_notify
AFTER INSERT ON ledger_entry
FOR EACH ROW EXECUTE FUNCTION ubl_tail_notify();


