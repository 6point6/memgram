use crate::file_parse;
use std::fs::File;
use std::io::prelude::*;

pub struct CStruct {
    pub name: String,
    pub fields: Vec<(String, String)>,
    pub toml_string: String,
}

impl CStruct {
    pub fn new() -> CStruct {
        CStruct {
            name: String::from(""),
            fields: Vec::new(),
            toml_string: String::from(""),
        }
    }

    pub fn parse_c_struct(&mut self, struct_filepath: &str) -> Result<&mut CStruct, ()> {
        let mut prev_index: usize = 0;
        let mut next_index: usize = 0;

        let struct_string: String = file_parse::read_file(struct_filepath)?;

        prev_index += match struct_string[prev_index..].find("struct ") {
            Some(matched_index) => matched_index + 7,
            None => {
                serror!("Invalid C struct: could not find 'struct' keyword");
                return Err(());
            }
        };

        next_index += match struct_string[prev_index..].find('{') {
            Some(matched_index) => matched_index + prev_index,
            None => {
                serror!("Invalid C struct: could not find opening '{'");
                return Err(());
            }
        };

        let last_index: usize = match struct_string[next_index..].find("};") {
            Some(matched_index) => matched_index + next_index,
            None => {
                serror!("Invalid C struct: could not find closing '};'");
                return Err(());
            }
        };

        let struct_name: &str = &struct_string[prev_index..next_index];

        self.name = struct_name.trim().to_string();

        let words = struct_string[next_index + 1..last_index].split_ascii_whitespace();

        let mut entry_num = 0;

        for word in words {
            if word.ends_with(';') {
                match self.fields.get_mut(entry_num) {
                    Some(value) => {
                        value.1.push_str(&word[..word.len() - 1]);
                        entry_num += 1;
                    }
                    None => {
                        serror!(format!(
                            "Type must be specified in C struct before field name: {}",
                            word
                        ));
                        return Err(());
                    }
                }
            } else {
                match self.fields.get_mut(entry_num) {
                    Some(value) => {
                        value.0.push_str(" ");
                        value.0.push_str(word)
                    }
                    None => self
                        .fields
                        .insert(entry_num, (word.to_string(), "".to_string())),
                }
            }
        }
        Ok(self)
    }

    pub fn build_toml_string(&mut self) -> Result<&mut CStruct, ()> {
        self.toml_string.push_str("[metadata]\r\n");
        self.toml_string
            .push_str(&format!("\tname = '{}'\r\n", self.name)[..]);
        self.toml_string.push_str("\tvariable_fields = ['']\r\n");
        self.toml_string.push_str("\trepeat_fields = ['']\r\n");

        for field in self.fields.iter() {
            self.toml_string.push_str("\r\n[[fields]]\r\n");
            self.toml_string
                .push_str(&format!("\tname = '{}'\r\n", field.1));
            self.toml_string
                .push_str(&format!("\tsize = {}\r\n", get_field_size(&field.0)?));
            self.toml_string
                .push_str(&format!("\tdata_type = '{}'\r\n", field.0));
            self.toml_string.push_str("\tdisplay_format = 'hex'\r\n");
            self.toml_string.push_str("\tdescription = 'N/A'\r\n");
        }
        Ok(self)
    }

    pub fn write_toml_file(&mut self, output_path: &str) -> Result<&mut CStruct, ()> {
        let mut grammer_file = match File::create(output_path) {
            Ok(f) => f,
            Err(e) => {
                serror!(format!(
                    "Could not create/open file {}, because {} ",
                    output_path, e
                ));
                return Err(());
            }
        };

        match grammer_file.write_all(self.toml_string.as_bytes()) {
            Ok(_) => {
                println!(
                    "[+] Successfully converted C struct {} to grammar file {}",
                    self.name, output_path
                );
                Ok(self)
            }
            Err(e) => {
                serror!(format!(
                    "Could not write to file: {}, because {}",
                    output_path, e
                ));
                Err(())
            }
        }
    }
}

fn get_field_size(field_type: &str) -> Result<&str, ()> {
    let l_field_type = field_type.to_lowercase();
    match &l_field_type[..] {
        "char" | "signed char" | "unsigned char" => Ok("0x01"),
        "short" | "short int" | "signed short" | "signed short int" | "unsigned short"
        | "unsigned short int" => Ok("0x02"),
        "float" | "int" | "signed" | "signed int" | "unsigned" | "unsigned int" | "long"
        | "long int" | "signed long" | "signed long int" | "unsigned long"
        | "unsigned long int" => Ok("0x04"),
        "double"
        | "long long"
        | "long long int"
        | "signed long long"
        | "signed long long int"
        | "unsigned long long"
        | "unsigned long long int" => Ok("0x08"),
        "long double" => Ok("0x10"),
        _ => {
            serror!(format!("Type: {}, is not supported", l_field_type));
            Err(())
        }
    }
}
