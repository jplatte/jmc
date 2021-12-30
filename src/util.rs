use std::{fmt, ops::Deref, sync::Arc};

use matrix_sdk::ruma::{EventId, RoomId};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, druid::Data)]
pub struct EventIdArc(#[data(eq)] Arc<EventId>);

impl fmt::Display for EventIdArc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for EventIdArc {
    type Target = Arc<EventId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&EventId> for EventIdArc {
    fn from(room_id: &EventId) -> Self {
        Self(room_id.to_owned().into())
    }
}

impl From<Box<EventId>> for EventIdArc {
    fn from(room_id: Box<EventId>) -> Self {
        Self(room_id.into())
    }
}

impl From<Arc<EventId>> for EventIdArc {
    fn from(arc: Arc<EventId>) -> Self {
        Self(arc)
    }
}

impl From<&Arc<EventId>> for EventIdArc {
    fn from(arc: &Arc<EventId>) -> Self {
        Self(arc.clone())
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, druid::Data)]
pub struct RoomIdArc(#[data(eq)] Arc<RoomId>);

impl fmt::Display for RoomIdArc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for RoomIdArc {
    type Target = Arc<RoomId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&RoomId> for RoomIdArc {
    fn from(room_id: &RoomId) -> Self {
        Self(room_id.to_owned().into())
    }
}

impl From<Box<RoomId>> for RoomIdArc {
    fn from(room_id: Box<RoomId>) -> Self {
        Self(room_id.into())
    }
}

impl From<Arc<RoomId>> for RoomIdArc {
    fn from(arc: Arc<RoomId>) -> Self {
        Self(arc)
    }
}

impl From<&Arc<RoomId>> for RoomIdArc {
    fn from(arc: &Arc<RoomId>) -> Self {
        Self(arc.clone())
    }
}
