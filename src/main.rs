use itertools::Itertools;
use std::collections::HashMap;

use config::{Condition, Configuration, OutputMode, load_config};
use search::search;
use vehicle::Vehicle;

mod config;
mod search;
mod vehicle;

#[tokio::main]
async fn main() {
    let configuration = load_config();

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

    // launch search
    let found_vehicles = search(&configuration).await.unwrap();
    println!("Found {} vehicles:", found_vehicles.len());

    match configuration.output() {
        OutputMode::Text | OutputMode::Json => {
            // filter cars by expected equipment name
            let filtered_vehicles: Vec<&Vehicle> = found_vehicles
                .values()
                .filter(|vehicle| vehicle_matches_equipment(vehicle, &configuration))
                .sorted_by(|a, b| sort_by_price(a, b))
                .collect();
            match configuration.output() {
                OutputMode::Text => print_text_output(&filtered_vehicles),
                OutputMode::Json => print_json_output(&filtered_vehicles),
                _ => unreachable!(),
            }
        }
        OutputMode::Ui => {
            print_ui_output(&configuration, &found_vehicles);
        }
    }
}

fn print_json_output(vehicles: &[&Vehicle]) {
    let json = serde_json::to_string_pretty(vehicles).unwrap();
    println!("{}", json);
}

// Order by price, none as last
fn sort_by_price(vehicle_a: &Vehicle, vehicle_b: &Vehicle) -> std::cmp::Ordering {
    vehicle_a
        .get_price()
        .partial_cmp(&vehicle_b.get_price())
        .unwrap_or(std::cmp::Ordering::Equal)
}

fn print_text_output(vehicles: &[&Vehicle]) {
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

fn vehicle_matches_equipment(vehicle: &Vehicle, configuration: &Configuration) -> bool {
    configuration
        .equipment_names()
        .map(|equipment_names| vehicle.has_equipment_names(equipment_names))
        .unwrap_or(true)
}

fn print_ui_output(configuration: &Configuration, vehicles: &HashMap<uuid::Uuid, Vehicle>) {
    println!("Search parameters:");
    println!("  Condition: {:?}", configuration.condition);
    println!("  Models: {}", configuration.models().join(", "));
    if let Some(limit) = configuration.limit {
        println!("  Limit: {}", limit);
    }
    println!("Filtered vehicles found: {}", vehicles.len());
}
