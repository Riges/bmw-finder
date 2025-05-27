//! Legacy module for text and JSON output in the BMW Finder application.
//! Contains the legacy mode execution logic and associated display functions.

use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::HashMap;

use crate::bmw::search::search;
use crate::config::{Condition, Configuration, OutputMode};
use crate::vehicle::Vehicle;

/// Runs the legacy (text/json) mode of the application.
pub async fn run(configuration: &Configuration) {
    print_header(configuration);
    let found_vehicles = fetch_and_report_vehicles(configuration).await;
    let filtered_vehicles = filter_and_sort_vehicles(&found_vehicles, configuration);
    match configuration.output() {
        OutputMode::Text => print_text_output(&filtered_vehicles),
        OutputMode::Json => print_json_output(&filtered_vehicles),
        _ => unreachable!(),
    }
}

/// Prints the search header for output.
fn print_header(configuration: &Configuration) {
    if configuration.limit.is_some() {
        println!("Limiting results to {}", configuration.limit.unwrap());
    }
    println!(
        "Searching for {} vehicles ({}) ...\n",
        match configuration.condition {
            Condition::New => "new",
            Condition::Used => "used",
        },
        configuration.models().join(", ")
    );
}

/// Fetches vehicles and prints the number found.
async fn fetch_and_report_vehicles(configuration: &Configuration) -> HashMap<uuid::Uuid, Vehicle> {
    let found_vehicles = search(configuration).await.unwrap();
    println!("Found {} vehicles:", found_vehicles.len());
    found_vehicles
}

/// Filters and sorts vehicles according to configuration.
fn filter_and_sort_vehicles<'a>(
    found_vehicles: &'a HashMap<uuid::Uuid, Vehicle>,
    configuration: &Configuration,
) -> Vec<&'a Vehicle> {
    found_vehicles
        .values()
        .filter(|vehicle| vehicle_matches_equipment(vehicle, configuration))
        .sorted_by(|a, b| sort_by_price(a, b))
        .collect()
}

/// Checks if a vehicle matches the expected equipment configuration.
pub fn vehicle_matches_equipment(vehicle: &Vehicle, configuration: &Configuration) -> bool {
    configuration
        .equipment_names()
        .map(|equipment_names| vehicle.has_equipment_names(equipment_names))
        .unwrap_or(true)
}

/// Sorts two vehicles by ascending price, None last.
pub fn sort_by_price(vehicle_a: &Vehicle, vehicle_b: &Vehicle) -> Ordering {
    vehicle_a
        .get_price()
        .partial_cmp(&vehicle_b.get_price())
        .unwrap_or(Ordering::Equal)
}

/// Displays the list of vehicles in text format.
pub fn print_text_output(vehicles: &[&Vehicle]) {
    println!(
        "{0: <36} | {1: <12} | {2: <8} | {3}",
        "Id", "Price", "Discount", "Link"
    );
    for vehicle in vehicles {
        println!(
            "{0: <36} | {1: <12} | {2: <8} | {3}",
            vehicle.vss_id,
            format!("{:.2} €", vehicle.get_price()),
            format!(
                "{:.2} %",
                vehicle.get_discount_percentage().unwrap_or_default()
            ),
            vehicle.get_link()
        );
    }
}

/// Displays the list of vehicles in JSON format.
pub fn print_json_output(vehicles: &[&Vehicle]) {
    let json = serde_json::to_string_pretty(vehicles).unwrap();
    println!("{}", json);
}
