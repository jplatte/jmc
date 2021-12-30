use std::{io::Cursor, sync::Arc};

use druid::{im::OrdMap, image::io::Reader as ImageReader, ImageBuf, Key};
use matrix_sdk::{
    media::{MediaFormat, MediaThumbnailSize},
    room::{self, Room},
    ruma::{
        api::client::r0::media::get_content_thumbnail::Method as ResizeMethod,
        events::room::message::SyncRoomMessageEvent, uint,
    },
    uuid::Uuid,
};
use tokio::sync::mpsc::Sender;
use tracing::error;

use crate::util::{EventIdArc, RoomIdArc};

// FIXME: Having to use `Arc` to fulfill the `ValueType` bound here feels wrong.
pub const LOGIN_TX: Key<Arc<Sender<LoginState>>> = Key::new("jmc.login_tx");

#[derive(Clone, druid::Data, druid::Lens)]
pub struct AppState {
    pub view: View,
    pub login_state: LoginState,
    pub user_state: UserState,
}

#[derive(Clone, Copy, PartialEq, Eq, druid::Data)]
pub enum View {
    Login,
    Loading,
    LoggedIn,
}

impl AppState {
    pub fn new(view: View) -> Self {
        Self { view, login_state: Default::default(), user_state: Default::default() }
    }
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
    pub icon: Option<ImageBuf>,

    #[data(ignore)]
    pub room: Room,
}

impl MinRoomState {
    pub async fn new(room: Room) -> Self {
        let display_name = match room.display_name().await {
            Ok(name) => name.into(),
            Err(e) => {
                error!("Failed to compute room display name: {}", e);
                "<error>".into()
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

        Self { id: room.room_id().into(), display_name, icon, room }
    }
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct ActiveRoomState {
    pub id: RoomIdArc,
    pub display_name: Arc<str>,
    pub timeline: OrdMap<EventOrTxnId, EventState>,
    pub kind: Option<JoinedRoomState>,
}

impl From<&MinRoomState> for ActiveRoomState {
    fn from(st: &MinRoomState) -> Self {
        let kind = match &st.room {
            Room::Joined(joined) => {
                Some(JoinedRoomState { message_input: Default::default(), room: joined.to_owned() })
            }
            Room::Left(_) => None,
            Room::Invited(_) => None,
        };

        Self {
            id: st.id.clone(),
            display_name: st.display_name.clone(),
            timeline: OrdMap::new(),
            kind,
        }
    }
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct EventState {
    pub id: EventOrTxnId,
    pub event_type: EventTypeState,
}

#[derive(Clone, druid::Data)]
pub enum EventTypeState {
    RoomMessage { display_string: Arc<str> },
}

impl From<SyncRoomMessageEvent> for EventState {
    fn from(ev: SyncRoomMessageEvent) -> Self {
        Self {
            id: EventOrTxnId::EventId(ev.event_id.into()),
            event_type: EventTypeState::RoomMessage {
                display_string: ev.content.msgtype.body().into(),
            },
        }
    }
}

//#[derive(Clone, druid::Data)]
//pub enum RoomKindState {
//    Joined(JoinedRoomState),
//    Left(LeftRoomState),
//    Invited(InvitedRoomState),
//}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct JoinedRoomState {
    pub message_input: Arc<String>,

    #[data(ignore)]
    pub room: room::Joined,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, druid::Data)]
pub enum EventOrTxnId {
    EventId(EventIdArc),
    TxnId(UuidWrap),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, druid::Data)]
pub struct UuidWrap(#[data(eq)] pub Uuid);
