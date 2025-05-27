//! Main module for the UI (app) mode of the BMW Finder application.
//! Contains the UI mode execution logic and associated display functions.

use std::collections::HashMap;

use crate::bmw::search::search;
use crate::config::Configuration;
use crate::vehicle::Vehicle;

/// Runs the UI mode of the application.
pub async fn run(configuration: &Configuration) {
    match search(configuration).await {
        Ok(vehicles) => print_ui_output(configuration, &vehicles),
        Err(e) => {
            eprintln!("Error during search: {}", e);
        }
    }
}

/// Displays the search parameters and the number of vehicles found in UI mode.
pub fn print_ui_output(configuration: &Configuration, vehicles: &HashMap<uuid::Uuid, Vehicle>) {
    println!("Search parameters:");
    println!("  Condition: {:?}", configuration.condition);
    println!("  Models: {}", configuration.models().join(", "));
    if let Some(limit) = configuration.limit {
        println!("  Limit: {}", limit);
    }
    if let Some(equipment_names) = configuration.equipment_names() {
        println!("  Equipment names: {}", equipment_names.join(", "));
    }
    println!("Filtered vehicles found: {}", vehicles.len());
}
