use crate::errors;
use std::collections::HashMap;
use std::env;
use std::path::Path;

/// All of the arguments which can be passed to memgram
pub const GRAMMER_FILE_FLAG: &str = "-g";
pub const CSTRUCT_FILE_FLAG: &str = "-c";
pub const OUTPUT_FILE_FLAG: &str = "-o";
pub const BINARY_FILE_FLAG: &str = "-b";
pub const STRUCT_OFFSET_FLAG: &str = "-s";
pub const FMT_ENDIAN_FLAG: &str = "-e";
pub const HEX_ENDIAN_FLAG: &str = "-E";
pub const DESCRIPTION_FLAG: &str = "-d";
pub const HELP_FLAG: &str = "-h";

/// Holds the state of all of the arguments passed to memgram.
/// At some point this will be replaced with the Clap crate: https://github.com/clap-rs/clap.
pub struct CMDArgParse {
    /// The raw cmdline args retrieved from env::args().collect().
    raw_args: Vec<String>,
    /// Holds a hashmap of the switch (-e,-d, etc) as the key and a Option<String> representing a value for the switch or no value.
    pub arg_map: HashMap<String, Option<String>>,
    /// A filepath to the grammar file.
    pub grammer_filepath: String,
    /// A filepath to the binary file.
    pub binary_filepath: String,
    /// A filepath to the C struct file.
    pub cstruct_filepath: String,
    /// A filepath for the output grammar file of the C struct conversion.
    pub output_filepath: String,
    /// Offset into binary file where the data structure starts.
    pub struct_offset: u64,
    /// Determines whether the description table will be printed to stdout or not.
    /// true = Print to stdout. This is set by specifying the switch as a CLI argument at runtime.
    /// false = Do not print to stdout. This is the default state if the switch is not passed as an argument.
    pub description: bool,
    /// Determines whether the endianess of non memgram display types will be printed with reversed endianess in the output fmt table.
    /// true = reverse the endianess.
    /// false = Do not reverse the endianess.This is the default state if the switch is not passed as an argument.
    pub fmt_endian: bool,
    /// Determines whether the endianess of non memgram display types will be printed with reversed endianess in the output hex view.
    /// true = reverse the endianess.
    /// false = Do not reverse the endianess.This is the default state if the switch is not passed as an argument.
    pub hex_endian: bool,
    /// Determines whether memgram usage information will be printed to stdout.
    /// true = print help information and quit.
    /// false = Do not print help information and quit. This is the default state if the switch is not passed as an argument.
    pub help_flag: bool,
}

/// memgram run options
pub enum RunOptions {
    /// Convert the C struct file to a grammar file. Do not display output fmt table/hex view.
    CStructConvertWrite,
    /// Convert the C struct and display the fmt table/hex view output. The converted C struct file is not written to disk.
    CStructConvertDisplay,
    /// Display the fmt table/hex view. No C structs are converted or displayed.
    DisplayNormal,
}

impl CMDArgParse {
    pub fn new() -> Self {
        Self {
            raw_args: env::args().collect(),
            arg_map: HashMap::new(),
            grammer_filepath: String::from(""),
            binary_filepath: String::from(""),
            cstruct_filepath: String::from(""),
            output_filepath: String::from(""),
            struct_offset: 0,
            description: false,
            fmt_endian: false,
            hex_endian: false,
            help_flag: false,
        }
    }

    /// Parses the cmdline arguments stored in self.raw_args into self.arg_map.
    /// If a switch is supplied, e.g "-g" and no value, then the arg will be stored as "-g",None in HashMap.
    /// If a switch is supplied, e.g "-g" and a value e.g "somegramfile.toml", then the argu will be stored as "-g",Some("somegramfile.toml") in HashMap.
    /// If no arguments are supplied, usage info is printed and Err(()) returned.
    pub fn parse_cmd_args(&mut self) -> Result<&mut Self, ()> {
        match self.raw_args.len() {
            1 => {
                errors::usage();
                Err(())
            }
            _ => {
                for (index, value) in self.raw_args.iter().enumerate() {
                    if &value[..1] == "-" {
                        self.arg_map.insert(value.to_string(), None);
                    } else if (index != 0) && (&self.raw_args[index - 1][..1] == "-") {
                        self.arg_map.insert(
                            self.raw_args[index - 1].to_string(),
                            Some(value.to_string()),
                        );
                    }
                }
                Ok(self)
            }
        }
    }

