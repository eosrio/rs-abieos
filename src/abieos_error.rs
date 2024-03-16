use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug)]
pub enum AbieosError {
    StringToNameError,
    NameToStringError,
    SetAbiError(String),
    JsonToHexError(String),
    HexToJsonError(String),
    UnknownError,
    NameTooLongError,
    GetTypeForActionError(String),
    GetTypeForTableError(String),
    GetTypeForActionResultError(String),
    AbiBinToJsonError(String),
    AbiJsonToBinError(String),
}

impl Display for AbieosError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            AbieosError::NameTooLongError => write!(f, "Name is too long"),
            AbieosError::StringToNameError => write!(f, "Failed to convert string to name"),
            AbieosError::NameToStringError => write!(f, "Failed to convert name to string"),
            AbieosError::SetAbiError(e) => write!(f, "Failed to set ABI: {}", e),
            AbieosError::JsonToHexError(e) => write!(f, "Failed to serialize JSON: {}", e),
            AbieosError::HexToJsonError(e) => write!(f, "Failed to deserialize Binary: {}", e),
            AbieosError::GetTypeForActionError(e) => write!(f, "Failed to get type for action: {}", e),
            AbieosError::GetTypeForTableError(e) => write!(f, "Failed to get type for table: {}", e),
            AbieosError::GetTypeForActionResultError(e) => write!(f, "Failed to get type for action result: {}", e),
            AbieosError::AbiBinToJsonError(e) => write!(f, "Failed to convert ABI binary to JSON: {}", e),
            AbieosError::AbiJsonToBinError(e) => write!(f, "Failed to convert ABI JSON to binary: {}", e),
            AbieosError::UnknownError => write!(f, "Unknown error occurred"),
        }
    }
}

impl Error for AbieosError {}

