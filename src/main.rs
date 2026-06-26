mod handlers;
mod errors;
mod models;
mod state;

use axum::{
    routing::get,
    Router,
};

use handlers::*;
use state::AppState;
use tracing::info;

use tower_http::cors::{
    Any,
    CorsLayer,
};

use sqlx::{
    PgPool,
    postgres::PgPoolOptions,
};


#[tokio::main]
async fn main() -> Result<(), std::io::Error>{

    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool: PgPool = PgPoolOptions::new().max_connections(5).connect(&database_url).await.expect("failed to connect to database");

    info!("database connection established");

    let state = AppState::new(pool,);

    let app = Router::new()
                .route("/health", get(health_check))
                .route("/links/count",get(count_links))
                .route("/links",get(list_links))
                .route("/links/{subdomain}",get(get_link_by_subdomain))
                .route("/debug/host", get(read_host_header))
                .route("/debug/subdomain",get(read_subdomain))
                .route("/analytics/clicks/total",get(get_click_count))
                .route("/analytics/clicks",get(list_click_events))
                .route("/analytics/visitors/unique", get(get_unique_visitor_count))
                .route("/analytics/devices",get(get_device_analytics))
                .route("/analytics/referrers",get(get_referrer_analytics))
                .fallback(get(redirect_by_host))
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any)
                )
                .with_state(state);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);

    let address = format!("0.0.0.0:{port}");

    let listener = tokio::net::TcpListener::bind(&address)
                    .await?;
    
    info!("server listening on {}", address);

    axum::serve(listener, app)
        .await

}