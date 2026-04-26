use tower_governor::{
    governor::GovernorConfigBuilder,
    GovernorLayer
};
use std::time::Duration;
use axum::{Router, routing};

use crate::db::create_pool;
use crate::types::AppState;
use crate::handlers;

pub async fn get_router() -> Router {
    let pool = create_pool().await.expect("Error while connecting DB pool");

    let state = AppState { db: pool };

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(3)
        .use_headers()
        .finish()
        .unwrap();

    let governor_limiter = governor_conf.limiter().clone();

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            governor_limiter.retain_recent();
        }
    });

    Router::new()
        .route("/", routing::get(handlers::serve_website))
        .route("/favicon.ico", routing::get(handlers::get_favicon))
        .route(
            "/login", 
            routing::post(handlers::login)
                .route_layer(GovernorLayer::new(governor_conf))
        )
        .route("/check-login", routing::get(handlers::check_login))
        .with_state(state)
}
