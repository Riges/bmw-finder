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

    /// Maximum number of results to return
    #[arg(short, long, default_value_t = 100)]
    count: u32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let new_car = !args.used;

    print!(
        "Searching for {} cars...\n",
        if new_car { "new" } else { "used" }
    );

    // replaced unstable usage with stable format macros
    let cars = search_cars(new_car, args.count).await.unwrap();
    print!("Found {} cars:\n", cars.len());

    let price_sorted_car = sort_by_price(cars);

    println!("{0: <36} | {1: <12} | {2}", "Id", "Price", "Link");
    for car in price_sorted_car {
        println!(
            "{0: <36} | {1: <12} | {2}",
            car.vss_id,
            format!("{:.2} €", car.get_price().unwrap_or_default()),
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
