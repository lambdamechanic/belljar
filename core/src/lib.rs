//! par-core: internal library for par-rs

/// Returns the semantic version of the core crate.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Placeholder for session options; will expand with compose settings.
#[derive(Debug, Default, Clone)]
pub struct SessionOptions {
    pub services: Vec<String>,
    pub keep: bool,
    pub parallel: Option<usize>,
}

/// Placeholder for initializing a session; will manage docker-compose lifecycle.
pub fn init_session(_opts: &SessionOptions) -> Result<(), &'static str> {
    // TODO: implement per-session docker compose orchestration
    Ok(())
}

