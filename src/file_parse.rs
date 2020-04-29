
pub struct FileData {
    pub grammar_contents: String,
}

impl FileData {
    pub fn new() -> Self {
        Self {
            grammar_contents: String::from(""),
        }
    }

    /// Reads the contents of a grammar file into self.grammar_contents
    pub fn read_grammar(&mut self, file_path: &str) -> Result<&mut Self, ()> {
        self.grammar_contents = fs::read_to_string(file_path)
            .map_err(|e| serror!(format!("Error opening file: {}, because:{}", file_path, e)))?;
        Ok(self)
    }
}
