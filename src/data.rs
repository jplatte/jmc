use std::{convert::Infallible, sync::Arc};

use druid::Key;
use matrix_sdk::{ruma::RoomId, Client as MatrixClient};
use task_group::TaskGroup;
use tokio::sync::mpsc::Sender;

use crate::ui::actions::UserData;

// FIXME: Having to use `Arc` to fulfill the `ValueType` bound here feels wrong.
pub const LOGIN_TX: Key<Arc<Sender<LoginState>>> = Key::new("jmc.login_tx");

#[derive(Clone, druid::Data, druid::Lens)]
pub struct AppState {
    pub view: View,
    pub login_state: LoginState,
    pub user_state: Option<UserState>,
}

#[derive(Clone, Copy, PartialEq, druid::Data)]
pub enum View {
    Login,
    Loading,
    Main,
}

impl AppState {
    pub fn new(view: View) -> Self {
        Self { view, login_state: Default::default(), user_state: None }
    }
}

#[derive(Clone, Default, druid::Data, druid::Lens)]
pub struct LoginState {
    pub user_id: Arc<String>,
    pub password: Arc<String>,
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct UserState {
    // For the sidebar
    pub rooms: Arc<Vec<RoomState>>,

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
            rooms: Default::default(),
            mtx_client: data.mtx_client.clone(),
            task_group: data.task_group.clone(),
        }
    }
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct RoomState {
    #[data(eq)]
    pub room_id: Arc<RoomId>,
    // icon
}
