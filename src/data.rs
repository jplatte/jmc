use std::{fmt, sync::Arc};

use druid::{
    im::{OrdMap, Vector},
    text::RichText,
    ImageBuf, Key,
};
use druid_widget_nursery::prism::Prism;
use matrix_sdk::{
    room::{self, Room},
    ruma::events::room::message::SyncRoomMessageEvent,
    uuid::Uuid,
};
use tokio::sync::mpsc::Sender;
use tracing::error;

use crate::util::{EventIdArc, RoomIdArc, UserIdArc};

// FIXME: Having to use `Arc` to fulfill the `ValueType` bound here feels wrong.
pub const LOGIN_TX: Key<Arc<Sender<LoginState>>> = Key::new("jmc.login_tx");

#[derive(Clone, druid::Data)]
pub enum AppState {
    Login(LoginState),
    LoggingIn,
    LoggedIn(UserState),
}

#[derive(Clone)]
pub struct AppStateLogin;

#[derive(Clone)]
pub struct AppStateLoggingIn;

#[derive(Clone)]
pub struct AppStateLoggedIn;

impl Prism<AppState, LoginState> for AppStateLogin {
    fn get(&self, data: &AppState) -> Option<LoginState> {
        match data {
            AppState::Login(value) => Some(value.clone()),
            _ => None,
        }
    }

    fn put(&self, data: &mut AppState, inner: LoginState) {
        *data = AppState::Login(inner);
    }
}

impl Prism<AppState, ()> for AppStateLoggingIn {
    fn get(&self, data: &AppState) -> Option<()> {
        match data {
            AppState::LoggingIn => Some(()),
            _ => None,
        }
    }

    fn put(&self, data: &mut AppState, _inner: ()) {
        *data = AppState::LoggingIn;
    }
}

impl Prism<AppState, UserState> for AppStateLoggedIn {
    fn get(&self, data: &AppState) -> Option<UserState> {
        match data {
            AppState::LoggedIn(value) => Some(value.clone()),
            _ => None,
        }
    }

    fn put(&self, data: &mut AppState, inner: UserState) {
        *data = AppState::LoggedIn(inner);
    }
}

#[derive(Clone, Default, druid::Data, druid::Lens)]
pub struct LoginState {
    pub user_id: Arc<String>,
    pub password: Arc<String>,
}

#[derive(Clone, Default, druid::Data, druid::Lens)]
pub struct UserState {
    pub rooms: OrdMap<RoomIdArc, MinRoomState>, // For the sidebar
    pub active_room: Option<ActiveRoomState>,
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct MinRoomState {
    pub id: RoomIdArc,
    pub display_name: Arc<str>,
    pub icon: ImageBuf,

    #[data(ignore)]
    pub room: Room,
}

impl MinRoomState {
    // FIXME: Don't grab the icon here as this makes the first load of the room list rather slow
    pub async fn new(room: Room) -> Self {
        let display_name = match room.display_name().await {
            Ok(name) => name.into(),
            Err(e) => {
                error!("Failed to compute room display name: {}", e);
                "<error>".into()
            }
        };

        Self { id: room.room_id().into(), display_name, icon: ImageBuf::empty(), room }
    }
}

/* fetching images (displaying is broken so this is not currently used)

use druid::image::io::Reader as ImageReader;
use matrix_sdk::{
    media::{MediaFormat, MediaThumbnailSize},
    ruma::{api::client::r0::media::get_content_thumbnail::Method as ResizeMethod, uint},
};
use std::io::Cursor;
use tokio::task;

let icon_format = MediaFormat::Thumbnail(MediaThumbnailSize {
    method: ResizeMethod::Scale,
    width: uint!(32),
    height: uint!(32),
});
let icon_bytes = match room.avatar(icon_format).await {
    Ok(b) => b,
    Err(e) => {
        error!("Failed to load room icon: {}", e);
        None
    }
};
let icon = match icon_bytes {
    Some(bytes) => match decode_image(bytes).await {
        Ok(image) => image,
        Err(e) => {
            error!("Failed to decode room icon: {}", e);
            ImageBuf::empty()
        }
    },
    None => ImageBuf::empty(),
};

async fn decode_image(bytes: Vec<u8>) -> anyhow::Result<ImageBuf> {
    task::spawn_blocking(move || {
        let cursor = Cursor::new(bytes);
        let reader = ImageReader::new(cursor).with_guessed_format()?;
        let image = reader.decode()?;

        Ok(ImageBuf::from_dynamic_image(image))
    })
    .await?
}
*/

#[derive(Clone, druid::Data, druid::Lens)]
pub struct ActiveRoomState {
    pub id: RoomIdArc,
    pub display_name: Arc<str>,
    pub timeline: Vector<EventGroupState>,
    pub kind: Option<JoinedRoomState>,
}

impl From<&MinRoomState> for ActiveRoomState {
    fn from(st: &MinRoomState) -> Self {
        let kind = match &st.room {
            Room::Joined(joined) => {
                Some(JoinedRoomState { message_input: Default::default(), room: joined.to_owned() })
            }
            Room::Left(_) => None,
            Room::Invited(_) => None,
        };

        Self {
            id: st.id.clone(),
            display_name: st.display_name.clone(),
            timeline: Vector::new(),
            kind,
        }
    }
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct EventGroupState {
    pub sender: UserIdArc,
    pub sender_display_name: RichText,
    pub events: Vector<EventState>,
}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct EventState {
    pub id: EventOrTxnId,
    pub event_type: EventTypeState,
}

#[derive(Clone, druid::Data)]
pub enum EventTypeState {
    RoomMessage { display_string: Arc<str> },
}

impl From<SyncRoomMessageEvent> for EventState {
    fn from(ev: SyncRoomMessageEvent) -> Self {
        Self {
            id: EventOrTxnId::EventId(ev.event_id.into()),
            event_type: EventTypeState::RoomMessage {
                display_string: ev.content.msgtype.body().into(),
            },
        }
    }
}

//#[derive(Clone, druid::Data)]
//pub enum RoomKindState {
//    Joined(JoinedRoomState),
//    Left(LeftRoomState),
//    Invited(InvitedRoomState),
//}

#[derive(Clone, druid::Data, druid::Lens)]
pub struct JoinedRoomState {
    pub message_input: Arc<String>,

    #[data(ignore)]
    pub room: room::Joined,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, druid::Data)]
pub enum EventOrTxnId {
    EventId(EventIdArc),
    TxnId(UuidWrap),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, druid::Data)]
pub struct UuidWrap(#[data(eq)] pub Uuid);

impl fmt::Debug for UuidWrap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
