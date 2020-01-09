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

    let cmdline_parse_result: arg_parse::ArgParseResult =
        arg_parse::parse_cmdline_args(args, &mut cmdline_hashmap);

    match cmdline_parse_result {
        arg_parse::ArgParseResult::NoArguments => {
            usage_info();
            process::exit(arg_parse::ArgParseResult::NoArguments as i32);
        }
        arg_parse::ArgParseResult::Success => (),
    }

    let mut binary_parse_result: gram_parse::CMDParseResult =
        gram_parse::check_mandatory_cmds(&mut cmdline_hashmap);

}
