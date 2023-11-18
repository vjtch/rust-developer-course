use super::convertors::remove_new_line;
use super::transformation::Transformation;
use super::transformation_input::TransformationInput;
use super::transformation_type::TransformationType;
use std::error::Error;
use std::io;
use std::marker::PhantomData;

pub enum Set {}

pub enum Unset {}

pub struct TransformationBuilder<T, I> {
    transformation: TransformationType,
    input: TransformationInput,
    type_state: PhantomData<(T, I)>,
}

impl TransformationBuilder<Unset, Unset> {
    pub fn new() -> TransformationBuilder<Unset, Unset> {
        TransformationBuilder {
            transformation: TransformationType::Lowercase,
            input: TransformationInput::StringInput(String::from("")),
            type_state: PhantomData,
        }
    }

    pub fn transformation(
        self,
        transformation_type: TransformationType,
    ) -> TransformationBuilder<Set, Unset> {
        TransformationBuilder {
            transformation: transformation_type,
            input: self.input,
            type_state: PhantomData,
        }
    }
}

impl TransformationBuilder<Set, Unset> {
    pub fn input(self, input: &str) -> Result<TransformationBuilder<Set, Set>, Box<dyn Error>> {
        let mut input = input.to_string();

        remove_new_line(&mut input);

        let transformation_input = match self.transformation {
            TransformationType::Csv => {
                println!("{:?}", input);
                let reader = csv::Reader::from_path(input)?;
                TransformationInput::CsvReaderInput(reader)
            }
            _ => TransformationInput::StringInput(input),
        };

        Ok(TransformationBuilder {
            transformation: self.transformation,
            input: transformation_input,
            type_state: PhantomData,
        })
    }

    pub fn input_read(self) -> Result<TransformationBuilder<Set, Set>, Box<dyn Error>> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        remove_new_line(&mut input);

        let transformation_input = match self.transformation {
            TransformationType::Csv => {
                let reader = csv::Reader::from_path(input)?;
                TransformationInput::CsvReaderInput(reader)
            }
            _ => TransformationInput::StringInput(input),
        };

        Ok(TransformationBuilder {
            transformation: self.transformation,
            input: transformation_input,
            type_state: PhantomData,
        })
    }
}

impl TransformationBuilder<Set, Set> {
    pub fn build(self) -> Transformation {
        Transformation {
            transformation: self.transformation,
            input: self.input,
        }
    }
}
