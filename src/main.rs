use std::time::Duration;

use druid::{AppLauncher, PlatformError, WindowDesc};
use tokio::{runtime::Runtime, sync::mpsc};

mod bg;
mod data;
mod ui;

fn main() -> Result<(), PlatformError> {
    tracing_subscriber::fmt::init();

    // Create a runtime and have it register its threadlocal magic.
    // The main thread will not block on a future like in most async
    // applications because it will run the GUI instead, which is not async.
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    let launcher = AppLauncher::with_window(WindowDesc::new(ui::build_ui()));

    let (login_tx, login_rx) = mpsc::channel(1);
    let event_sink = launcher.get_external_handle();

    tokio::spawn(bg::main(login_rx, event_sink));
    launcher.launch(data::AppState::new(login_tx))?;

    // After the GUI is closed, shut down all pending async tasks.
    rt.shutdown_timeout(Duration::from_secs(5));
    Ok(())
}
