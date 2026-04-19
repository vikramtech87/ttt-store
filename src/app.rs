use crate::db::users::UserRepo;
use axum::{Router, routing::{get}};
use std::sync::Arc;
use crate::handlers::auth::{self, AuthServiceProvier};
use crate::handlers::users::{self, UserState};
use crate::services::auth::AuthService;

#[derive(Clone)]
pub struct AppState {
    pub user_repo: UserRepo,
    pub auth_service: AuthService,
}

pub type SharedState = Arc<AppState>;

impl UserState for SharedState {
    fn user_repo(&self) -> &UserRepo {
        &self.user_repo
    }
}

impl AuthServiceProvier for SharedState {
    fn auth_service(&self) -> &AuthService {
        &self.auth_service
    }
}

pub fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .nest("/users", users::router())
        .nest("/auth", auth::router())
        .with_state(state)
}

async fn health_check() -> &'static str {
    "Ok"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_pool;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use dotenvy::dotenv;
    use tower::ServiceExt;
    use crate::db::auth::AuthRepo;

    async fn get_configured_router() -> Router {
        let pool = init_pool().await.unwrap();
        let user_repo = UserRepo::new(pool.clone());
        let auth_repo = AuthRepo::new(pool);
        let auth_service = AuthService::new(auth_repo);
        let state = Arc::new(AppState { user_repo, auth_service });
        create_router(state)
    }

    #[tokio::test]
    async fn test_health_check() {
        dotenv().ok();
        let app = get_configured_router().await;
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }


}
