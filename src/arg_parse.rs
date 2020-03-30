use crate::errors;
use std::collections::HashMap;
use std::env;
use std::path::Path;

pub const GRAMMER_FILE_FLAG: &str = "-g";
pub const CSTRUCT_FILE_FLAG: &str = "-c";
pub const OUTPUT_FILE_FLAG: &str = "-o";
pub const BINARY_FILE_FLAG: &str = "-b";
pub const OFFSET_FLAG: &str = "-s";
pub const FMT_ENDIAN_FLAG: &str = "-e";
pub const HEX_ENDIAN_FLAG: &str = "-E";
pub const DESCRIPTION_FLAG: &str = "-d";
pub const HELP_FLAG: &str = "-h";

pub struct CMDArgParse {
    raw_args: Vec<String>,
    pub arg_map: HashMap<String, Option<String>>,
    pub grammer_filepath: String,
    pub binary_filepath: String,
    pub cstruct_filepath: String,
    pub output_filepath: String,
    pub struct_offset: u64,
    pub description: bool,
    pub fmt_endian: bool,
    pub hex_endian: bool,
    pub help_flag: bool,
}

pub enum CMDOptions {
    ConvertWrite,
    ConvertDisplay,
    DisplayNormal,
}

impl CMDArgParse {
    pub fn new() -> CMDArgParse {
        CMDArgParse {
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

    pub fn parse_cmd_args(&mut self) -> Result<&mut CMDArgParse, ()> {
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

    pub fn parse_file_flag(&mut self, flag: &str) -> Result<&mut CMDArgParse, ()> {
        let file_path = self
            .arg_map
            .get(flag)
            .ok_or_else(|| {
                serror!(format!("You need to specify a value for flag {}", flag));
                panic!();
            })?
            .clone()
            .ok_or_else(|| serror!(format!("A file path must be specified for flag: {}", flag)))?;

        if Path::new(&file_path).exists() {
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

    pub fn run_cmds(&mut self) -> Result<CMDOptions, ()> {
        if !self.arg_map.contains_key(CSTRUCT_FILE_FLAG)
            && self.arg_map.contains_key(GRAMMER_FILE_FLAG)
            && self.arg_map.contains_key(BINARY_FILE_FLAG)
        {
            Ok(CMDOptions::DisplayNormal)
        } else {
            if self.arg_map.contains_key(OUTPUT_FILE_FLAG)
                && self.arg_map.contains_key(CSTRUCT_FILE_FLAG)
                && !self.arg_map.contains_key(OFFSET_FLAG)
                && !self.arg_map.contains_key(BINARY_FILE_FLAG)
            {
                return Ok(CMDOptions::ConvertWrite);
            } else if self.arg_map.contains_key(BINARY_FILE_FLAG)
                && self.arg_map.contains_key(CSTRUCT_FILE_FLAG)
                && !self.arg_map.contains_key(OUTPUT_FILE_FLAG)
                && !self.arg_map.contains_key(GRAMMER_FILE_FLAG)
            {
                return Ok(CMDOptions::ConvertDisplay);
            } else {
                serror!("Unsupported flag combination");
                errors::usage();
                Err(())
            }
        }
    }

    pub fn parse_offset_flag(&mut self, offset_flag: &str) -> Result<&mut CMDArgParse, ()> {
        if self.arg_map.contains_key(offset_flag) {
            match self.arg_map.get(offset_flag).unwrap() {
                Some(offset) => {
                    self.struct_offset = offset
                        .parse::<u64>()
                        .or_else(|e| {
                            serror!(format!("Invalid offset: {}, because: {}", offset, e));
                            return Err(());
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

    pub fn parse_help_flag(&mut self, help_flag: &str) -> &mut CMDArgParse {
        if self.arg_map.contains_key(help_flag) {
            self.help_flag = true
        }
        self
    }

    pub fn parse_bool_flags(
        &mut self,
        fmt_endian_flag: &str,
        hex_endian_flag: &str,
        description_flag: &str,
    ) -> &mut CMDArgParse {
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
}
