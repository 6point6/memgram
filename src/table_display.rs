use crate::arg_parse;
use crate::gram_parse;
use hex::ToHex;
use prettytable::Table;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

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
    pub fn new() -> Self {
        Self {
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
        parsed_gram: &gram_parse::Grammar,
        mut struct_offset: usize,
    ) -> Result<&mut Self, ()> {
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

    pub fn fill_description_table(&mut self, parsed_gram: &gram_parse::Grammar) -> &mut Self {
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
        parsed_gram: &mut gram_parse::Grammar,
        binary_file: &mut File,
        var_sized_fields_vec: &mut Vec<gram_parse::VariableSizeEntry>,
    ) -> Result<(), ()> {
        let eof = binary_file.metadata().unwrap().len() as i64;

        for field in &mut parsed_gram.fields {
            for entry in var_sized_fields_vec.iter_mut() {
                if field.name == entry.var_field_name {
                    match entry.variable_options {
                        gram_parse::VariableOptions::NullChar => {
                            let current_position =
                                binary_file.seek(SeekFrom::Current(0)).unwrap() as i64;
                            let mut read_size: i64 = 512;
                            let mut byte_buffer: Vec<u8> = Vec::new();
                            let mut prev_null = false;

                            if read_size + current_position <= eof {
                                read_size = 512;
                            } else {
                                read_size = eof - current_position;
                            }

                            byte_buffer.append(
                                &mut binary_file
                                    .bytes()
                                    .take(read_size as usize)
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
                        gram_parse::VariableOptions::NoOptions => {
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

                            if &entry.source_field_display[..] == gram_parse::HEXLE_TYPE {
                                entry.convert_field_size(
                                    &raw_field_data,
                                    gram_parse::ConvertEndianess::LittleEndian,
                                )?;
                            } else {
                                entry.convert_field_size(
                                    &raw_field_data,
                                    gram_parse::ConvertEndianess::BigEndian,
                                )?;
                            }

                            field.size = entry.calculate_variable_size();
                        }
                    }
                }

                let pos_after_read =
                    binary_file.seek(SeekFrom::Current(0)).unwrap() as usize + field.size;

                if eof <= pos_after_read as i64 {
                    serror!("Reached EOF");
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
        parsed_gram: &mut gram_parse::Grammar,
        cmd_args: &arg_parse::CMDArgParse,
    ) -> Result<&mut Self, ()> {
        let binary_file: &mut File = &mut File::open(&cmd_args.binary_filepath)
            .map_err(|_| serror!(format!("Could not open file: {}", cmd_args.binary_filepath)))?;

        let eof = binary_file.metadata().unwrap().len() as i64;

        binary_file
            .seek(SeekFrom::Start(cmd_args.struct_offset))
            .unwrap();

        let mut var_sized_fields_vec: Vec<gram_parse::VariableSizeEntry> = Vec::new();
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
        parsed_gram: &gram_parse::Grammar,
        fmt_endian_flag: bool,
    ) -> Result<&mut Self, ()> {
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
                gram_parse::HEXLE_TYPE => reverse_hex_string(),
                gram_parse::ASCII_TYPE => {
                    raw_field_data.iter().map(|ascii| *ascii as char).collect()
                }
                gram_parse::IPV4BE_TYPE => gram_parse::format_ipv4_string(&raw_field_data)?,
                gram_parse::IPV4LE_TYPE => {
                    let mut reversed_raw_field_data: Vec<u8> = raw_field_data.clone();
                    reversed_raw_field_data.reverse();
                    gram_parse::format_ipv4_string(&reversed_raw_field_data)?
                }
                gram_parse::UTF16BE_TYPE => gram_parse::format_utf16_string(raw_field_data, false)?,
                gram_parse::UTF16LE_TYPE => gram_parse::format_utf16_string(raw_field_data, true)?,
                gram_parse::X86_TYPE => {
                    let mut x86_disassembly = gram_parse::DissassOutput::new();
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
