use serde::Serialize;

use uuid::Uuid;

use chrono::{DateTime, Utc};

#[derive(Debug, Serialize)]
pub struct ClickCountResponse {
    pub total_clicks: usize,
}

#[derive(Debug, Serialize)]
pub struct UniqueVisitorCountResponse {
    pub unique_visitors: usize,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AnalyticsCountResponse {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClickEvent {
    pub subdomain: String,
    pub clicked_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub referrer: Option<String>,
    pub device_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ClickEventRecord {
    pub id: Uuid,
    pub subdomain: String,
    pub clicked_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub referrer: Option<String>,
    pub device_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Link {
    pub subdomain: String,
    pub destination_url: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LinkRecord {
    pub id: Uuid,
    pub subdomain: String,
    pub destination_url: String,
    pub created_at: DateTime<Utc>,
}

pub fn find_link<'a>(links: &'a [Link], subdomain: &str) -> Option<&'a Link> {
    links.iter().find(|link| link.subdomain == subdomain)
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]

    fn find_existing_link() {
        let links = vec![
            Link {
                subdomain: "me".to_string(),
                destination_url: "www.pablomendoza.site".to_string(),
            },
            Link {
                subdomain: "info".to_string(),
                destination_url: "www.pablomendoza.site".to_string(),
            },
        ];

        let result = find_link(&links, "me");

        assert!(result.is_some());

        let link = result.unwrap();

        assert_eq!(link.subdomain, "me");
    }
}
