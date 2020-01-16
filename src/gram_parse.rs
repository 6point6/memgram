use hex::ToHex;
use prettytable::{Cell, Row, Table};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use crate::errorh;

#[derive(PartialEq, Debug)]
pub enum ParseResult {
    GrammerParseFail,
    FillingHashMapFail,
    FlagNotSpecified,
    OffsetToLarge,
    FieldValueEmpty,
    SeekError,
    OpenFileError,
    FileNotFound,
    FilePathNotSpecified,
    OffsetNotSpecified,
    OffsetFlagNotSpecified,
    FeatureNotImplemented,
    OffsetTooLarge,
    Success,
}

#[derive(Deserialize, Debug)]
pub struct Grammer {
    pub metadata: GrammerMetadata,
    pub fields: Vec<GrammerField>,
}

#[derive(Deserialize, Debug)]
pub struct GrammerMetadata {
    pub name: String,
    pub fixed_size: bool,
    pub size: u64,
    pub big_endian: bool,
}

#[derive(Deserialize, Debug)]
pub struct GrammerField {
    pub name: String,
    pub offset: i64,
    pub size: usize,
    pub data_type: String,
    pub display_format: String,
    pub description: String,
}

pub const GRAMMER_FILE_FLAG: &str = "-g";
pub const BINARY_FILE_FLAG: &str = "-b";
pub const OFFSET_FLAG: &str = "-o";

pub const ASCII_TYPE: &str = "ascii";

pub fn check_file_large_enough(struct_offset: u64, struct_size: u64 ,binary_file: &mut File, binary_path: &String) -> Result<ParseResult, ParseResult> {
    let binary_file_size = binary_file.seek(SeekFrom::End(0)).unwrap();

    if binary_file_size > struct_offset + struct_size { 
        match binary_file.seek(SeekFrom::Start(struct_offset)) {
            Ok(_) => return Ok(ParseResult::Success),
            Err(error) => {
                eprintln!(
                    "{} seeking to offset in file {}: {}",
                    errorh::ERROR_START, binary_path, error
                );
                return Err(ParseResult::SeekError);
            }
        }
    } else {
        eprintln!(
            "{} offset + structure size is larger than the file {}:",
            errorh::ERROR_START, binary_path
        );
        return Err(ParseResult::OffsetTooLarge);
    }
}


pub fn parse_grammer(gram_file_contents: &String) -> Option<Grammer> {
    match toml::from_str(gram_file_contents) {
        Ok(gram) => return Some(gram),
        Err(error) => {
            eprintln!("{} failed to parse grammer file {}", errorh::ERROR_START, error);
            return None;
        }
    }
}

fn print_filled_table(
    parsed_gram: &Grammer,
    field_hashmap: &HashMap<String, Vec<u8>>,
) -> Result<ParseResult, ParseResult> {
    let mut table = Table::new();

    table.add_row(row!["Field", "Offset", "Size","Data Type", "Raw Data","Formatted Data", "Description"]);

    for (index, field) in parsed_gram.fields.iter().enumerate() {
        let mut hex_string: String = match field_hashmap.get(&field.name) {
            Some(raw_data) => raw_data.encode_hex::<String>(),
            None => return Err(ParseResult::FieldValueEmpty),
        };
    
        if hex_string.len() > 40 {
            hex_string = hex_string[..40].to_string();
            hex_string.push_str("...");
        }

        let formatted_data: String = match field_hashmap.get(&field.name) {
            Some(raw_data) => match &field.display_format[..] {
                ASCII_TYPE => raw_data.into_iter().map(|ascii| *ascii as char).collect(),
                _ => String::from("N/A")
            }
            None => return Err(ParseResult::FieldValueEmpty)
        };


        let _row = match index % 2 {
            0 => table.add_row(
                row![bFW->field.name,bFW->field.offset,bFW->field.size,bFW->field.data_type,bFW->hex_string,bFW->formatted_data,bFW->field.description],
            ),
            _ => table.add_row(
                row![bFC->field.name,bFC->field.offset,bFC->field.size,bFC->field.data_type,bFC->hex_string,bFC->formatted_data,bFC->field.description],
            ),
        };
    
    }
    table.printstd();

    Ok(ParseResult::Success)
}

