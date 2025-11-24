ALTER TABLE link_analytics ADD COLUMN IF NOT EXISTS browser VARCHAR(50);
ALTER TABLE link_analytics ADD COLUMN IF NOT EXISTS os VARCHAR(50);
ALTER TABLE link_analytics ADD COLUMN IF NOT EXISTS device_type VARCHAR(20);
ALTER TABLE link_analytics ADD COLUMN IF NOT EXISTS city VARCHAR(100);

CREATE INDEX IF NOT EXISTS idx_analytics_browser ON link_analytics(browser);
CREATE INDEX IF NOT EXISTS idx_analytics_os ON link_analytics(os);
CREATE INDEX IF NOT EXISTS idx_analytics_device_type ON link_analytics(device_type);
CREATE INDEX IF NOT EXISTS idx_analytics_country_code ON link_analytics(country_code);

CREATE TABLE IF NOT EXISTS analytics_summary (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    link_id UUID NOT NULL REFERENCES links(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    total_clicks INTEGER NOT NULL DEFAULT 0,
    unique_visitors INTEGER NOT NULL DEFAULT 0,
    UNIQUE(link_id, date)
);

CREATE INDEX idx_analytics_summary_link_date ON analytics_summary(link_id, date DESC);
CREATE INDEX idx_analytics_summary_date ON analytics_summary(date DESC);

