use std::path::PathBuf;

/// Navigation request surfaced by a feature module back to the app shell. The
/// shell is responsible for switching the active tab and calling into the
/// target module's `open`/`select` entry point.
pub enum Open {
    Vault(PathBuf),
    Task(String),
}
