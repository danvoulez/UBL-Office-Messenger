-- C.Messenger Boundary Support Tables
-- Version: 1.0
-- Date: 2024-12-28

-- ============================================================================
-- Message Content Storage
-- ============================================================================
-- The ledger stores only content_hash for privacy.
-- Actual content is stored here for retrieval.

CREATE TABLE IF NOT EXISTS message_content (
    message_id      TEXT PRIMARY KEY,
    content         TEXT NOT NULL,
    content_hash    TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_message_content_hash ON message_content(content_hash);

-- ============================================================================
-- Conversation Projection
-- ============================================================================
-- Derived from conversation.* events

CREATE TABLE IF NOT EXISTS projection_conversations (
    conversation_id TEXT PRIMARY KEY,
    name            TEXT,
    is_group        BOOLEAN NOT NULL DEFAULT false,
    participants    TEXT[] NOT NULL DEFAULT '{}',
    created_by      TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL,
    last_message_id TEXT,
    last_message_at TIMESTAMPTZ,
    last_event_hash TEXT NOT NULL,
    last_event_seq  BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_conv_participants ON projection_conversations USING GIN(participants);
CREATE INDEX IF NOT EXISTS idx_conv_last_message ON projection_conversations(last_message_at DESC);

-- ============================================================================
-- Entity Projection (for Messenger visibility)
-- ============================================================================
-- Derived from entity.registered events

CREATE TABLE IF NOT EXISTS projection_entities (
    entity_id       TEXT PRIMARY KEY,
    display_name    TEXT NOT NULL,
    entity_type     TEXT NOT NULL,
    avatar_hash     TEXT,
    status          TEXT DEFAULT 'online',
    registered_at   TIMESTAMPTZ NOT NULL,
    last_event_hash TEXT NOT NULL,
    last_event_seq  BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_entity_type ON projection_entities(entity_type);

-- ============================================================================
-- Comments
-- ============================================================================
COMMENT ON TABLE message_content IS 'Stores actual message content. Ledger stores only hash for privacy.';
COMMENT ON TABLE projection_conversations IS 'Derived state from conversation.* events in C.Messenger';
COMMENT ON TABLE projection_entities IS 'Derived state from entity.registered events visible to Messenger';

