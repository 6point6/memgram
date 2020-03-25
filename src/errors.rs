#[macro_export]

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

const USAGE_STRING: &str = "memgram [OPTION] [VALUE]";
const OPTIONS: [&str; 7] = [
    "-g     grammar filepath",
    "-b     binary filepath",
    "-s     offset into binary the file structure starts at",
    "-e     reverse the endianess for table formatted data",
    "-E     reverse the endianess for hex view formatted data",
    "-c     c struct filepath",
    "-o     output filepath for conversion",
];

const EXAMPLES: [&str; 3] = [
    "memgram -b ./examples/test_formats.bin -g ./grammar/test_formats.toml -o 0",
    "memgram -c ./examples/COFFHeader.h -o ./grammar/COFFHeader.toml",
    "memgram -c ./examples/COFFHeader.h -b .~/Downloads/binary.exe -o 244 -e",
];

pub fn usage() {
    println!(
        "USAGE: {:}\n\nOPTIONS:\n{:#?}\n\nEXAMPLE:\n{:#?}",
        USAGE_STRING, OPTIONS, EXAMPLES
    );
}
