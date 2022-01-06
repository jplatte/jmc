use std::ops::Deref;

use druid::Target;
use matrix_sdk::{
    event_handler::Ctx,
    room::Room,
    ruma::events::room::{
        create::{RoomType, SyncRoomCreateEvent},
        message::SyncRoomMessageEvent,
        name::SyncRoomNameEvent,
    },
    uuid::Uuid,
};
use tracing::{error, info};

use crate::{
    data::{EventOrTxnId, MinRoomState, UuidWrap},
    ui::actions::{ADD_EVENT, ADD_OR_UPDATE_ROOM, REMOVE_EVENT},
};

pub async fn on_room_create(
    event: SyncRoomCreateEvent,
    room: Room,
    Ctx(ui_handle): Ctx<druid::ExtEventSink>,
) {
    // Ignore rooms with a type (i.e. not regular chat rooms)
    if let Some(t) = event.content.room_type {
        if t != RoomType::Space {
            info!("Ignoring room of unknown type `{}`", t);
        }

        return;
    }

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
    if let Some(txn_id) = &event.unsigned.transaction_id {
        match txn_id.parse::<Uuid>() {
            Ok(txn_id) => {
                if let Err(e) = ui_handle.submit_command(
                    REMOVE_EVENT,
                    EventOrTxnId::TxnId(UuidWrap(txn_id)),
                    Target::Auto,
                ) {
                    error!("{}", e);
                }
            }
            Err(e) => error!("{}", e),
        }
    }

    let event = (room.room_id().into(), event.sender.deref().into(), event.into());
    if let Err(e) = ui_handle.submit_command(ADD_EVENT, event, Target::Auto) {
        error!("{}", e);
    }
}
