use std::ops::ControlFlow;

use anyhow::bail;
use druid::Target;
use matrix_sdk::{
    config::SyncSettings,
    store::{make_store_config, OpenStoreError},
    Client as MatrixClient, ClientBuilder as MatrixClientBuilder, Session,
};
use ruma::{
    api::client::{
        filter::{FilterDefinition, LazyLoadOptions, RoomEventFilter, RoomFilter},
        session::login::v3::Response as LoginResponse,
        sync::sync_events::v3::Filter,
    },
    assign, UserId,
};
use tokio::{fs, sync::mpsc::Receiver, task};
use tracing::error;

use crate::{
    config::{self, Config, CONFIG_DIR_PATH},
    data::{LoginState, MinRoomState},
    ui::actions::{ADD_OR_UPDATE_ROOM, FINISH_LOGIN},
};

pub mod event_handlers;

enum State {
    LoggedOut,
    LoggedIn { mtx_client: MatrixClient },
}

pub async fn main(
    config: Config,
    mut login_rx: Receiver<LoginState>,
    ui_handle: druid::ExtEventSink,
) {
    if let Err(e) = fs::create_dir_all(&*CONFIG_DIR_PATH).await {
        error!("Failed to create store directory: {e}");
        return;
    }

    let mut state = if let Some(session) = config.session {
        match restore_login(session.clone()).await {
            Ok(mtx_client) => State::LoggedIn { mtx_client },
            Err(e) => {
                error!("{e}");
                // FIXME: Display an error message on the login screen
                State::LoggedOut
            }
        }
    } else {
        State::LoggedOut
    };

    loop {
        let res = match state {
            State::LoggedOut => logged_out_main(&mut login_rx).await,
            State::LoggedIn { mtx_client } => logged_in_main(mtx_client, ui_handle.clone()).await,
        };

        match res {
            ControlFlow::Continue(new_state) => state = new_state,
            ControlFlow::Break(_) => break,
        }
    }
}

async fn logged_out_main(login_rx: &mut Receiver<LoginState>) -> ControlFlow<(), State> {
    let login_state = match login_rx.recv().await {
        Some(s) => s,
        None => return ControlFlow::Break(()),
    };

    match login(login_state).await {
        Ok((mtx_client, login_response)) => {
            let session = Session {
                access_token: login_response.access_token,
                user_id: login_response.user_id,
                device_id: login_response.device_id,
                refresh_token: login_response.refresh_token,
            };
            let config = Config { session: Some(session) };

            if let Err(e) = task::spawn_blocking(move || config::save(&config)).await {
                error!("Failed to save config: {e:?}");
            }

            ControlFlow::Continue(State::LoggedIn { mtx_client })
        }
        Err(e) => {
            error!("{e:?}");
            ControlFlow::Continue(State::LoggedOut)
        }
    }
}

async fn logged_in_main(
    mtx_client: MatrixClient,
    ui_handle: druid::ExtEventSink,
) -> ControlFlow<(), State> {
    if let Err(e) = ui_handle.submit_command(FINISH_LOGIN, (), Target::Auto) {
        error!("{e}");
    }

    task::spawn({
        let ui_handle = ui_handle.clone();
        let mtx_client = mtx_client.clone();

        async move {
            let joined_rooms = mtx_client.rooms();

            if !joined_rooms.is_empty() {
                for r in joined_rooms {
                    if r.tombstone().is_some() {
                        // Skip rooms that have been superseded (for now)
                        // FIXME: Find a better solution.
                        continue;
                    }

                    let room_state = MinRoomState::new(r).await;
                    if let Err(e) =
                        ui_handle.submit_command(ADD_OR_UPDATE_ROOM, room_state, Target::Auto)
                    {
                        error!("{e}");
                    }
                }
            }
        }
    });

    let filter = assign!(FilterDefinition::default(), {
        room: assign!(RoomFilter::default(), {
            state: assign!(RoomEventFilter::default(), {
                lazy_load_options: LazyLoadOptions::Enabled { include_redundant_members: false }
            })
        })
    });
    let filter_id = mtx_client.get_or_upload_filter("jmc_sync", filter).await.unwrap();

    mtx_client.add_event_handler_context(ui_handle.clone());
    mtx_client.add_event_handler(event_handlers::on_room_create);
    mtx_client.add_event_handler(event_handlers::on_room_name);
    mtx_client.add_event_handler(event_handlers::on_room_message);

    let sync_settings = SyncSettings::new().filter(Filter::FilterId(&filter_id));
    if let Err(e) = mtx_client.sync(sync_settings).await {
        error!("sync failed: {e}");
    }

    ControlFlow::Break(())
}

async fn login(login_data: LoginState) -> anyhow::Result<(MatrixClient, LoginResponse)> {
    let user_id = match UserId::parse(login_data.user_id.as_str()) {
        Ok(id) => id,
        Err(e) => {
            // FIXME: Show error in UI
            bail!("Can't log in due to invalid User ID: {e}");
        }
    };

    let mtx_client = matrix_client_builder()?.server_name(user_id.server_name()).build().await?;
    let response = mtx_client
        .login_username(user_id.localpart(), &login_data.password)
        .initial_device_display_name("jmc")
        .send()
        .await?;

    Ok((mtx_client, response))
}

async fn restore_login(session: Session) -> anyhow::Result<MatrixClient> {
    let mtx_client =
        matrix_client_builder()?.server_name(session.user_id.server_name()).build().await?;
    mtx_client.restore_login(session).await?;

    Ok(mtx_client)
}

fn matrix_client_builder() -> Result<MatrixClientBuilder, OpenStoreError> {
    let store_config = make_store_config(&*CONFIG_DIR_PATH, None)?;
    Ok(MatrixClient::builder().store_config(store_config))
}
