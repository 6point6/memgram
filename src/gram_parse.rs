use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use toml::Value;

#[derive(PartialEq, Debug)]
pub enum ParseResult {
    FlagNotSpecified,
    OffsetToLarge,
    SeekError,
    OpenFileError,
    FileNotFound,
    FilePathNotSpecified,
    OffsetNotSpecified,
    OffsetFlagNotSpecified,
    Success,
}

#[derive(Deserialize, Debug)]
pub struct Grammer {
    metadata: GrammerMetadata,
    fields: Vec<GrammerField>,
}

#[derive(Deserialize, Debug)]
pub struct GrammerMetadata {
    name: String,
    fixed_size: bool,
    size: u32,
    big_endian: bool,
}

#[derive(Deserialize, Debug)]
pub struct GrammerField {
    name: String,
    size: u32,
    display_format: String,
    description: String,
}

pub const GRAMMER_FILE_FLAG: &str = "-g";
pub const BINARY_FILE_FLAG: &str = "-b";
pub const OFFSET_FLAG: &str = "-o";

pub const ERROR_START: &str = "[-] Error:";

pub fn parse_grammer(gram_file_contents: &String) -> Option<Grammer> {
    match toml::from_str(gram_file_contents) {
        Ok(gram) => return Some(gram),
        Err(error) => {
            eprintln!("{} failed to parse grammer file {}", ERROR_START, error);
            return None;
        }
    }
}

pub fn print_hex_gram(
    gram_file_contents: &String,
    binary_path: &String,
    struct_offset: u64,
) -> Result<ParseResult, ParseResult> {
    let mut binary_file = match File::open(binary_path) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("{} opening file {}: {}", ERROR_START, binary_path, error);
            return Err(ParseResult::OpenFileError);
        }
    };

    let binary_file_end_offset = binary_file.seek(SeekFrom::End(0)).unwrap();

    if binary_file_end_offset >= struct_offset {
        match binary_file.seek(SeekFrom::Start(struct_offset)) {
            Ok(offset) => (),
            Err(error) => {
                eprintln!(
                    "{} seeking to offset in file {}: {}",
                    ERROR_START, binary_path, error
                );
                return Err(ParseResult::SeekError);
            }
        }
    } else {
        eprintln!(
            "{} provided offset {} is larger than size of file {}: {}",
            ERROR_START, struct_offset, binary_path, binary_file_end_offset
        );
        return Err(ParseResult::OffsetToLarge);
    }

    return Ok(ParseResult::Success);
}

pub fn check_mandatory_cmds(cmdline_hashmap: &mut HashMap<String, Option<String>>) -> Result<ParseResult,ParseResult> {
    match check_flag_and_file_exists(cmdline_hashmap, GRAMMER_FILE_FLAG) {
        Ok(ParseResult::Success) => (),
        cmd_parse_result @ _ => return cmd_parse_result,
    }

    match check_flag_and_file_exists(cmdline_hashmap, BINARY_FILE_FLAG) {
        Ok(ParseResult::Success) => (),
        cmd_parse_result @ _ => return cmd_parse_result,
    }

    match check_flag_and_value_exists(cmdline_hashmap, OFFSET_FLAG) {
        Ok(ParseResult::Success) => (),
        cmd_parse_result @ _ => return cmd_parse_result,
    }

    return Ok(ParseResult::Success);
}

fn check_flag_and_value_exists(
    cmdline_hashmap: &mut HashMap<String, Option<String>>,
    key: &str,
) -> Result<ParseResult, ParseResult> {
    match cmdline_hashmap.contains_key(key) {
        true => match cmdline_hashmap.get(key).unwrap() {
            Some(entry) => return Ok(ParseResult::Success),
            None => {
                eprintln!(
                    "{} You need to specify an offset into the binary file",
                    ERROR_START
                );
                return Err(ParseResult::OffsetNotSpecified);
            }
        },
        false => {
            eprintln!(
                "{} You need to specify the offset flag {}",
                ERROR_START, OFFSET_FLAG
            );
            return Err(ParseResult::OffsetFlagNotSpecified);
        }
    }
}

fn check_flag_and_file_exists(
    cmdline_hashmap: &mut HashMap<String, Option<String>>,
    key: &str,
) -> Result<ParseResult, ParseResult> {
    match cmdline_hashmap.contains_key(key) {
        true => match cmdline_hashmap.get(key).unwrap() {
            Some(entry) => match Path::new(entry).exists() {
                true => return Ok(ParseResult::Success),
                false => {
                    eprintln!("{} Could not find file {:#?}", ERROR_START, entry);
                    return Err(ParseResult::FileNotFound);
                }
            },
            None => {
                eprintln!(
                    "{} You need to specify a value for flag {}",
                    ERROR_START, key
                );
                return Err(ParseResult::FilePathNotSpecified);
            }
        },
        false => {
            eprintln!("{} You need to specify the flag {}", ERROR_START, key);
            return Err(ParseResult::FlagNotSpecified);
        }
    }
}
