mod arg_parse;
mod gram_parse;
mod hexfmt;
use std::collections::HashMap;
use std::env;
use std::process;
use std::fs;
use std::fs::File;

fn usage_and_exit(exit_code: i32) {
    println!("usage: blah blah");
    process::exit(exit_code);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut cmdline_hashmap: HashMap<String, Option<String>> = HashMap::new();

    match arg_parse::parse_cmdline_args(args, &mut cmdline_hashmap) {
        arg_parse::ArgParseResult::Success => (),
        cmdline_parse_result @ _ => usage_and_exit(cmdline_parse_result as i32)
    };

    match gram_parse::check_mandatory_cmds(&mut cmdline_hashmap) {
        gram_parse::CMDParseResult::Success => (),
        binary_parse_result @ _ => usage_and_exit(binary_parse_result as i32)
    };

    let gram_file = match cmdline_hashmap.get(gram_parse::GRAMMER_FILE_FLAG).unwrap() {
        Some(path) => {
            match fs::read_to_string(path) {
                Ok(file) => file,
                Err(error) => panic!("[-] Error opening file: {}",error)
            }
        }
        None => panic!("[-] No value for -g flag") 
    };


}
