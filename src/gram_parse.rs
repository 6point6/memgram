use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

#[derive(PartialEq)]
pub enum CMDParseResult {
    FileNotFound,
    FilePathNotSpecified,
    FileFound,
    Success,
}

pub fn check_mandatory_cmds(cmdline_hashmap: &mut HashMap<String, Option<String>>) -> CMDParseResult {
    let cmd_parse_result: CMDParseResult = check_cmd_and_file_exists(cmdline_hashmap,"-g");

    if cmd_parse_result != CMDParseResult::FileFound {
        return cmd_parse_result;
    }else {
    }

    let cmd_parse_result: CMDParseResult = check_cmd_and_file_exists(cmdline_hashmap,"-b");

    if cmd_parse_result != CMDParseResult::FileFound {
        return cmd_parse_result;
    }else {
    }

    return CMDParseResult::Success;

}

pub fn check_cmd_and_file_exists(
    cmdline_hashmap: &mut HashMap<String, Option<String>>, key: &str 
) -> CMDParseResult {
    match cmdline_hashmap.contains_key(key) {
        true => match cmdline_hashmap.get(key).unwrap() {
            Some(entry) => match Path::new(entry).exists() {
                true => return CMDParseResult::FileFound,
                false => {
                    eprintln!("Error: Could not find file {:#?}",entry);
                    return CMDParseResult::FileNotFound
                }
            },
            None => {
                eprintln!("Error: You need to specify a value for flag {}",key);
                return CMDParseResult::FilePathNotSpecified
            } 
        },
        false => {
            eprintln!("Error: You need to specify the flag {}",key);
            return CMDParseResult::FilePathNotSpecified
        }
    }
}
