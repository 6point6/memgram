use crate::gram_parse;
use hexplay::HexViewBuilder;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

pub fn print_hex_table(
    parsed_gram: &gram_parse::Grammer,
    binary_path: &str,
    field_offset: usize,
) -> Result<(), ()> {
    let mut  binary_file = match File::open(binary_path) {
        Ok(file) => file,
        Err(e) => {
            serror!(format!(
                "Error opening file: {}, because {}",
                binary_path, e
            ));
            return Err(())
        }
    };

    match binary_file.seek(SeekFrom::Start(field_offset as u64)) {
        Ok (_) => (),
        Err(e) => {
            serror!(format!("Could not seek to offset: {}, because {}",field_offset,e));
            return Err(())
        }
    }

    let struct_size = parsed_gram.get_struct_size();

    let raw_data: Vec<u8> = binary_file
        .bytes()
        .take(struct_size)
        .map(|r: Result<u8, _>| r.unwrap())
        .collect();

    let mut color_vector = Vec::new();

    let mut color_offset: usize = 0;

    for (index, field) in parsed_gram.fields.iter().enumerate() {
        match index % 2 {
            0 => color_vector.append(&mut vec![(
                hexplay::color::green_bold(),
                color_offset..color_offset + field.size,
            )]),
            _ => color_vector.append(&mut vec![(
                hexplay::color::magenta_bold(),
                color_offset..color_offset + field.size,
            )]),
        };

        color_offset += field.size;
    }

    let hex_view = HexViewBuilder::new(&raw_data[..struct_size])
        .address_offset(field_offset)
        .row_width(0x10)
        .add_colors(color_vector)
        .finish();

    hex_view.print().unwrap();

    Ok(())
}
