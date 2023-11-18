use crate::libs::transformation::Transformation;
use std::{env, error::Error};

mod libs;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        let err_text = "There should be exactly one CLI argument!";
        eprintln!("{err_text}");
        return Err(err_text.into());
    }

    let transformation = Transformation::from_str(&args[1].as_str());

    match transformation {
        Ok(t) => {
            let output = t.run();
            match output {
                Ok(o) => {
                    println!("{o}");
                }
                Err(e) => {
                    eprintln!("{e}");
                    return Err(e.into());
                }
            }
        }
        Err(e) => {
            eprintln!("{e}");
            return Err(e.into());
        }
    }

    Ok(())
}
