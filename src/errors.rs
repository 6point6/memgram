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

const usage_string: &str = "memgram [OPTION] [VALUE]";
const options: [&str;3] = ["-g     grammer file","-b     binary file","-o     offset into binary file"];

pub fn usage() {
    println!("USAGE:\n\t{}\n\nOPTIONS:{:#?}\n\nEXAMPLE:\n\tmemgram -b mbrdump.bin -g ./grammer/mbr.toml -o 0",usage_string,options);
}
