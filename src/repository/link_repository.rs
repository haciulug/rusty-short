use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};
use uuid::Uuid;
use crate::domain::{Link, LinkAnalytics};

#[derive(Clone)]
pub struct LinkRepository {
    pool: PgPool,
}

impl LinkRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, key: String, original_url: String, expires_at: Option<DateTime<Utc>>, owner_id: Option<Uuid>) -> Result<Link> {
        let link = sqlx::query_as::<_, Link>(
            r#"
            INSERT INTO links (key, original_url, expires_at, owner_id)
            VALUES ($1, $2, $3, $4)
            RETURNING id, key, original_url, created_at, expires_at, click_count, owner_id
            "#
        )
        .bind(&key)
        .bind(&original_url)
        .bind(expires_at)
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(link)
    }

    pub async fn find_by_key(&self, key: &str) -> Result<Option<Link>> {
        let link = sqlx::query_as::<_, Link>(
            r#"
            SELECT id, key, original_url, created_at, expires_at, click_count, owner_id
            FROM links
            WHERE key = $1
            "#
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(link)
    }

    pub async fn exists(&self, key: &str) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM links WHERE key = $1)"
        )
        .bind(key)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    pub async fn increment_click_count(&self, key: &str) -> Result<()> {
        sqlx::query(
            "UPDATE links SET click_count = click_count + 1 WHERE key = $1"
        )
        .bind(key)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM links WHERE key = $1"
        )
        .bind(key)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Link>> {
        let links = sqlx::query_as::<_, Link>(
            r#"
            SELECT id, key, original_url, created_at, expires_at, click_count, owner_id
            FROM links
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(links)
    }

    pub async fn record_analytics(
        &self,
        link_id: Uuid,
        referrer: Option<String>,
        user_agent: Option<String>,
        ip_hash: Option<String>,
        browser: Option<String>,
        os: Option<String>,
        device_type: Option<String>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO link_analytics (link_id, referrer, user_agent, ip_hash, browser, os, device_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(link_id)
        .bind(referrer)
        .bind(user_agent)
        .bind(ip_hash)
        .bind(browser)
        .bind(os)
        .bind(device_type)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_analytics(&self, key: &str, limit: i64) -> Result<Vec<LinkAnalytics>> {
        let analytics = sqlx::query_as::<_, LinkAnalytics>(
            r#"
            SELECT la.id, la.link_id, la.clicked_at, la.referrer, la.user_agent, 
                   la.ip_hash, la.country_code, la.browser, la.os, la.device_type, la.city
            FROM link_analytics la
            JOIN links l ON la.link_id = l.id
            WHERE l.key = $1
            ORDER BY la.clicked_at DESC
            LIMIT $2
            "#
        )
        .bind(key)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(analytics)
    }
    
    pub async fn get_total_clicks(&self, key: &str) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM link_analytics la
            JOIN links l ON la.link_id = l.id
            WHERE l.key = $1
            "#
        )
        .bind(key)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }
    
    pub async fn get_unique_visitors(&self, key: &str) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT ip_hash)
            FROM link_analytics la
            JOIN links l ON la.link_id = l.id
            WHERE l.key = $1 AND la.ip_hash IS NOT NULL
            "#
        )
        .bind(key)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }
    
    pub async fn get_top_referrers(&self, key: &str, limit: i64) -> Result<Vec<(String, i64)>> {
        let referrers = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT la.referrer, COUNT(*) as count
            FROM link_analytics la
            JOIN links l ON la.link_id = l.id
            WHERE l.key = $1 AND la.referrer IS NOT NULL AND la.referrer != ''
            GROUP BY la.referrer
            ORDER BY count DESC
            LIMIT $2
            "#
        )
        .bind(key)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(referrers)
    }
    
    pub async fn get_device_breakdown(&self, key: &str) -> Result<Vec<(String, i64)>> {
        let devices = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT COALESCE(la.device_type, 'other') as device, COUNT(*) as count
            FROM link_analytics la
            JOIN links l ON la.link_id = l.id
            WHERE l.key = $1
            GROUP BY device
            ORDER BY count DESC
            "#
        )
        .bind(key)
        .fetch_all(&self.pool)
        .await?;

        Ok(devices)
    }
    
    pub async fn get_browser_stats(&self, key: &str, limit: i64) -> Result<Vec<(String, i64)>> {
        let browsers = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT la.browser, COUNT(*) as count
            FROM link_analytics la
            JOIN links l ON la.link_id = l.id
            WHERE l.key = $1 AND la.browser IS NOT NULL
            GROUP BY la.browser
            ORDER BY count DESC
            LIMIT $2
            "#
        )
        .bind(key)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(browsers)
    }
    
    pub async fn get_country_stats(&self, key: &str, limit: i64) -> Result<Vec<(String, i64)>> {
        let countries = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT la.country_code, COUNT(*) as count
            FROM link_analytics la
            JOIN links l ON la.link_id = l.id
            WHERE l.key = $1 AND la.country_code IS NOT NULL
            GROUP BY la.country_code
            ORDER BY count DESC
            LIMIT $2
            "#
        )
        .bind(key)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(countries)
    }
    
    pub async fn get_time_series(&self, key: &str, days: i32) -> Result<Vec<(String, i64, i64)>> {
        let time_series = sqlx::query_as::<_, (String, i64, i64)>(
            r#"
            SELECT 
                DATE(la.clicked_at)::TEXT as date,
                COUNT(*)::BIGINT as clicks,
                COUNT(DISTINCT la.ip_hash)::BIGINT as unique_visitors
            FROM link_analytics la
            JOIN links l ON la.link_id = l.id
            WHERE l.key = $1 
                AND la.clicked_at >= NOW() - INTERVAL '1 day' * $2
            GROUP BY DATE(la.clicked_at)
            ORDER BY date DESC
            "#
        )
        .bind(key)
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        Ok(time_series)
    }

    pub async fn cleanup_expired(&self) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM links WHERE expires_at IS NOT NULL AND expires_at < NOW()"
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

