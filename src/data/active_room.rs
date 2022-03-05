use std::sync::Arc;

use druid::{im::Vector, text::RichText, ImageBuf};
use druid_widget_nursery::prism::Prism;
use matrix_sdk::room::{self, Room};
use ruma::events::room::message::SyncRoomMessageEvent;

use crate::{
    ui::actions::NewActiveRoomState,
    util::{EventIdArc, RoomIdArc, TransactionIdArc, UserIdArc},
};

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
