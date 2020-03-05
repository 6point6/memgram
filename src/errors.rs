#[macro_export]

macro_rules! serror {
    ($cause:expr) => {
        eprintln!(
            "[-] Error: \n\t- Cause: {}\n\t- Line: {}\n\t- File: {}",
            $cause,
            line!(),
            file!(),
            // backtrace::Backtrace::new()
        );
    };
}

const USAGE_STRING: &str = "memgram [OPTION] [VALUE]";
const OPTIONS: [&str; 3] = [
    "-g     grammer file",
    "-b     binary file",
    "-o     offset into binary file",
];

pub fn usage() {
    println!("USAGE:\n\t{}\n\nOPTIONS:{:#?}\n\nEXAMPLE:\n\tmemgram -b ./examples/test_formats.bin -g ./grammer/test_formats.toml -o 0",USAGE_STRING,OPTIONS);
}
