use std::sync::Arc;

use druid::{im::Vector, text::RichText, ImageBuf};
use druid_widget_nursery::prism::Prism;
use matrix_sdk::{
    room::{self, timeline::Timeline, Room},
    ruma,
};
use ruma::events::room::message::OriginalSyncRoomMessageEvent;

use crate::{
    ui::actions::NewActiveRoomState,
    util::{EventId, RoomId, TransactionId, UserId},
};

#[derive(Clone, druid::Data, druid::Lens)]
pub struct ActiveRoomState {
    pub id: RoomId,
    pub icon: ImageBuf,
    pub display_name: Arc<str>,
    pub timeline: Vector<EventState>,
    pub kind: RoomKindState,
    pub show_spinner: bool,

    #[data(ignore)]
    pub sdk_timeline: Arc<Timeline>,
}

impl ActiveRoomState {
    pub fn room(&self) -> &room::Common {
        match &self.kind {
            RoomKindState::Joined(j) => &j.room,
            RoomKindState::Left(l) => &l.room,
            RoomKindState::Invited(i) => &i.room,
        }
    }
}

impl From<&NewActiveRoomState> for ActiveRoomState {
    fn from(new: &NewActiveRoomState) -> Self {
        Self {
            id: new.id.clone(),
            icon: new.icon.clone(),
            display_name: new.display_name.clone(),
            timeline: Vector::new(),
            kind: new.kind.clone(),
            show_spinner: false,
            sdk_timeline: new.sdk_timeline.clone(),
        }
    }
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct EventState {
    pub id: EventOrTxnId,
    pub sender: UserId,
    pub sender_display_name: RichText,
    pub event_type: EventTypeState,
}

#[derive(Clone, druid::Data)]
pub enum EventTypeState {
    RoomMessage { display_string: Arc<str> },
}

impl From<OriginalSyncRoomMessageEvent> for EventState {
    fn from(ev: OriginalSyncRoomMessageEvent) -> Self {
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
    EventId(EventId),
    TxnId(TransactionId),
}
