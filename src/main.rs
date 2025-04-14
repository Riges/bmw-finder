use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Search for new cars
    #[arg(short, long, default_value_t = true)]
    new_car: bool,
}

fn main() {
    let args = Args::parse();

    println!("Hello, world!");

    println!("New car ? {:?}", args.new_car);
}
