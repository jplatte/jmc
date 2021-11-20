use std::convert::Infallible;

use druid::Selector;
use matrix_sdk::Client as MatrixClient;
use task_group::TaskGroup;

use crate::data::RoomState;

pub struct UserData {
    pub mtx_client: MatrixClient,
    pub task_group: TaskGroup<Infallible>,
}

pub const FINISH_LOGIN: Selector<UserData> = Selector::new("finish-login");

pub const ADD_OR_UPDATE_ROOM: Selector<RoomState> = Selector::new("add-room");
pub const ADD_OR_UPDATE_ROOMS: Selector<Vec<RoomState>> = Selector::new("add-rooms");
