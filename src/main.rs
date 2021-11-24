use std::{sync::Arc, time::Duration};

use data::LOGIN_TX;
use druid::{AppLauncher, PlatformError, WindowDesc};
use tokio::{runtime::Runtime, sync::mpsc};

mod bg;
mod config;
mod data;
mod ui;
mod util;

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

    let logged_in = config.session.is_some();
    tokio::spawn(bg::main(config, login_rx, event_sink));
    launcher.launch(data::AppState::new(logged_in))?;

    // After the GUI is closed, shut down all pending async tasks.
    rt.shutdown_timeout(Duration::from_secs(5));
    Ok(())
}
