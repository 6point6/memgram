use crate::gram_parse;
use hexplay::HexViewBuilder;
use std::collections::HashMap;

pub fn print_hex_table(
    parsed_gram: &gram_parse::Grammar,
    field_hashmap: &HashMap<String, Vec<u8>>,
    field_offset: usize,
    hex_endianess: bool,
) -> Result<(), ()> {
    let struct_size = parsed_gram.get_struct_size();

    let mut hex_data: Vec<u8> = Vec::new();

    for field in parsed_gram.fields.iter() {
        let mut data: Vec<u8> = field_hashmap
            .get(&field.name)
            .ok_or_else(|| {
                serror!(format!("Could not get value for field: {}", field.name));
            })?
            .clone();

        if hex_endianess {
            if &field.display_format[..] != gram_parse::ASCII_TYPE {
                data.reverse()
            }
        }

        hex_data.append(&mut data);
    }

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

    let hex_view = HexViewBuilder::new(&hex_data[..struct_size])
        .address_offset(field_offset)
        .row_width(0x10)
        .add_colors(color_vector)
        .finish();

    hex_view.print().unwrap();
    println!("");
    Ok(())
}
