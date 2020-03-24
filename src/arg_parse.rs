use crate::errors;
use std::collections::HashMap;
use std::env;
use std::path::Path;

pub const GRAMMER_FILE_FLAG: &str = "-g";
pub const CSTRUCT_FILE_FLAG: &str = "-c";
pub const OUTPUT_FILE_FLAG: &str = "-o";
pub const BINARY_FILE_FLAG: &str = "-b";
pub const OFFSET_FLAG: &str = "-o";
pub const ENDIAN_FLAG: &str = "-e";

pub struct CMDArgParse {
    raw_args: Vec<String>,
    pub arg_map: HashMap<String, Option<String>>,
    pub grammer_filepath: String,
    pub binary_filepath: String,
    pub cstruct_filepath: String,
    pub output_filepath: String,
    pub struct_offset: u64,
    pub reverse_endian: bool,
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
            reverse_endian: false,
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
        let file_path_option = self
            .arg_map
            .get(flag)
            .ok_or_else(|| {
                serror!(format!("You need to specify a value for flag {}", flag));
            })?
            .clone();

        let file_path = match file_path_option {
            Some(path) => path,
            None => {
                serror!(format!("A file path must be specified for flag: {}", flag));
                return Err(());
            }
        };

        if Path::new(&file_path).exists() {
                match flag {
                    GRAMMER_FILE_FLAG => self.grammer_filepath = file_path,
                    BINARY_FILE_FLAG => self.binary_filepath = file_path,
                    CSTRUCT_FILE_FLAG => self.cstruct_filepath = file_path,
                    _ => (serror!(format!("The flag is not a file flag: {}", flag))),
                }
                Ok(self)
            } else  {
                serror!(format!(
                    "Could not find file {} for flag {}",
                    file_path, flag
                ));
                Err(())
            }
    }

    pub fn check_convert_flags(&mut self) -> Result<Option<&mut CMDArgParse>, ()> {
        if !self.arg_map.contains_key(CSTRUCT_FILE_FLAG) {
            Ok(None)
        } else {
            let file_path_option = self
            .arg_map
            .get(OUTPUT_FILE_FLAG)
            .ok_or_else(|| {
                serror!("You must provide an output file for conversion");
            })?
            .clone();

            self.output_filepath = match file_path_option {
                Some(path) => path,
                None => {
                    serror!(format!("A file path must be specified for flag: {}", OUTPUT_FILE_FLAG));
                    return Err(());
                }
            };

            Ok(Some(self))
  
        }
    }

    pub fn parse_offset_flag(&mut self, offset_flag: &str) -> Result<&mut CMDArgParse, ()> {
        if self.arg_map.contains_key(offset_flag) {
            match self.arg_map.get(offset_flag).unwrap() {
                Some(offset) => {
                    self.struct_offset = offset
                        .parse::<u64>()
                        .or_else(|e| {
                            serror!(format!("Invalid offset: {}, because:{}", offset, e));
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
            serror!(format!("You need to specify the flag {}", offset_flag));
            Err(())
        }
    }

    pub fn parse_endian_flag(&mut self, endian_flag: &str) -> &mut CMDArgParse {
        if self.arg_map.contains_key(endian_flag) {
            self.reverse_endian = true;
            self
        } else {
            self
        }
    }
}
