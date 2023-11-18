use std::fs::File;

pub enum TransformationInput {
    StringInput(String),
    CsvReaderInput(csv::Reader<File>),
}
