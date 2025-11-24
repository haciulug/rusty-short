use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use nanoid::nanoid;
use url::Url;
use uuid::Uuid;

use crate::{
    cache::LinkCache,
    domain::{CreateLinkRequest, Link, LinkResponse, LinkStats, AnalyticsSummary, ReferrerStats, DeviceBreakdown, CountryStats, BrowserStats, TimeSeriesPoint},
    repository::LinkRepository,
    services::AnalyticsService,
};

const DEFAULT_KEY_LENGTH: usize = 7;
const CUSTOM_ALIAS_MAX_LENGTH: usize = 10;
const MAX_URL_LENGTH: usize = 2048;

#[derive(Clone)]
pub struct LinkService {
    repository: LinkRepository,
    cache: LinkCache,
    pub base_url: String,
}

impl LinkService {
    pub fn new(repository: LinkRepository, cache: LinkCache, base_url: String) -> Self {
        Self {
            repository,
            cache,
            base_url,
        }
    }

    pub async fn create_link(&self, request: CreateLinkRequest) -> Result<LinkResponse> {
        self.validate_url(&request.url)?;

        let key = if let Some(custom_alias) = request.custom_alias {
            self.validate_custom_alias(&custom_alias)?;
            if self.repository.exists(&custom_alias).await? {
                return Err(anyhow!("Custom alias already exists"));
            }
            custom_alias
        } else {
            self.generate_unique_key().await?
        };

        let expires_at = request.expires_in.map(|seconds| {
            Utc::now() + Duration::seconds(seconds)
        });

        let link = self.repository.create(
            key.clone(),
            request.url.clone(),
            expires_at,
            request.owner_id,
        ).await?;

        self.cache.set(key.clone(), link.clone()).await;

        Ok(self.link_to_response(link))
    }

    pub async fn get_link(&self, key: &str) -> Result<Option<Link>> {
        if let Some(link) = self.cache.get(key).await {
            if !link.is_expired() {
                return Ok(Some(link));
            }
            self.cache.invalidate(key).await;
        }

        if let Some(link) = self.repository.find_by_key(key).await? {
            if !link.is_expired() {
                self.cache.set(key.to_string(), link.clone()).await;
                return Ok(Some(link));
            }
        }

        Ok(None)
    }

    pub async fn increment_click(&self, key: &str) -> Result<()> {
        self.repository.increment_click_count(key).await?;
        self.cache.invalidate(key).await;
        Ok(())
    }

    pub async fn record_analytics(
        &self,
        link_id: Uuid,
        referrer: Option<String>,
        user_agent: Option<String>,
        ip_hash: Option<String>,
    ) -> Result<()> {
        let (browser, os, device_type) = if let Some(ref ua) = user_agent {
            AnalyticsService::parse_user_agent(ua)
        } else {
            (None, None, None)
        };
        
        self.repository.record_analytics(
            link_id,
            referrer,
            user_agent,
            ip_hash,
            browser,
            os,
            device_type,
        ).await?;
        Ok(())
    }
    
