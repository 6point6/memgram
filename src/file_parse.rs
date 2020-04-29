use std::fs;

pub struct FileData {
    pub grammer_contents: String,
}

impl FileData {
    pub fn new() -> Self {
        Self {
            grammer_contents: String::from(""),
        }
    }

    pub fn read_grammer(&mut self, file_path: &str) -> Result<&mut Self, ()> {
        self.grammer_contents = fs::read_to_string(file_path)
            .map_err(|e| serror!(format!("Error opening file: {}, because:{}", file_path, e)))?;
        Ok(self)
    }
}
