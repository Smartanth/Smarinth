use std::{env, io};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use ntex::web;
use ntex::web::{App, scope};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::configs::database::Database;
use crate::configs::password::{Argon2Hash, Password};
use crate::configs::settings::Settings;
use crate::handlers::auth_handler;
use crate::middlewares::auth_middleware::JWTAuth;
use crate::repository::user_repository::UserRepository;
use crate::services::{auth_service, control_service, token_service, user_service};
use crate::states::{auth_state, user_state};

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
    let database = Arc::new(Database::new(&settings).await.expect("Fail to init database."));
    let hasher = Arc::new(Argon2Hash::new()) as Arc<dyn Password>;

    let user_repo = Arc::new(UserRepository::new(&hasher, &database));

    let auth_service = Arc::new(auth_service::AuthService::new(&hasher));
    let control_service = Arc::new(control_service::ControlService::new(&settings).await);
    let token_service = Arc::new(token_service::TokenService::new(&settings));
    let user_service = Arc::new(user_service::UserService::new(&user_repo));

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            let app_name = env!("CARGO_PKG_NAME").replace('-', "_");
            let level = settings.logger.level.as_str();

            format!("{app_name}={level},tower_http={level}").into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let ip_addr = settings.server.host.parse::<IpAddr>().unwrap();

    let address = SocketAddr::from((ip_addr, settings.server.port));

    tracing::debug!("listening on {}", address);

    web::HttpServer::new(move || {
        let auth_state = auth_state::AuthState {
            auth_service: auth_service.clone(),
            token_service: token_service.clone(),
            user_service: user_service.clone(),
        };
        let user_state = user_state::UserState {
            user_service: user_service.clone(),
        };

        App::new()
            .state(auth_state.clone())
            .state(user_state.clone())
            .service(
                scope("/auth")
                    .service(auth_handler::auth)
                    .service(auth_handler::register)
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
