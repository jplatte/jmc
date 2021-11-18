use std::{convert::Infallible, sync::Arc};

use matrix_sdk::{ruma::RoomId, Client as MatrixClient};
use task_group::TaskGroup;
use tokio::sync::mpsc::Sender;
use tracing::error;

use crate::ui::actions::UserData;

#[derive(Clone, druid::Data, druid::Lens)]
pub struct AppState {
    pub login_state: LoginState,
    pub user_state: Option<UserState>,

    #[data(ignore)]
    #[lens(ignore)]
    login_tx: Sender<LoginState>,
}

impl AppState {
    pub fn new(login_tx: Sender<LoginState>) -> Self {
        Self { login_state: Default::default(), user_state: None, login_tx }
    }

    pub fn login(&self) {
        let send_res = self.login_tx.try_send(self.login_state.clone());

        if let Err(e) = send_res {
            error!("Sending login data failed: {}", e);
        }
    }
}

#[derive(Clone, Default, druid::Data, druid::Lens)]
pub struct LoginState {
    pub user_id: Arc<String>,
    pub password: Arc<String>,
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct UserState {
    #[data(eq)]
    focused_room: Option<Arc<RoomId>>,

    #[data(ignore)]
    #[lens(ignore)]
    pub mtx_client: MatrixClient,

    #[data(ignore)]
    #[lens(ignore)]
    pub task_group: TaskGroup<Infallible>,
}

impl From<&UserData> for UserState {
    fn from(data: &UserData) -> Self {
        Self {
            focused_room: None,
            mtx_client: data.mtx_client.clone(),
            task_group: data.task_group.clone(),
        }
    }
}
