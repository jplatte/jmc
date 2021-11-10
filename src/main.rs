use std::{thread, time::Duration};

use druid::{AppLauncher, PlatformError, Selector, Target, WindowDesc};
use tracing::error;

mod data;
mod ui;

pub use data::{AppState, LoginState, UserState};

const FINISH_LOGIN: Selector<()> = Selector::new("finish-login");

fn main() -> Result<(), PlatformError> {
    tracing_subscriber::fmt::init();

    let launcher = AppLauncher::with_window(WindowDesc::new(ui::build_ui()));

    let event_sink = launcher.get_external_handle();
    thread::spawn(move || switch_view_soon(event_sink));

    launcher.launch(AppState::default())
}

fn switch_view_soon(event_sink: druid::ExtEventSink) {
    thread::sleep(Duration::from_secs(3));

    if let Err(e) = event_sink.submit_command(FINISH_LOGIN, (), Target::Auto) {
        error!("{}", e);
    }
}
