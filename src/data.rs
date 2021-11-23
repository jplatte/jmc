use std::{convert::Infallible, sync::Arc};

use druid::{im::OrdMap, Key};
use matrix_sdk::{room, ruma::events::room::message::SyncRoomMessageEvent, Client as MatrixClient};
use task_group::TaskGroup;
use tokio::sync::mpsc::Sender;
use tracing::error;

use crate::{
    ui::actions::UserData,
    util::{EventIdArc, RoomIdArc},
};

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
    pub rooms: OrdMap<RoomIdArc, MinRoomState>, // For the sidebar
    pub active_room: Option<ActiveRoomState>,

    // FIXME: Use Env for these as well
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
            active_room: None,
            mtx_client: data.mtx_client.clone(),
            task_group: data.task_group.clone(),
        }
    }
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct MinRoomState {
    pub id: RoomIdArc,
    pub display_name: String,
    // icon
}

impl MinRoomState {
    pub async fn new(room: &room::Common) -> Self {
        let display_name = match room.display_name().await {
            Ok(name) => name,
            Err(e) => {
                error!("Failed to compute room display name: {}", e);
                "<error>".to_owned()
            }
        };

        Self { id: room.room_id().into(), display_name }
    }
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct ActiveRoomState {
    pub id: RoomIdArc,
    pub display_name: String,
    pub timeline: OrdMap<EventIdArc, EventState>,
}

impl From<&MinRoomState> for ActiveRoomState {
    fn from(st: &MinRoomState) -> Self {
        Self { id: st.id.clone(), display_name: st.display_name.clone(), timeline: OrdMap::new() }
    }
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct EventState {
    pub event_id: EventIdArc,
    pub event_type: EventTypeState,
}

#[derive(Clone, druid::Data)]
pub enum EventTypeState {
    RoomMessage { display_string: String },
}

impl From<SyncRoomMessageEvent> for EventState {
    fn from(ev: SyncRoomMessageEvent) -> Self {
        Self {
            event_id: ev.event_id.into(),
            event_type: EventTypeState::RoomMessage {
                display_string: ev.content.msgtype.body().to_owned(),
            },
        }
    }
}
