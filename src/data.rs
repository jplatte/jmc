use std::sync::Arc;

#[derive(Clone, druid::Data)]
pub enum AppState {
    LoggedOut(LoginState),
    LoggedIn(UserState),
}

impl Default for AppState {
    fn default() -> Self {
        Self::LoggedOut(LoginState::default())
    }
}

#[derive(Clone, Default, druid::Data, druid::Lens)]
pub struct LoginState {
    user_id: Arc<String>,
    password: Arc<String>,
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct UserState {}
