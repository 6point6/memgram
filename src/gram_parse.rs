use crate::arg_parse;
use hex::ToHex;
use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, NasmFormatter};
use prettytable::Table;
use serde::Deserialize;
use serde::Serialize;
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Grammar {
    pub metadata: GrammerMetadata,
    pub fields: Vec<GrammerFields>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GrammerMetadata {
    pub name: String,
    pub variable_fields: Vec<String>,
    pub multiply_fields: Vec<(String, String)>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
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
            variable_fields: Vec::new(),
            multiply_fields: Vec::new(),
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

        Ok(self)
    }

    pub fn get_struct_size(&self) -> usize {
        let mut struct_size: usize = 0;

        for field in &self.fields {
            struct_size += field.size;
        }

        struct_size
    }

    pub fn post_parse_toml(&mut self) -> Result<&mut Grammar, ()> {
        
        self.multiply_fields()?;

        self.add_field_id();

        Ok(self)
    }

    fn add_field_id(&mut self) {
        let mut field_id: u32 = 0;

        for field in &mut self.fields {
            field.name.push_str(&format!("{:03X}", field_id)[..]);
            field_id += 1;
        }
    }

    fn multiply_fields(&mut self) -> Result<(), ()> {
        if self.metadata.multiply_fields[0].0.is_empty()
            && self.metadata.multiply_fields[0].1.is_empty()
        {
            return Ok(());
        }

        for multiplier_entry in self.metadata.multiply_fields.iter() {
            let mut field_multiply = FieldMultiply::new();

            let entry_0 = multiplier_entry.0.as_str();
            let entry_1 = multiplier_entry.1.as_str();

            for (index, field) in self.fields.iter().enumerate() {
                if &field.name[..] == entry_0 {
                    field_multiply.field_name = field.name.clone();
                    field_multiply.field_index = index;

                    match entry_1.parse::<i32>() {
                        Ok(multiplier) => {
                            field_multiply.multiplier = multiplier;
                            break;
                        }
                        Err(_) => {
                            serror!(format!(
                                "Could not convert multiplier for field: {} to an interger",
                                field.name
                            ));
                            return Err(());
                        }
                    }
                } else if &field.name[..] == entry_1 {
                    field_multiply.field_name = field.name.clone();
                    field_multiply.field_index = index;

                    match entry_0.parse::<i32>() {
                        Ok(multiplier) => {
                            field_multiply.multiplier = multiplier;
                            break;
                        }
                        Err(_) => {
                            serror!(format!(
                                "Could not convert multiplier for field: {} to an interger",
                                field.name
                            ));
                            return Err(());
                        }
                    }
                } else {
                    continue;
                }
            }

            if field_multiply.field_name.is_empty() || field_multiply.multiplier == 0 {
                serror! {"Could not find multiply field name or multiplier is 0"};
                return Err(());
            }

            for _x in 1..field_multiply.multiplier {
                self.fields.insert(field_multiply.field_index,self.fields[field_multiply.field_index].clone());
            }
            
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct FieldMultiply {
    pub field_name: String,
    pub field_index: usize,
    pub multiplier: i32,
}

impl FieldMultiply {
    fn new() -> FieldMultiply {
        FieldMultiply {
            field_name: String::from(""),
            field_index: 0,
            multiplier: 0,
        }
    }
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
