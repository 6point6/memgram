pub mod arg_parse {
    use std::collections::HashMap;
    use std::env;
    use std::path::Path;
    use crate::errors;

    pub const GRAMMER_FILE_FLAG: &str = "-g";
    pub const BINARY_FILE_FLAG: &str = "-b";
    pub const OFFSET_FLAG: &str = "-o";

    pub struct CMDArgParse {
        raw_args: Vec<String>,
        arg_map: HashMap<String, Option<String>>,
        pub grammer_filepath: String,
        pub binary_filepath: String,
        pub struct_offset: u64,
    }

    impl CMDArgParse {
        pub fn new() -> CMDArgParse {
            CMDArgParse {
                raw_args: env::args().collect(),
                arg_map: HashMap::new(),
                grammer_filepath: String::from(""),
                binary_filepath: String::from(""),
                struct_offset: 0,
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
                })?
                .clone()
                .unwrap();

            match Path::new(&file_path).exists() {
                true => {
                    match flag {
                        GRAMMER_FILE_FLAG => self.grammer_filepath = file_path,
                        BINARY_FILE_FLAG => self.binary_filepath = file_path,
                        _ => (serror!(format!("The flag is not a file flag: {}", flag))),
                    }
                    Ok(self)
                }
                false => {
                    serror!(format!(
                        "Could not find file {} for flag {}",
                        file_path, flag
                    ));
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
    }
}

pub mod file_parse {
    use crate::parse::arg_parse::{BINARY_FILE_FLAG, GRAMMER_FILE_FLAG,CMDArgParse};
    use std::fs;
    use std::fs::File;

    pub struct FileData {
        pub grammer_contents: String,
        grammer_size: u64,
    }

    impl FileData {
        pub fn new() -> FileData {
            FileData {
                grammer_contents: String::from(""),
                grammer_size: 0,
            }
        }
        pub fn read_grammer(
            &mut self,
            file_path: &str,
        ) -> Result<&mut FileData, ()> {
                self.grammer_contents = read_file(file_path)?;
                Ok(self)
            }
        }

    pub fn read_file(file_path: &str) -> Result<String, ()> {
        fs::read_to_string(file_path).or_else(|e| {
            serror!(format!("Error opening file: {}, because:{}", file_path, e));
            Err(())
        })
    }

    // pub fn check_binary_filesize(&mut self,cmd_args: &CMDArgParse) {
    //     let binary_file_size =  binary_file.seek(SeekFrom::End(0)).unwrap();

    //     if binary_file_size > cmd_args.struct_offset + cmd_args.struct_size {
            

    //     } else {
    //         serror!(format!("Offset + structure size is larger than the file: {}",cmd_args.binary_filepath));
    //         Err(())
    //     }
    // } 
}

// pub mod hex_parse {

// }


pub mod gram_parse {
    use crate::arg_parse::CMDArgParse;
    use crate::file_parse::FileData;
    use hex::ToHex;
    use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, NasmFormatter};
    use prettytable::Table;
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Read;
    use std::net::{IpAddr, Ipv4Addr};
    use widestring::U16CString;

    pub const HEXLE_TYPE: &str = "hexle";
    pub const ASCII_TYPE: &str = "ascii";
    pub const IPV4BE_TYPE: &str = "ipv4be";
    pub const IPV4LE_TYPE: &str = "ipv4le";
    pub const UTF16LE_TYPE: &str = "utf16be";
    pub const UTF16BE_TYPE: &str = "utf16le";
    pub const X86_TYPE: &str = "x86_32";

    #[derive(Deserialize, Debug, Clone)]
    pub struct Grammer {
        pub metadata: GrammerMetadata,
        pub fields: Vec<GrammerFields>,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct GrammerMetadata {
        pub name: String,
        pub fixed_size: bool,
        pub size: u64,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct GrammerFields {
        pub name: String,
        pub offset: i64,
        pub size: usize,
        pub data_type: String,
        pub display_format: String,
        pub description: String,
    }

    pub struct TableData {
        field_hashmap: HashMap<String, Vec<u8>>,
        field_fmt_hashmap: HashMap<String, String>,
        field_raw_hashmap: HashMap<String, String>,
        description_table: Table,
        standard_table: Table,
    }

    pub enum Tables {
        Standard,
        Description,
    }

    impl TableData {
        pub fn new() -> TableData {
            TableData {
                field_hashmap: HashMap::new(),
                field_fmt_hashmap: HashMap::new(),
                field_raw_hashmap: HashMap::new(),
                description_table: Table::new(),
                standard_table: Table::new(),
            }
        }


        pub fn print_table(&self, table: Tables) {
            match table {
                Tables::Standard => self.standard_table.printstd(),
                Tables::Description => self.description_table.printstd(),
            };
        }

        pub fn fill_standard_table(&mut self, parsed_gram: &Grammer) -> Result<&mut TableData, ()> {
            self.standard_table.add_row(row![
                "Field",
                "Offset",
                "Size",
                "Data Type",
                "Raw Data",
                "Formatted Data",
            ]);

            for (index, field) in parsed_gram.fields.iter().enumerate() {
                let raw_hex_string = self.field_raw_hashmap.get(&field.name).ok_or_else(|| {
                    serror!(format!("Could not get value for field: {}", field.name));
                })?;

                let formatted_data = self.field_fmt_hashmap.get(&field.name).ok_or_else(|| {
                    serror!(format!("Could not get value for field: {}", field.name));
                })?;

                if index % 2 == 0 {
                    self.standard_table
                        .add_row(row![bFG->field.name,bFG->format!("{:#X}", field.offset),bFG->field.size,bFG->field.data_type,bFG->raw_hex_string,bFG->formatted_data]);
                } else {
                    self.standard_table
                        .add_row(row![bFM->field.name,bFM->format!("{:#X}", field.offset),bFM->field.size,bFM->field.data_type,bFM->raw_hex_string,bFM->formatted_data]);
                }
            }

            Ok(self)
        }

        pub fn fill_description_table(&mut self, parsed_gram: &Grammer) -> &mut TableData {
            self.description_table.add_row(row!["Field", "Description"]);

            for (index, field) in parsed_gram.fields.iter().enumerate() {
                if index % 2 == 0 {
                    self.description_table
                        .add_row(row![bFG->field.name,bFG->field.description]);
                } else {
                    self.description_table
                        .add_row(row![bFM->field.name,bFM->field.description]);
                }
            }
            self
        }

        pub fn create_field_hashmap(
            &mut self,
            parsed_gram: &Grammer,
            cmd_args: &CMDArgParse,
        ) -> Result<&mut TableData, ()> {
            let binary_file: &File = &File::open(&cmd_args.binary_filepath).map_err(|_| {
                serror!(format!(
                    "Could not open file: {}",
                    cmd_args.binary_filepath
                ))
            })?;

            for field in &parsed_gram.fields {
                self.field_hashmap.insert(
                    field.name.to_string(),
                    binary_file
                        .bytes()
                        .take(field.size)
                        .map(|r: Result<u8, _>| r.unwrap())
                        .collect(),
                );
            }
            Ok(self)
        }

        pub fn format_fields(&mut self, parsed_gram: &Grammer) -> Result<&mut TableData, ()> {
            for field in parsed_gram.fields.iter() {
                let mut  raw_hex_string: String = self
                    .field_hashmap
                    .get(&field.name)
                    .ok_or_else(|| serror!(format!("Failed to value for field: {}", field.name)))?
                    .encode_hex::<String>()
                    .to_uppercase();

                if raw_hex_string.len() > 25 {
                    raw_hex_string = raw_hex_string[..25].to_string();
                    raw_hex_string.push_str("...");
                }

                self.field_raw_hashmap
                    .insert(field.name.clone(), raw_hex_string.clone());

                let raw_data: &Vec<u8> = self.field_hashmap.get(&field.name).ok_or_else(|| {
                    serror!(format!("Could not get value for field: {}", field.name));
                })?;

                let formatted_data = match &field.display_format[..] {
                    HEXLE_TYPE => {
                        let mut reversed_raw_data: Vec<u8> = raw_data.clone();
                        reversed_raw_data.reverse();
                        reversed_raw_data.encode_hex::<String>().to_uppercase()
                    }
                    ASCII_TYPE => raw_data.into_iter().map(|ascii| *ascii as char).collect(),
                    IPV4BE_TYPE => format_ipv4_string(&raw_data)?,
                    IPV4LE_TYPE => {
                        let mut reversed_raw_data: Vec<u8> = raw_data.clone();
                        reversed_raw_data.reverse();
                        format_ipv4_string(&reversed_raw_data)?
                    }
                    UTF16BE_TYPE => format_utf16_string(raw_data, false)?,
                    UTF16LE_TYPE => format_utf16_string(raw_data, true)?,
                    X86_TYPE => {
                        let mut x86_disassembly = DissassOutput::new();
                        x86_disassembly.format_x86(16, raw_data);

                        if x86_disassembly.line_count > 5 {
                            String::from("See table above ;)")
                        } else {
                            x86_disassembly.output
                        }
                    }
                    _ => raw_hex_string.clone(),
                };
                self.field_fmt_hashmap
                    .insert(field.name.clone(), formatted_data);
            }
            Ok(self)
        }
    }

    pub fn format_ipv4_string(ipv4_bytes: &[u8]) -> Result<String, ()> {
        match ipv4_bytes.len() {
            4 => Ok(format!(
                "{}",
                IpAddr::V4(Ipv4Addr::new(
                    ipv4_bytes[0],
                    ipv4_bytes[1],
                    ipv4_bytes[2],
                    ipv4_bytes[3]
                ))
            )),
            _ => {
                serror!("Invalid IPv4 address {}");
                Err(())
            }
        }
    }

    fn format_utf16_string(utf16_bytes: &[u8], little_endian: bool) -> Result<String, ()> {
        let raw_iter = utf16_bytes.chunks_exact(2).into_iter();

        if little_endian {
            let le_raw_data: Vec<u16> = raw_iter
                .map(|word| u16::from_le_bytes([word[0], word[1]]))
                .collect();

            match U16CString::from_vec_with_nul(le_raw_data) {
                Ok(le_data) => Ok(le_data.to_string_lossy()),
                Err(_) => {
                    serror!("Error constructing UTF16_LE string");
                    Err(())
                }
            }
        } else {
            let le_raw_data: Vec<u16> = raw_iter
                .map(|word| u16::from_be_bytes([word[0], word[1]]))
                .collect();

            match U16CString::from_vec_with_nul(le_raw_data) {
                Ok(le_data) => Ok(le_data.to_string_lossy()),
                Err(_) => {
                    serror!("Error constructing UTF16_BE string");
                    Err(())
                }
            }
        }
    }

    impl GrammerMetadata {
        pub fn new() -> GrammerMetadata {
            GrammerMetadata {
                name: String::from(""),
                fixed_size: false,
                size: 0,
            }
        }
    }

    impl Grammer {
        pub fn new() -> Grammer {
            Grammer {
                metadata: GrammerMetadata::new(),
                fields: Vec::new(),
            }
        }

        pub fn parse_toml(&mut self, file_contents: &FileData) -> Result<&mut Grammer, ()> {
            match toml::from_str::<Grammer>(&file_contents.grammer_contents) {
                Ok(gram) => {
                    *self = gram;
                    Ok(self)
                }
                Err(_) => {
                    serror!("Could not parse grammer file");
                    Err(())
                }
            }
        }

        // pub mod Format {

        // }
    }

    pub struct DissassOutput {
        pub output: String,
        pub line_count: u32,
    }

    impl DissassOutput {
        fn new() -> DissassOutput {
            DissassOutput {
                output: String::from(""),
                line_count: 0,
            }
        }

        fn format_x86(&mut self, bitness: u32, machine_code: &[u8]) {
            let mut decoder = Decoder::new(bitness, machine_code, DecoderOptions::NONE);
            let mut formatter = NasmFormatter::new();
            let mut instruction = Instruction::default();

            while decoder.can_decode() {
                decoder.decode_out(&mut instruction);

                formatter.format(&instruction, &mut self.output);
                self.output.push_str("\n");
                self.line_count += 1;
            }
        }
    }
}
