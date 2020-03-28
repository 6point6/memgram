use crate::arg_parse;
use hex::ToHex;
use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, NasmFormatter};
use prettytable::Table;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

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
pub struct Grammar {
    pub metadata: GrammerMetadata,
    pub fields: Vec<GrammerFields>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GrammerMetadata {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GrammerFields {
    pub name: String,
    pub size: usize,
    pub data_type: String,
    pub display_format: String,
    pub description: String,
}

pub struct TableData {
    pub field_hashmap: HashMap<String, Vec<u8>>,
    field_fmt_hashmap: HashMap<String, String>,
    field_str_hashmap: HashMap<String, String>,
    description_table: Table,
    standard_table: Table,
}

pub enum Tables {
    Standard,
    Description,
}

pub enum MatchResult {
    FailHard,
    FailSoft,
}

impl TableData {
    pub fn new() -> TableData {
        TableData {
            field_hashmap: HashMap::new(),
            field_fmt_hashmap: HashMap::new(),
            field_str_hashmap: HashMap::new(),
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

    pub fn fill_standard_table(
        &mut self,
        parsed_gram: &Grammar,
        mut struct_offset: usize,
    ) -> Result<&mut TableData, ()> {
        self.standard_table.add_row(row![
            "ID",
            "Field",
            "Offset",
            "Size",
            "Data Type",
            "Raw Data",
            "Formatted Data",
        ]);

        for (index, field) in parsed_gram.fields.iter().enumerate() {
            let raw_hex_string = self.field_str_hashmap.get(&field.name).ok_or_else(|| {
                serror!(format!("Could not get value for field: {}", field.name));
            })?;

            let formatted_data = self.field_fmt_hashmap.get(&field.name).ok_or_else(|| {
                serror!(format!("Could not get value for field: {}", field.name));
            })?;

            let name_length: usize = field.name.len() - 3;
            let id: String = field.name[name_length..].to_string();

            if index % 2 == 0 {
                self.standard_table
                        .add_row(row![bFG->id,bFG->field.name[..name_length],bFG->format!("{:#X}", struct_offset),bFG->format!("{:#X}",field.size),bFG->field.data_type,bFG->raw_hex_string,bFG->formatted_data]);
            } else {
                self.standard_table
                        .add_row(row![bFM->id,bFM->field.name[..name_length],bFM->format!("{:#X}", struct_offset),bFM->format!("{:#X}",field.size),bFM->field.data_type,bFM->raw_hex_string,bFM->formatted_data]);
            }

            struct_offset += field.size;
        }

        Ok(self)
    }

    pub fn fill_description_table(&mut self, parsed_gram: &Grammar) -> &mut TableData {
        self.description_table
            .add_row(row!["ID", "Field", "Description"]);

        for (index, field) in parsed_gram.fields.iter().enumerate() {
            let name_length: usize = field.name.len() - 3;
            let id: String = field.name[name_length..].to_string();

            if index % 2 == 0 {
                self.description_table
                    .add_row(row![bFG->id,bFG->field.name[..name_length],bFG->field.description]);
            } else {
                self.description_table
                    .add_row(row![bFM->id,bFM->field.name[..name_length],bFM->field.description]);
            }
        }
        self
    }

    pub fn create_field_hashmap(
        &mut self,
        parsed_gram: &Grammar,
        cmd_args: &arg_parse::CMDArgParse,
    ) -> Result<&mut TableData, ()> {
        let binary_file: &mut File = &mut File::open(&cmd_args.binary_filepath)
            .map_err(|_| serror!(format!("Could not open file: {}", cmd_args.binary_filepath)))?;

        check_filesize(
            binary_file,
            &cmd_args.binary_filepath,
            cmd_args.struct_offset,
            parsed_gram.get_struct_size() as u64,
        )?;

        binary_file
            .seek(SeekFrom::Start(cmd_args.struct_offset))
            .unwrap();

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

    pub fn format_fields(
        &mut self,
        parsed_gram: &Grammar,
        fmt_endian_flag: bool,
    ) -> Result<&mut TableData, ()> {
        for field in parsed_gram.fields.iter() {
            let mut raw_hex_string: String = self
                .field_hashmap
                .get(&field.name)
                .ok_or_else(|| serror!(format!("Failed to value for field: {}", field.name)))?
                .encode_hex::<String>()
                .to_uppercase();

            if raw_hex_string.len() > 25 {
                raw_hex_string = raw_hex_string[..25].to_string();
                raw_hex_string.push_str("...");
            }

            self.field_str_hashmap
                .insert(field.name.clone(), raw_hex_string.clone());

            let raw_data: &Vec<u8> = self.field_hashmap.get(&field.name).ok_or_else(|| {
                serror!(format!("Could not get value for field: {}", field.name));
            })?;

            let reverse_hex_string = || {
                let mut reversed_raw_data: Vec<u8> = raw_data.clone();
                reversed_raw_data.reverse();
                reversed_raw_data.encode_hex::<String>().to_uppercase()
            };

            let formatted_data = match &field.display_format[..] {
                HEXLE_TYPE => reverse_hex_string(),
                ASCII_TYPE => raw_data.iter().map(|ascii| *ascii as char).collect(),
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
                _ => {
                    if fmt_endian_flag {
                        reverse_hex_string()
                    } else {
                        raw_hex_string.clone()
                    }
                }
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
    let raw_iter = utf16_bytes.chunks_exact(2);

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

fn check_filesize(
    binary_file: &mut File,
    binary_path: &str,
    struct_offset: u64,
    struct_size: u64,
) -> Result<(), ()> {
    let file_size = binary_file.seek(SeekFrom::End(0)).unwrap();

    if file_size >= struct_offset + struct_size {
        Ok(())
    } else {
        serror!(format!(
            "((offset: {}) + (structure size: {})) is larger than the filesize: {} of {}",
            struct_offset, struct_size, file_size, binary_path
        ));
        Err(())
    }
}

impl GrammerMetadata {
    pub fn new() -> GrammerMetadata {
        GrammerMetadata {
            name: String::from(""),
        }
    }
}

impl Grammar {
    pub fn new() -> Grammar {
        Grammar {
            metadata: GrammerMetadata::new(),
            fields: Vec::new(),
        }
    }

    pub fn parse_toml(&mut self, file_contents: &str) -> Result<&mut Grammar, ()> {
        match toml::from_str::<Grammar>(file_contents) {
            Ok(gram) => {
                *self = gram;
            }
            Err(e) => {
                serror!(format!("Could not parse grammar file, because {}", e));
                return Err(());
            }
        }

        let mut field_id: u32 = 0;

        for field in &mut self.fields {
            field.name.push_str(&format!("{:03X}", field_id)[..]);
            field_id += 1;
        }

        Ok(self)
    }

    pub fn get_struct_size(&self) -> usize {
        let mut struct_size: usize = 0;

        for field in &self.fields {
            struct_size += field.size;
        }

        struct_size
    }

    pub fn pre_parse_toml(&mut self, file_contents: &mut String) -> Result<&mut Grammar, ()> {
        self.parse_variable_length(file_contents)?
            .expand_fields(file_contents)?;
        Ok(self)
    }

    fn parse_variable_length(&mut self, file_contents: &mut String) -> Result<&mut Grammar, ()> {
        let mut search_start_index: usize = 0;

        loop {
            let field_start_end: (usize, usize) =
                match get_match_start_end_newline(search_start_index, &file_contents, "size =") {
                    Ok(matched_start_end) => matched_start_end,
                    Err(e) => match e {
                        MatchResult::FailSoft => break,
                        MatchResult::FailHard => return Err(()),
                    },
                };

            search_start_index = field_start_end.1;

            let var_entry_start_end: (usize, usize) = match get_match_start_end(
                field_start_end.0,
                field_start_end.1,
                &file_contents,
                "(",
                ")",
                false,
            ) {
                Ok(start_end) => start_end,
                Err(match_result) => match match_result {
                    MatchResult::FailHard => return Err(()),
                    MatchResult::FailSoft => continue,
                },
            };

            println!(
                "START: {}\nMATCHSTART: {}\nMATCHEND: {}\n",
                search_start_index, var_entry_start_end.0, var_entry_start_end.1
            );

            let var_name_start_end: (usize, usize) = match get_match_start_end( // remove this and parse one byte at a time
                var_entry_start_end.0,
                var_entry_start_end.1,
                &file_contents,
                "'",
                "'",
                true,
            ) {
                Ok(start_end) => start_end,
                Err(match_result) => match match_result {
                    MatchResult::FailHard => return Err(()),
                    MatchResult::FailSoft => return Ok(self),
                },
            };

            println!(
                "START: {}\nMATCHSTART: {}\nMATCHEND: {}\n",
                search_start_index, var_name_start_end.0, var_name_start_end.1
            );

            let variable_name = 
                file_contents[var_name_start_end.0..var_name_start_end.1].to_string(); // Save var name, save offset bracket start, bracket end ( in a vector of struct?), then search names match replace all

            println!("var name: {}", variable_name);
        }

        Ok(self)
    }

    fn expand_fields(&mut self, file_contents: &mut String) -> Result<&mut Grammar, ()> {
        let mut search_index: usize = 0; // Note, implement get_field_start_end() for this function and support for just \n
        // let mut crlf_flag = false;
        
        loop {
            search_index += match file_contents[search_index..].find("[[fields]] *") {
                Some(matched_index) => matched_index,
                None => return Ok(self),
            };

            let next_line_index = match file_contents[search_index + 13..].find("\r\n") {
                Some(matched_index) => matched_index + search_index + 13,
                None => {
                    serror!("Could not find CRLF after field multiplier");
                    return Err(());
                }
            };

            let multiple: u32 = match file_contents[search_index + 13..next_line_index]
                .trim()
                .parse::<u32>()
            {
                Ok(mul) => mul,
                Err(_) => {
                    serror!("Could not parse field multiplier");
                    return Err(());
                }
            };

            file_contents.replace_range(search_index + 10..next_line_index, "    ");

            let field_end_index: usize = file_contents[search_index..]
                .find("\r\n\r\n")
                .ok_or_else(|| serror!("Could not find CRLF after multiplied field"))?;

            let mut multiplied_field = String::from("");

            for _i in 1..multiple {
                multiplied_field.push_str(
                    &file_contents[search_index..search_index + field_end_index].to_string(),
                );
                multiplied_field.push_str("\r\n\r\n");
            }

            file_contents.insert_str(search_index, &multiplied_field[..]);
        }
    }
}

fn get_match_start_end_newline(
    search_start: usize,
    file_contents: &str,
    match_start: &str,
) -> Result<(usize, usize), MatchResult> {
    let match_start_index = match file_contents[search_start..].find(match_start) {
        Some(matched_index) => search_start + matched_index,
        None => return Err(MatchResult::FailSoft),
    };

    let match_end_index: usize = match file_contents[match_start_index..].find("\r\n") {
        Some(matched_index) => match_start_index + matched_index,
        None => match file_contents[match_start_index..].find("\n") {
            Some(matched_index) => match_start_index + matched_index,
            None => {
                serror!(format!("Could not find newline after: {}", match_start));
                return Err(MatchResult::FailHard);
            }
        },
    };

    Ok((match_start_index, match_end_index))
}

fn get_match_start_end(
    search_start: usize,
    search_end: usize,
    file_contents: &str,
    match_start: &str,
    match_end: &str,
    fail_no_find: bool,
) -> Result<(usize, usize), MatchResult> {
    let match_start_index = match file_contents[search_start..search_end].find(match_start) {
        Some(matched_index) => search_start + matched_index + 1,
        None => {
            if fail_no_find {
                serror!(format!("Could not find match start: {}", match_start));
                return Err(MatchResult::FailHard);
            }
            return Err(MatchResult::FailSoft);
        }
    };

    let match_end_index: usize = match file_contents[match_start_index..search_end].find(match_end)
    {
        Some(matched_index) => match_start_index + matched_index,
        None => {
            serror!(format!(
                "Could not find match end: {} for match start: {}",
                match_end, match_start
            ));
            return Err(MatchResult::FailHard);
        }
    };

    Ok((match_start_index, match_end_index))
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
