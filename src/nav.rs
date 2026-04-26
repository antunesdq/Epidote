use std::path::PathBuf;

use crate::menu::Nav;

/// Navigation request surfaced by a feature module back to the app shell. The
/// shell is responsible for switching the active tab and calling into the
/// target module's `open`/`select` entry point.
#[derive(Clone)]
pub enum Open {
    /// Switch to a top-level tab without opening any specific item.
    Tab(Nav),
    Vault(PathBuf),
    Task(String),
    Meeting(String),
    Proposal(String),
}
