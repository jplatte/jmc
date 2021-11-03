use std::{env, process};

fn main() {
    let native_tls_active = env::var_os("CARGO_FEATURE_NATIVE_TLS").is_some();
    let rustls_tls_active = env::var_os("CARGO_FEATURE_RUSTLS_TLS").is_some();

    if !native_tls_active && !rustls_tls_active {
        eprintln!("error: At least one TLS backend has to be activated.");
        process::exit(1);
    }
}
