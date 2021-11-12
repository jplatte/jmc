use druid::{AppLauncher, PlatformError, Selector, Target, WindowDesc};
use tokio::{
    runtime::Runtime,
    time::{self, Duration},
};
use tracing::error;

mod data;
mod ui;

pub use data::{AppState, LoginState, UserState};

const FINISH_LOGIN: Selector<()> = Selector::new("finish-login");

fn main() -> Result<(), PlatformError> {
    tracing_subscriber::fmt::init();

    // Create a runtime and have it register its threadlocal magic.
    // The main thread will not block on a future like in most async
    // applications because it will run the GUI instead, which is not async.
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    let launcher = AppLauncher::with_window(WindowDesc::new(ui::build_ui()));

    let event_sink = launcher.get_external_handle();
    tokio::spawn(switch_view_soon(event_sink));

    launcher.launch(AppState::default())?;

    // After the GUI is closed, shut down all pending async tasks.
    rt.shutdown_timeout(Duration::from_secs(5));
    Ok(())
}

async fn switch_view_soon(event_sink: druid::ExtEventSink) {
    time::sleep(Duration::from_secs(3)).await;

    if let Err(e) = event_sink.submit_command(FINISH_LOGIN, (), Target::Auto) {
        error!("{}", e);
    }
}
