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
        
    }
}