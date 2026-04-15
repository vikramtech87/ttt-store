CREATE TABLE IF NOT EXISTS store.user_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES store.users (id) ON DELETE CASCADE,
    provider TEXT NOT NULL, -- e.g., google, email
    provider_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(provider, provider_id)
);

CREATE INDEX IF NOT EXISTS idx_identities_user_id ON store.user_identities (user_id);