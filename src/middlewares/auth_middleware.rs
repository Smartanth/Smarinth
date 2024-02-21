use std::sync::Arc;

use ntex::{http, Middleware, Service, ServiceCtx};
use ntex::web::{Error, ErrorRenderer, WebRequest, WebResponse};

use crate::entities::user::User;
use crate::errors::token_error::TokenError;
use crate::payload::user_dto::UserIdentity;
use crate::states::auth_state::AuthState;

pub struct JWTAuth {
    state: Arc<AuthState>,
}

impl JWTAuth {
    pub fn new(state: &Arc<AuthState>) -> Self {
        Self {
            state: Arc::clone(state),
        }
    }
}

impl<S> Middleware<S> for JWTAuth {
    type Service = JWTAuthMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        JWTAuthMiddleware {
            service,
            state: self.state.clone(),
        }
    }
}

pub struct JWTAuthMiddleware<S> {
    state: Arc<AuthState>,
    service: S,
}

impl<S, Err> Service<WebRequest<Err>> for JWTAuthMiddleware<S>
    where
        S: Service<WebRequest<Err>, Response = WebResponse, Error = Error> + 'static,
        Err: ErrorRenderer + 'static,
{
    type Response = WebResponse;
    type Error = Error;

    ntex::forward_poll_ready!(service);
    ntex::forward_poll_shutdown!(service);

    async fn call(
        &self,
        req: WebRequest<Err>,
        ctx: ServiceCtx<'_, Self>
    ) -> Result<Self::Response, Self::Error> {
        let token = match req.headers().get(http::header::AUTHORIZATION) {
            Some(header_value) => {
                if let Ok(str_value) = header_value.to_str() {
                    if str_value.starts_with("Bearer ") {
                        Some(str_value[7..].trim())
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            None => None,
        };

        match token {
            Some(token) => {
                let token_data = self.state.token_service.retrieve_token_claims(token)?;

                let identity = UserIdentity::Id(token_data.claims.sub.parse().unwrap());

                req.extensions_mut().insert(self.state.user_service
                    .find_user(identity)
                    .await?);

                let res = ctx.call(&self.service, req).await?;

                res.request().extensions_mut().remove::<User>();

                Ok(res)
            }
            _ => Err(TokenError::MissingToken)?,
        }
    }
}