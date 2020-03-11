use crate::file_parse;
use std::collections::HashMap;
use regex::Regex;

pub struct CStruct {
    pub name: String,
    pub fields: HashMap<usize,(String,String)>,
}

impl CStruct {
    pub fn new() -> CStruct {
        CStruct {
            name: String::from(""),
            fields: HashMap::new(),
        }
    }

    pub fn parse_c_struct(&mut self, struct_filepath: &String) -> Result<&mut CStruct, ()> {
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

        next_index += match struct_string[prev_index..].find("{") {
            Some(matched_index) => matched_index + prev_index,
            None => {
                serror!("Invalid C struct: could not find opening '{'");
                return Err(());
            }
        };

        let last_index: usize = match struct_string[next_index..].find("}") {
            Some(matched_index) => matched_index + next_index,
            None => {
                serror!("Invalid C struct: could not find closing '}'");
                return Err(());
            }
        };

        let struct_name: &str = &struct_string[prev_index..next_index];

        self.name = struct_name.clone().trim().to_string();

        let re = Regex::new(r"([^\s;]+)([^;]+)").unwrap();

        for (index,cap) in re.captures_iter(&struct_string[next_index + 1..last_index]).enumerate() {
            self.fields.insert(index,(cap[1].to_string(), cap[2].to_string()));
        }

        Ok(self)
    }
}
