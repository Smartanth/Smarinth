use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::{env, io};

use ntex::web;
use ntex::web::{scope, App};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::configs::{Argon2Hash, Database, Password, SchemaManager, Settings};
use crate::handlers::{auth, register};
use crate::middlewares::{JWTAuth, JWTAuthMiddleware};
use crate::repository::UserRepository;
use crate::services::{AuthService, TokenService, UserService};
use crate::states::{AuthState, UserState};

mod configs;
mod entities;
mod errors;
mod handlers;
mod middlewares;
mod payload;
mod repository;
mod services;
mod states;

#[ntex::main]
async fn main() -> io::Result<()> {
    let settings = Arc::new(Settings::new().expect("Failed to load settings."));
    let database = Arc::new(Database::new(&settings, &SchemaManager::default()).await.expect("Fail to init database."), );
    let hasher = Arc::new(Argon2Hash::new()) as Arc<dyn Password>;

    let user_repo = Arc::new(UserRepository::new(&hasher, &database));

    let auth_service = Arc::new(AuthService::new(&hasher));
    let token_service = Arc::new(TokenService::new(&settings));
    let user_service = Arc::new(UserService::new(&user_repo));

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                let app_name = env!("CARGO_PKG_NAME").replace('-', "_");
                let level = settings.logger.level.as_str();

                format!("{app_name}={level},tower_http={level}").into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let ip_addr = settings.server.host.parse::<IpAddr>().unwrap();

    let address = SocketAddr::from((ip_addr, settings.server.port));

    tracing::debug!("listening on {}", address);

    web::HttpServer::new(move || {
        let auth_state = AuthState {
            auth_service: auth_service.clone(),
            token_service: token_service.clone(),
            user_service: user_service.clone(),
        };
        let user_state = UserState {
            user_service: user_service.clone(),
        };

        App::new()
            .state(auth_state.clone())
            .state(user_state.clone())
            .service(
                scope("/auth")
                    .service(auth)
                    .service(register),
            )
            .service(
                scope("api")
                    .wrap(JWTAuth::new(&Arc::new(auth_state)))
            )
    })
        .bind(address)?
        .run()
        .await
}
