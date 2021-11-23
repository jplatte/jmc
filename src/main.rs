use std::{sync::Arc, time::Duration};

use druid::{AppLauncher, PlatformError, WindowDesc};
use tokio::{runtime::Runtime, sync::mpsc};

mod bg;
mod config;
mod data;
mod ui;
mod util;

use data::{AppState, LOGIN_TX};

fn main() -> Result<(), PlatformError> {
    tracing_subscriber::fmt::init();
    let config = config::load()?;

    // Create a runtime and have it register its threadlocal magic.
    // The main thread will not block on a future like in most async
    // applications because it will run the GUI instead, which is not async.
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    let (login_tx, login_rx) = mpsc::channel(1);

    let login_tx = Arc::new(login_tx);
    let launcher = AppLauncher::with_window(WindowDesc::new(ui::build_ui()))
        .configure_env(move |env, _state| env.set(LOGIN_TX, login_tx.clone()));
    let event_sink = launcher.get_external_handle();

    let initial_state = if config.session.is_some() {
        AppState::LoggingIn
    } else {
        AppState::Login(Default::default())
    };

    tokio::spawn(bg::main(config, login_rx, event_sink));
    launcher.launch(initial_state)?;

    // After the GUI is closed, shut down all pending async tasks.
    rt.shutdown_timeout(Duration::from_secs(5));
    Ok(())
}
