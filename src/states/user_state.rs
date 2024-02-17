use crate::services::user_service::UserService;

#[derive(Clone)]
pub struct UserState<R> {
    pub user_service: UserService<R>,
}