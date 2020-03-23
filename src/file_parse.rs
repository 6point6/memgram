use std::fs;

pub struct FileData {
    pub grammer_contents: String,
}

impl FileData {
    pub fn new() -> FileData {
        FileData {
            grammer_contents: String::from(""),
        }
    }
    pub fn read_grammer(&mut self, file_path: &str) -> Result<&mut FileData, ()> {
        self.grammer_contents = read_file(file_path)?;
        Ok(self)
    }
}

pub fn read_file(file_path: &str) -> Result<String, ()> {
    fs::read_to_string(file_path).or_else(|e| {
        serror!(format!("Error opening file: {}, because:{}", file_path, e));
        Err(())
    })
}
