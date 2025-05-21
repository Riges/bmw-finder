use config::Condition;

use search::{search, search_by_vss_id};
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
        configuration.models.join(", ")
    );

    // launch search
    let found_vehicles = search(&configuration).await.unwrap();
    println!("Found {} vehicles:", found_vehicles.len());

    // filter cars by expected equipment
    let mut filtered_vehicles = found_vehicles
        .iter()
        .filter(|(_, vehicle)| {
            if configuration
                .filter_equipment
                .clone()
                .is_some_and(|equipment_names| !vehicle.has_equipment_names(equipment_names))
            {
                return false;
            }
            true
        })
        .map(|(_, vehicle)| vehicle.clone())
        .collect::<Vec<Vehicle>>();

    filtered_vehicles.sort_by(sort_by_price);

    println!(
        "{0: <36} | {1: <12} | {2: <8} | {3}",
        "Id", "Price", "Discount", "Link"
    );

    for vehicle in filtered_vehicles {
        // if !vehicle.has_equipment_name_like("Pack Innovation") {
        //     continue;
        // }

        let vehicle = match vehicle.get_price() {
            Some(_) => vehicle.clone(),
            None => search_by_vss_id(&configuration, &vehicle.vss_id)
                .await
                .unwrap_or_else(|_| Some(vehicle.clone()))
                .unwrap(),
        };

        println!(
            "{0: <36} | {1: <12} | {2: <8} | {3}",
            vehicle.vss_id,
            format!("{:.2} €", vehicle.get_price().unwrap_or_default()),
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
    let price_a = vehicle_a.get_price();
    let price_b = vehicle_b.get_price();

    match (price_a, price_b) {
        (Some(p1), Some(p2)) => p1.partial_cmp(&p2).unwrap(),
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, None) => std::cmp::Ordering::Equal,
    }
}
