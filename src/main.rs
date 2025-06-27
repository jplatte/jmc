use std::time::Duration;

use tokio::runtime::Runtime;

mod config;

fn main() {
    tracing_subscriber::fmt::init();
    let _config = config::load().unwrap();

    // Create a runtime and have it register its threadlocal magic.
    // The main thread will not block on a future like in most async
    // applications because it will run the GUI instead, which is not async.
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    // launch the ui?

    // After the GUI is closed, shut down all pending async tasks.
    rt.shutdown_timeout(Duration::from_secs(5));
}
