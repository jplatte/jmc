use druid::Target;
use matrix_sdk::{
    event_handler::Ctx,
    room::Room,
    ruma::events::room::{
        create::SyncRoomCreateEvent, message::SyncRoomMessageEvent, name::SyncRoomNameEvent,
    },
};
use tracing::error;

use crate::{
    data::MinRoomState,
    ui::actions::{ADD_EVENT, ADD_OR_UPDATE_ROOM},
};

pub async fn on_room_create(
    _event: SyncRoomCreateEvent,
    room: Room,
    Ctx(ui_handle): Ctx<druid::ExtEventSink>,
) {
    if let Err(e) =
        ui_handle.submit_command(ADD_OR_UPDATE_ROOM, MinRoomState::new(room).await, Target::Auto)
    {
        error!("{}", e);
    }
}

pub async fn on_room_name(
    _event: SyncRoomNameEvent,
    room: Room,
    Ctx(ui_handle): Ctx<druid::ExtEventSink>,
) {
    if let Err(e) =
        ui_handle.submit_command(ADD_OR_UPDATE_ROOM, MinRoomState::new(room).await, Target::Auto)
    {
        error!("{}", e);
    }
}

pub async fn on_room_message(
    event: SyncRoomMessageEvent,
    room: Room,
    Ctx(ui_handle): Ctx<druid::ExtEventSink>,
) {
    if let Err(e) =
        ui_handle.submit_command(ADD_EVENT, (room.room_id().into(), event.into()), Target::Auto)
    {
        error!("{}", e);
    }
}
