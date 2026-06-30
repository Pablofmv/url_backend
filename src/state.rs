use sqlx::PgPool;

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use crate::models::{
    AnalyticsCountResponse, ClickEvent, ClickEventRecord, Link, LinkRecord, find_link,
};

#[derive(Clone)]
pub struct AppState {
    pub links: Vec<Link>,
    click_events: Arc<Mutex<Vec<ClickEvent>>>,
    pub pool: PgPool,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            links: vec![
                Link {
                    subdomain: "me".to_string(),
                    destination_url: "https://www.pablomendoza.site".to_string(),
                },
                Link {
                    subdomain: "info".to_string(),
                    destination_url: "https://www.pablomendoza.site".to_string(),
                },
            ],
            click_events: Arc::new(Mutex::new(Vec::new())),
            pool,
        }
    }

    pub async fn get_click_count_db(&self) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
                SELECT COUNT(*)
                FROM click_events
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    pub async fn get_unique_visitor_count_db(&self) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT ip_address)
            FROM click_events
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    pub async fn list_click_events_db(&self) -> Result<Vec<ClickEventRecord>, sqlx::Error> {
        sqlx::query_as::<_, ClickEventRecord>(
            r#"
                SELECT id, subdomain, clicked_at, ip_address, referrer, device_type
                FROM click_events
                ORDER BY clicked_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_device_analytics_db(
        &self,
    ) -> Result<Vec<AnalyticsCountResponse>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AnalyticsCountResponse>(
            r#"
                SELECT COALESCE(device_type, 'Unknwon') AS name, COUNT(*)::BIGINT AS count
                FROM click_events
                GROUP BY device_type
                ORDER BY count DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_referrer_analytics_db(
        &self,
    ) -> Result<Vec<AnalyticsCountResponse>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AnalyticsCountResponse>(
            r#"
                SELECT COALESCE(referrer, 'Direct') AS name, COUNT(*)::BIGINT AS count
                FROM click_events
                GROUP BY referrer
                ORDER BY count DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_all_links_from_db(&self) -> Result<Vec<LinkRecord>, sqlx::Error> {
        sqlx::query_as::<_, LinkRecord>(
            r#"
            SELECT
                id,
                subdomain,
                destination_url,
                created_at
            FROM links
            ORDER BY subdomain
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn save_click_event_db(&self, click_event: &ClickEvent) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO click_events (
                subdomain,
                clicked_at,
                ip_address,
                referrer,
                device_type
            )
            VALUES(
            $1,
            $2,
            $3,
            $4,
            $5
            )
            "#,
        )
        .bind(&click_event.subdomain)
        .bind(&click_event.clicked_at)
        .bind(&click_event.ip_address)
        .bind(&click_event.referrer)
        .bind(&click_event.device_type)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub fn find_link(&self, subdomain: &str) -> Option<&Link> {
        find_link(&self.links, subdomain)
    }

    pub fn save_click_event(&self, click_event: ClickEvent) -> Result<(), String> {
        let mut events = self
            .click_events
            .lock()
            .map_err(|_| "failed to lock click events".to_string())?;

        events.push(click_event);

        Ok(())
    }

    pub fn get_click_count(&self) -> Result<usize, String> {
        let events = self
            .click_events
            .lock()
            .map_err(|_| "failed to lock click events".to_string())?;

        Ok(events.len())
    }

    pub fn get_unique_visitor_count(&self) -> Result<usize, String> {
        let events = self
            .click_events
            .lock()
            .map_err(|_| "failed to lock click events".to_string())?;

        let unique_ips: HashSet<&str> = events
            .iter()
            .filter_map(|event| event.ip_address.as_deref())
            .collect();

        Ok(unique_ips.len())
    }

    pub fn get_device_analytics(&self) -> Result<Vec<AnalyticsCountResponse>, String> {
        let events = self
            .click_events
            .lock()
            .map_err(|_| "failed to lock click events".to_string())?;

        let mut counts: HashMap<String, usize> = HashMap::new();

        for event in events.iter() {
            let device = event.device_type.as_deref().unwrap_or("Unknown");

            *counts.entry(device.to_string()).or_insert(0) += 1
        }

        let analytics = counts
            .into_iter()
            .map(|(name, count)| AnalyticsCountResponse {
                name,
                count: count as i64,
            })
            .collect();

        Ok(analytics)
    }

    pub fn get_referrer_analytics(&self) -> Result<Vec<AnalyticsCountResponse>, String> {
        let events = self
            .click_events
            .lock()
            .map_err(|_| "failed to lock click events".to_string())?;

        let mut counts: HashMap<String, usize> = HashMap::new();

        for event in events.iter() {
            let referrer = event.referrer.as_deref().unwrap_or("Direct");

            *counts.entry(referrer.to_string()).or_insert(0) += 1;
        }

        let analytics = counts
            .into_iter()
            .map(|(name, count)| AnalyticsCountResponse {
                name,
                count: count as i64,
            })
            .collect();

        Ok(analytics)
    }

    pub fn list_click_events(&self) -> Result<Vec<ClickEvent>, String> {
        let events = self
            .click_events
            .lock()
            .map_err(|_| "failed to lock click events".to_string())?;

        Ok(events.clone())
    }

    pub async fn find_link_by_subdomain_db(
        &self,
        subdomain: &str,
    ) -> Result<Option<LinkRecord>, sqlx::Error> {
        sqlx::query_as::<_, LinkRecord>(
            r#"
                SELECT
                    id,
                    subdomain,
                    destination_url
                FROM links
                WHERE subdomain =$1
            "#,
        )
        .bind(subdomain)
        .fetch_optional(&self.pool)
        .await
    }
}

/*

#[cfg(test)]

mod tests {

    use super::*;
    use chrono::Utc;

    fn click_event(ip: Option<&str>) -> ClickEvent {
        ClickEvent {
            subdomain: "me".to_string(),
            clicked_at: Utc::now(),
            ip_address: ip.map(String::from),
            referrer: None,
            device_type: None,
        }
    }

    #[test]
    fn counts_unique_visitors() {
        let state = AppState::new();

        state
            .save_click_event(click_event(Some("34.12.55.9")))
            .unwrap();

        state
            .save_click_event(click_event(Some("88.10.20.1")))
            .unwrap();

        let count = state.get_unique_visitor_count().unwrap();

        assert_eq!(count, 2);
    }

    #[test]
    fn save_click_event() {
        let state = AppState::new();

        let click = ClickEvent {
            subdomain: "me".to_string(),
            clicked_at: Utc::now(),
            ip_address: None,
            referrer: None,
            device_type: None,
        };

        let result = state.save_click_event(click);

        assert!(result.is_ok());

        let count = state.get_click_count().unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn app_state_starts_with_default_links() {
        let state = AppState::new();
        assert_eq!(state.links.len(), 2);
    }

    #[test]
    fn app_state_returns_none() {
        let state = AppState::new();

        let result = state.find_link("blog");

        assert!(result.is_none());
    }
}

*/
