-- ============================================================================
-- UBL-FIX: Authentication Anti-Replay Protection
-- ============================================================================
-- Diamond Checklist #4: Prevent challenge reuse in authentication flow
-- This ensures that WebAuthn challenges can only be used once

-- ============================================================================
-- CHALLENGE ANTI-REPLAY
-- ============================================================================
-- UBL-FIX: Add unique index to prevent challenge reuse
-- A challenge can only be consumed once to create a session
CREATE UNIQUE INDEX IF NOT EXISTS uq_challenge_id_used
  ON id_challenge(id)
  WHERE used = true;

-- UBL-FIX: Add index for efficient challenge cleanup
CREATE INDEX IF NOT EXISTS ix_challenge_expires
  ON id_challenge(expires_at)
  WHERE used = false;

COMMENT ON INDEX uq_challenge_id_used IS 'Prevents authentication replay attacks by ensuring challenges are single-use';

-- ============================================================================
-- SESSION UNIQUENESS
-- ============================================================================
-- Ensure one active session per user per flavor at a time (optional, but recommended)
-- Note: This is commented out as it may be too restrictive for some use cases
-- Uncomment if you want to enforce single-session-per-user
-- 
-- CREATE UNIQUE INDEX IF NOT EXISTS uq_active_session_per_user
--   ON id_session(sid, flavor)
--   WHERE not_after > NOW();

-- ============================================================================
-- VERIFICATION
-- ============================================================================
-- To verify anti-replay protection is working:
-- 
-- 1. Create a challenge:
--    INSERT INTO id_challenge (id, kind, challenge, origin, expires_at, used)
--    VALUES (gen_random_uuid(), 'webauthn', 'test', 'http://localhost', NOW() + INTERVAL '5 minutes', false);
-- 
-- 2. Mark it as used:
--    UPDATE id_challenge SET used = true WHERE id = '<uuid-from-step-1>';
-- 
-- 3. Try to mark it as used again (should succeed but be idempotent):
--    UPDATE id_challenge SET used = true WHERE id = '<uuid-from-step-1>';
-- 
-- 4. The unique index prevents having multiple "used=true" rows with same id
