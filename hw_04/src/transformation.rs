use std::error::Error;
use std::io;
use std::str::FromStr;
use std::thread;

#[allow(unused_imports)]
use super::convertors::{
    remove_new_line, to_csv, to_csv_cli_table_crate, to_lowercase, to_no_spaces, to_reverse,
    to_sha256, to_slugify, to_uppercase,
};
use super::transformation_builder::TransformationBuilder;
use super::transformation_input::TransformationInput;
use super::transformation_type::TransformationType;

pub struct Transformation {
    pub transformation: TransformationType,
    pub input: TransformationInput,
}

impl Transformation {
    pub fn run() -> Result<(), Box<dyn Error>> {
        let (tx, rx) = flume::unbounded::<Transformation>();

        let input_handle = thread::spawn(move || loop {
            // read line from stdin
            let mut buffer = String::new();
            match io::stdin().read_line(&mut buffer) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Could not read line.");
                    eprintln!("{}", e.to_string());
                    continue;
                }
            }

            remove_new_line(&mut buffer);

            // exit command ends this thread
            if buffer.eq("exit") {
                return;
            }

            // split line into command and input parts
            let mut parts = buffer.split(' ');

            let command = match parts.next() {
                Some(s) => s,
                None => {
                    eprintln!("Could not parse <command>.");
                    continue;
                }
            };

            let input = match parts.next() {
                Some(s) => s,
                None => {
                    eprintln!("Could not parse <input>.");
                    continue;
                }
            };

            // if valid command create Transformation object and send it to channel
            // if can not send into channel exit thread
            // if any other error print error message and continue
            match TransformationType::from_str(&command) {
                Ok(t) => {
                    let transformation =
                        TransformationBuilder::new().transformation(t).input(&input);

                    let transformation = match transformation {
                        Ok(t) => t.build(),
                        Err(e) => {
                            eprintln!("Could not create transformation object.");
                            eprintln!("{}", e.to_string());
                            continue;
                        }
                    };

                    match tx.send(transformation) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Could not send transformation object to channel. Exiting.");
                            eprintln!("{}", e.to_string());
                            return;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Command not found.");
                    eprintln!("{}", e.to_string());
                    continue;
                }
            }
        });

        let convert_handle = thread::spawn(move || loop {
            let mut transformation = match rx.recv() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Could not read from channel. Exiting.");
                    eprintln!("{}", e.to_string());
                    return;
                }
            };

            let transformation_result = transformation.convert();
            match Transformation::process_conversion_result(transformation_result) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to process the conversion result.");
                    eprintln!("{}", e.to_string());
                    continue;
                }
            }
        });

        match input_handle.join() {
            Ok(_) => {}
            Err(_) => {
                return Err("Failed to join thread.".into());
            }
        }

        match convert_handle.join() {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to join thread.".into()),
        }
    }

    pub fn convert(&mut self) -> Result<String, Box<dyn Error>> {
        match (&self.transformation, &mut self.input) {
            (TransformationType::Lowercase, TransformationInput::StringInput(input)) => {
                to_lowercase(&input)
            }
            (TransformationType::Uppercase, TransformationInput::StringInput(input)) => {
                to_uppercase(&input)
            }
            (TransformationType::NoSpaces, TransformationInput::StringInput(input)) => {
                to_no_spaces(&input)
            }
            (TransformationType::Slugify, TransformationInput::StringInput(input)) => {
                to_slugify(&input)
            }
            (TransformationType::Reverse, TransformationInput::StringInput(input)) => {
                to_reverse(&input)
            }
            (TransformationType::Sha256, TransformationInput::StringInput(input)) => {
                to_sha256(&input)
            }
            (TransformationType::Csv, TransformationInput::CsvReaderInput(ref mut input)) => {
                to_csv(input)
                // to_csv_cli_table_crate(&mut input)
            }
            (_, _) => Err(
                "Internal error: invalid combination of TransformationType and TransformationInput."
                    .into(),
            ),
        }
    }

    pub fn process_conversion_result(
        transformation: Result<String, Box<dyn Error>>,
    ) -> Result<(), Box<dyn Error>> {
        match transformation {
            Ok(o) => {
                println!("{o}");
            }
            Err(e) => {
                eprintln!("{e}");
                return Err(e.into());
            }
        }

        Ok(())
    }
}
