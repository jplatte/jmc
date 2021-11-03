use std::sync::Arc;

use druid::{AppLauncher, PlatformError, WindowDesc};

mod ui;

#[derive(Clone, Default, druid::Data, druid::Lens)]
struct AppState {
    user_id: Arc<String>,
    password: Arc<String>,
}

fn main() -> Result<(), PlatformError> {
    tracing_subscriber::fmt::init();

    AppLauncher::with_window(WindowDesc::new(ui::build_ui())).launch(AppState::default())?;
    Ok(())
}
