use clap::Parser;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Condition {
    New,
    Used,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputMode {
    Ui,
    Text,
    Json,
}

impl std::str::FromStr for OutputMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "ui" => Ok(OutputMode::Ui),
            "text" => Ok(OutputMode::Text),
            "json" => Ok(OutputMode::Json),
            _ => Err(format!("Invalid output mode: {}", s)),
        }
    }
}

type ModelList = Vec<String>;
type EquipmentNameList = Vec<String>;

#[derive(Clone, Debug)]
pub struct Configuration {
    pub condition: Condition,
    pub limit: Option<u32>,
    output: OutputMode,
    models: ModelList,
    equipment_names: Option<EquipmentNameList>,
}

impl Configuration {
    pub fn models(&self) -> &[String] {
        &self.models
    }

    pub fn equipment_names(&self) -> Option<&[String]> {
        self.equipment_names.as_deref()
    }

    pub fn output(&self) -> OutputMode {
        self.output
    }

    pub fn new(args: Args) -> Self {
        Self {
            condition: match args.used {
                true => Condition::Used,
                false => Condition::New,
            },
            models: args.model,
            limit: args.limit,
            equipment_names: args.equipment_names,
            output: match (args.json, args.text) {
                (true, _) => OutputMode::Json,
                (false, true) => OutputMode::Text,
                _ => args.output,
            },
        }
    }
}

pub fn load_config() -> Configuration {
    Configuration::new(Args::parse())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(group(
    clap::ArgGroup::new("output_mode")
        .args(["output", "json", "text"])
        .required(false)
        .multiple(false)
))]
pub struct Args {
    /// Models to search for
    #[arg(long, default_value = "iX2_U10E")]
    model: Vec<String>,

    /// Search for used cars
    #[arg(long)]
    used: bool,

    /// Maximum number of results to fetch
    #[arg(short, long)]
    limit: Option<u32>,

    /// Filter by equipment/pack name on all found cars
    #[arg(long = "equipment-name", value_name = "NAME")]
    equipment_names: Option<Vec<String>>,

    /// Output mode: Ui (default), text, or json
    #[arg(long, value_enum, default_value = "ui", group = "output_mode")]
    output: OutputMode,

    /// Shortcut for --output text
    #[arg(long, group = "output_mode")]
    text: bool,

    /// Shortcut for --output json
    #[arg(long, group = "output_mode")]
    json: bool,
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
                equipment_names: Some(vec![String::from("Pack Innovation")]),
                output: OutputMode::Text,
                text: false,
                json: false,
            };

            let config = Configuration::new(args);

            assert_eq!(config.models, vec![String::from("My Model")]);
            assert_eq!(config.condition, Condition::Used);
            assert_eq!(config.limit, Some(5));
            assert_eq!(
                config.equipment_names,
                Some(vec![String::from("Pack Innovation")])
            );
            assert_eq!(config.output, OutputMode::Text);
        }
    }

    mod args {
        use super::*;
        use clap::error::ErrorKind;

        #[test]
        fn should_error_on_output_and_text() {
            let res = Args::try_parse_from(["test", "--output", "json", "--text"]);
            assert!(res.is_err());
            let err = res.unwrap_err();
            assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
        }

        #[test]
        fn should_error_on_output_and_json() {
            let res = Args::try_parse_from(["test", "--output", "text", "--json"]);
            assert!(res.is_err());
            let err = res.unwrap_err();
            assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
        }

        #[test]
        fn should_error_on_json_and_text() {
            let res = Args::try_parse_from(["test", "--json", "--text"]);
            assert!(res.is_err());
            let err = res.unwrap_err();
            assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
        }

        #[test]
        fn should_be_parsed() {
            let args = Args::parse_from(vec![
                "test",
                "--model",
                "My Model",
                "--used",
                "--limit",
                "5",
                "--equipment-name",
                "Pack Innovation",
                "--equipment-name",
                "Pack M Sport",
                "--model",
                "My second Model",
                "--output",
                "json",
            ]);

            assert_eq!(
                args.model,
                vec![String::from("My Model"), String::from("My second Model")]
            );
            assert_eq!(args.used, true);
            assert_eq!(args.limit, Some(5));
            assert_eq!(
                args.equipment_names,
                Some(vec![
                    String::from("Pack Innovation"),
                    String::from("Pack M Sport")
                ])
            );
            assert_eq!(args.output, OutputMode::Json);
        }

        #[test]
        fn should_use_default_values() {
            let args = Args::parse_from(vec!["test"]);

            assert_eq!(args.model, vec![String::from("iX2_U10E")]);
            assert_eq!(args.used, false);
            assert_eq!(args.limit, None);
            assert_eq!(args.equipment_names, None);
            assert_eq!(args.output, OutputMode::Ui);
        }
    }

    mod output_mode_fromstr {
        use super::*;
        use std::str::FromStr;

        #[test]
        fn parses_ui_case_insensitive() {
            assert_eq!(OutputMode::from_str("ui"), Ok(OutputMode::Ui));
            assert_eq!(OutputMode::from_str("UI"), Ok(OutputMode::Ui));
            assert_eq!(OutputMode::from_str("Ui"), Ok(OutputMode::Ui));
        }

        #[test]
        fn parses_text_case_insensitive() {
            assert_eq!(OutputMode::from_str("text"), Ok(OutputMode::Text));
            assert_eq!(OutputMode::from_str("TEXT"), Ok(OutputMode::Text));
            assert_eq!(OutputMode::from_str("Text"), Ok(OutputMode::Text));
        }

        #[test]
        fn parses_json_case_insensitive() {
            assert_eq!(OutputMode::from_str("json"), Ok(OutputMode::Json));
            assert_eq!(OutputMode::from_str("JSON"), Ok(OutputMode::Json));
            assert_eq!(OutputMode::from_str("Json"), Ok(OutputMode::Json));
        }

        #[test]
        fn returns_err_on_invalid_value() {
            assert!(OutputMode::from_str("foo").is_err());
            assert!(OutputMode::from_str("").is_err());
            assert!(OutputMode::from_str("123").is_err());
        }
    }
}
