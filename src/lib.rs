#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod abieos_error;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use crate::abieos_error::AbieosError;

include!("bindings.rs");

fn convert_str(my_str: &str) -> *const c_char {
    let c_string = CString::new(my_str).expect("Failed to create CString");
    c_string.as_ptr()
}

fn string_from_ptr(ptr: *const c_char) -> String {
    unsafe {
        CStr::from_ptr(ptr).to_str().expect("Failed to convert CStr to str").to_string()
    }
}

/// Abieos is a Rust wrapper for the abieos C library
pub struct Abieos {
    pub context: Option<*mut abieos_context>,
}

impl Default for Abieos {
    fn default() -> Self {
        Abieos::new()
    }
}

impl Abieos {
    pub fn new() -> Abieos {
        Abieos {
            context: Some(abieos::create())
        }
    }

    pub fn from_context(context: *mut abieos_context) -> Abieos {
        Abieos {
            context: Some(context)
        }
    }

    fn get_error(&self) -> String {
        let ctx = self.ctx();
        unsafe {
            let error = abieos_get_error(ctx);
            CStr::from_ptr(error).to_str().unwrap().to_string()
        }
    }

    fn ctx(&self) -> *mut abieos_context {
        self.context.unwrap()
    }

    /// Convert a string slice into an u64 native name
    pub fn string_to_name(&self, name: &str) -> Result<u64, AbieosError> {
        let ctx = self.ctx();
        if name.len() > 12 {
            return Err(AbieosError::NameTooLongError);
        }
        unsafe {
            let c_buf = CString::new(name.as_bytes()).unwrap();
            let name = abieos_string_to_name(ctx, c_buf.as_ptr());
            if name == 0 {
                Err(AbieosError::StringToNameError)
            } else {
                Ok(name)
            }
        }
    }

    /// Convert an u64 native name into a string slice
    pub fn name_to_string(&self, name: u64) -> Result<&str, AbieosError> {
        let ctx = self.ctx();
        unsafe {
            let c_buf = abieos_name_to_string(ctx, name);
            match CStr::from_ptr(c_buf).to_str() {
                Ok(x) => Ok(x),
                Err(_) => Err(AbieosError::NameToStringError),
            }
        }
    }

    /// Load a contract ABI to memory (JSON format)
    pub fn set_abi(&self, contract: &str, abi_json: &str) -> Result<bool, AbieosError> {
        match self.string_to_name(contract) {
            Ok(contract_u64) => unsafe {
                let abi_content_cs = CString::new(abi_json.as_bytes()).unwrap();
                match abieos_set_abi(self.ctx(), contract_u64, abi_content_cs.as_ptr()) {
                    1 => Ok(true),
                    _ => Err(AbieosError::SetAbiError(self.get_error()))
                }
            }
            Err(_) => Err(AbieosError::StringToNameError)
        }
    }

    /// Load a contract ABI to memory (HEX format)
    pub fn set_abi_hex(&self, contract: &str, abi_hex: String) -> Result<bool, AbieosError> {
        let abi_hex_cs = CString::new(abi_hex).unwrap();
        match self.string_to_name(contract) {
            Ok(contract_u64) => unsafe {
                match abieos_set_abi_hex(self.ctx(), contract_u64, abi_hex_cs.as_ptr()) {
                    1 => Ok(true),
                    _ => Err(AbieosError::SetAbiError(self.get_error()))
                }
            }
            Err(_) => Err(AbieosError::StringToNameError)
        }
    }
    
    /// Load a contract ABI to memory (binary format)
    pub fn set_abi_bin(&self, contract: &str, abi_bin: Vec<u8>) -> Result<bool, AbieosError> {
        match self.string_to_name(contract) {
            Ok(contract_u64) => unsafe {
                let abi_bin_data: *const c_char = abi_bin.as_ptr() as *const c_char;
                let abi_bin_size: usize = abi_bin.len();
                match abieos_set_abi_bin(self.ctx(), contract_u64, abi_bin_data, abi_bin_size) {
                    1 => Ok(true),
                    _ => Err(AbieosError::SetAbiError(self.get_error()))
                }
            }
            Err(_) => Err(AbieosError::StringToNameError)
        }
    }

