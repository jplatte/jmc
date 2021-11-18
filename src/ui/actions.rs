use std::convert::Infallible;

use druid::Selector;
use matrix_sdk::Client as MatrixClient;
use task_group::TaskGroup;

pub struct UserData {
    pub mtx_client: MatrixClient,
    pub task_group: TaskGroup<Infallible>,
}

pub const FINISH_LOGIN: Selector<UserData> = Selector::new("finish-login");
