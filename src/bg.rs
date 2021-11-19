use std::ops::ControlFlow;

use anyhow::bail;
use druid::Target;
use matrix_sdk::{
    ruma::{api::client::r0::session::login::Response as LoginResponse, UserId},
    Client as MatrixClient, Session,
};
use task_group::TaskGroup;
use tokio::{sync::mpsc::Receiver, task};
use tracing::error;

use crate::{
    config::{self, Config},
    data::LoginState,
    ui::actions::{UserData, FINISH_LOGIN},
};

// FIXME: Make MatrixClient smaller
#[allow(clippy::large_enum_variant)]
enum State {
    LoggedOut,
    LoggedIn { mtx_client: MatrixClient },
}

pub async fn main(
    config: Config,
    mut login_rx: Receiver<LoginState>,
    event_sink: druid::ExtEventSink,
) {
    let mut state = if let Some(session) = config.session {
        match restore_login(session).await {
            Ok(mtx_client) => State::LoggedIn { mtx_client },
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
            State::LoggedIn { mtx_client } => logged_in_main(mtx_client, &event_sink).await,
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
            let save_res = task::spawn_blocking(move || {
                config::save(&Config {
                    session: Some(Session {
                        access_token: login_response.access_token,
                        user_id: login_response.user_id,
                        device_id: login_response.device_id,
                    }),
                    // ..config.clone()
                })
            })
            .await;

            if let Err(e) = save_res {
                error!("Failed to save config: {:?}", e);
            }

            ControlFlow::Continue(State::LoggedIn { mtx_client })
        }
        Err(e) => {
            error!("{:?}", e);
            ControlFlow::Continue(State::LoggedOut)
        }
    }
}

async fn logged_in_main(
    mtx_client: MatrixClient,
    event_sink: &druid::ExtEventSink,
) -> ControlFlow<(), State> {
    let (task_group, _task_manager) = TaskGroup::new();
    let user_data = UserData { mtx_client, task_group };
    if let Err(e) = event_sink.submit_command(FINISH_LOGIN, user_data, Target::Auto) {
        error!("{}", e);
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

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

    let mtx_client = MatrixClient::new_from_user_id(&user_id).await?;
    let response =
        mtx_client.login(user_id.localpart(), &login_data.password, None, Some("jmc")).await?;

    Ok((mtx_client, response))
}

async fn restore_login(session: Session) -> matrix_sdk::Result<MatrixClient> {
    let mtx_client = MatrixClient::new_from_user_id(&session.user_id).await?;
    mtx_client.restore_login(session).await?;

    Ok(mtx_client)
}