    /// Serialize JSON into binary
    pub fn json_to_hex(&self, account: &str, datatype: &str, json: String) -> Result<String, AbieosError> {
        let ctx = self.ctx();
        let account = self.string_to_name(account)?;

        let datatype = CString::new(datatype).unwrap();
        let json = CString::new(json).unwrap();
        unsafe {
            match abieos_json_to_bin_reorderable(ctx, account, datatype.as_ptr(), json.as_ptr()) {
                1 => {
                    let p = abieos_get_bin_hex(ctx);
                    Ok(string_from_ptr(p))
                }
                _ => Err(AbieosError::JsonToHexError(self.get_error()))
            }
        }
    }

    /// Deserialize HEX string into JSON
    pub fn hex_to_json(&self, account: &str, datatype: &str, hex: String) -> Result<String, AbieosError> {
        let ctx = self.ctx();
        let account = self.string_to_name(account).unwrap();

        let datatype = CString::new(datatype).unwrap();
        let hex = CString::new(hex).unwrap();
        unsafe {
            let p = abieos_hex_to_json(ctx, account, datatype.as_ptr(), hex.as_ptr());
            if p.is_null() {
                Err(AbieosError::HexToJsonError(self.get_error()))
            } else {
                Ok(string_from_ptr(p))
            }
        }
    }


    /// Get the type for an action
    pub fn get_type_for_action(&self, contract: &str, action: &str) -> Result<String, AbieosError> {
        let ctx = self.ctx();
        let contract = self.string_to_name(contract).unwrap();
        let action = self.string_to_name(action).unwrap();
        unsafe {
            let p = abieos_get_type_for_action(ctx, contract, action);
            if p.is_null() {
                Err(AbieosError::GetTypeForActionError(self.get_error()))
            } else {
                Ok(string_from_ptr(p))
            }
        }
    }

    /// Get the type for a table
    pub fn get_type_for_table(&self, contract: &str, table: &str) -> Result<String, AbieosError> {
        let ctx = self.ctx();
        let contract = self.string_to_name(contract).unwrap();
        let table = self.string_to_name(table).unwrap();
        unsafe {
            let p = abieos_get_type_for_table(ctx, contract, table);
            if p.is_null() {
                Err(AbieosError::GetTypeForTableError(self.get_error()))
            } else {
                Ok(string_from_ptr(p))
            }
        }
    }

    /// Get the type for an action result
    pub fn get_type_for_action_result(&self, contract: &str, action: &str) -> Result<String, AbieosError> {
        let ctx = self.ctx();
        let contract = self.string_to_name(contract).unwrap();
        let action_res = self.string_to_name(action).unwrap();
        unsafe {
            let p = abieos_get_type_for_action_result(ctx, contract, action_res);
            if p.is_null() {
                Err(AbieosError::GetTypeForActionResultError(self.get_error()))
            } else {
                Ok(string_from_ptr(p))
            }
        }
    }

    pub fn abi_bin_to_json(&self, abi: Vec<u8>) -> Result<String, AbieosError> {
        let ctx = self.ctx();
        let abi_bin_data: *const c_char = abi.as_ptr() as *const c_char;
        let abi_bin_size: usize = abi.len();
        unsafe {
            let p = abieos_abi_bin_to_json(
                ctx,
                abi_bin_data,
                abi_bin_size
            );
            if p.is_null() {
                Err(AbieosError::AbiBinToJsonError(self.get_error()))
            } else {
                Ok(string_from_ptr(p))
            }
        }
    }

    pub fn abi_json_to_bin(&self, json: String) -> Result<Vec<u8>, AbieosError> {
        let ctx = self.ctx();
        let abi_json: CString = CString::new(json).unwrap();
        unsafe {
            match abieos_abi_json_to_bin(ctx, abi_json.as_ptr()) {
                1 => {
                    let p = abieos_get_bin_data(ctx);
                    let len = abieos_get_bin_size(ctx);
                    let mut result: Vec<c_char> = Vec::with_capacity(len as usize);
                    std::ptr::copy_nonoverlapping(p, result.as_mut_ptr(), len as usize);
                    result.set_len(len as usize);
                    Ok(result.iter().map(|&c| c as u8).collect())
                }
                _ => Err(AbieosError::AbiJsonToBinError(self.get_error()))
            }
        }
    }
}

pub mod abieos {
    use crate::{abieos_context, abieos_create};

    pub fn create() -> *mut abieos_context {
        unsafe {
            abieos_create()
        }
    }
}
