pub enum GeneralError {
    SeekError,
    OpenFileError,
    FileNotFound,
}

pub const ERROR_START: &str = "[-] Error:";