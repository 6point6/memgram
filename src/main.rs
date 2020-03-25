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

    match cmd_args.run_cmds() {
        Ok(r) => match r {
            arg_parse::CMDOptions::ConvertWrite => {
                let mut c_struct = struct_convert::CStruct::new();

                cmd_args
                    .parse_file_flag(arg_parse::OUTPUT_FILE_FLAG)?
                    .parse_file_flag(arg_parse::CSTRUCT_FILE_FLAG)?;

                c_struct
                    .parse_c_struct(&cmd_args.cstruct_filepath)?
                    .build_toml_string()?
                    .write_toml_file(&cmd_args.output_filepath)?;

                Ok(())
            }
            arg_parse::CMDOptions::ConvertDisplay => {
                let mut c_struct = struct_convert::CStruct::new();

                cmd_args
                    .parse_file_flag(arg_parse::CSTRUCT_FILE_FLAG)?
                    .parse_file_flag(arg_parse::BINARY_FILE_FLAG)?
                    .parse_offset_flag(arg_parse::OFFSET_FLAG)?
                    .parse_endian_flags(arg_parse::FMT_ENDIAN_FLAG, arg_parse::HEX_ENDIAN_FLAG);

                c_struct
                    .parse_c_struct(&cmd_args.cstruct_filepath)?
                    .build_toml_string()?;

                let mut parsed_gram = gram_parse::Grammar::new();

                parsed_gram
                    .pre_parse_toml(&mut c_struct.toml_string)?
                    .parse_toml(&c_struct.toml_string)?;

                let mut table_data = gram_parse::TableData::new();

                table_data
                    .create_field_hashmap(&parsed_gram, &cmd_args)?
                    .format_fields(&parsed_gram, cmd_args.fmt_endian)?
                    .fill_standard_table(&parsed_gram, cmd_args.struct_offset as usize)?
                    .print_table(gram_parse::Tables::Standard);

                hex_display::print_hex_table(
                    &parsed_gram,
                    &table_data.field_hashmap,
                    cmd_args.struct_offset as usize,
                    cmd_args.hex_endian,
                )?;

                Ok(())
            }
            arg_parse::CMDOptions::DisplayNormal => {
                cmd_args
                    .parse_file_flag(arg_parse::GRAMMER_FILE_FLAG)?
                    .parse_file_flag(arg_parse::BINARY_FILE_FLAG)?
                    .parse_offset_flag(arg_parse::OFFSET_FLAG)?
                    .parse_endian_flags(arg_parse::FMT_ENDIAN_FLAG, arg_parse::HEX_ENDIAN_FLAG);

                let mut file_contents = file_parse::FileData::new();

                file_contents.read_grammer(&cmd_args.grammer_filepath)?;

                let mut parsed_gram = gram_parse::Grammar::new();
                parsed_gram
                    .pre_parse_toml(&mut file_contents.grammer_contents)?
                    .parse_toml(&file_contents.grammer_contents)?;

                let mut table_data = gram_parse::TableData::new();

                table_data
                    .fill_description_table(&parsed_gram)
                    .print_table(gram_parse::Tables::Description);

                table_data
                    .create_field_hashmap(&parsed_gram, &cmd_args)?
                    .format_fields(&parsed_gram, cmd_args.fmt_endian)?
                    .fill_standard_table(&parsed_gram, cmd_args.struct_offset as usize)?
                    .print_table(gram_parse::Tables::Standard);

                hex_display::print_hex_table(
                    &parsed_gram,
                    &table_data.field_hashmap,
                    cmd_args.struct_offset as usize,
                    cmd_args.hex_endian,
                )?;

                Ok(())
            }
        },
        Err(_) => Err(()),
    }
}
