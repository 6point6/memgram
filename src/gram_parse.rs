use std::collections::HashMap;
use std::path::Path;

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

pub const GRAMMER_FILE_FLAG: &str = "-g";
pub const BINARY_FILE_FLAG: &str = "-b";
pub const OFFSET_FLAG: &str = "-o";

pub const ERROR_START: &str = "[-] Error:";

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
