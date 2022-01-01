use std::sync::Arc;

use druid::Selector;
use matrix_sdk::ruma::RoomId;

use crate::data::{EventOrTxnId, EventState, MinRoomState};

pub const FINISH_LOGIN: Selector<()> = Selector::new("finish-login");

pub const ADD_OR_UPDATE_ROOM: Selector<MinRoomState> = Selector::new("add-room");
pub const ADD_OR_UPDATE_ROOMS: Selector<Vec<MinRoomState>> = Selector::new("add-rooms");

pub const SET_ACTIVE_ROOM: Selector<MinRoomState> = Selector::new("set-active-room");

pub const ADD_EVENT: Selector<(Arc<RoomId>, EventState)> = Selector::new("add-event");
pub const REMOVE_EVENT: Selector<EventOrTxnId> = Selector::new("remove-event");
// FIXME: Maybe have `REPLACE_EVENT` (instead)?
