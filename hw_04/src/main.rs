use libs::transformation::Transformation;
use libs::transformation_builder::TransformationBuilder;
use libs::transformation_type::TransformationType;
use std::{env, error::Error, str::FromStr};

mod libs;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            // <>
            Transformation::run()
        }
        2 | 3 => {
            let transformation;

            // <command>
            if args.len() == 2 {
                transformation = TransformationBuilder::new()
                    .transformation(TransformationType::from_str(&args[1].as_str())?)
                    .input_read();
            }
            // <command> <input>
            else {
                transformation = TransformationBuilder::new()
                    .transformation(TransformationType::from_str(&args[1].as_str())?)
                    .input(&args[2].as_str());
            }

            let mut transformation = match transformation {
                Ok(t) => t.build(),
                Err(e) => {
                    eprintln!("Could not create transformation object.");
                    eprintln!("{}", e.to_string());
                    return Ok(());
                }
            };

            let transformation_result = transformation.convert();
            Transformation::process_conversion_result(transformation_result)
        }
        _ => {
            let err_text = "There should be exactly zero, one (<command>) or two (<command> <input>) CLI arguments!";
            eprintln!("{err_text}");
            Err(err_text.into())
        }
    }
}
