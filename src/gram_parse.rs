use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use toml::Value;
use serde::{Deserialize};

#[derive(PartialEq)]
pub enum CMDParseResult {
    FileNotFound,
    FilePathNotSpecified,
    FileFound,
    OffsetFlagExists,
    OffsetNotSpecified,
    OffsetFlagNotSpecified,
    Success,
}

#[derive(Deserialize, Debug)]
pub struct Grammer {
    metadata: GrammerMetadata,
    fields: Vec<GrammerField>
}

#[derive(Deserialize, Debug)]
pub struct GrammerMetadata {
    name: String,
    fixed_size: bool,
    size: u32,
    big_endian: bool
}

#[derive(Deserialize, Debug)]
pub struct GrammerField {
    name: String,
    size: u32,
    display_format: String,
    description: String
}

pub const GRAMMER_FILE_FLAG: &str = "-g";
pub const BINARY_FILE_FLAG: &str = "-b";
pub const OFFSET_FLAG: &str = "-o";

pub const ERROR_START: &str = "[-] Error:";

pub fn parse_grammer(gram_file_contents: &String) -> Grammer {
    let gram_parsed: Grammer = match toml::from_str(gram_file_contents) {
        Ok(gram) => gram,
        Err(error) => panic!("{} failed to parse grammer file {}",ERROR_START,error)
    };

    return gram_parsed
} 


pub fn print_hex_gram(gram_file_contents: &String, binary_path: &String, struct_offset: u64) {
    let mut binary_file = match File::open(binary_path) {
        Ok(file) => file,
        Err(error) => panic!("{} opening file {}: {}",ERROR_START, binary_path, error),
    };

    let binary_file_end_offset = binary_file.seek(SeekFrom::End(0)).unwrap();

    if binary_file_end_offset >= struct_offset {
        match binary_file.seek(SeekFrom::Start(struct_offset)) {
            Ok(offset) => (),
            Err(error) => panic!(
                "{} seeking to offset in file {}: {}",ERROR_START ,
                binary_path, error
            ),
        }
    } else {
        panic!(
            "{} provided offset {} is larger than size of file {}: {}", ERROR_START ,
            struct_offset, binary_path, binary_file_end_offset
        );
    }

}

pub fn check_mandatory_cmds(
    cmdline_hashmap: &mut HashMap<String, Option<String>>,
) -> CMDParseResult {
    match check_flag_and_file_exists(cmdline_hashmap, GRAMMER_FILE_FLAG) {
        CMDParseResult::FileFound => (),
        cmd_parse_result @ _ => return cmd_parse_result,
    }

    match check_flag_and_file_exists(cmdline_hashmap, BINARY_FILE_FLAG) {
        CMDParseResult::FileFound => (),
        cmd_parse_result @ _ => return cmd_parse_result,
    }

    match check_flag_and_value_exists(cmdline_hashmap, OFFSET_FLAG) {
        CMDParseResult::OffsetFlagExists => (),
        cmd_parse_result @ _ => return cmd_parse_result,
    }

    return CMDParseResult::Success;
}

fn check_flag_and_value_exists(
    cmdline_hashmap: &mut HashMap<String, Option<String>>,
    key: &str,
) -> CMDParseResult {
    match cmdline_hashmap.contains_key(key) {
        true => match cmdline_hashmap.get(key).unwrap() {
            Some(entry) => return CMDParseResult::OffsetFlagExists,
            None => {
                eprintln!(
                    "{} You need to specify an offset into the binary file",
                    ERROR_START
                );
                return CMDParseResult::OffsetNotSpecified;
            }
        },
        false => {
            eprintln!(
                "{} You need to specify the offset flag {}",
                ERROR_START, OFFSET_FLAG
            );
            return CMDParseResult::OffsetFlagNotSpecified;
        }
    }
}

fn check_flag_and_file_exists(
    cmdline_hashmap: &mut HashMap<String, Option<String>>,
    key: &str,
) -> CMDParseResult {
    match cmdline_hashmap.contains_key(key) {
        true => match cmdline_hashmap.get(key).unwrap() {
            Some(entry) => match Path::new(entry).exists() {
                true => return CMDParseResult::FileFound,
                false => {
                    eprintln!("{} Could not find file {:#?}", ERROR_START, entry);
                    return CMDParseResult::FileNotFound;
                }
            },
            None => {
                eprintln!(
                    "{} You need to specify a value for flag {}",
                    ERROR_START, key
                );
                return CMDParseResult::FilePathNotSpecified;
            }
        },
        false => {
            eprintln!("{} You need to specify the flag {}", ERROR_START, key);
            return CMDParseResult::FilePathNotSpecified;
        }
    }
}
