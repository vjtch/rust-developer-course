use sha256::digest;
use slug::slugify;
use std::{env, io};
use unicode_segmentation::UnicodeSegmentation;

enum Transformation {
    Lowercase,
    Uppercase,
    NoSpaces,
    Slugify,
    Reverse,
    Sha256,
}

impl Transformation {
    fn from_str(string: &str) -> Result<Self, &'static str> {
        match string {
            "lowercase" => Ok(Transformation::Lowercase),
            "uppercase" => Ok(Transformation::Uppercase),
            "no-spaces" => Ok(Transformation::NoSpaces),
            "slugify" => Ok(Transformation::Slugify),
            "reverse" => Ok(Transformation::Reverse),
            "sha256" => Ok(Transformation::Sha256),
            _ => Err("lowercase | uppercase | no-spaces | slugify | reverse | sha256"),
        }
    }

    fn action(&self, string: &str) -> String {
        match self {
            Transformation::Lowercase => string.to_lowercase(),
            Transformation::Uppercase => string.to_uppercase(),
            Transformation::NoSpaces => string.replace(" ", ""),
            Transformation::Slugify => slugify(string),
            Transformation::Reverse => string.graphemes(true).rev().collect(),
            Transformation::Sha256 => String::from(digest(string)),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("There should be exactly one CLI argument!");
        return;
    }

    let transformation =
        Transformation::from_str(&args[1].as_str()).expect("Invalid transformation name");

    let mut input = String::new();

    match io::stdin().read_line(&mut input) {
        Ok(_) => println!(
            "{}",
            transformation.action(
                &input
                    .trim_end_matches("\r\n")
                    .trim_end_matches('\n')
                    .trim_end_matches('\r')
            )
        ),
        Err(error) => println!("Could not read input. (error: {error})"),
    }
}
