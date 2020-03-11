use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

pub struct CStruct {
    pub name: String,
    pub fields: HashMap<String, String>,
}

impl CStruct {
    pub fn new() -> CStruct {
        CStruct {
            name: String::from(""),
            fields: HashMap::new(),
        }
    }

    pub fn parse_c_struct(&mut self, struct_string: &String) -> Result<&mut CStruct, ()> {
        let mut prev_index: usize = 0;

        prev_index += match struct_string[prev_index..].find("struct") {
            Some(matched_index) => matched_index,
            None => {
                serror("Invalid C struct: could not find 'struct' keyword");
                return Err(())
            }
        }

        next_index += match struct_string[prev_index..].find("{") {
            Some(matched_index) => matched_index,
            None => {
                serror("Invalid C struct: could not find opening '{}'");
                return Err(());
            }
        }

        struct_name = struct_string[prev_index..next_index].clone();

        println!("struct name : {}\n",struct_name);


    }
}