//! Entry point for the BMW Finder application.
//! Routes to legacy (text/json) or app (UI) mode depending on configuration.

mod app;
mod config;
mod legacy;
mod search;
mod vehicle;

use config::{OutputMode, load_config};

#[tokio::main]
async fn main() {
    let configuration = load_config();
    match configuration.output() {
        OutputMode::Text | OutputMode::Json => legacy::run(&configuration).await,
        OutputMode::Ui => app::run(&configuration).await,
    }
}