pub fn fill_field_hashmap(
    field_hashmap: &mut HashMap<String, Vec<u8>>,
    parsed_gram: &Grammer,
    binary_file: &mut File,
) -> Result<ParseResult, ParseResult> {
    for field in &parsed_gram.fields {
        field_hashmap.insert(
            field.name.to_string(),
            binary_file
                .bytes()
                .take(field.size)
                .map(|r: Result<u8, _>| r.unwrap())
                .collect(),
        ); // Need to sort this out, will panic if error
    }
    // println!("{:#x?}",field_hashmap);
    return Ok(ParseResult::Success);
}

pub fn print_hex_gram(
    gram_file_contents: &String,
    binary_path: &String,
    struct_offset: u64,
) -> Result<ParseResult, ParseResult> {

    let parsed_gram = match parse_grammer(gram_file_contents) {
        Some(parsed) => parsed,
        None => return Err(ParseResult::GrammerParseFail),
    };

    match parsed_gram.metadata.fixed_size {
        true => (),
        false => { 
            eprintln!(
                "{} variable size data structures currently not supported",
                errorh::ERROR_START
            );
            return Err(ParseResult::FeatureNotImplemented)
        }
    }

    let mut binary_file = match File::open(binary_path) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("{} opening file {}: {}", errorh::ERROR_START, binary_path, error);
            return Err(ParseResult::OpenFileError);
        }
    };

    match check_file_large_enough(struct_offset, parsed_gram.metadata.size, &mut binary_file, &binary_path) {
        Ok(_) => (),
        Err(error) => return Err(error)
    }

    let mut field_hashmap = HashMap::new();

    match fill_field_hashmap(&mut field_hashmap, &parsed_gram, &mut binary_file) {
        Ok(_) => (),
        Err(_) => {
            eprintln!("{} filling hashmap fields:", errorh::ERROR_START);
            return Err(ParseResult::FillingHashMapFail);
        }
    }

    match print_filled_table(&parsed_gram, &field_hashmap) {
        Ok(_) => (),
        Err(error) => return Err(error) 
    }

    return Ok(ParseResult::Success);
}

pub fn check_mandatory_cmds(
    cmdline_hashmap: &mut HashMap<String, Option<String>>,
) -> Result<ParseResult, ParseResult> {
    match check_flag_and_file_exists(cmdline_hashmap, GRAMMER_FILE_FLAG) {
        Ok(_) => (),
        cmd_parse_result @ _ => return cmd_parse_result,
    }

    match check_flag_and_file_exists(cmdline_hashmap, BINARY_FILE_FLAG) {
        Ok(_) => (),
        cmd_parse_result @ _ => return cmd_parse_result,
    }

    match check_flag_and_value_exists(cmdline_hashmap, OFFSET_FLAG) {
        Ok(_) => (),
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
            Some(_) => return Ok(ParseResult::Success),
            None => {
                eprintln!(
                    "{} You need to specify an offset into the binary file",
                    errorh::ERROR_START
                );
                return Err(ParseResult::OffsetNotSpecified);
            }
        },
        false => {
            eprintln!(
                "{} You need to specify the offset flag {}",
                errorh::ERROR_START, OFFSET_FLAG
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
                    eprintln!("{} Could not find file {:#?}", errorh::ERROR_START, entry);
                    return Err(ParseResult::FileNotFound);
                }
            },
            None => {
                eprintln!(
                    "{} You need to specify a value for flag {}",
                    errorh::ERROR_START, key
                );
                return Err(ParseResult::FilePathNotSpecified);
            }
        },
        false => {
            eprintln!("{} You need to specify the flag {}", errorh::ERROR_START, key);
            return Err(ParseResult::FlagNotSpecified);
        }
    }
}
