use config::Condition;

use search::{search_cars, search_cars_by_vss_id};
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

    let found_cars = search_cars(&configuration).await.unwrap();
    print!("Found {} cars:\n", found_cars.len());

    // filter cars by expected equipment
    let mut filtered_cars = found_cars
        .iter()
        .filter(|(_, car)| {
            if configuration
                .filter_equipment
                .clone()
                .is_some_and(|equipment_names| !car.has_equipment_names(equipment_names))
            {
                return false;
            }
            true
        })
        .map(|(_, car)| car.clone())
        .collect::<Vec<Vehicle>>();

    filtered_cars.sort_by(sort_by_price);

    println!(
        "{0: <36} | {1: <12} | {2: <8} | {3}",
        "Id", "Price", "Discount", "Link"
    );

    for car in filtered_cars {
        // if !car.has_equipment_name_like("Pack Innovation") {
        //     continue;
        // }

        let car = match car.get_price() {
            Some(_) => car.clone(),
            None => search_cars_by_vss_id(&configuration, &car.vss_id.to_string().as_str())
                .await
                .unwrap_or_else(|_| Some(car.clone()))
                .unwrap(),
        };

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
fn sort_by_price(a: &Vehicle, b: &Vehicle) -> std::cmp::Ordering {
    let a_price = a.get_price();
    let b_price = b.get_price();

    match (a_price, b_price) {
        (Some(a), Some(b)) => a.partial_cmp(&b).unwrap(),
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, None) => std::cmp::Ordering::Equal,
    }
}
