use std::error::Error;
use std::fmt::Write;

use cli_table::TableStruct;
use csv::StringRecord;
use sha256::digest;
use slug::slugify;
use unicode_segmentation::UnicodeSegmentation;

pub fn to_lowercase(string: &str) -> Result<String, Box<dyn Error>> {
    Ok(format!("{}", string.to_lowercase()))
}

pub fn to_uppercase(string: &str) -> Result<String, Box<dyn Error>> {
    Ok(format!("{}", string.to_uppercase()))
}

pub fn to_no_spaces(string: &str) -> Result<String, Box<dyn Error>> {
    Ok(format!("{}", string.replace(" ", "")))
}

pub fn to_slugify(string: &str) -> Result<String, Box<dyn Error>> {
    Ok(format!("{}", slugify(string)))
}

pub fn to_reverse(string: &str) -> Result<String, Box<dyn Error>> {
    Ok(format!(
        "{}",
        string.graphemes(true).rev().collect::<String>()
    ))
}

pub fn to_sha256(string: &str) -> Result<String, Box<dyn Error>> {
    Ok(format!("{}", String::from(digest(string))))
}

pub fn to_csv(csv: &mut csv::Reader<std::io::Stdin>) -> Result<String, Box<dyn Error>> {
    // count total columns
    let mut columns = 0;
    let headers = csv.headers()?.clone();
    headers.iter().for_each(|_| columns += 1);

    // count width for every column
    let mut widths = vec![0; columns];

    // headers width
    headers.iter().enumerate().for_each(|(i, cell)| {
        widths[i] = cell.len();
    });

    // records width
    let records: Vec<StringRecord> = csv.records().map(|rec| rec.unwrap()).collect();

    records.iter().for_each(|row| {
        row.iter().enumerate().for_each(|(i, cell)| {
            if widths[i] < cell.len() {
                widths[i] = cell.len();
            }
        })
    });

    // create separation line between rows
    let mut separation_line = String::new();

    widths.iter().for_each(|width| {
        write!(&mut separation_line, "{}", format!("+{:-<width$}", "")).err();
    });
    write!(&mut separation_line, "+")?;

    // output table formatting
    let mut output = String::new();

    // write headers into table
    write_record_into_table(&mut output, &headers, &widths, &separation_line)?;

    // write records into table
    records.iter().for_each(|row| {
        write_record_into_table(&mut output, row, &widths, &separation_line).err();
    });

    // write last line into table
    writeln!(&mut output, "{separation_line}")?;

    // return output table
    Ok(output)
}

fn write_record_into_table(
    output: &mut String,
    record: &StringRecord,
    widths: &Vec<usize>,
    separation_line: &String,
) -> Result<(), Box<dyn Error>> {
    writeln!(output, "{separation_line}")?;

    record.iter().enumerate().for_each(|(i, cell)| {
        write!(output, "|").err();
        write!(
            output,
            "{}",
            format!("{string: <width$}", string = cell, width = widths[i])
        )
        .err();
    });

    writeln!(output, "|")?;

    Ok(())
}

pub fn to_csv_cli_table_crate(
    csv: &mut csv::Reader<std::io::Stdin>,
) -> Result<String, Box<dyn Error>> {
    let table = TableStruct::try_from(csv).unwrap();
    let table = table.display();
    let table = table.unwrap().to_string();
    Ok(String::from(table))
}
