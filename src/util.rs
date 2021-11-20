use std::{fmt, sync::Arc};

use matrix_sdk::ruma::RoomId;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, druid::Data)]
pub struct RoomIdArc(#[data(eq)] Arc<RoomId>);

impl fmt::Display for RoomIdArc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<RoomId> for RoomIdArc {
    fn from(room_id: RoomId) -> Self {
        Self(room_id.into())
    }
}

impl From<&RoomId> for RoomIdArc {
    fn from(room_id: &RoomId) -> Self {
        Self(room_id.to_owned().into())
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
