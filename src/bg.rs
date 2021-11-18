use anyhow::bail;
use druid::Target;
use matrix_sdk::{ruma::UserId, Client as MatrixClient};
use task_group::TaskGroup;
use tokio::sync::mpsc::Receiver;
use tracing::error;

use crate::{
    data::LoginState,
    ui::actions::{UserData, FINISH_LOGIN},
};

pub async fn main(mut login_rx: Receiver<LoginState>, event_sink: druid::ExtEventSink) {
    #[allow(clippy::while_let_loop)]
    loop {
        let login_state = match login_rx.recv().await {
            Some(s) => s,
            None => break,
        };

        let mtx_client = match login(login_state).await {
            Ok(c) => c,
            Err(e) => {
                error!("{}", e);
                continue;
            }
        };

        let (task_group, _task_manager) = TaskGroup::new();

        let user_data = UserData { mtx_client, task_group };
        if let Err(e) = event_sink.submit_command(FINISH_LOGIN, user_data, Target::Auto) {
            error!("{}", e);
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        break;
    }
}

async fn login(login_data: LoginState) -> anyhow::Result<MatrixClient> {
    let user_id = match UserId::try_from(login_data.user_id.as_str()) {
        Ok(id) => id,
        Err(e) => {
            // FIXME: Show error in UI
            bail!("Can't log in due to invalid User ID: {}", e);
        }
    };

    let mtx_client = MatrixClient::new_from_user_id(&user_id).await?;
    mtx_client.login(user_id.localpart(), &login_data.password, None, Some("jmc")).await?;

    Ok(mtx_client)
}
