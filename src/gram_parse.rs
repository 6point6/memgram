use serde::Deserialize;
use serde::Serialize;
use std::convert::TryInto;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Grammar {
    pub metadata: GrammerMetadata,
    pub fields: Vec<GrammerFields>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GrammerMetadata {
    pub name: String,
    pub variable_size_fields: Vec<(String, String, String, String)>,
    pub multiply_fields: Vec<(String, String)>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GrammerFields {
    pub name: String,
    pub size: usize,
    pub data_type: String,
    pub display_format: String,
    pub description: String,
}

impl GrammerMetadata {
    pub fn new() -> GrammerMetadata {
        GrammerMetadata {
            name: String::from(""),
            variable_size_fields: Vec::new(),
            multiply_fields: Vec::new(),
        }
    }
}

impl Grammar {
    pub fn new() -> Self {
        Self {
            metadata: GrammerMetadata::new(),
            fields: Vec::new(),
        }
    }

    pub fn parse_toml(&mut self, file_contents: &str) -> Result<&mut Self, ()> {
        match toml::from_str::<Self>(file_contents) {
            Ok(gram) => {
                *self = gram;
            }
            Err(e) => {
                serror!(format!("Could not parse grammar file, because {}", e));
                return Err(());
            }
        }

        Ok(self)
    }

    pub fn get_struct_size(&self) -> usize {
        let mut struct_size: usize = 0;

        for field in &self.fields {
            struct_size += field.size;
        }

        struct_size
    }

    pub fn post_parse_toml(&mut self) -> Result<&mut Self, ()> {
        if !self.metadata.multiply_fields[0].0.is_empty()
            && !self.metadata.multiply_fields[0].1.is_empty()
        {
            self.multiply_fields()?;
        }

        Ok(self)
    }

    // fn add_field_id(&mut self) {
    //     let mut field_id: u32 = 0;

    //     for field in &mut self.fields {
    //         field.name.push_str(&format!("{:03X}", field_id)[..]);
    //         field_id += 1;
    //     }
    // }

    pub fn create_var_size_entry_vector(
        &mut self,
        var_size_entry_vec: &mut Vec<VariableSizeEntry>,
    ) -> Result<(), ()> {
        if self.metadata.variable_size_fields[0].0.is_empty()
            && self.metadata.variable_size_fields[0].1.is_empty()
            && self.metadata.variable_size_fields[0].2.is_empty()
            && self.metadata.variable_size_fields[0].3.is_empty()
        {
            return Ok(());
        }

        let mut variable_size_vector: Vec<VariableSizeEntry> = Vec::new();

        for entry in self.metadata.variable_size_fields.iter() {
            let mut variable_size_entry = VariableSizeEntry::new();

            let entry_0 = entry.0.as_str();
            let entry_1 = entry.1.as_str();
            let entry_2 = entry.2.as_str();
            let entry_3 = entry.3.as_str();

            for (index, field) in self.fields.iter().enumerate() {
                if &field.name[..] == entry_0 {
                    variable_size_entry.source_field_name = field.name.clone();
                    variable_size_entry.source_field_display = field.display_format.clone();
                    variable_size_entry.source_field_index = index;

                    if !entry_1.is_empty() && !entry_2.is_empty() {
                        variable_size_entry.arithmetic_operator =
                            get_var_arithmetic_operator(entry_1.trim())?;

                        variable_size_entry.adjustment = match entry_2.trim().parse::<usize>() {
                            Ok(adjustment) => adjustment,
                            Err(e) => {
                                serror!(format!(
                                    "Could not convert variable size adjustment: {} because, {}",
                                    entry_2.trim(),
                                    e
                                ));
                                return Err(());
                            }
                        };

                        variable_size_entry.arithemitc_order =
                            VariableSizeArithmeticOrder::Forwards;
                    }
                } else if &field.name[..] == entry_2 {
                    variable_size_entry.source_field_name = field.name.clone();
                    variable_size_entry.source_field_display = field.display_format.clone();
                    variable_size_entry.source_field_index = index;

                    if !entry_0.is_empty() && !entry_1.is_empty() {
                        variable_size_entry.arithmetic_operator =
                            get_var_arithmetic_operator(entry_1.trim())?;

                        variable_size_entry.adjustment = match entry_0.trim().parse::<usize>() {
                            Ok(adjustment) => adjustment,
                            Err(e) => {
                                serror!(format!(
                                    "Could not convert variable size adjustment: {} because, {}",
                                    entry_2.trim(),
                                    e
                                ));
                                return Err(());
                            }
                        };

                        variable_size_entry.arithemitc_order =
                            VariableSizeArithmeticOrder::Backwards;
                    }
                } else if &field.name[..] == entry_3 {
                    variable_size_entry.var_field_name = field.name.clone();
                    if entry_1.trim().to_lowercase() == "null" {
                        variable_size_entry.variable_options = VariableOptions::NullChar;
                    }
                }
            }

            if variable_size_entry.var_field_name.is_empty() {
                serror!(format!(
                    "Variable field name: {}, does not exist for variable size fields",
                    entry_3
                ));
                return Err(());
            } else if variable_size_entry.source_field_name.is_empty() {
                match variable_size_entry.variable_options {
                    VariableOptions::NullChar => (),
                    VariableOptions::NoOptions => match entry_0.parse::<i32>() {
                        Ok(_) => {
                            serror!(format!(
                                "Source field name: {} does not exist as a field in grammar",
                                entry_2
                            ));
                            return Err(());
                        }
                        Err(_) => {
                            serror!(format!(
                                "Source field name: {} does not exist as a field in grammar",
                                entry_0
                            ));
                            return Err(());
                        }
                    },
                }
            }

            variable_size_vector.push(variable_size_entry);
        }

        *var_size_entry_vec = variable_size_vector;

        Ok(())
    }

    fn multiply_fields(&mut self) -> Result<(), ()> {
        for entry in self.metadata.multiply_fields.iter() {
            let mut field_multiply = FieldMultiply::new();

            let entry_0 = entry.0.as_str();
            let entry_1 = entry.1.as_str();

            for (index, field) in self.fields.iter().enumerate() {
                if &field.name[..] == entry_0 {
                    field_multiply.field_name = field.name.clone();
                    field_multiply.field_index = index;

                    match entry_1.parse::<i32>() {
                        Ok(multiplier) => {
                            field_multiply.multiplier = multiplier;
                            break;
                        }
                        Err(e) => {
                            serror!(format!(
                                "Could not convert multiplier for field: {} to an interger, because {}",
                                field.name, e
                            ));
                            return Err(());
                        }
                    }
                } else if &field.name[..] == entry_1 {
                    field_multiply.field_name = field.name.clone();
                    field_multiply.field_index = index;

                    match entry_0.parse::<i32>() {
                        Ok(multiplier) => {
                            field_multiply.multiplier = multiplier;
                            break;
                        }
                        Err(e) => {
                            serror!(format!(
                                "Could not convert multiplier for field: {} to an interger, because {}",
                                field.name, e));
                            return Err(());
                        }
                    }
                } else {
                    continue;
                }
            }

            if field_multiply.field_name.is_empty() || field_multiply.multiplier == 0 {
                serror! {"Could not find multiply field name or multiplier is 0"};
                return Err(());
            }

            for _x in 0..field_multiply.multiplier - 1 {
                self.fields.insert(
                    field_multiply.field_index,
                    self.fields[field_multiply.field_index].clone(),
                );
            }
        }
        Ok(())
    }
}

fn get_var_arithmetic_operator(arithmetic_op_str: &str) -> Result<ArithmeticOperators, ()> {
    if arithmetic_op_str.len() > 1 {
        serror!(format!("Invalid arithmetic operator legnth: {} for variable size fields: {}, must be one of the following (+, -, *, /)", arithmetic_op_str.len(),arithmetic_op_str));
        return Err(());
    }
    match arithmetic_op_str {
        "+" => Ok(ArithmeticOperators::Addition),
        "-" => Ok(ArithmeticOperators::Subtraction),
        "*" => Ok(ArithmeticOperators::Multiplication),
        "/" => Ok(ArithmeticOperators::Division),
        _ => {
            serror!(format!("Invalid arithmetic operator for variable size fields: {}, must be one of the following (+, -, *, /)",arithmetic_op_str));
            Err(())
        }
    }
}

///
#[derive(Debug)]
pub struct VariableSizeEntry {
    pub source_field_name: String,
    pub source_field_index: usize,
    pub source_field_real_size: usize,
    pub source_field_display: String,
    pub var_field_name: String,
    pub variable_options: VariableOptions,
    pub arithemitc_order: VariableSizeArithmeticOrder,
    pub arithmetic_operator: ArithmeticOperators,
    pub adjustment: usize,
}

pub enum ConvertEndianess {
    BigEndian,
    LittleEndian,
}

#[derive(Debug)]
pub enum VariableOptions {
    NoOptions,
    NullChar,
}

impl VariableSizeEntry {
    fn new() -> Self {
        Self {
            source_field_name: String::from(""),
            source_field_index: 0,
            source_field_real_size: 0,
            source_field_display: String::from(""),
            var_field_name: String::from(""),
            variable_options: VariableOptions::NoOptions,
            arithemitc_order: VariableSizeArithmeticOrder::Unset,
            arithmetic_operator: ArithmeticOperators::Addition,
            adjustment: 0,
        }
    }

    /// Performs arithmetic operation on variable size field yielding the result
    pub fn calculate_variable_size(&mut self) -> usize {
        match self.arithemitc_order {
            VariableSizeArithmeticOrder::Unset => self.source_field_real_size,
            VariableSizeArithmeticOrder::Forwards => match self.arithmetic_operator {
                ArithmeticOperators::Addition => self.source_field_real_size + self.adjustment,
                ArithmeticOperators::Subtraction => self.source_field_real_size - self.adjustment,
                ArithmeticOperators::Multiplication => {
                    self.source_field_real_size * self.adjustment
                }
                ArithmeticOperators::Division => self.source_field_real_size / self.adjustment,
            },
            VariableSizeArithmeticOrder::Backwards => match self.arithmetic_operator {
                ArithmeticOperators::Addition => self.adjustment + self.source_field_real_size,
                ArithmeticOperators::Subtraction => self.adjustment - self.source_field_real_size,
                ArithmeticOperators::Multiplication => {
                    self.adjustment * self.source_field_real_size
                }
                ArithmeticOperators::Division => self.adjustment / self.source_field_real_size,
            },
        }
    }

    pub fn convert_field_size(
        &mut self,
        raw_field_data: &[u8],
        endianess: ConvertEndianess,
    ) -> Result<(), ()> {
        match endianess {
            ConvertEndianess::LittleEndian => {
                self.source_field_real_size = match raw_field_data.len() {
                    2 => i16::from_le_bytes(raw_field_data[..2].try_into().unwrap()) as usize,
                    4 => i32::from_le_bytes(raw_field_data[..4].try_into().unwrap()) as usize,
                    8 => i64::from_le_bytes(raw_field_data[..8].try_into().unwrap()) as usize,
                    16 => i128::from_le_bytes(raw_field_data[..16].try_into().unwrap()) as usize,
                    _ => {
                        serror!(format!(
                            "Could not convert raw_field data to little endian because unsupported variable field size: {}",
                            raw_field_data.len()
                        ));
                        return Err(());
                    }
                };
            }
            ConvertEndianess::BigEndian => {
                self.source_field_real_size = match raw_field_data.len() {
                    2 => i16::from_be_bytes(raw_field_data[..2].try_into().unwrap()) as usize,
                    4 => i32::from_be_bytes(raw_field_data[..4].try_into().unwrap()) as usize,
                    8 => i64::from_be_bytes(raw_field_data[..8].try_into().unwrap()) as usize,
                    16 => i128::from_be_bytes(raw_field_data[..16].try_into().unwrap()) as usize,
                    _ => {
                        serror!(format!(
                            "Could not convert raw_field data to big endian because unsupported variable field size: {}",
                            raw_field_data.len()
                        ));
                        return Err(());
                    }
                };
            }
        }
        Ok(())
    }
}

/// Specifys the arithmetic order of the variable_size_fields entry in metadata.
#[derive(Debug)]
pub enum VariableSizeArithmeticOrder {
    /// The aritmetic order is forwards (number [+,-,*,/] variable size field).
    Forwards,
    /// The aritmetic order is backwards (variable size field) [+,-,*,/ ] number).
    Backwards,
    /// There is no arithmetic operation to be performed.
    Unset,
}

/// Abstraction for aritmetic operators.
#[derive(Debug)]
pub enum ArithmeticOperators {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

/// Holds data needed to multiply a field.
pub struct FieldMultiply {
    /// Name of field to be multiplied.
    pub field_name: String,
    /// The index/offset into the file the field occurs at.
    pub field_index: usize,
    /// How many times the field should be multiplied.
    pub multiplier: i32,
}

impl FieldMultiply {
    fn new() -> Self {
        Self {
            field_name: String::from(""),
            field_index: 0,
            multiplier: 0,
        }
    }
}
