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
    use crate::repository::UserRepository;
    use crate::services::{AuthService, TokenService};
    use super::*;

    #[ntex::test]
    async fn test_auth() -> Result<(), Error> {
        let settings = Arc::new(Settings::new()?);
        let database = Arc::new(Database::new(&settings, &SchemaManager::default()).await?);
        let hasher = Arc::new(Argon2Hash::new()) as Arc<dyn Password>;

        let user_repo = Arc::new(UserRepository::new(&hasher, &database));

        let auth_service = Arc::new(AuthService::new(&user_repo, &hasher));
        let token_service = Arc::new(TokenService::new(&settings));

        let auth_state = AuthState {
            auth_service: auth_service.clone(),
            token_service: token_service.clone(),
        };
        let app = App::new().state(auth_state.clone()).service(auth);
        let container = test::init_service(app).await;

        let payload = json!({
            "username": "new_user",
            "email": "new_user@sieluna.com",
            "password": "new_password"
        });

        let req = test::TestRequest::post().uri("/register").set_json(&payload).to_request();
        let resp = container.call(req).await?;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: Value = from_slice(test::read_body(resp).await.as_ref())?;

        assert_eq!(body["username"], "new_user");
        assert_eq!(body["email"], "new_user@sieluna.com");
        Ok(())
    }
}