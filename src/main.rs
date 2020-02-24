mod arg_parse;
mod errorh;
mod gram_parse;
mod hexfmt;
use std::collections::HashMap;
use std::env;
use std::fs;
#[macro_use]
extern crate prettytable;

pub const USAGE: &str = "usage: blah blah";

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut cmdline_hashmap: HashMap<String, Option<String>> = HashMap::new();

    match arg_parse::parse_cmdline_args(args, &mut cmdline_hashmap) {
        Ok(arg_parse::ArgParseResult::Success) => (),
        _ => {
            eprintln!("{}",USAGE);
            return
        },
    };

    match gram_parse::check_mandatory_cmds(&mut cmdline_hashmap) {
        Ok(gram_parse::ParseResult::Success) => (),
        _ => {
            eprintln!("{}",USAGE);
            return
        },
    };

    let gram_file_contents: String =
        match cmdline_hashmap.get(gram_parse::GRAMMER_FILE_FLAG).unwrap() {
            Some(path) => match fs::read_to_string(path) {
                Ok(file) => file,
                Err(error) => panic!("{} Error opening file: {}", errorh::ERROR_START, error),
            },
            None => panic!(
                "{} No value for {} flag",
                errorh::ERROR_START,
                gram_parse::GRAMMER_FILE_FLAG
            ),
        };

    let binary_file_path: &String = match cmdline_hashmap.get(gram_parse::BINARY_FILE_FLAG).unwrap()
    {
        Some(path) => path,
        None => panic!(
            "{} No value for {} flag",
            errorh::ERROR_START,
            gram_parse::BINARY_FILE_FLAG
        ),
    };

    let struct_offset: u64 = match cmdline_hashmap.get(gram_parse::OFFSET_FLAG).unwrap() {
        Some(string_offset) => match string_offset.parse::<u64>() {
            Ok(u64_offset) => u64_offset,
            Err(error) => panic!(
                "{} Invalid offset for {} flag: {}",
                errorh::ERROR_START,
                string_offset,
                error
            ),
        },
        None => panic!(
            "{} No value for {} flag",
            errorh::ERROR_START,
            gram_parse::OFFSET_FLAG
        ),
    };

    match gram_parse::print_hex_gram(&gram_file_contents, &binary_file_path, struct_offset) {
        Ok(_) => (),
        Err(_) => panic!(
            "{} Failed to print grammer table for {}",
            errorh::ERROR_START,
            binary_file_path
        ),
    }

    match hexfmt::print_hex_table(&gram_file_contents, &binary_file_path, struct_offset) {
        Ok(_) => (),
        Err(_) => panic!(
            "{} Failed to print hex table for {}",
            errorh::ERROR_START,
            binary_file_path
        ),
    }
}
