//! Module that deals with converting raw u8 arrays into formatted strings. e.g utf16 byte array to utf16 string.
use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, NasmFormatter};
use std::net::{IpAddr, Ipv4Addr};
use widestring::U16CString;

/// Display data in little endian hex string format.
pub const HEXLE_TYPE: &str = "hexle";
/// Display data in ASCII format.
pub const ASCII_TYPE: &str = "ascii";
/// Display data in IPv4 big endian format.
pub const IPV4BE_TYPE: &str = "ipv4be";
/// Display data in IPv4 little endian format.
pub const IPV4LE_TYPE: &str = "ipv4le";
/// Display data in UTF16 little endian format.
pub const UTF16LE_TYPE: &str = "utf16be";
/// Display data in UTF16  big endian format.
pub const UTF16BE_TYPE: &str = "utf16le";
/// Display x86_32 assembly format.
pub const X86_TYPE: &str = "x86_32";

/// Converts a 4 byte u8 array into a ipv4 string
pub fn ipv4_string(ipv4_bytes: &[u8]) -> Result<String, ()> {
    match ipv4_bytes.len() {
        4 => Ok(format!(
            "{}",
            IpAddr::V4(Ipv4Addr::new(
                ipv4_bytes[0],
                ipv4_bytes[1],
                ipv4_bytes[2],
                ipv4_bytes[3]
            ))
        )),
        _ => {
            serror!("Invalid IPv4 address {}");
            Err(())
        }
    }
}

/// Converts a utf16 byte array into a utf16 string.
///
/// If little_endian is set to true, the utf16 byte array will be converted to a utf16_le string.
///
/// If little endian is set to false, the utf16 byte array will be converted to a utf16_be string.
pub fn utf16_string(utf16_bytes: &[u8], little_endian: bool) -> Result<String, ()> {
    let raw_iter = utf16_bytes.chunks_exact(2);

    if little_endian {
        let le_raw_field_data: Vec<u16> = raw_iter
            .map(|word| u16::from_le_bytes([word[0], word[1]]))
            .collect();

        match U16CString::from_vec_with_nul(le_raw_field_data) {
            Ok(le_data) => Ok(le_data.to_string_lossy()),
            Err(_) => {
                serror!("Error constructing UTF16_LE string");
                Err(())
            }
        }
    } else {
        let le_raw_field_data: Vec<u16> = raw_iter
            .map(|word| u16::from_be_bytes([word[0], word[1]]))
            .collect();

        match U16CString::from_vec_with_nul(le_raw_field_data) {
            Ok(le_data) => Ok(le_data.to_string_lossy()),
            Err(_) => {
                serror!("Error constructing UTF16_BE string");
                Err(())
            }
        }
    }
}

/// Holds the outputted assembly as well as the line count.
pub struct DissassOutput {
    pub output: String,
    pub line_count: u32,
}

impl DissassOutput {
    pub fn new() -> Self {
        Self {
            output: String::from(""),
            line_count: 0,
        }
    }

    /// Converts u8 array into x86 assembly string and populates `self.output` and `self.line_count`.
    pub fn format_x86(&mut self, bitness: u32, machine_code: &[u8]) {
        let mut decoder = Decoder::new(bitness, machine_code, DecoderOptions::NONE);
        let mut formatter = NasmFormatter::new();
        let mut instruction = Instruction::default();

        while decoder.can_decode() {
            decoder.decode_out(&mut instruction);

            formatter.format(&instruction, &mut self.output);
            self.output.push_str("\n");
            self.line_count += 1;
        }
    }
}
