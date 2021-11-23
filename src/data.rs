use std::{convert::Infallible, io::Cursor, sync::Arc};

use druid::{im::OrdMap, image::io::Reader as ImageReader, ImageBuf, Key};
use matrix_sdk::{
    media::{MediaFormat, MediaThumbnailSize},
    room,
    ruma::{
        api::client::r0::media::get_content_thumbnail::Method as ResizeMethod,
        events::room::message::SyncRoomMessageEvent, uint,
    },
    Client as MatrixClient,
};
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
    pub icon: Option<ImageBuf>,
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

        let icon_format = MediaFormat::Thumbnail(MediaThumbnailSize {
            method: ResizeMethod::Scale,
            width: uint!(32),
            height: uint!(32),
        });
        let icon_bytes = match room.avatar(icon_format).await {
            Ok(b) => b,
            Err(e) => {
                error!("Failed to load room icon: {}", e);
                None
            }
        };
        let icon =
            icon_bytes.and_then(|bytes| match ImageReader::new(Cursor::new(bytes)).decode() {
                Ok(image) => Some(ImageBuf::from_dynamic_image(image)),
                Err(e) => {
                    error!("Failed to decode room icon: {}", e);
                    None
                }
            });

        Self { id: room.room_id().into(), display_name, icon }
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
