use std::{io::Cursor, sync::Arc};

use druid::{image::io::Reader as ImageReader, ExtEventSink, ImageBuf, Selector, Target};
use futures_signals::signal_vec::VecDiff;
use futures_util::StreamExt;
use matrix_sdk::{
    media::{MediaFormat, MediaThumbnailSize},
    room::timeline::{Timeline, TimelineItem},
    ruma,
};
use ruma::{api::client::media::get_content_thumbnail::v3::Method as ResizeMethod, uint};
use tokio::task;
use tracing::error;

use crate::{
    data::{active_room::RoomKindState, MinRoomState},
    util::RoomId,
};

pub const FINISH_LOGIN: Selector<()> = Selector::new("finish-login");

pub const ADD_OR_UPDATE_ROOM: Selector<MinRoomState> = Selector::new("add-room");

pub const SET_ACTIVE_ROOM: Selector<NewActiveRoomState> = Selector::new("set-active-room");

pub const APPLY_TIMELINE_DIFF: Selector<(RoomId, VecDiff<Arc<TimelineItem>>)> =
    Selector::new("apply-timeline-diff");

pub struct NewActiveRoomState {
    pub id: RoomId,
    pub icon: ImageBuf,
    pub display_name: Arc<str>,
    pub kind: RoomKindState,
    pub sdk_timeline: Arc<Timeline>,
}

impl NewActiveRoomState {
    pub async fn new(st: MinRoomState, ui_handle: ExtEventSink) -> Self {
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

        let sdk_timeline = Arc::new(st.room.timeline().await);

        let room_id: RoomId = st.room.room_id().into();
        let mut timeline_stream = sdk_timeline.stream();
        tokio::spawn(async move {
            while let Some(diff) = timeline_stream.next().await {
                if let Err(e) = ui_handle.submit_command(
                    APPLY_TIMELINE_DIFF,
                    (room_id.clone(), diff),
                    Target::Auto,
                ) {
                    error!("{e}");
                }
            }
        });

        Self {
            id: st.id.clone(),
            icon,
            display_name: st.display_name.clone(),
            kind: st.room.into(),
            sdk_timeline,
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
