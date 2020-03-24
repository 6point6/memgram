use hexplay::HexViewBuilder;
use std::fs::File;
use std::io::prelude::*;
use crate::gram_parse;

pub fn print_hex_table(
    parsed_gram: &mut gram_parse::Grammer,
    binary_path: &str,
    mut field_offset: usize,
) -> Result<(), ()> {

    let binary_file = match File::open(binary_path) {
        Ok(file) => file,
        Err(e) => {
            serror!(format!("Error opening file: {}, because {}",binary_path,e));
            return Err(())
        }
    };

    let struct_size = parsed_gram.get_struct_size();

    let raw_data: Vec<u8> = binary_file
        .bytes()
        .take(struct_size)
        .map(|r: Result<u8, _>| r.unwrap())
        .collect(); 

    let mut color_vector = Vec::new();

    for (index, field) in parsed_gram.fields.iter().enumerate() {
        match index % 2 {
            0 => color_vector.append(&mut vec![(
                hexplay::color::green_bold(),
                field_offset..field_offset+ field.size,
            )]),
            _ => color_vector.append(&mut vec![(
                hexplay::color::magenta_bold(),
                field_offset..field_offset+ field.size,
            )]),
        };

        field_offset += field.size;
    }

    let hex_view = HexViewBuilder::new(&raw_data[..struct_size])
        .address_offset(0x00)
        .row_width(0x10)
        .add_colors(color_vector)
        .finish();

    hex_view.print().unwrap();

    Ok(())
}
