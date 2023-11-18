use std::str::FromStr;

pub enum TransformationType {
    Lowercase,
    Uppercase,
    NoSpaces,
    Slugify,
    Reverse,
    Sha256,
    Csv,
}

impl FromStr for TransformationType {
    type Err = &'static str;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "lowercase" => Ok(TransformationType::Lowercase),
            "uppercase" => Ok(TransformationType::Uppercase),
            "no-spaces" => Ok(TransformationType::NoSpaces),
            "slugify" => Ok(TransformationType::Slugify),
            "reverse" => Ok(TransformationType::Reverse),
            "sha256" => Ok(TransformationType::Sha256),
            "csv" => Ok(TransformationType::Csv),
            _ => Err("Invalid transformation name. Valid options: lowercase | uppercase | no-spaces | slugify | reverse | sha256 | csv."),
        }
    }
}
