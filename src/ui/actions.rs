use std::convert::Infallible;

use druid::Selector;
use matrix_sdk::{ruma::RoomId, Client as MatrixClient};
use task_group::TaskGroup;

use crate::data::{EventState, MinRoomState};

pub struct UserData {
    pub mtx_client: MatrixClient,
    pub task_group: TaskGroup<Infallible>,
}

pub const FINISH_LOGIN: Selector<UserData> = Selector::new("finish-login");

pub const ADD_OR_UPDATE_ROOM: Selector<MinRoomState> = Selector::new("add-room");
pub const ADD_OR_UPDATE_ROOMS: Selector<Vec<MinRoomState>> = Selector::new("add-rooms");

pub const SET_ACTIVE_ROOM: Selector<MinRoomState> = Selector::new("set-active-room");

pub const ADD_EVENT: Selector<(RoomId, EventState)> = Selector::new("add-event");
