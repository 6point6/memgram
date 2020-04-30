//! Module that deals with parsing a grammar file into a Grammar data structure
use serde::Deserialize;
use serde::Serialize;
use std::convert::TryInto;

/// Parent structure which holds the metadata and fields of the grammar
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Grammar {
    /// Holds metadata ([metadata) portion of the grammar file.
    pub metadata: GrammerMetadata,
    /// Each GrammarField entry corrosponds to a [[fields]] entry in the grammar file.
    pub fields: Vec<GrammerFields>,
}

/// Holds metadata ([metadata) portion of the grammar file.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GrammerMetadata {
    /// The name of the data structure.
    pub name: String,
    /// Specifies which fields if any are variable sized.
    pub variable_size_fields: Vec<(String, String, String, String)>,
    /// Specifies which fields if any should be multiplied/repeated.
    pub multiply_fields: Vec<(String, String)>,
}

/// Each GrammarField entry corrosponds to a [[fields]] entry in the grammar file.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GrammerFields {
    /// The name of the field.
    pub name: String,
    /// How large the field is in bytes.
    pub size: usize,
    /// The data type of the field.
    pub data_type: String,
    /// The display format of the field.
    pub display_format: String,
    /// The description of the field.
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

    /// Parses the contents of a grammar into the Grammer structure.
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

    /// Get's the total size in bytes of all the fields in the grammer file.
    pub fn get_struct_size(&self) -> usize {
        let mut struct_size: usize = 0;

        for field in &self.fields {
            struct_size += field.size;
        }

        struct_size
    }

    /// Further parses the grammar in the Grammar structure.
    ///
    /// multiply_fields is run here if mulitplying fields was specified in the grammar file.
    pub fn post_parse_toml(&mut self) -> Result<&mut Self, ()> {
        if !self.metadata.multiply_fields[0].0.is_empty()
            && !self.metadata.multiply_fields[0].1.is_empty()
        {
            self.multiply_fields()?;
        }

        Ok(self)
    }

    /// Populates a Vec<VariableSizeEntry>.
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

    /// Mulitplies (copys) a field of the grammar by the number of times specified in the grammar file.
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

/// Matches an arthmetic operator in char format (+,-,*,/) with one of the ArithmeticOperator enum variants.
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

/// Holds of the information relating to a variable size entry.
#[derive(Debug)]
pub struct VariableSizeEntry {
    /// The name of the field used as a source for the variable sized field.
    pub source_field_name: String,
    /// Index of source field in Grammar.fields.
    pub source_field_index: usize,
    /// The interger value of the data stored at the source field.
    pub source_field_real_size: usize,
    /// The display_format of the source field.
    pub source_field_display: String,
    /// The name of the variable sized field.
    pub var_field_name: String,
    /// The options for the variable sized field.
    pub variable_options: VariableOptions,
    /// The order of arthimetic operations on the source field real size.
    pub arithemitc_order: VariableSizeArithmeticOrder,
    /// The type of arthimetic operation to perform (Addition, Subtraction, Multiplication, Division).
    pub arithmetic_operator: ArithmeticOperators,
    /// The adjustment in the arithmetic operation.
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
