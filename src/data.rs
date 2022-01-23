use std::{fmt, sync::Arc};

use druid::{
    im::{OrdMap, Vector},
    text::RichText,
    ImageBuf, Key,
};
use druid_widget_nursery::prism::Prism;
use matrix_sdk::{
    room::{self, Room},
    ruma::events::room::message::SyncRoomMessageEvent,
    uuid::Uuid,
};
use tokio::sync::mpsc::Sender;
use tracing::error;

use crate::{
    ui::actions::NewActiveRoomState,
    util::{EventIdArc, RoomIdArc, UserIdArc},
};

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
                error!("Failed to compute room display name: {}", e);
                "<error>".into()
            }
        };

        Self { id: room.room_id().into(), display_name, icon: ImageBuf::empty(), room }
    }
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct ActiveRoomState {
    pub id: RoomIdArc,
    pub icon: ImageBuf,
    pub display_name: Arc<str>,
    pub timeline: Vector<EventGroupState>,
    pub kind: Option<JoinedRoomState>,
}

impl From<&NewActiveRoomState> for ActiveRoomState {
    fn from(new: &NewActiveRoomState) -> Self {
        Self {
            id: new.id.clone(),
            icon: new.icon.clone(),
            display_name: new.display_name.clone(),
            timeline: Vector::new(),
            kind: new.kind.clone(),
        }
    }
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct EventGroupState {
    pub sender: UserIdArc,
    pub sender_display_name: RichText,
    pub events: Vector<EventState>,
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, druid::Data)]
pub enum EventOrTxnId {
    EventId(EventIdArc),
    TxnId(UuidWrap),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, druid::Data)]
pub struct UuidWrap(#[data(eq)] pub Uuid);

impl fmt::Debug for UuidWrap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
