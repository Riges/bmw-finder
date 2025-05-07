use std::collections::HashMap;

use clap::Parser;

mod search;
use search::search_cars;
use uuid::Uuid;
use vehicle::Vehicle;

mod vehicle;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Search for used cars
    #[arg(long)]
    used: bool,

    /// Maximum number of results to fetch
    #[arg(short, long)]
    limit: Option<u32>,

    /// Equipment filter on all found cars
    #[arg(long)]
    filter_equipment: Option<Vec<String>>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let new_car = !args.used;

    if args.limit.is_some() {
        println!("Limiting results to {}", args.limit.unwrap());
    }

    print!(
        "Searching for {} cars...\n",
        if new_car { "new" } else { "used" }
    );

    let cars = search_cars(new_car, args.limit).await.unwrap();
    print!("Found {} cars:\n", cars.len());

    let price_sorted_car = sort_by_price(cars);

    println!(
        "{0: <36} | {1: <12} | {2: <8} | {3}",
        "Id", "Price", "Discount", "Link"
    );

    for car in price_sorted_car {
        // filter by equipment name
        if let Some(filter_equipment) = &args.filter_equipment {
            if !filter_equipment
                .iter()
                .any(|name| car.has_equipment_name_like(name))
            {
                continue;
            }
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

// Order by price, none last
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
