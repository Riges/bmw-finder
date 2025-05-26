use config::Condition;

use search::search;
use vehicle::Vehicle;

mod config;
mod search;
mod vehicle;

#[tokio::main]
async fn main() {
    let configuration = config::load_config();

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

    // filter cars by expected equipment name
    let mut filtered_vehicles: Vec<&Vehicle> = found_vehicles
        .values()
        .filter(|vehicle| {
            configuration
                .equipment_names()
                .map(|equipment_names| vehicle.has_equipment_names(equipment_names))
                .unwrap_or(true)
        })
        .collect();

    filtered_vehicles.sort_by(|a, b| sort_by_price(a, b));

    println!(
        "{0: <36} | {1: <12} | {2: <8} | {3}",
        "Id", "Price", "Discount", "Link"
    );

    for vehicle in filtered_vehicles {
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

// Order by price, none as last
fn sort_by_price(vehicle_a: &Vehicle, vehicle_b: &Vehicle) -> std::cmp::Ordering {
    vehicle_a
        .get_price()
        .partial_cmp(&vehicle_b.get_price())
        .unwrap_or(std::cmp::Ordering::Equal)
}
