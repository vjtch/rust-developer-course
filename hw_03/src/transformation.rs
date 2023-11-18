use std::error::Error;
use std::io;

#[allow(unused_imports)]
use crate::convertors::{
    to_csv, to_csv_cli_table_crate, to_lowercase, to_no_spaces, to_reverse, to_sha256, to_slugify,
    to_uppercase,
};

pub enum Transformation {
    Lowercase,
    Uppercase,
    NoSpaces,
    Slugify,
    Reverse,
    Sha256,
    Csv,
}

enum TransformationInput {
    StringInput(String),
    CsvReaderInput(csv::Reader<std::io::Stdin>),
}

impl Transformation {
    pub fn from_str(string: &str) -> Result<Self, &'static str> {
        match string {
            "lowercase" => Ok(Transformation::Lowercase),
            "uppercase" => Ok(Transformation::Uppercase),
            "no-spaces" => Ok(Transformation::NoSpaces),
            "slugify" => Ok(Transformation::Slugify),
            "reverse" => Ok(Transformation::Reverse),
            "sha256" => Ok(Transformation::Sha256),
            "csv" => Ok(Transformation::Csv),
            _ => Err("Invalid transformation name. Valid options: lowercase | uppercase | no-spaces | slugify | reverse | sha256 | csv."),
        }
    }

    pub fn run(&self) -> Result<String, Box<dyn Error>> {
        let mut input = self.get_input()?;
        self.convert(&mut input)
    }

    fn get_input(&self) -> Result<TransformationInput, Box<dyn Error>> {
        match self {
            Transformation::Lowercase
            | Transformation::Uppercase
            | Transformation::NoSpaces
            | Transformation::Slugify
            | Transformation::Reverse
            | Transformation::Sha256 => {
                let mut input = String::new();

                io::stdin().read_line(&mut input)?;

                Ok(TransformationInput::StringInput(
                    input
                        .trim_end_matches("\r\n")
                        .trim_end_matches('\n')
                        .trim_end_matches('\r')
                        .to_string(),
                ))
            }
            Transformation::Csv => {
                let reader = csv::ReaderBuilder::new().from_reader(io::stdin());
                Ok(TransformationInput::CsvReaderInput(reader))
            }
        }
    }

    fn convert(&self, input: &mut TransformationInput) -> Result<String, Box<dyn Error>> {
        match (self, input) {
            (Transformation::Lowercase, TransformationInput::StringInput(input)) => {
                to_lowercase(input)
            }
            (Transformation::Uppercase, TransformationInput::StringInput(input)) => {
                to_uppercase(input)
            }
            (Transformation::NoSpaces, TransformationInput::StringInput(input)) => {
                to_no_spaces(input)
            }
            (Transformation::Slugify, TransformationInput::StringInput(input)) => to_slugify(input),
            (Transformation::Reverse, TransformationInput::StringInput(input)) => to_reverse(input),
            (Transformation::Sha256, TransformationInput::StringInput(input)) => to_sha256(input),
            (Transformation::Csv, TransformationInput::CsvReaderInput(input)) => {
                to_csv(input)
                // to_csv_cli_table_crate(input)
            }
            (_, _) => Err(
                "Internal error: invalid combination of Transformation and TransformationInput."
                    .into(),
            ),
        }
    }
}
