use crate::errorh;
use crate::gram_parse;
use hexplay::HexViewBuilder;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

pub enum TableResult {
    OpenFileError,
    OffsetTooLarge,
    GrammerParseFail,
    FeatureNotImplemented,
    Success,
} // This is messy need to sort out repeated error enums!

pub fn print_hex_table(
    gram_file_contents: &String,
    binary_path: &String,
    struct_offset: u64,
) -> Result<TableResult, TableResult> {
    let parsed_gram = match gram_parse::parse_grammer(gram_file_contents) {
        Some(parsed) => parsed,
        None => return Err(TableResult::GrammerParseFail),
    };

    match parsed_gram.metadata.fixed_size {
        true => (),
        false => {
            eprintln!(
                "{} variable size data structures currently not supported",
                errorh::ERROR_START
            );
            return Err(TableResult::FeatureNotImplemented);
        }
    }

    let mut binary_file = match File::open(binary_path) {
        Ok(file) => file,
        Err(error) => {
            eprintln!(
                "{} opening file {}: {}",
                errorh::ERROR_START,
                binary_path,
                error
            );
            return Err(TableResult::OpenFileError);
        }
    };

    match gram_parse::check_file_large_enough(
        struct_offset,
        parsed_gram.metadata.size,
        &mut binary_file,
        &binary_path,
    ) {
        Ok(_) => (),
        _ => return Err(TableResult::OffsetTooLarge),
    }
    let raw_data: Vec<u8> = binary_file
        .bytes()
        .take(parsed_gram.metadata.size as usize)
        .map(|r: Result<u8, _>| r.unwrap())
        .collect(); // change this into match?

    let mut color_vector = Vec::new();

    for (index, field) in parsed_gram.fields.iter().enumerate() {
        let _row = match index % 2 {
            0 => color_vector.append(&mut vec![(
                hexplay::color::green_bold(),
                field.offset as usize..field.offset as usize + field.size,
            )]),
            _ => color_vector.append(&mut vec![(
                hexplay::color::magenta_bold(),
                field.offset as usize..field.offset as usize + field.size,
            )]),
        };
    }

    let hex_view = HexViewBuilder::new(&raw_data[..parsed_gram.metadata.size as usize])
        .address_offset(0x00)
        .row_width(0x10)
        .add_colors(color_vector)
        .finish();

    hex_view.print().unwrap();

    Ok(TableResult::Success)
}
