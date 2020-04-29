#[macro_use]
mod errors;
mod arg_parse;
mod gram_parse;
mod hex_display;
mod struct_convert;
use std::fs;

#[macro_use]
extern crate prettytable;

fn main() {
    if let Err(()) = run() {}
}

fn run() -> Result<(), ()> {
    let mut cmd_args = arg_parse::CMDArgParse::new();

    cmd_args
        .parse_cmd_args()?
        .parse_help_flag(arg_parse::HELP_FLAG);

    if cmd_args.help_flag {
        errors::usage();
        return Ok(());
    }

    match cmd_args.run_cmds() {
        Ok(r) => match r {
            arg_parse::RunOptions::CStructConvertWrite => {
                let mut c_struct = struct_convert::CStruct::new();

                cmd_args
                    .parse_file_arg(arg_parse::OUTPUT_FILE_FLAG)?
                    .parse_file_arg(arg_parse::CSTRUCT_FILE_FLAG)?;

                c_struct
                    .parse_c_struct(&cmd_args.cstruct_filepath)?
                    .build_grammar_contents()?
                    .write_grammar_file(&cmd_args.output_filepath)?;

                Ok(())
            }
            arg_parse::RunOptions::CStructConvertDisplay => {
                let mut c_struct = struct_convert::CStruct::new();

                cmd_args
                    .parse_file_arg(arg_parse::CSTRUCT_FILE_FLAG)?
                    .parse_file_arg(arg_parse::BINARY_FILE_FLAG)?
                    .parse_offset_flag(arg_parse::STRUCT_OFFSET_FLAG)?
                    .parse_bool_flags(
                        arg_parse::FMT_ENDIAN_FLAG,
                        arg_parse::HEX_ENDIAN_FLAG,
                        arg_parse::DESCRIPTION_FLAG,
                    );

                c_struct
                    .parse_c_struct(&cmd_args.cstruct_filepath)?
                    .build_grammar_contents()?;

                let mut parsed_gram = gram_parse::Grammar::new();

                parsed_gram.parse_toml(&c_struct.grammar_contents)?;

                let mut table_data = gram_parse::TableData::new();

                table_data
                    .create_field_hashmap(&mut parsed_gram, &cmd_args)?
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
            arg_parse::RunOptions::DisplayNormal => {
                cmd_args
                    .parse_file_arg(arg_parse::GRAMMER_FILE_FLAG)?
                    .parse_file_arg(arg_parse::BINARY_FILE_FLAG)?
                    .parse_offset_flag(arg_parse::STRUCT_OFFSET_FLAG)?
                    .parse_bool_flags(
                        arg_parse::FMT_ENDIAN_FLAG,
                        arg_parse::HEX_ENDIAN_FLAG,
                        arg_parse::DESCRIPTION_FLAG,
                    );

                let file_contents =
                    fs::read_to_string(&cmd_args.grammar_filepath).map_err(|e| {
                        serror!(format!(
                            "Error opening file: {}, because:{}",
                            &cmd_args.grammar_filepath, e
                        ))
                    })?;

                let mut parsed_gram = gram_parse::Grammar::new();

                parsed_gram.parse_toml(&file_contents)?.post_parse_toml()?;

                let mut table_data = gram_parse::TableData::new();

                if cmd_args.description {
                    table_data
                        .fill_description_table(&parsed_gram)
                        .print_table(gram_parse::Tables::Description)
                }

                table_data
                    .create_field_hashmap(&mut parsed_gram, &cmd_args)?
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
