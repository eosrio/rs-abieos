use std::fmt::{Display, Formatter, Result as FmtResult};

/// Error types
#[derive(Debug)]
pub enum AbieosError {
    Unknown,
    StringToName,
    NameToString,
    NameTooLong,
    AbiNotLoaded,
    FileRead,
    SetAbi(String),
    JsonToHex(String),
    HexToJson(String),
    GetTypeForAction(String),
    GetTypeForTable(String),
    GetTypeForActionResult(String),
    AbiBinToJson(String),
    AbiJsonToBin(String),
    BinToJson(String),
}

impl Display for AbieosError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            AbieosError::NameTooLong => write!(f, "Name is too long"),
            AbieosError::StringToName => write!(f, "Failed to convert string to name"),
            AbieosError::NameToString => write!(f, "Failed to convert name to string"),
            AbieosError::SetAbi(e) => write!(f, "Failed to set ABI: {}", e),
            AbieosError::JsonToHex(e) => write!(f, "Failed to serialize JSON: {}", e),
            AbieosError::HexToJson(e) => write!(f, "Failed to deserialize Binary: {}", e),
            AbieosError::GetTypeForAction(e) => write!(f, "Failed to get type for action: {}", e),
            AbieosError::GetTypeForTable(e) => write!(f, "Failed to get type for table: {}", e),
            AbieosError::GetTypeForActionResult(e) => write!(f, "Failed to get type for action result: {}", e),
            AbieosError::AbiBinToJson(e) => write!(f, "Failed to convert ABI binary to JSON: {}", e),
            AbieosError::AbiJsonToBin(e) => write!(f, "Failed to convert ABI JSON to binary: {}", e),
            AbieosError::AbiNotLoaded => write!(f, "ABI not loaded in this contract"),
            AbieosError::FileRead => write!(f, "Failed to read file"),
            AbieosError::BinToJson(e) => write!(f, "Failed to convert binary to JSON: {}", e),
            AbieosError::Unknown => write!(f, "Unknown error occurred"),
        }
    }
}