    pub async fn get_analytics_summary(&self, key: &str, days: i32) -> Result<Option<AnalyticsSummary>> {
        if self.repository.find_by_key(key).await?.is_none() {
            return Ok(None);
        }
        
        let total_clicks = self.repository.get_total_clicks(key).await?;
        let unique_visitors = self.repository.get_unique_visitors(key).await?;
        
        let referrers = self.repository.get_top_referrers(key, 10).await?;
        let top_referrers = referrers.into_iter().map(|(domain, count)| {
            let percentage = if total_clicks > 0 {
                (count as f64 / total_clicks as f64) * 100.0
            } else {
                0.0
            };
            ReferrerStats {
                domain,
                count,
                percentage,
            }
        }).collect();
        
        let devices = self.repository.get_device_breakdown(key).await?;
        let mut device_breakdown = DeviceBreakdown {
            desktop: 0,
            mobile: 0,
            tablet: 0,
            bot: 0,
            other: 0,
        };
        for (device_type, count) in devices {
            match device_type.as_str() {
                "desktop" => device_breakdown.desktop = count,
                "mobile" => device_breakdown.mobile = count,
                "tablet" => device_breakdown.tablet = count,
                "bot" => device_breakdown.bot = count,
                _ => device_breakdown.other = count,
            }
        }
        
        let countries = self.repository.get_country_stats(key, 10).await?;
        let geographic_distribution = countries.into_iter().map(|(country_code, count)| {
            let percentage = if total_clicks > 0 {
                (count as f64 / total_clicks as f64) * 100.0
            } else {
                0.0
            };
            CountryStats {
                country_code,
                count,
                percentage,
            }
        }).collect();
        
        let browsers = self.repository.get_browser_stats(key, 10).await?;
        let browser_stats = browsers.into_iter().map(|(browser, count)| {
            let percentage = if total_clicks > 0 {
                (count as f64 / total_clicks as f64) * 100.0
            } else {
                0.0
            };
            BrowserStats {
                browser,
                count,
                percentage,
            }
        }).collect();
        
        let time_data = self.repository.get_time_series(key, days).await?;
        let time_series = time_data.into_iter().map(|(date, clicks, unique)| {
            TimeSeriesPoint {
                date,
                clicks,
                unique_visitors: unique,
            }
        }).collect();
        
        Ok(Some(AnalyticsSummary {
            total_clicks,
            unique_visitors,
            top_referrers,
            device_breakdown,
            geographic_distribution,
            browser_stats,
            time_series,
        }))
    }

    pub async fn get_stats(&self, key: &str) -> Result<Option<LinkStats>> {
        let link = self.repository.find_by_key(key).await?;
        
        Ok(link.map(|l| LinkStats {
            key: l.key,
            original_url: l.original_url,
            click_count: l.click_count,
            created_at: l.created_at,
            expires_at: l.expires_at,
        }))
    }

    pub async fn delete_link(&self, key: &str) -> Result<bool> {
        let deleted = self.repository.delete(key).await?;
        if deleted {
            self.cache.invalidate(key).await;
        }
        Ok(deleted)
    }

    pub async fn list_links(&self, limit: i64, offset: i64) -> Result<Vec<LinkResponse>> {
        let links = self.repository.list(limit, offset).await?;
        Ok(links.into_iter().map(|l| self.link_to_response(l)).collect())
    }

    async fn generate_unique_key(&self) -> Result<String> {
        for _ in 0..10 {
            let key = nanoid!(DEFAULT_KEY_LENGTH, &nanoid::alphabet::SAFE);
            if !self.repository.exists(&key).await? {
                return Ok(key);
            }
        }
        Err(anyhow!("Failed to generate unique key after 10 attempts"))
    }

    fn validate_url(&self, url_str: &str) -> Result<()> {
        if url_str.len() > MAX_URL_LENGTH {
            return Err(anyhow!("URL exceeds maximum length of {} characters", MAX_URL_LENGTH));
        }

        let url = Url::parse(url_str)
            .map_err(|_| anyhow!("Invalid URL format"))?;

        if !matches!(url.scheme(), "http" | "https") {
            return Err(anyhow!("URL must use HTTP or HTTPS protocol"));
        }

        if url.host_str().is_none() {
            return Err(anyhow!("URL must have a valid host"));
        }

        Ok(())
    }

    fn validate_custom_alias(&self, alias: &str) -> Result<()> {
        if alias.is_empty() {
            return Err(anyhow!("Custom alias cannot be empty"));
        }

        if alias.len() > CUSTOM_ALIAS_MAX_LENGTH {
            return Err(anyhow!("Custom alias exceeds maximum length of {}", CUSTOM_ALIAS_MAX_LENGTH));
        }

        if !alias.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(anyhow!("Custom alias can only contain alphanumeric characters, hyphens, and underscores"));
        }

        Ok(())
    }

    fn link_to_response(&self, link: Link) -> LinkResponse {
        LinkResponse {
            key: link.key.clone(),
            short_url: format!("{}/{}", self.base_url, link.key),
            original_url: link.original_url,
            qr_code_url: format!("{}/qr/{}", self.base_url, link.key),
            created_at: link.created_at,
            expires_at: link.expires_at,
        }
    }
}

