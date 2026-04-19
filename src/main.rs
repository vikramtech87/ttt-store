use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use crate::{
    app::{create_router, AppState},
    db::{init_pool, users::UserRepo, auth::AuthRepo},
    services::auth::AuthService,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod db;
pub mod models;
pub mod app;
pub mod handlers;
pub mod error;
pub mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "store_backend=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();


    let pool = init_pool().await?;

    let state = Arc::new(AppState {
        user_repo: UserRepo::new(pool.clone()),
        auth_service: AuthService::new(AuthRepo::new(pool)),
    });

    let app = create_router(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("🚀 Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;

    Ok(())
}