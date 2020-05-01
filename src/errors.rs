//! Module for printing memgram error and usage information.

#[macro_export]
/// Prints error information to stderr including cause, line number and file.
macro_rules! serror {
    ($cause:expr) => {
        eprintln!(
            "[-] Error: \n\t- Cause: {}\n\t- Line: {}\n\t- File: {}\n",
            $cause,
            line!(),
            file!(),
        );
    };
}

/// Specifies how memgram takes CLI arguments.
const USAGE_STRING: &str = "memgram [OPTION] [VALUE]";

/// Memgram CLI argument options and their corresponding descriptions.
const OPTIONS: [&str; 9] = [
    "-b     binary filepath",
    "-c     c struct filepath",
    "-d     show description table",
    "-e     reverse the endianess for table formatted data",
    "-E     reverse the endianess for hex view formatted data",
    "-g     grammar filepath",
    "-h     display usage information",
    "-o     output filepath for conversion",
    "-s     offset into binary the file structure starts at",
];

/// Example usage strings for memgram.
const EXAMPLES: [&str; 3] = [
    "memgram -b ./examples/test_formats.bin -g ./grammar/test_formats.toml -s 0",
    "memgram -c ./examples/COFFHeader.h -o ./grammar/COFFHeader.toml",
    "memgram -c ./examples/COFFHeader.h -b .~/Downloads/binary.exe -o 244 -e -E -d",
];

/// Prints memgram usage information.
pub fn usage() {
    println!(
        "USAGE: {:}\n\nOPTIONS:\n{:#?}\n\nEXAMPLE:\n{:#?}",
        USAGE_STRING, OPTIONS, EXAMPLES
    );
}
