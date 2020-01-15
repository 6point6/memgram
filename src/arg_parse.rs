use std::collections::HashMap;

#[derive(Debug)]
pub enum ArgParseResult {
    NoArguments,
    Success,
}

pub fn parse_cmdline_args(
    args: Vec<String>,
    cmdline_hashmap: &mut HashMap<String, Option<String>>,
) -> Result<ArgParseResult, ArgParseResult> {
    match args.len() {
        1 => {
            return Err(ArgParseResult::NoArguments);
        }
        _ => {
            for (index, value) in args.iter().enumerate() {
                if &value[..1] == "-" {
                    cmdline_hashmap.insert(value.to_string(), None);
                } else {
                    if (index != 0) && (&args[index - 1][..1]) == "-" {
                        cmdline_hashmap
                            .insert(args[index - 1].to_string(), Some(value.to_string()));
                    }
                }
            }
            return Ok(ArgParseResult::Success);
        }
    }
}
