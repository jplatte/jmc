use druid::{AppLauncher, PlatformError, Selector, Target, WindowDesc};
use tokio::time::{self, Duration};
use tracing::error;

mod data;
mod ui;

pub use data::{AppState, LoginState, UserState};

const FINISH_LOGIN: Selector<()> = Selector::new("finish-login");

#[tokio::main]
async fn main() -> Result<(), PlatformError> {
    tracing_subscriber::fmt::init();

    let launcher = AppLauncher::with_window(WindowDesc::new(ui::build_ui()));

    let event_sink = launcher.get_external_handle();
    tokio::spawn(switch_view_soon(event_sink));

    launcher.launch(AppState::default())
}

async fn switch_view_soon(event_sink: druid::ExtEventSink) {
    time::sleep(Duration::from_secs(3)).await;

    if let Err(e) = event_sink.submit_command(FINISH_LOGIN, (), Target::Auto) {
        error!("{}", e);
    }
}
