use config::Condition;
use std::collections::HashMap;
use uuid::Uuid;

use search::search_cars;
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
        "Searching for {} cars ({}) ...\n",
        match configuration.condition {
            Condition::New => "new",
            Condition::Used => "used",
        },
        configuration.models.join(", ")
    );

    let cars = search_cars(&configuration).await.unwrap();
    print!("Found {} cars:\n", cars.len());

    let price_sorted_car = sort_by_price(cars);

    println!(
        "{0: <36} | {1: <12} | {2: <8} | {3}",
        "Id", "Price", "Discount", "Link"
    );

    for car in price_sorted_car {
        // filter by equipment name
        if configuration
            .filter_equipment
            .clone()
            .is_some_and(|equipment_names| !has_expected_equipment(car.clone(), equipment_names))
        {
            continue;
        }

        // if !car.has_equipment_name_like("Pack Innovation") {
        //     continue;
        // }

        println!(
            "{0: <36} | {1: <12} | {2: <8} | {3}",
            car.vss_id,
            format!("{:.2} €", car.get_price().unwrap_or_default()),
            format!("{:.2} %", car.get_discount_percentage().unwrap_or_default()),
            car.get_link()
        );
    }
}

// Order by price, none as last
fn sort_by_price(cars: HashMap<Uuid, Vehicle>) -> Vec<Vehicle> {
    let mut vehicles: Vec<Vehicle> = cars.values().cloned().collect();

    vehicles.sort_by(|a, b| {
        let a_price = a.get_price();
        let b_price = b.get_price();

        match (a_price, b_price) {
            (Some(a), Some(b)) => a.partial_cmp(&b).unwrap(),
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });

    vehicles
}

fn has_expected_equipment(car: Vehicle, equipment_names: Vec<String>) -> bool {
    if equipment_names.is_empty() {
        return true;
    }

    equipment_names
        .iter()
        .all(|equipment_name| car.has_equipment_name_like(equipment_name))
}
