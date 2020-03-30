use crate::arg_parse;
use hex::ToHex;
use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, NasmFormatter};
use prettytable::Table;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::{self, BufReader};

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
    pub variable_size_fields: Vec<(String, String, String, String)>,
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

            let field_id = format!("{:03X}", index);

            if index % 2 == 0 {
                self.standard_table
                        .add_row(row![bFG->field_id,bFG->field.name,bFG->format!("{:#X}", struct_offset),bFG->format!("{:#X}",field.size),bFG->field.data_type,bFG->raw_hex_string,bFG->formatted_data]);
            } else {
                self.standard_table
                        .add_row(row![bFM->field_id,bFM->field.name,bFM->format!("{:#X}", struct_offset),bFM->format!("{:#X}",field.size),bFM->field.data_type,bFM->raw_hex_string,bFM->formatted_data]);
            }

            struct_offset += field.size;
        }

        Ok(self)
    }

    pub fn fill_description_table(&mut self, parsed_gram: &Grammar) -> &mut TableData {
        self.description_table
            .add_row(row!["ID", "Field", "Description"]);

        for (index, field) in parsed_gram.fields.iter().enumerate() {
            let field_id = format!("{:03X}", index);

            if index % 2 == 0 {
                self.description_table
                    .add_row(row![bFG->field_id,bFG->field.name[..],bFG->field.description]);
            } else {
                self.description_table
                    .add_row(row![bFM->field_id,bFM->field.name[..],bFM->field.description]);
            }
        }
        self
    }

    pub fn create_var_size_hashmap(
        &mut self,
        parsed_gram: &mut Grammar,
        binary_file: &mut File,
        var_sized_fields_vec: &mut Vec<VariableSizeEntry>,
    ) -> Result<(), ()> {
        let eof = binary_file.metadata().unwrap().len() as i64;

        for field in &mut parsed_gram.fields {
            for entry in var_sized_fields_vec.iter_mut() {
                if field.name == entry.var_field_name {
                    match entry.variable_options {
                        VariableOptions::NullChar => {
                            let current_position =
                                binary_file.seek(SeekFrom::Current(0)).unwrap() as i64;
                            let mut read_size: i64 = 512;
                            let mut byte_buffer: Vec<u8> = Vec::new();
                            let mut prev_null = false;

                            if read_size + current_position <= eof {
                                read_size = 512;
                            } else {
                                read_size =
                                    eof - binary_file.seek(SeekFrom::Current(0)).unwrap() as i64;
                            }

                            byte_buffer.append(
                                &mut binary_file
                                    .bytes()
                                    .take(eof as usize - current_position as usize)
                                    .map(|r: Result<u8, _>| r.unwrap())
                                    .collect(),
                            );

                            for (index, byte) in byte_buffer.iter().enumerate() {
                                if *byte == 0x00u8 {
                                    prev_null = true;
                                } else if (*byte != 0x00) && prev_null {
                                    field.size = index;
                                    binary_file
                                        .seek(SeekFrom::Start(current_position as u64))
                                        .unwrap();
                                    break;
                                }
                            }
                        }
                        VariableOptions::NoOptions => {
                            let raw_field_data: Vec<u8> = self
                            .field_hashmap
                            .get(&entry.source_field_name)
                            .ok_or_else(|| {
                                serror!(format!(
                                    "Source field name: {}, should appear before variable field name: {}",
                                    entry.source_field_name, entry.var_field_name
                                ))
                            })?
                            .clone();

                            if &entry.source_field_display[..] == HEXLE_TYPE {
                                entry.convert_field_size(
                                    &raw_field_data,
                                    ConvertEndianess::LittleEndian,
                                )?;
                            } else {
                                entry.convert_field_size(
                                    &raw_field_data,
                                    ConvertEndianess::BigEndian,
                                )?;
                            }

                            field.size = entry.calculate_variable_size();
                        }
                    }
                }

                let pos_after_read =
                    binary_file.seek(SeekFrom::Current(0)).unwrap() as usize + field.size;

                if eof <= pos_after_read as i64 {
                    serror!(format!("Reached EOF"));
                    return Ok(());
                }
                self.field_hashmap.insert(
                    field.name.to_string(),
                    binary_file
                        .bytes()
                        .take(field.size)
                        .map(|r: Result<u8, _>| r.unwrap())
                        .collect(),
                );
            }
        }

        Ok(())
    }

    pub fn create_field_hashmap(
        &mut self,
        parsed_gram: &mut Grammar,
        cmd_args: &arg_parse::CMDArgParse,
    ) -> Result<&mut TableData, ()> {
        let binary_file: &mut File = &mut File::open(&cmd_args.binary_filepath)
            .map_err(|_| serror!(format!("Could not open file: {}", cmd_args.binary_filepath)))?;

        let eof = binary_file.metadata().unwrap().len() as i64;

        binary_file
            .seek(SeekFrom::Start(cmd_args.struct_offset))
            .unwrap();

        let mut var_sized_fields_vec: Vec<VariableSizeEntry> = Vec::new();
        parsed_gram.create_var_size_entry_vector(&mut var_sized_fields_vec)?;

        if !parsed_gram.metadata.variable_size_fields[0].3.is_empty() {
            self.create_var_size_hashmap(parsed_gram, binary_file, &mut var_sized_fields_vec)?;
        } else {
            for field in &parsed_gram.fields {
                let pos_after_read =
                    binary_file.seek(SeekFrom::Current(0)).unwrap() as usize + field.size;

                if eof <= pos_after_read as i64 {
                    serror!(format!("Structure size after read: {}, will be larger than file size: {} after next read",pos_after_read,eof));
                    return Err(());
                }
                self.field_hashmap.insert(
                    field.name.to_string(),
                    binary_file
                        .bytes()
                        .take(field.size)
                        .map(|r: Result<u8, _>| r.unwrap())
                        .collect(),
                );
            }
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
                .ok_or_else(|| serror!(format!("Failed to get value for field: {}", field.name)))?
                .encode_hex::<String>()
                .to_uppercase();

            if raw_hex_string.len() > 25 {
                raw_hex_string = raw_hex_string[..25].to_string();
                raw_hex_string.push_str("...");
            }

            self.field_str_hashmap
                .insert(field.name.clone(), raw_hex_string.clone());

            let raw_field_data: &Vec<u8> =
                self.field_hashmap.get(&field.name).ok_or_else(|| {
                    serror!(format!("Could not get value for field: {}", field.name));
                })?;

            let reverse_hex_string = || {
                let mut reversed_raw_field_data: Vec<u8> = raw_field_data.clone();
                reversed_raw_field_data.reverse();
                reversed_raw_field_data
                    .encode_hex::<String>()
                    .to_uppercase()
            };

            let formatted_data = match &field.display_format[..] {
                HEXLE_TYPE => reverse_hex_string(),
                ASCII_TYPE => raw_field_data.iter().map(|ascii| *ascii as char).collect(),
                IPV4BE_TYPE => format_ipv4_string(&raw_field_data)?,
                IPV4LE_TYPE => {
                    let mut reversed_raw_field_data: Vec<u8> = raw_field_data.clone();
                    reversed_raw_field_data.reverse();
                    format_ipv4_string(&reversed_raw_field_data)?
                }
                UTF16BE_TYPE => format_utf16_string(raw_field_data, false)?,
                UTF16LE_TYPE => format_utf16_string(raw_field_data, true)?,
                X86_TYPE => {
                    let mut x86_disassembly = DissassOutput::new();
                    x86_disassembly.format_x86(16, raw_field_data);

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
        let le_raw_field_data: Vec<u16> = raw_iter
            .map(|word| u16::from_le_bytes([word[0], word[1]]))
            .collect();

        match U16CString::from_vec_with_nul(le_raw_field_data) {
            Ok(le_data) => Ok(le_data.to_string_lossy()),
            Err(_) => {
                serror!("Error constructing UTF16_LE string");
                Err(())
            }
        }
    } else {
        let le_raw_field_data: Vec<u16> = raw_iter
            .map(|word| u16::from_be_bytes([word[0], word[1]]))
            .collect();

        match U16CString::from_vec_with_nul(le_raw_field_data) {
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
            variable_size_fields: Vec::new(),
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
        if !self.metadata.multiply_fields[0].0.is_empty()
            && !self.metadata.multiply_fields[0].1.is_empty()
        {
            self.multiply_fields()?;
        }

        Ok(self)
    }

    // fn add_field_id(&mut self) {
    //     let mut field_id: u32 = 0;

    //     for field in &mut self.fields {
    //         field.name.push_str(&format!("{:03X}", field_id)[..]);
    //         field_id += 1;
    //     }
    // }

    fn create_var_size_entry_vector(
        &mut self,
        var_size_entry_vec: &mut Vec<VariableSizeEntry>,
    ) -> Result<(), ()> {

        if self.metadata.variable_size_fields[0].0.is_empty() && self.metadata.variable_size_fields[0].1.is_empty() && self.metadata.variable_size_fields[0].2.is_empty() && self.metadata.variable_size_fields[0].3.is_empty() {
            return Ok(());
        }

        let mut variable_size_vector: Vec<VariableSizeEntry> = Vec::new();

        for entry in self.metadata.variable_size_fields.iter() {
            let mut variable_size_entry = VariableSizeEntry::new();

            let entry_0 = entry.0.as_str();
            let entry_1 = entry.1.as_str();
            let entry_2 = entry.2.as_str();
            let entry_3 = entry.3.as_str();

            for (index, field) in self.fields.iter().enumerate() {
                if &field.name[..] == entry_0 {
                    variable_size_entry.source_field_name = field.name.clone();
                    variable_size_entry.source_field_display = field.display_format.clone();
                    variable_size_entry.source_field_index = index;

                    if !entry_1.is_empty() && !entry_2.is_empty() {
                        variable_size_entry.arithmetic_operator =
                            get_var_arithmetic_operator(entry_1.trim())?;

                        variable_size_entry.adjustment = match entry_2.trim().parse::<usize>() {
                            Ok(adjustment) => adjustment,
                            Err(e) => {
                                serror!(format!(
                                    "Could not convert variable size adjustment: {} because, {}",
                                    entry_2.trim(),
                                    e
                                ));
                                return Err(());
                            }
                        };

                        variable_size_entry.arithemitc_order =
                            VariableSizeArithmeticOrder::Forwards;
                    }
                } else if &field.name[..] == entry_2 {
                    variable_size_entry.source_field_name = field.name.clone();
                    variable_size_entry.source_field_display = field.display_format.clone();
                    variable_size_entry.source_field_index = index;

                    if !entry_0.is_empty() && !entry_1.is_empty() {
                        variable_size_entry.arithmetic_operator =
                            get_var_arithmetic_operator(entry_1.trim())?;

                        variable_size_entry.adjustment = match entry_0.trim().parse::<usize>() {
                            Ok(adjustment) => adjustment,
                            Err(e) => {
                                serror!(format!(
                                    "Could not convert variable size adjustment: {} because, {}",
                                    entry_2.trim(),
                                    e
                                ));
                                return Err(());
                            }
                        };

                        variable_size_entry.arithemitc_order =
                            VariableSizeArithmeticOrder::Backwards;
                    }
                } else if &field.name[..] == entry_3 {
                    variable_size_entry.var_field_name = field.name.clone();
                    if entry_1.trim().to_lowercase() == "null" {
                        variable_size_entry.variable_options = VariableOptions::NullChar;
                    }
                }
            }

            if variable_size_entry.var_field_name.is_empty() {
                serror!(format!(
                    "Variable field name: {}, does not exist for variable size fields",
                    entry_3
                ));
                return Err(());
            } else if variable_size_entry.source_field_name.is_empty() {
                match variable_size_entry.variable_options {
                    VariableOptions::NullChar => (),
                    VariableOptions::NoOptions => match entry_0.parse::<i32>() {
                        Ok(_) => {
                            serror!(format!(
                                "Source field name: {} does not exist as a field in grammar",
                                entry_2
                            ));
                            return Err(());
                        }
                        Err(_) => {
                            serror!(format!(
                                "Source field name: {} does not exist as a field in grammar",
                                entry_0
                            ));
                            return Err(());
                        }
                    },
                }
            }

            variable_size_vector.push(variable_size_entry);
        }

        *var_size_entry_vec = variable_size_vector;

        Ok(())
    }

    fn multiply_fields(&mut self) -> Result<(), ()> {
        for entry in self.metadata.multiply_fields.iter() {
            let mut field_multiply = FieldMultiply::new();

            let entry_0 = entry.0.as_str();
            let entry_1 = entry.1.as_str();

            for (index, field) in self.fields.iter().enumerate() {
                if &field.name[..] == entry_0 {
                    field_multiply.field_name = field.name.clone();
                    field_multiply.field_index = index;

                    match entry_1.parse::<i32>() {
                        Ok(multiplier) => {
                            field_multiply.multiplier = multiplier;
                            break;
                        }
                        Err(e) => {
                            serror!(format!(
                                "Could not convert multiplier for field: {} to an interger, because {}",
                                field.name, e
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
                        Err(e) => {
                            serror!(format!(
                                "Could not convert multiplier for field: {} to an interger, because {}",
                                field.name, e));
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

            for _x in 0..field_multiply.multiplier - 1 {
                self.fields.insert(
                    field_multiply.field_index,
                    self.fields[field_multiply.field_index].clone(),
                );
            }
        }
        Ok(())
    }
}

fn get_var_arithmetic_operator(arithmetic_op_str: &str) -> Result<ArithmeticOperators, ()> {
    if arithmetic_op_str.len() > 1 {
        serror!(format!("Invalid arithmetic operator legnth: {} for variable size fields: {}, must be one of the following (+, -, *, /)", arithmetic_op_str.len(),arithmetic_op_str));
        return Err(());
    }
    match arithmetic_op_str {
        "+" => Ok(ArithmeticOperators::Addition),
        "-" => Ok(ArithmeticOperators::Subtraction),
        "*" => Ok(ArithmeticOperators::Multiplication),
        "/" => Ok(ArithmeticOperators::Division),
        _ => {
            serror!(format!("Invalid arithmetic operator for variable size fields: {}, must be one of the following (+, -, *, /)",arithmetic_op_str));
            Err(())
        }
    }
}

#[derive(Debug)]
pub struct VariableSizeEntry {
    pub source_field_name: String,
    pub source_field_index: usize,
    pub source_field_real_size: usize,
    pub source_field_display: String,
    pub var_field_name: String,
    pub variable_options: VariableOptions,
    pub arithemitc_order: VariableSizeArithmeticOrder,
    pub arithmetic_operator: ArithmeticOperators,
    pub adjustment: usize,
}

pub enum ConvertEndianess {
    BigEndian,
    LittleEndian,
}

#[derive(Debug)]
pub enum VariableOptions {
    NoOptions,
    NullChar,
}

impl VariableSizeEntry {
    fn new() -> VariableSizeEntry {
        VariableSizeEntry {
            source_field_name: String::from(""),
            source_field_index: 0,
            source_field_real_size: 0,
            source_field_display: String::from(""),
            var_field_name: String::from(""),
            variable_options: VariableOptions::NoOptions,
            arithemitc_order: VariableSizeArithmeticOrder::Unset,
            arithmetic_operator: ArithmeticOperators::Addition,
            adjustment: 0,
        }
    }

    pub fn calculate_variable_size(&mut self) -> usize {
        match self.arithemitc_order {
            VariableSizeArithmeticOrder::Unset => self.source_field_real_size,
            VariableSizeArithmeticOrder::Forwards => match self.arithmetic_operator {
                ArithmeticOperators::Addition => self.source_field_real_size + self.adjustment,
                ArithmeticOperators::Subtraction => self.source_field_real_size - self.adjustment,
                ArithmeticOperators::Multiplication => {
                    self.source_field_real_size * self.adjustment
                }
                ArithmeticOperators::Division => self.source_field_real_size / self.adjustment,
            },
            VariableSizeArithmeticOrder::Backwards => match self.arithmetic_operator {
                ArithmeticOperators::Addition => self.adjustment + self.source_field_real_size,
                ArithmeticOperators::Subtraction => self.adjustment - self.source_field_real_size,
                ArithmeticOperators::Multiplication => {
                    self.adjustment * self.source_field_real_size
                }
                ArithmeticOperators::Division => self.adjustment / self.source_field_real_size,
            },
        }
    }

    pub fn convert_field_size(
        &mut self,
        raw_field_data: &Vec<u8>,
        endianess: ConvertEndianess,
    ) -> Result<(), ()> {
        match endianess {
            ConvertEndianess::LittleEndian => {
                self.source_field_real_size = match raw_field_data.len() {
                    2 => i16::from_le_bytes(raw_field_data[..2].try_into().unwrap()) as usize,
                    4 => i32::from_le_bytes(raw_field_data[..4].try_into().unwrap()) as usize,
                    8 => i64::from_le_bytes(raw_field_data[..8].try_into().unwrap()) as usize,
                    16 => i128::from_le_bytes(raw_field_data[..16].try_into().unwrap()) as usize,
                    _ => {
                        serror!(format!(
                            "Could not convert raw_field data to little endian because unsupported variable field size: {}",
                            raw_field_data.len()
                        ));
                        return Err(());
                    }
                };
            }
            ConvertEndianess::BigEndian => {
                self.source_field_real_size = match raw_field_data.len() {
                    2 => i16::from_be_bytes(raw_field_data[..2].try_into().unwrap()) as usize,
                    4 => i32::from_be_bytes(raw_field_data[..4].try_into().unwrap()) as usize,
                    8 => i64::from_be_bytes(raw_field_data[..8].try_into().unwrap()) as usize,
                    16 => i128::from_be_bytes(raw_field_data[..16].try_into().unwrap()) as usize,
                    _ => {
                        serror!(format!(
                            "Could not convert raw_field data to big endian because unsupported variable field size: {}",
                            raw_field_data.len()
                        ));
                        return Err(());
                    }
                };
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum VariableSizeArithmeticOrder {
    Forwards,
    Backwards,
    Unset,
}

#[derive(Debug)]
pub enum ArithmeticOperators {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

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
