use anyhow::{Context, Result};
use sqlx::{SqlitePool, Row};
use super::models::*;

pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url)
            .await
            .context("Failed to connect to database")?;

        // Run migrations
        sqlx::query(include_str!("schema.sql"))
            .execute(&pool)
            .await
            .context("Failed to run database migrations")?;

        Ok(Self { pool })
    }

    // Account operations
    pub async fn insert_account(&self, account: &Account) -> Result<()> {
        sqlx::query(
            "INSERT INTO accounts (id, provider, name, credentials_encrypted, created_at, last_synced)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&account.id)
        .bind(&account.provider)
        .bind(&account.name)
        .bind(&account.credentials_encrypted)
        .bind(account.created_at)
        .bind(account.last_synced)
        .execute(&self.pool)
        .await
        .context("Failed to insert account")?;

        Ok(())
    }

    pub async fn get_all_accounts(&self) -> Result<Vec<Account>> {
        let accounts = sqlx::query_as::<_, Account>(
            "SELECT id, provider, name, credentials_encrypted, created_at, last_synced FROM accounts"
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch accounts")?;

        Ok(accounts)
    }

    pub async fn get_account(&self, id: &str) -> Result<Option<Account>> {
        let account = sqlx::query_as::<_, Account>(
            "SELECT id, provider, name, credentials_encrypted, created_at, last_synced
             FROM accounts WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch account")?;

        Ok(account)
    }

    pub async fn delete_account(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM accounts WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete account")?;

        Ok(())
    }

    pub async fn update_account_sync_time(&self, id: &str, timestamp: i64) -> Result<()> {
        sqlx::query("UPDATE accounts SET last_synced = ? WHERE id = ?")
            .bind(timestamp)
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to update account sync time")?;

        Ok(())
    }

    // Quota snapshot operations
    pub async fn insert_quota_snapshot(&self, snapshot: &QuotaSnapshot) -> Result<()> {
        sqlx::query(
            "INSERT INTO quota_snapshots
             (account_id, timestamp, tokens_input, tokens_output, cost_usd, quota_limit, quota_remaining, metadata)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&snapshot.account_id)
        .bind(snapshot.timestamp)
        .bind(snapshot.tokens_input)
        .bind(snapshot.tokens_output)
        .bind(snapshot.cost_usd)
        .bind(snapshot.quota_limit)
        .bind(snapshot.quota_remaining)
        .bind(&snapshot.metadata)
        .execute(&self.pool)
        .await
        .context("Failed to insert quota snapshot")?;

        Ok(())
    }

    pub async fn get_latest_snapshot(&self, account_id: &str) -> Result<Option<QuotaSnapshot>> {
        let snapshot = sqlx::query_as::<_, QuotaSnapshot>(
            "SELECT id, account_id, timestamp, tokens_input, tokens_output, cost_usd,
                    quota_limit, quota_remaining, metadata
             FROM quota_snapshots
             WHERE account_id = ?
             ORDER BY timestamp DESC
             LIMIT 1"
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch latest snapshot")?;

        Ok(snapshot)
    }

    pub async fn get_snapshots_since(&self, account_id: &str, since: i64) -> Result<Vec<QuotaSnapshot>> {
        let snapshots = sqlx::query_as::<_, QuotaSnapshot>(
            "SELECT id, account_id, timestamp, tokens_input, tokens_output, cost_usd,
                    quota_limit, quota_remaining, metadata
             FROM quota_snapshots
             WHERE account_id = ? AND timestamp >= ?
             ORDER BY timestamp ASC"
        )
        .bind(account_id)
        .bind(since)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch snapshots")?;

        Ok(snapshots)
    }

    // Model usage operations
    pub async fn insert_model_usage(&self, usage: &ModelUsage) -> Result<()> {
        sqlx::query(
            "INSERT INTO model_usage
             (account_id, model_name, timestamp, tokens_input, tokens_output, cost_usd, request_count)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&usage.account_id)
        .bind(&usage.model_name)
        .bind(usage.timestamp)
        .bind(usage.tokens_input)
        .bind(usage.tokens_output)
        .bind(usage.cost_usd)
        .bind(usage.request_count)
        .execute(&self.pool)
        .await
        .context("Failed to insert model usage")?;

        Ok(())
    }

    pub async fn get_model_usage_since(&self, account_id: &str, since: i64) -> Result<Vec<ModelUsage>> {
        let usage = sqlx::query_as::<_, ModelUsage>(
            "SELECT id, account_id, model_name, timestamp, tokens_input, tokens_output, cost_usd, request_count
             FROM model_usage
             WHERE account_id = ? AND timestamp >= ?
             ORDER BY timestamp ASC"
        )
        .bind(account_id)
        .bind(since)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch model usage")?;

        Ok(usage)
    }

    // Notification state operations
    pub async fn get_notification_state(&self, account_id: &str) -> Result<Option<NotificationState>> {
        let state = sqlx::query_as::<_, NotificationState>(
            "SELECT account_id, last_75_percent_notified, last_90_percent_notified, last_95_percent_notified
             FROM notification_state
             WHERE account_id = ?"
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch notification state")?;

        Ok(state)
    }

    pub async fn update_notification_state(&self, state: &NotificationState) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO notification_state
             (account_id, last_75_percent_notified, last_90_percent_notified, last_95_percent_notified)
             VALUES (?, ?, ?, ?)"
        )
        .bind(&state.account_id)
        .bind(state.last_75_percent_notified)
        .bind(state.last_90_percent_notified)
        .bind(state.last_95_percent_notified)
        .execute(&self.pool)
        .await
        .context("Failed to update notification state")?;

        Ok(())
    }

    // Settings operations
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let row = sqlx::query("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to fetch setting")?;

        Ok(row.map(|r| r.get("value")))
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await
            .context("Failed to set setting")?;

        Ok(())
    }

    pub async fn cleanup_old_data(&self, days: i64) -> Result<()> {
        let cutoff = chrono::Utc::now().timestamp() - (days * 86400);

        sqlx::query("DELETE FROM quota_snapshots WHERE timestamp < ?")
            .bind(cutoff)
            .execute(&self.pool)
            .await
            .context("Failed to cleanup quota snapshots")?;

        sqlx::query("DELETE FROM model_usage WHERE timestamp < ?")
            .bind(cutoff)
            .execute(&self.pool)
            .await
            .context("Failed to cleanup model usage")?;

        Ok(())
    }
}

// Implement sqlx::FromRow for custom types
impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for Account {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> sqlx::Result<Self> {
        Ok(Account {
            id: row.try_get("id")?,
            provider: row.try_get("provider")?,
            name: row.try_get("name")?,
            credentials_encrypted: row.try_get("credentials_encrypted")?,
            created_at: row.try_get("created_at")?,
            last_synced: row.try_get("last_synced")?,
        })
    }
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for QuotaSnapshot {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> sqlx::Result<Self> {
        Ok(QuotaSnapshot {
            id: row.try_get("id")?,
            account_id: row.try_get("account_id")?,
            timestamp: row.try_get("timestamp")?,
            tokens_input: row.try_get("tokens_input")?,
            tokens_output: row.try_get("tokens_output")?,
            cost_usd: row.try_get("cost_usd")?,
            quota_limit: row.try_get("quota_limit")?,
            quota_remaining: row.try_get("quota_remaining")?,
            metadata: row.try_get("metadata")?,
        })
    }
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for ModelUsage {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> sqlx::Result<Self> {
        Ok(ModelUsage {
            id: row.try_get("id")?,
            account_id: row.try_get("account_id")?,
            model_name: row.try_get("model_name")?,
            timestamp: row.try_get("timestamp")?,
            tokens_input: row.try_get("tokens_input")?,
            tokens_output: row.try_get("tokens_output")?,
            cost_usd: row.try_get("cost_usd")?,
            request_count: row.try_get("request_count")?,
        })
    }
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for NotificationState {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> sqlx::Result<Self> {
        Ok(NotificationState {
            account_id: row.try_get("account_id")?,
            last_75_percent_notified: row.try_get("last_75_percent_notified")?,
            last_90_percent_notified: row.try_get("last_90_percent_notified")?,
            last_95_percent_notified: row.try_get("last_95_percent_notified")?,
        })
    }
}