    /// Parses a single file arg into a &str and saves it in self.MATCHED_FILEPATH
    /// An Err(()) is returned if the key-value pair are not both Some or the filepath does not exist on the OS (except for the OUTPUT_FILE_FLAG)
    pub fn parse_file_arg(&mut self, flag: &str) -> Result<&mut Self, ()> {
        let file_path = self
            .arg_map
            .get(flag)
            .ok_or_else(|| {
                serror!(format!("You need to specify a value for flag {}", flag));
                panic!();
            })?
            .clone()
            .ok_or_else(|| serror!(format!("A file path must be specified for flag: {}", flag)))?;

        if Path::new(&file_path).exists() || flag == OUTPUT_FILE_FLAG {
            match flag {
                GRAMMER_FILE_FLAG => self.grammer_filepath = file_path,
                BINARY_FILE_FLAG => self.binary_filepath = file_path,
                CSTRUCT_FILE_FLAG => self.cstruct_filepath = file_path,
                OUTPUT_FILE_FLAG => self.output_filepath = file_path,
                _ => (serror!(format!("The flag is not a file flag: {}", flag))),
            }
            Ok(self)
        } else {
            serror!(format!(
                "Could not find file {} for flag {}",
                file_path, flag
            ));
            Err(())
        }
    }

    /// Parses a single offset flag into u64 and saves result in self.struct_offset.
    /// An Err(()) is returned if the offset could not be coverted into a u64.
    /// If an offset is not found in self.arg_map, self.struct_offset is set to 0.
    pub fn parse_offset_flag(&mut self, offset_flag: &str) -> Result<&mut Self, ()> {
        if self.arg_map.contains_key(offset_flag) {
            match self.arg_map.get(offset_flag).unwrap() {
                Some(offset) => {
                    self.struct_offset = offset
                        .parse::<u64>()
                        .or_else(|e| {
                            serror!(format!("Invalid offset: {}, because: {}", offset, e));
                            Err(())
                        })
                        .unwrap();
                    Ok(self)
                }
                None => {
                    serror!("You need to specify an offset into the binary file");
                    Err(())
                }
            }
        } else {
            self.struct_offset = 0;
            Ok(self)
        }
    }

    /// Sets self.help_flag to true if the help flag/switch is in self.arg_map
    pub fn parse_help_flag(&mut self, help_flag: &str) -> &mut Self {
        if self.arg_map.contains_key(help_flag) {
            self.help_flag = true
        }
        self
    }

    /// Sets the corrosponding flag in self if it's in self.arg_map
    pub fn parse_bool_flags(
        &mut self,
        fmt_endian_flag: &str,
        hex_endian_flag: &str,
        description_flag: &str,
    ) -> &mut Self {
        if self.arg_map.contains_key(fmt_endian_flag) {
            self.fmt_endian = true
        }

        if self.arg_map.contains_key(hex_endian_flag) {
            self.hex_endian = true
        }

        if self.arg_map.contains_key(description_flag) {
            self.description = true
        }
        self
    }

    /// Used to determine which methods to run in main.rs.
    /// Returns a RunOption variant depending on which switches/flags were passed on the cmdline.
    /// Descriptions of each variant are contained in the RunOption enum documentation
    pub fn run_cmds(&mut self) -> Result<RunOptions, ()> {
        if !self.arg_map.contains_key(CSTRUCT_FILE_FLAG)
            && self.arg_map.contains_key(GRAMMER_FILE_FLAG)
            && self.arg_map.contains_key(BINARY_FILE_FLAG)
        {
            Ok(RunOptions::DisplayNormal)
        } else if self.arg_map.contains_key(OUTPUT_FILE_FLAG)
            && self.arg_map.contains_key(CSTRUCT_FILE_FLAG)
            && !self.arg_map.contains_key(STRUCT_OFFSET_FLAG)
            && !self.arg_map.contains_key(BINARY_FILE_FLAG)
        {
            Ok(RunOptions::CStructConvertWrite)
        } else if self.arg_map.contains_key(BINARY_FILE_FLAG)
            && self.arg_map.contains_key(CSTRUCT_FILE_FLAG)
            && !self.arg_map.contains_key(OUTPUT_FILE_FLAG)
            && !self.arg_map.contains_key(GRAMMER_FILE_FLAG)
        {
            Ok(RunOptions::CStructConvertDisplay)
        } else {
            serror!("Unsupported flag combination");
            errors::usage();
            Err(())
        }
    }
}
