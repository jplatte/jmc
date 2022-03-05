use std::sync::Arc;

use druid::{im::OrdMap, ImageBuf, Key};
use druid_widget_nursery::prism::Prism;
use matrix_sdk::room::Room;
use tokio::sync::mpsc::Sender;
use tracing::error;

use crate::util::RoomIdArc;

pub mod active_room;

use active_room::ActiveRoomState;

// FIXME: Having to use `Arc` to fulfill the `ValueType` bound here feels wrong.
pub const LOGIN_TX: Key<Arc<Sender<LoginState>>> = Key::new("jmc.login_tx");

#[allow(clippy::large_enum_variant)] // for now
#[derive(Clone, druid::Data, Prism)]
pub enum AppState {
    Login(LoginState),
    LoggingIn,
    LoggedIn(UserState),
}

#[derive(Clone, Default, druid::Data, druid::Lens)]
pub struct LoginState {
    pub user_id: Arc<String>,
    pub password: Arc<String>,
}

#[derive(Clone, Default, druid::Data, druid::Lens)]
pub struct UserState {
    pub rooms: OrdMap<RoomIdArc, MinRoomState>, // For the sidebar
    pub active_room: Option<ActiveRoomState>,
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct MinRoomState {
    pub id: RoomIdArc,
    pub display_name: Arc<str>,
    pub icon: ImageBuf,

    #[data(ignore)]
    pub room: Room,
}

impl MinRoomState {
    // FIXME: Don't grab the icon here as this makes the first load of the room list rather slow
    pub async fn new(room: Room) -> Self {
        let display_name = match room.display_name().await {
            Ok(name) => name.into(),
            Err(e) => {
                error!("Failed to compute room display name: {e}");
                "<error>".into()
            }
        };

        Self { id: room.room_id().into(), display_name, icon: ImageBuf::empty(), room }
    }
}
