use std::ops::ControlFlow;

use anyhow::bail;
use druid::Target;
use matrix_sdk::{
    config::{ClientConfig as MatrixClientConfig, SyncSettings},
    ruma::{
        api::client::r0::{
            filter::{FilterDefinition, LazyLoadOptions, RoomEventFilter, RoomFilter},
            session::login::Response as LoginResponse,
            sync::sync_events::Filter,
        },
        assign, UserId,
    },
    Client as MatrixClient, LoopCtrl, Session,
};
use task_group::TaskGroup;
use tokio::{fs, sync::mpsc::Receiver, task};
use tracing::error;

use crate::{
    config::{self, Config, CONFIG_DIR_PATH},
    data::LoginState,
    ui::actions::{UserData, FINISH_LOGIN},
};

pub mod event_handlers;

// FIXME: Make MatrixClient smaller
#[allow(clippy::large_enum_variant)]
enum State {
    LoggedOut,
    LoggedIn { mtx_client: MatrixClient, session: Session },
}

pub async fn main(
    config: Config,
    mut login_rx: Receiver<LoginState>,
    ui_handle: druid::ExtEventSink,
) {
    if let Err(e) = fs::create_dir_all(&*CONFIG_DIR_PATH).await {
        error!("Failed to create store directory: {}", e);
        return;
    }

    let mut state = if let Some(session) = config.session {
        match restore_login(session.clone()).await {
            Ok(mtx_client) => State::LoggedIn { mtx_client, session },
            Err(e) => {
                error!("{}", e);
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
            State::LoggedIn { mtx_client, session } => {
                logged_in_main(mtx_client, session, ui_handle.clone()).await
            }
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
            };

            let save_res = task::spawn_blocking({
                let session = session.clone();
                move || config::save(&Config { session: Some(session), sync_token: None })
            })
            .await;

            if let Err(e) = save_res {
                error!("Failed to save config: {:?}", e);
            }

            ControlFlow::Continue(State::LoggedIn { mtx_client, session })
        }
        Err(e) => {
            error!("{:?}", e);
            ControlFlow::Continue(State::LoggedOut)
        }
    }
}

async fn logged_in_main(
    mtx_client: MatrixClient,
    session: Session,
    ui_handle: druid::ExtEventSink,
) -> ControlFlow<(), State> {
    let (task_group, _task_manager) = TaskGroup::new();
    let user_data = UserData { mtx_client: mtx_client.clone(), task_group };
    if let Err(e) = ui_handle.submit_command(FINISH_LOGIN, user_data, Target::Auto) {
        error!("{}", e);
    }

    let filter = assign!(FilterDefinition::default(), {
        room: assign!(RoomFilter::default(), {
            state: assign!(RoomEventFilter::default(), {
                lazy_load_options: LazyLoadOptions::Enabled { include_redundant_members: false }
            })
        })
    });
    let filter_id = mtx_client.get_or_upload_filter("jmc_sync", filter).await.unwrap();

    mtx_client
        .register_event_handler_context(ui_handle.clone())
        .register_event_handler(event_handlers::on_room_create)
        .await;

    let sync_settings = SyncSettings::new().filter(Filter::FilterId(&filter_id));
    mtx_client
        .sync_with_callback(sync_settings, |sync_response| async {
            let session = session.clone();
            let save_res = task::spawn_blocking(move || {
                config::save(&Config {
                    session: Some(session),
                    sync_token: Some(sync_response.next_batch),
                })
            })
            .await;

            if let Err(e) = save_res {
                error!("Failed to save config: {:?}", e);
            }

            LoopCtrl::Continue
        })
        .await;

    ControlFlow::Break(())
}

async fn login(login_data: LoginState) -> anyhow::Result<(MatrixClient, LoginResponse)> {
    let user_id = match UserId::try_from(login_data.user_id.as_str()) {
        Ok(id) => id,
        Err(e) => {
            // FIXME: Show error in UI
            bail!("Can't log in due to invalid User ID: {}", e);
        }
    };

    let mtx_client =
        MatrixClient::new_from_user_id_with_config(&user_id, matrix_client_config()).await?;
    let response =
        mtx_client.login(user_id.localpart(), &login_data.password, None, Some("jmc")).await?;

    Ok((mtx_client, response))
}

async fn restore_login(session: Session) -> matrix_sdk::Result<MatrixClient> {
    let mtx_client =
        MatrixClient::new_from_user_id_with_config(&session.user_id, matrix_client_config())
            .await?;
    mtx_client.restore_login(session).await?;

    Ok(mtx_client)
}

fn matrix_client_config() -> MatrixClientConfig {
    MatrixClientConfig::new().store_path(&*CONFIG_DIR_PATH)
}
