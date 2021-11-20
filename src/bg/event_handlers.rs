use std::sync::Arc;

use druid::Target;
use matrix_sdk::{event_handler::Ctx, room::Room, ruma::events::room::create::SyncRoomCreateEvent};
use tracing::error;

use crate::ui::actions::ADD_ROOM;

pub async fn on_room_create(
    _event: SyncRoomCreateEvent,
    room: Room,
    Ctx(ui_handle): Ctx<druid::ExtEventSink>,
) {
    if let Err(e) =
        ui_handle.submit_command(ADD_ROOM, Arc::new(room.room_id().to_owned()), Target::Auto)
    {
        error!("{}", e);
    }
}
