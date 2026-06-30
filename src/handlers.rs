use axum::{
    extract::{Path, State},
    http::{
        HeaderMap, StatusCode,
        header::{HOST, REFERER, USER_AGENT},
    },
    response::{IntoResponse, Json, Redirect},
};

use crate::{
    models::{ClickCountResponse, ClickEvent, Link, UniqueVisitorCountResponse},
    state::AppState,
};

use chrono::Utc;

use tracing::info;

const X_FORWARDED_FOR: &str = "x-forwarded-for";
const X_REAL_IP: &str = "x-real-ip";

pub async fn get_device_analytics(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let analytics = state
        .get_device_analytics_db()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(analytics))
}

pub async fn get_referrer_analytics(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let analytics = state
        .get_referrer_analytics_db()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(analytics))
}

pub async fn get_unique_visitor_count(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let count = state
        .get_unique_visitor_count_db()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(UniqueVisitorCountResponse {
        unique_visitors: count as usize,
    }))
}

pub fn log_click_event(click_event: &ClickEvent) {
    info!(
        subdomain = %click_event.subdomain,
        clicked_at = %click_event.clicked_at,
        ip_address = ?click_event.ip_address,
        referrer = ?click_event.referrer,
        device_type = ?click_event.device_type,
        "click event captured"
    );
}

pub fn build_click_event(subdomain: &str, headers: &HeaderMap) -> ClickEvent {
    let device_type = extract_user_agent(headers)
        .as_deref()
        .and_then(parse_device_type);

    ClickEvent {
        subdomain: subdomain.to_string(),
        clicked_at: current_timestamp(),
        ip_address: extract_ip(headers),
        referrer: extract_referrer(headers),
        device_type,
    }
}

pub async fn get_click_count(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let count = state
        .get_click_count_db()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ClickCountResponse {
        total_clicks: count as usize,
    }))
}

pub async fn list_click_events(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let events = state
        .list_click_events_db()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(events))
}

pub fn current_timestamp() -> chrono::DateTime<Utc> {
    Utc::now()
}

pub fn extract_ip(headers: &HeaderMap) -> Option<String> {
    if let Some(ip) = headers.get(X_FORWARDED_FOR).and_then(|v| v.to_str().ok()) {
        return ip.split(",").next().map(|s| s.trim().to_string());
    }

    headers
        .get(X_REAL_IP)
        .and_then(|v| v.to_str().ok())
        .map(String::from)
}

pub fn extract_referrer(headers: &HeaderMap) -> Option<String> {
    headers
        .get(REFERER)
        .and_then(|value| value.to_str().ok())
        .map(String::from)
}

pub fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(String::from)
}

pub fn parse_device_type(user_agent: &str) -> Option<String> {
    if user_agent.contains("iPad") || user_agent.contains("Tablet") {
        Some("Tablet".to_string())
    } else if user_agent.contains("Mobile")
        || user_agent.contains("iPhone")
        || user_agent.contains("Android")
    {
        Some("Mobile".to_string())
    } else {
        Some("Desktop".to_string())
    }
}

pub async fn redirect_by_host(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("redirect_by_host called");

    let Some(host) = extract_host(&headers) else {
        tracing::error!("missing host header");
        return Err(StatusCode::BAD_REQUEST);
    };

    tracing::info!(host = %host, "host received");

    let Some(subdomain) = extract_subdomain(host) else {
        tracing::error!(host = %host, "could not extract subdomain");
        return Err(StatusCode::BAD_REQUEST);
    };

    tracing::info!(subdomain = %subdomain, "subdomain extracted");

    let link_record = match state.find_link_by_subdomain_db(subdomain).await {
        Ok(Some(link_record)) => {
            tracing::info!(
                subdomain = %subdomain,
                destination_url = %link_record.destination_url,
                "link found in database"
            );

            link_record
        }
        Ok(None) => {
            tracing::error!(subdomain = %subdomain, "subdomain not found in database");
            return Err(StatusCode::NOT_FOUND);
        }
        Err(error) => {
            tracing::error!(?error, "database lookup failed");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let destination_url = link_record.destination_url.clone();

    let click_event = build_click_event(subdomain, &headers);

    log_click_event(&click_event);

    match state.save_click_event(click_event.clone()) {
        Ok(_) => tracing::info!("click event saved in memory"),
        Err(error) => {
            tracing::error!(?error, "failed to save click event in memory");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    match state.save_click_event_db(&click_event).await {
        Ok(_) => tracing::info!("click event saved in database"),
        Err(error) => {
            tracing::error!(?error, "failed to save click event in database");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    tracing::info!(destination_url = %destination_url, "redirecting user");

    Ok(Redirect::temporary(&destination_url))
}

pub fn extract_subdomain_from_headers(headers: &HeaderMap) -> Option<&str> {
    let host = extract_host(headers)?;

    extract_subdomain(host)
}

pub fn extract_host(headers: &HeaderMap) -> Option<&str> {
    headers.get(HOST).and_then(|value| value.to_str().ok())
}

pub fn extract_subdomain(host: &str) -> Option<&str> {
    let host_without_port = host.split(":").next()?;

    let parts: Vec<&str> = host_without_port.split(".").collect();

    if parts.len() < 3 {
        return None;
    }

    parts.first().copied()
}

pub async fn read_subdomain(headers: HeaderMap) -> impl IntoResponse {
    let Some(subdomain) = extract_subdomain_from_headers(&headers) else {
        return (StatusCode::BAD_REQUEST, "Missing subdomain".to_string());
    };

    (StatusCode::OK, format!("Subdomain {}", subdomain))
}

pub async fn health_check() -> impl IntoResponse {
    "OK"
}

pub async fn count_links(State(state): State<AppState>) -> impl IntoResponse {
    let count = state.links.len();

    format!("Total links: {}", count)
}

pub async fn list_links(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let records = state
        .get_all_links_from_db()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let links: Vec<Link> = records
        .into_iter()
        .map(|record| Link {
            subdomain: record.subdomain,
            destination_url: record.destination_url,
        })
        .collect();

    Ok(Json(links))
}

pub async fn get_link_by_subdomain(
    State(state): State<AppState>,
    Path(subdomain): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let record = state
        .find_link_by_subdomain_db(&subdomain)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match record {
        Some(record) => {
            let link = Link {
                subdomain: record.subdomain,
                destination_url: record.destination_url,
            };

            Ok(Json(link))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn read_host_header(headers: HeaderMap) -> impl IntoResponse {
    match extract_host(&headers) {
        Some(host_value) => (StatusCode::OK, format!("Host header {}", host_value)),
        None => (StatusCode::BAD_REQUEST, "Missing host header".to_string()),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use axum::http::{HeaderMap, HeaderValue, header::HOST};

    #[test]
    fn extracts_subdomain_from_headers() {
        let mut headers = HeaderMap::new();

        headers.insert(HOST, HeaderValue::from_static("me.pablomendoza.site"));

        let result = extract_subdomain_from_headers(&headers);

        assert_eq!(result, Some("me"));
    }

    #[test]
    fn extract_subdomain_with_port() {
        let result = extract_subdomain("me.pablomendoza.site:443");

        assert_eq!(result, Some("me"));
    }

    #[test]
    fn extract_subdomain_successfully() {
        let result = extract_subdomain("me.pablomendoza.site");

        assert_eq!(result, Some("me"));
    }

    #[test]
    fn extract_host_successfully() {
        let mut headers = HeaderMap::new();

        headers.insert(HOST, HeaderValue::from_static("me.pablomendoza.site"));

        let result = extract_host(&headers);

        assert_eq!(result, Some("me.pablomendoza.site"),);
    }
}
