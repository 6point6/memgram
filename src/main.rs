mod arg_parse;
mod gram_parse;
mod hexfmt;
use std::collections::HashMap;
use std::env;
use std::process;

fn usage_info() {
    println!("usage: blah blah");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut cmdline_hashmap: HashMap<String, Option<String>> = HashMap::new();

    match arg_parse::parse_cmdline_args(args, &mut cmdline_hashmap) {
        arg_parse::ArgParseResult::Success => (),
        cmdline_parse_result @ _ => {
            usage_info();
            process::exit(cmdline_parse_result as i32);
        }
    };

    match gram_parse::check_mandatory_cmds(&mut cmdline_hashmap) {
        gram_parse::CMDParseResult::Success => (),
        binary_parse_result @ _ =>  {
            usage_info();
            process::exit(binary_parse_result as i32);
        }
    };
}
