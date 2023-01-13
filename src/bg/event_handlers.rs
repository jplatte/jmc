use druid::Target;
use matrix_sdk::{event_handler::Ctx, room::Room, ruma};
use ruma::{
    events::room::{create::OriginalSyncRoomCreateEvent, name::OriginalSyncRoomNameEvent},
    room::RoomType,
};
use tracing::{error, info};

use crate::{data::MinRoomState, ui::actions::ADD_OR_UPDATE_ROOM};

pub async fn on_room_create(
    event: OriginalSyncRoomCreateEvent,
    room: Room,
    Ctx(ui_handle): Ctx<druid::ExtEventSink>,
) {
    // Ignore rooms with a type (i.e. not regular chat rooms)
    if let Some(t) = event.content.room_type {
        if t != RoomType::Space {
            info!("Ignoring room of unknown type `{t}`");
        }

        return;
    }

    if let Err(e) =
        ui_handle.submit_command(ADD_OR_UPDATE_ROOM, MinRoomState::new(room).await, Target::Auto)
    {
        error!("{e}");
    }
}

pub async fn on_room_name(
    _event: OriginalSyncRoomNameEvent,
    room: Room,
    Ctx(ui_handle): Ctx<druid::ExtEventSink>,
) {
    if let Err(e) =
        ui_handle.submit_command(ADD_OR_UPDATE_ROOM, MinRoomState::new(room).await, Target::Auto)
    {
        error!("{e}");
    }
}
