#[macro_use]
mod errors;
mod arg_parse;
mod file_parse;
mod gram_parse;

// use backtrace::Backtrace;
// use arg_parse;;

#[macro_use]
extern crate prettytable;

fn main() -> Result<(), ()> {
    let mut cmd_args = arg_parse::CMDArgParse::new();

    cmd_args
        .parse_cmd_args()?
        .parse_file_flag(arg_parse::GRAMMER_FILE_FLAG)?
        .parse_file_flag(arg_parse::BINARY_FILE_FLAG)?
        .parse_offset_flag(arg_parse::OFFSET_FLAG)?;

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
        .format_fields(&parsed_gram)?
        .fill_standard_table(&parsed_gram, cmd_args.struct_offset as usize)?
        .print_table(gram_parse::Tables::Standard);

    Ok(())
}
