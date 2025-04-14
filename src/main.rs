use clap::Parser;

mod search;
use search::search_cars;

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

    println!("{0: <36} | {1: <12} | {2}", "Id", "Price", "Link");
    for car in cars {
        println!(
            "{0: <36} | {1: <12} | {2}",
            car.vss_id,
            format!("{:.2} €", car.get_price().unwrap_or_default()),
            car.get_link()
        );
    }
}
