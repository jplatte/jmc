use std::{convert::Infallible, sync::Arc};

use druid::Selector;
use matrix_sdk::{ruma::RoomId, Client as MatrixClient};
use task_group::TaskGroup;

pub struct UserData {
    pub mtx_client: MatrixClient,
    pub task_group: TaskGroup<Infallible>,
}

pub const FINISH_LOGIN: Selector<UserData> = Selector::new("finish-login");

pub const ADD_ROOM: Selector<Arc<RoomId>> = Selector::new("add-room");
pub const ADD_ROOMS: Selector<Vec<Arc<RoomId>>> = Selector::new("add-rooms");
