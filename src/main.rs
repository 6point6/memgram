#[macro_use]
mod errors;
mod arg_parse;
mod file_parse;
mod gram_parse;
mod hex_display;
mod struct_convert;

#[macro_use]
extern crate prettytable;

fn main() {
    if let Err(()) = run() {}
}

fn run() -> Result<(), ()> {
    let mut cmd_args = arg_parse::CMDArgParse::new();

    cmd_args.parse_cmd_args()?;

    match cmd_args.check_convert_flags() {
        Ok(r) => {
            if r.is_some() {
                cmd_args.parse_file_flag(arg_parse::CSTRUCT_FILE_FLAG)?;

                let mut c_struct = struct_convert::CStruct::new();

                c_struct
                    .parse_c_struct(&cmd_args.cstruct_filepath)?
                    .build_toml_string()?
                    .write_toml_file(&cmd_args.output_filepath)?;

                return Ok(());
            }
        }
        Err(_) => return Err(()),
    }

    cmd_args
        .parse_file_flag(arg_parse::GRAMMER_FILE_FLAG)?
        .parse_file_flag(arg_parse::BINARY_FILE_FLAG)?
        .parse_offset_flag(arg_parse::OFFSET_FLAG)?
        .parse_endian_flag(arg_parse::ENDIAN_FLAG);

    let mut file_contents = file_parse::FileData::new();

    file_contents.read_grammer(&cmd_args.grammer_filepath)?;

    let mut parsed_gram = gram_parse::Grammer::new();
    parsed_gram
        .pre_parse_toml(&mut file_contents.grammer_contents)?
        .parse_toml(&file_contents.grammer_contents)?;

    let mut table_data = gram_parse::TableData::new();

    table_data
        .fill_description_table(&parsed_gram)
        .print_table(gram_parse::Tables::Description);

    table_data
        .create_field_hashmap(&parsed_gram, &cmd_args)?
        .format_fields(&parsed_gram, cmd_args.reverse_endian)?
        .fill_standard_table(&parsed_gram, cmd_args.struct_offset as usize)?
        .print_table(gram_parse::Tables::Standard);

    hex_display::print_hex_table(
        &parsed_gram,
        &cmd_args.binary_filepath,
        cmd_args.struct_offset as usize,
    )?;

    Ok(())
}
