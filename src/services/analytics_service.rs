use sha2::{Digest, Sha256};
use woothee::parser::Parser;

#[derive(Debug, Clone)]
pub struct AnalyticsData {
    pub referrer: Option<String>,
    pub user_agent: Option<String>,
    pub ip_hash: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub device_type: Option<String>,
}

pub struct AnalyticsService;

impl AnalyticsService {
    pub fn parse_user_agent(user_agent: &str) -> (Option<String>, Option<String>, Option<String>) {
        let parser = Parser::new();
        
        if let Some(result) = parser.parse(user_agent) {
            let browser = if result.name != "UNKNOWN" {
                Some(result.name.to_string())
            } else {
                None
            };
            
            let os = if result.os != "UNKNOWN" {
                Some(result.os.to_string())
            } else {
                None
            };
            
            let device_type = match result.category {
                "pc" => Some("desktop".to_string()),
                "smartphone" => Some("mobile".to_string()),
                "mobilephone" => Some("mobile".to_string()),
                "appliance" => Some("tablet".to_string()),
                "crawler" => Some("bot".to_string()),
                _ => Some("other".to_string()),
            };
            
            (browser, os, device_type)
        } else {
            (None, None, None)
        }
    }
    
    pub fn hash_ip(ip: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(ip.as_bytes());
        hex::encode(hasher.finalize())
    }
    
    pub fn extract_referrer_domain(referrer: &str) -> Option<String> {
        if let Ok(url) = url::Url::parse(referrer) {
            url.host_str().map(|s| s.to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_user_agent_chrome() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        let (browser, os, device) = AnalyticsService::parse_user_agent(ua);
        assert!(browser.is_some());
        assert!(os.is_some());
        assert_eq!(device, Some("desktop".to_string()));
    }
    
    #[test]
    fn test_hash_ip() {
        let hash1 = AnalyticsService::hash_ip("192.168.1.1");
        let hash2 = AnalyticsService::hash_ip("192.168.1.1");
        let hash3 = AnalyticsService::hash_ip("192.168.1.2");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64);
    }
    
    #[test]
    fn test_extract_referrer_domain() {
        assert_eq!(
            AnalyticsService::extract_referrer_domain("https://www.google.com/search?q=test"),
            Some("www.google.com".to_string())
        );
        assert_eq!(
            AnalyticsService::extract_referrer_domain("https://github.com/"),
            Some("github.com".to_string())
        );
    }
}

