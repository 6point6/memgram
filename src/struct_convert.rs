//! Module for converting a C struct to the grammar format.
use std::fs;
use std::io::prelude::*;

/// Holds the contents of the parsed C struct fields and converted grammar contents.
pub struct CStruct {
    /// Name of the C struct (Also name of grammar).
    pub name: String,
    /// Vector of (C type, field_name) created by `parse_c_struct`.
    pub fields: Vec<(String, String)>,
    /// String containing the resulting grammar contents.
    pub grammar_contents: String,
}

impl CStruct {
    pub fn new() -> Self {
        Self {
            name: String::from(""),
            fields: Vec::new(),
            grammar_contents: String::from(""),
        }
    }

    /// Parses the contents of a file containing a C struct.
    ///
    /// A `Vector` of `(String, String)` is built in `self.fields` in the format (c type, field_name)
    pub fn parse_c_struct(&mut self, struct_filepath: &str) -> Result<&mut Self, ()> {
        let mut prev_index: usize = 0;
        let mut next_index: usize = 0;

        let struct_string: String = fs::read_to_string(struct_filepath).map_err(|e| {
            serror!(format!(
                "Error opening file: {}, because:{}",
                struct_filepath, e
            ))
        })?;

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

    /// Builds the contents of the output grammar file line by line.
    pub fn build_grammar_contents(&mut self) -> Result<&mut Self, ()> {
        self.grammar_contents.push_str("[metadata]\r\n");
        self.grammar_contents
            .push_str(&format!("\tname = '{}'\r\n", self.name)[..]);
        self.grammar_contents
            .push_str("\tvariable_size_fields = [['','','','']]\r\n");
        self.grammar_contents
            .push_str("\tmultiply_fields = [['','']]\r\n");

        for field in self.fields.iter() {
            self.grammar_contents.push_str("\r\n[[fields]]\r\n");
            self.grammar_contents
                .push_str(&format!("\tname = '{}'\r\n", field.1));
            self.grammar_contents
                .push_str(&format!("\tsize = {}\r\n", get_field_size(&field.0)?));
            self.grammar_contents
                .push_str(&format!("\tdata_type = '{}'\r\n", field.0));
            self.grammar_contents
                .push_str("\tdisplay_format = 'hex'\r\n");
            self.grammar_contents.push_str("\tdescription = 'N/A'\r\n");
        }
        Ok(self)
    }

    /// Writes the newly created grammar contents in `self.grammar_contents` to a toml file specified by `output_path`.
    pub fn write_grammar_file(&mut self, output_path: &str) -> Result<&mut Self, ()> {
        let mut grammer_file = match fs::File::create(output_path) {
            Ok(f) => f,
            Err(e) => {
                serror!(format!(
                    "Could not create/open file {}, because {} ",
                    output_path, e
                ));
                return Err(());
            }
        };

        match grammer_file.write_all(self.grammar_contents.as_bytes()) {
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

/// Get's the field size based on the C basic type.
///
/// This assumes that a char is 1 byte and an int is 4 bytes etc.
/// This may not allways be the case!
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
