use std::{io::Cursor, sync::Arc};

use druid::{image::io::Reader as ImageReader, ImageBuf, Selector};
use matrix_sdk::media::{MediaFormat, MediaThumbnailSize};
use ruma::{api::client::media::get_content_thumbnail::v3::Method as ResizeMethod, uint};
use tokio::task;
use tracing::error;

use crate::{
    data::{
        active_room::{EventOrTxnId, EventState, RoomKindState},
        MinRoomState,
    },
    util::{RoomId, UserId},
};

pub const FINISH_LOGIN: Selector<()> = Selector::new("finish-login");

pub const ADD_OR_UPDATE_ROOM: Selector<MinRoomState> = Selector::new("add-room");

pub const SET_ACTIVE_ROOM: Selector<NewActiveRoomState> = Selector::new("set-active-room");

pub const APPEND_EVENT: Selector<(RoomId, UserId, EventState)> = Selector::new("append-event");
pub const PREPEND_EVENT: Selector<(RoomId, UserId, EventState)> = Selector::new("prepend-event");
pub const REMOVE_EVENT: Selector<EventOrTxnId> = Selector::new("remove-event");
// FIXME: Maybe have `REPLACE_EVENT` (instead)?

pub struct NewActiveRoomState {
    pub id: RoomId,
    pub icon: ImageBuf,
    pub display_name: Arc<str>,
    pub kind: RoomKindState,
}

impl NewActiveRoomState {
    pub async fn new(st: MinRoomState) -> Self {
        let icon_format = MediaFormat::Thumbnail(MediaThumbnailSize {
            method: ResizeMethod::Scale,
            width: uint!(32),
            height: uint!(32),
        });
        let icon_bytes = match st.room.avatar(icon_format).await {
            Ok(b) => b,
            Err(e) => {
                error!("Failed to load room icon: {e}");
                None
            }
        };
        let icon = match icon_bytes {
            Some(bytes) => match decode_image(bytes).await {
                Ok(image) => image,
                Err(e) => {
                    error!("Failed to decode room icon: {e}");
                    ImageBuf::empty()
                }
            },
            None => ImageBuf::empty(),
        };

        Self {
            id: st.id.clone(),
            icon,
            display_name: st.display_name.clone(),
            kind: st.room.into(),
        }
    }
}

async fn decode_image(bytes: Vec<u8>) -> anyhow::Result<ImageBuf> {
    task::spawn_blocking(move || {
        let cursor = Cursor::new(bytes);
        let reader = ImageReader::new(cursor).with_guessed_format()?;
        let image = reader.decode()?;

        Ok(ImageBuf::from_dynamic_image(image))
    })
    .await?
}
