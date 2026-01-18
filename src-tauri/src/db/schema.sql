-- Account configurations
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    name TEXT NOT NULL,
    credentials_encrypted BLOB NOT NULL,
    created_at INTEGER NOT NULL,
    last_synced INTEGER,
    CHECK (provider IN ('openai', 'anthropic', 'google', 'github'))
);

-- Historical quota snapshots (account-level aggregates)
CREATE TABLE IF NOT EXISTS quota_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    tokens_input INTEGER,
    tokens_output INTEGER,
    cost_usd REAL,
    quota_limit INTEGER,
    quota_remaining INTEGER,
    metadata TEXT,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_quota_snapshots_account_timestamp
ON quota_snapshots(account_id, timestamp DESC);

-- Per-model usage tracking
CREATE TABLE IF NOT EXISTS model_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,
    model_name TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    tokens_input INTEGER DEFAULT 0,
    tokens_output INTEGER DEFAULT 0,
    cost_usd REAL DEFAULT 0.0,
    request_count INTEGER DEFAULT 0,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_model_usage_account_timestamp
ON model_usage(account_id, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_model_usage_model_name
ON model_usage(model_name);

-- Notification state tracking
CREATE TABLE IF NOT EXISTS notification_state (
    account_id TEXT PRIMARY KEY,
    last_75_percent_notified INTEGER,
    last_90_percent_notified INTEGER,
    last_95_percent_notified INTEGER,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

-- Application settings
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Insert default settings
INSERT OR IGNORE INTO settings (key, value) VALUES
    ('refresh_interval_seconds', '300'),
    ('notifications_enabled', 'true'),
    ('threshold_75_enabled', 'true'),
    ('threshold_90_enabled', 'true'),
    ('threshold_95_enabled', 'true'),
    ('quiet_hours_start', ''),
    ('quiet_hours_end', ''),
    ('data_retention_days', '90');
