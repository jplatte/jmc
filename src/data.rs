use std::sync::Arc;

use druid::{
    im::{OrdMap, Vector},
    text::RichText,
    ImageBuf, Key,
};
use druid_widget_nursery::prism::Prism;
use matrix_sdk::room::{self, Room};
use ruma::events::room::message::SyncRoomMessageEvent;
use tokio::sync::mpsc::Sender;
use tracing::error;

use crate::{
    ui::actions::NewActiveRoomState,
    util::{EventIdArc, RoomIdArc, TransactionIdArc, UserIdArc},
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
                error!("Failed to compute room display name: {e}");
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
    pub kind: RoomKindState,
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

#[derive(Clone, druid::Data, Prism)]
pub enum RoomKindState {
    Joined(JoinedRoomState),
    Left(LeftRoomState),
    Invited(InvitedRoomState),
}

impl RoomKindState {
    fn joined(room: room::Joined) -> Self {
        Self::Joined(JoinedRoomState { message_input: Default::default(), room })
    }

    fn left(room: room::Left) -> Self {
        Self::Left(LeftRoomState { _dummy: (), room })
    }

    fn invited(room: room::Invited) -> Self {
        Self::Invited(InvitedRoomState { _dummy: (), room })
    }
}

impl From<Room> for RoomKindState {
    fn from(room: Room) -> Self {
        match room {
            Room::Joined(j) => Self::joined(j),
            Room::Left(l) => Self::left(l),
            Room::Invited(i) => Self::invited(i),
        }
    }
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct JoinedRoomState {
    pub message_input: Arc<String>,

    #[data(ignore)]
    pub room: room::Joined,
}

#[derive(Clone, druid::Data)]
#[allow(clippy::manual_non_exhaustive)]
pub struct LeftRoomState {
    _dummy: (),

    #[data(ignore)]
    pub room: room::Left,
}

#[derive(Clone, druid::Data)]
#[allow(clippy::manual_non_exhaustive)]
pub struct InvitedRoomState {
    _dummy: (),

    #[data(ignore)]
    pub room: room::Invited,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, druid::Data)]
pub enum EventOrTxnId {
    EventId(EventIdArc),
    TxnId(TransactionIdArc),
}
