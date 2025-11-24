CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS links (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    key VARCHAR(10) NOT NULL UNIQUE,
    original_url TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    click_count BIGINT NOT NULL DEFAULT 0,
    owner_id UUID,
    CONSTRAINT key_length CHECK (length(key) <= 10),
    CONSTRAINT url_not_empty CHECK (length(original_url) > 0)
);

CREATE INDEX idx_links_key ON links(key);
CREATE INDEX idx_links_expires_at ON links(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_links_created_at ON links(created_at DESC);
CREATE INDEX idx_links_owner_id ON links(owner_id) WHERE owner_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS link_analytics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    link_id UUID NOT NULL REFERENCES links(id) ON DELETE CASCADE,
    clicked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    referrer TEXT,
    user_agent TEXT,
    ip_hash VARCHAR(64),
    country_code VARCHAR(2)
);

CREATE INDEX idx_analytics_link_id ON link_analytics(link_id);
CREATE INDEX idx_analytics_clicked_at ON link_analytics(clicked_at DESC);

CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    key VARCHAR(64) NOT NULL UNIQUE,
    owner_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX idx_api_keys_key ON api_keys(key) WHERE is_active = true;
CREATE INDEX idx_api_keys_owner_id ON api_keys(owner_id);

