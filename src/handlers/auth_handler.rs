use ntex::web::{post, types, Error, HttpResponse, Responder};

use crate::payload::{UserAuthDto, UserCreateDto};
use crate::states::AuthState;

#[post("/login")]
pub async fn auth(
    payload: types::Json<UserAuthDto>,
    auth_state: types::State<AuthState>,
) -> Result<impl Responder, Error> {
    let types::Json(user_data) = payload;

    let user = auth_state.auth_service.authorization_user(user_data).await?;

    let result = auth_state.token_service.generate_token(user)?;

    Ok(HttpResponse::Ok().json(&result))
}

#[post("/register")]
pub async fn register(
    payload: types::Json<UserCreateDto>,
    auth_state: types::State<AuthState>,
) -> Result<impl Responder, Error> {
    let types::Json(create_data) = payload;

    let result = auth_state.auth_service.create_user(create_data).await?;

    Ok(HttpResponse::Ok().json(&result))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ntex::http::StatusCode;
    use ntex::web::{test, App, Error};
    use serde_json::{from_slice, json, Value};

    use crate::configs::{Argon2Hash, Database, Password, SchemaManager, Settings};
    use crate::errors::{ApiError, DatabaseError};
    use crate::repository::UserRepository;
    use crate::services::{AuthService, TokenService};
    use crate::sql;
    use super::*;

    struct AuthEnvironment {
        user_repo: Arc<UserRepository>,
        auth_state: AuthState,
    }

    impl AuthEnvironment {
        async fn new() -> Result<Self, ApiError> {
            let settings = Arc::new(Settings::new()?);
            let database = Arc::new(Database::new(&settings, &SchemaManager::default()).await?);
            let hasher = Arc::new(Argon2Hash::new()) as Arc<dyn Password>;

            let user_repo = Arc::new(UserRepository::new(&hasher, &database));

            let auth_state = AuthState {
                auth_service: Arc::new(AuthService::new(&user_repo, &hasher)),
                token_service: Arc::new(TokenService::new(&settings)),
            };

            Ok(Self { user_repo, auth_state })
        }
    }

    #[ntex::test]
    async fn test_auth() -> Result<(), Error> {
        let AuthEnvironment { user_repo, auth_state } = AuthEnvironment::new().await?;
    
        let app = App::new().state(auth_state).service(auth);
        let container = test::init_service(app).await;
    
        let username = "test_auth_user";
        let email = "test_auth_user@sieluna.com";
        let password = "test_auth_password";
    
        let statement = sql!(
            user_repo.database.scheme,
            "INSERT INTO users (username, email, password) VALUES ($1, $2, $3)"
        );
        sqlx::query(&statement)
            .bind(&username)
            .bind(&email)
            .bind(&user_repo.password.hash(&password)?)
            .execute(&user_repo.database.pool)
            .await
            .map_err(DatabaseError::from)?;
    
        let payload = json!({
            "identity": { "email": email },
            "password": password
        });
    
        let req = test::TestRequest::post().uri("/login").set_json(&payload).to_request();
        let resp = container.call(req).await?;
    
        assert_eq!(resp.status(), StatusCode::OK);
    
        let body: Value = from_slice(test::read_body(resp).await.as_ref())?;
    
        assert!(body["token"].is_string());
        assert!(body["iat"].is_number());
        assert!(body["exp"].is_number());
        Ok(())
    }

    #[ntex::test]
    async fn test_register() -> Result<(), Error> {
        let AuthEnvironment { auth_state, .. } = AuthEnvironment::new().await?;

        let app = App::new().state(auth_state).service(register);
        let container = test::init_service(app).await;

        let payload = json!({
            "username": "test_register_user",
            "email": "test_register_user@sieluna.com",
            "password": "test_register_password"
        });

        let req = test::TestRequest::post().uri("/register").set_json(&payload).to_request();
        let resp = container.call(req).await?;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: Value = from_slice(test::read_body(resp).await.as_ref())?;

        assert_eq!(body["username"], "test_register_user");
        assert_eq!(body["email"], "test_register_user@sieluna.com");
        Ok(())
    }
}
