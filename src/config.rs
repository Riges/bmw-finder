use clap::Parser;

#[derive(Clone, Copy)]
pub enum Condition {
    New,
    Used,
}

pub struct Configuration {
    pub models: Vec<String>,
    pub condition: Condition,
    pub limit: Option<u32>,
    pub filter_equipment: Option<Vec<String>>,
}

pub fn load_config() -> Configuration {
    let args = Args::parse();

    // Load configuration from a file or environment variables
    Configuration {
        models: args.model,
        condition: if args.used {
            Condition::Used
        } else {
            Condition::New
        },
        limit: args.limit,
        filter_equipment: args.filter_equipment,
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Models to search for
    #[arg(long, default_value = "iX2_U10E")]
    model: Vec<String>,

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
