#[macro_export]

macro_rules! serror {
    ($cause:expr) => {
        eprintln!(
            "[-] Error: \n\t- Cause: {}\n\t- Line: {}\n\t- File: {}",
            $cause,
            line!(),
            file!(),
        );
    };
}

const USAGE_STRING: &str = "memgram [OPTION] [VALUE]";
const OPTIONS: [&str; 6] = [
    "-g     grammer filepath",
    "-b     binary filepath",
    "-o     offset into binary file structure starts at",
    "-e     reverse endianess for formatted data",
    "-c     c struct filepath for conversion",
    "-o     output filepath for conversion",
];

const EXAMPLES: [&str; 2] = [
    "memgram -b ./examples/test_formats.bin -g ./grammer/test_formats.toml -o 0",
    "memgram -c ./examples/COFFHeader.h -o ./grammer/COFFHeader.toml",
];

pub fn usage() {
    println!(
        "USAGE: {:}\n\nOPTIONS:\n{:#?}\n\nEXAMPLE:\n{:#?}",
        USAGE_STRING, OPTIONS, EXAMPLES
    );
}
