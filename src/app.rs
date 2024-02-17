use std::sync::Arc;

use axum::http::{StatusCode, Uri};
use axum::Router;
use axum::routing::post;
use tower_http::cors::CorsLayer;

use crate::configs::database::Database;
use crate::configs::password::{Argon2Hash, Hasher};
use crate::configs::settings::Settings;
use crate::handlers::{auth_handler, register_handler};
use crate::repository::user_repository::{RemoteUserRepository, UserRepository};
use crate::services::auth_service::AuthService;
use crate::services::token_service::TokenService;
use crate::services::user_service::UserService;
use crate::states::auth_state::AuthState;
use crate::states::user_state::UserState;

/// Coding strategy [axum example](https://github.com/tokio-rs/axum/blob/main/examples/dependency-injection/src/main.rs)
///
/// 1. Using trait objects (`dyn SomeTrait`)
/// 2. Using generics (`T where T: SomeTrait`)
pub async fn create_app(settings: &Arc<Settings>) -> Router {
    let db_conn = Arc::new(Database::new(settings).await.unwrap());
    let hasher = Arc::new(Argon2Hash::new()) as Arc<dyn Hasher>;

    let routers = {
        let user_repo = RemoteUserRepository::new(&hasher, &db_conn);

        let auth_service = AuthService::new(&hasher);
        let token_service = TokenService::new(settings);
        let user_service = UserService::new(user_repo.clone());

        let auth_state = AuthState { token_service, auth_service, user_repo };
        let user_state = UserState { user_service };

        Router::new()
            .merge(Router::new().route("/auth", post(auth_handler::auth)).with_state(auth_state))
            .merge(Router::new().route("/register", post(register_handler::register)).with_state(user_state))
    };

    Router::new()
        .nest("/api", routers)
        .layer(CorsLayer::permissive())
        .fallback(|uri: Uri| async move {
            (StatusCode::NOT_FOUND, format!("No route for {uri}"))
        })
}