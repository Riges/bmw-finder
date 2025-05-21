use clap::Parser;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Condition {
    New,
    Used,
}

#[derive(Clone)]
pub struct Configuration {
    pub models: Vec<String>,
    pub condition: Condition,
    pub limit: Option<u32>,
    pub filter_equipment: Option<Vec<String>>,
}

impl Configuration {
    fn new(args: Args) -> Self {
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
}

pub fn load_config() -> Configuration {
    Configuration::new(Args::parse())
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

#[cfg(test)]
mod tests {
    use super::*;

    mod configuration {
        use super::*;

        #[test]
        fn should_use_args_to_create_configuration() {
            let args = Args {
                model: vec![String::from("My Model")],
                used: true,
                limit: Some(5),
                filter_equipment: Some(vec![String::from("Pack Innovation")]),
            };

            let config = Configuration::new(args);

            assert_eq!(config.models, vec![String::from("My Model")]);
            assert_eq!(config.condition, Condition::Used);
            assert_eq!(config.limit, Some(5));
            assert_eq!(
                config.filter_equipment,
                Some(vec![String::from("Pack Innovation")])
            );
        }
    }

    mod args {
        use super::*;

        #[test]
        fn should_be_parsed() {
            let args = Args::parse_from(vec![
                "test",
                "--model",
                "My Model",
                "--used",
                "--limit",
                "5",
                "--filter-equipment",
                "Pack Innovation",
                "--filter-equipment",
                "Pack M Sport",
                "--model",
                "My second Model",
            ]);

            assert_eq!(
                args.model,
                vec![String::from("My Model"), String::from("My second Model")]
            );
            assert_eq!(args.used, true);
            assert_eq!(args.limit, Some(5));
            assert_eq!(
                args.filter_equipment,
                Some(vec![
                    String::from("Pack Innovation"),
                    String::from("Pack M Sport")
                ])
            );
        }

        #[test]
        fn should_use_default_values() {
            let args = Args::parse_from(vec!["test"]);

            assert_eq!(args.model, vec![String::from("iX2_U10E")]);
            assert_eq!(args.used, false);
            assert_eq!(args.limit, None);
            assert_eq!(args.filter_equipment, None);
        }
    }
}
