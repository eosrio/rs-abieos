#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! # Rust Abieos
//!
//! Abieos is a Rust wrapper for the abieos C library

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

mod abieos_error;
pub use abieos_error::AbieosError;

pub mod bindings {
    include!("bindings.rs");
}

pub use bindings::*;

fn string_from_ptr(ptr: *const c_char) -> String {
    unsafe {
        CStr::from_ptr(ptr).to_str().expect("Failed to convert CStr to str").to_string()
    }
}

/// Abieos is a Rust wrapper for the abieos C library
pub struct Abieos {
    pub context: Option<*mut abieos_context>,
    pub is_destroyed: bool,
}

unsafe impl Send for Abieos {}

/// Accepted name formats
pub enum NameLike {
    String(String),
    StringRef(&'static str),
    U64(u64),
    I32(i32),
}

/// Accepted ABI formats
pub enum AbiLike {
    Json(String),
    Hex(String),
    Bin(Vec<u8>),
}

/// Abieos Contract reference
pub struct AbieosContract {
    pub context: *mut abieos_context,
    pub name: u64,
    pub abiLoaded: bool,
}

impl AbieosContract {

    //! # Abieos Contract
    //!
    //!  AbieosContract is a contract reference that can be used to load ABIs and serialize/deserialize data
    //!

    /// Load an ABI from a JSON file
    pub fn load_json_file(&mut self, path: &str) -> Result<&mut Self, AbieosError> {
        let ref_abieos = Abieos::from_context(self.context);
        match std::fs::read_to_string(path) {
            Ok(file) => {
                match ref_abieos.set_abi_json_native(self.name, file) {
                    Ok(_) => {
                        self.abiLoaded = true;
                        Ok(self)
                    }
                    Err(e) => Err(e)
                }
            }
            Err(_) => Err(AbieosError::FileRead),
        }
    }

    /// Load an ABI
    pub fn load_abi(&mut self, abi: AbiLike) -> Result<&mut Self, AbieosError> {
        let ref_abieos = Abieos::from_context(self.context);
        let result = match abi {
            AbiLike::Json(abi_json) => {
                ref_abieos.set_abi_json_native(self.name, abi_json)
            }
            AbiLike::Hex(abi_hex) => {
                ref_abieos.set_abi_hex_native(self.name, abi_hex)
            }
            AbiLike::Bin(abi_bin) => {
                ref_abieos.set_abi_bin_native(self.name, abi_bin)
            }
        };
        match result {
            Ok(_) => {
                self.abiLoaded = true;
                Ok(self)
            }
            Err(e) => Err(e)
        }
    }

    /// Get data type for an action
    pub fn get_type_for_action(&self, action: &str) -> Result<String, AbieosError> {
        let ref_abieos = Abieos::from_context(self.context);
        match ref_abieos.string_to_name(action) {
            Ok(x) => ref_abieos.get_type_for_action_native(self.name, x),
            Err(_) => Err(AbieosError::StringToName)
        }
    }

    /// Serialize JSON into binary (output as HEX)
    pub fn json_to_hex(&self, datatype: &str, json: String) -> Result<String, AbieosError> {
        let ref_abieos = Abieos::from_context(self.context);
        ref_abieos.json_to_hex_native(self.name, datatype, json)
    }

    /// Deserialize HEX string into JSON
    pub fn hex_to_json(&self, datatype: &str, hex: String) -> Result<String, AbieosError> {
        let ref_abieos = Abieos::from_context(self.context);
        ref_abieos.hex_to_json_native(self.name, datatype, hex)
    }
}

impl AbieosContract {
    pub fn new(context: *mut abieos_context, name: u64) -> AbieosContract {
        AbieosContract { context, name, abiLoaded: false }
    }
}

impl Abieos {
    /// Reference a contract by name
    pub fn contract(&self, account_name: NameLike) -> AbieosContract {
        match account_name {
            NameLike::StringRef(name) => {
                let name_u64 = self.string_to_name(name)
                    .expect("Failed to convert name to u64");
                AbieosContract::new(self.ctx(), name_u64)
            }
            NameLike::String(name) => {
                let name_u64 = self.string_to_name(name.as_str())
                    .expect("Failed to convert name to u64");
                AbieosContract::new(self.ctx(), name_u64)
            }
            NameLike::U64(name) => {
                AbieosContract::new(self.ctx(), name)
            }
            NameLike::I32(name) => {
                AbieosContract::new(self.ctx(), name as u64)
            }
        }
    }
}

impl Default for Abieos {
    fn default() -> Self {
        Abieos::new()
    }
}

impl Abieos {
    /// Create a new Abieos instance
    pub fn new() -> Abieos {
        Abieos {
            context: Some(abieos::create()),
            is_destroyed: false,
        }
    }

    /// Create a new Abieos instance from a context pointer
    pub fn from_context(context: *mut abieos_context) -> Abieos {
        Abieos {
            context: Some(context),
            is_destroyed: false,
        }
    }


    /// Destroy the Abieos instance
    pub fn destroy(&self) {
        unsafe {
            abieos_destroy(self.context.unwrap());
        }
    }

    /// Get the last error message
    fn get_error(&self) -> String {
        let ctx = self.ctx();
        unsafe {
            let error = abieos_get_error(ctx);
            CStr::from_ptr(error).to_str().unwrap().to_string()
        }
    }

    /// Get the context pointer
    fn ctx(&self) -> *mut abieos_context {
        self.context.unwrap()
    }

    /// Convert a string slice into an u64 native name
    pub fn string_to_name(&self, name: &str) -> Result<u64, AbieosError> {
        let ctx = self.ctx();
        if name.len() > 13 {
            return Err(AbieosError::NameTooLong);
        }
        unsafe {
            let c_buf = CString::new(name.as_bytes()).unwrap();
            Ok(abieos_string_to_name(ctx, c_buf.as_ptr()))
        }
    }

    /// Convert an u64 native name into a string slice
    pub fn name_to_string(&self, name: u64) -> Result<&str, AbieosError> {
        let ctx = self.ctx();
        unsafe {
            let c_buf = abieos_name_to_string(ctx, name);
            match CStr::from_ptr(c_buf).to_str() {
                Ok(x) => Ok(x),
                Err(_) => Err(AbieosError::NameToString),
            }
        }
    }

    /// Load a contract ABI to memory (JSON format)
    pub fn set_abi_json(&self, contract: &str, abi_json: String) -> Result<bool, AbieosError> {
        match self.string_to_name(contract) {
            Ok(contract_u64) => unsafe {
                let abi_content_cs = CString::new(abi_json).unwrap();
                match abieos_set_abi(self.ctx(), contract_u64, abi_content_cs.as_ptr()) {
                    1 => Ok(true),
                    _ => Err(AbieosError::SetAbi(self.get_error()))
                }
            }
            Err(_) => Err(AbieosError::StringToName)
        }
    }

    /// Load a contract ABI to memory (JSON format)
    pub fn set_abi_json_native(&self, contract_u64: u64, abi_json: String) -> Result<bool, AbieosError> {
        let abi_content_cs = CString::new(abi_json).unwrap();
        unsafe {
            match abieos_set_abi(self.ctx(), contract_u64, abi_content_cs.as_ptr()) {
                1 => Ok(true),
                _ => Err(AbieosError::SetAbi(self.get_error()))
            }
        }
    }

    /// Load a contract ABI to memory (HEX format)
    pub fn set_abi_hex(&self, contract: &str, abi_hex: String) -> Result<bool, AbieosError> {
        let abi_hex_cs = CString::new(abi_hex).unwrap();
        match self.string_to_name(contract) {
            Ok(contract_u64) => unsafe {
                match abieos_set_abi_hex(self.ctx(), contract_u64, abi_hex_cs.as_ptr()) {
                    1 => Ok(true),
                    _ => Err(AbieosError::SetAbi(self.get_error()))
                }
            }
            Err(_) => Err(AbieosError::StringToName)
        }
    }

    /// Load a contract ABI to memory (HEX format, u64 contract name)
    pub fn set_abi_hex_native(&self, contract: u64, abi_hex: String) -> Result<bool, AbieosError> {
        let abi_hex_cs = CString::new(abi_hex).unwrap();
        unsafe {
            match abieos_set_abi_hex(self.ctx(), contract, abi_hex_cs.as_ptr()) {
                1 => Ok(true),
                _ => Err(AbieosError::SetAbi(self.get_error()))
            }
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
                    _ => Err(AbieosError::SetAbi(self.get_error()))
                }
            }
            Err(_) => Err(AbieosError::StringToName)
        }
    }

    /// Load a contract ABI to memory (binary format, u64 contract name)
    pub fn set_abi_bin_native(&self, contract: u64, abi_bin: Vec<u8>) -> Result<bool, AbieosError> {
        let abi_bin_data: *const c_char = abi_bin.as_ptr() as *const c_char;
        let abi_bin_size: usize = abi_bin.len();
        unsafe {
            match abieos_set_abi_bin(self.ctx(), contract, abi_bin_data, abi_bin_size) {
                1 => Ok(true),
                _ => Err(AbieosError::SetAbi(self.get_error()))
            }
        }
    }

    /// Serialize JSON into binary (output as HEX)
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
                _ => Err(AbieosError::JsonToHex(self.get_error()))
            }
        }
    }

    /// Serialize JSON into binary (output as HEX, u64 account name)
    pub fn json_to_hex_native(&self, account: u64, datatype: &str, json: String) -> Result<String, AbieosError> {
        let ctx = self.ctx();
        let datatype = CString::new(datatype).unwrap();
        let json = CString::new(json).unwrap();
        unsafe {
            match abieos_json_to_bin_reorderable(ctx, account, datatype.as_ptr(), json.as_ptr()) {
                1 => {
                    let p = abieos_get_bin_hex(ctx);
                    Ok(string_from_ptr(p))
                }
                _ => Err(AbieosError::JsonToHex(self.get_error()))
            }
        }
    }

    /// Serialize JSON into binary (output as binary)
    pub fn json_to_bin(&self, account: &str, datatype: &str, json: String) -> Result<Vec<u8>, AbieosError> {
        let ctx = self.ctx();
        let account = self.string_to_name(account)?;
        let datatype = CString::new(datatype).unwrap();
        let json = CString::new(json).unwrap();
        unsafe {
            match abieos_json_to_bin_reorderable(ctx, account, datatype.as_ptr(), json.as_ptr()) {
                1 => {
                    let p = abieos_get_bin_data(ctx);
                    let len = abieos_get_bin_size(ctx);
                    let mut result: Vec<c_char> = Vec::with_capacity(len as usize);
                    std::ptr::copy_nonoverlapping(p, result.as_mut_ptr(), len as usize);
                    result.set_len(len as usize);
                    Ok(result.iter().map(|&c| c as u8).collect())
                }
                _ => Err(AbieosError::JsonToHex(self.get_error()))
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
                Err(AbieosError::HexToJson(self.get_error()))
            } else {
                Ok(string_from_ptr(p))
            }
        }
    }

    /// Deserialize Binary into JSON
    pub fn bin_to_json(&self, account: &str, datatype: &str, bin: Vec<u8>) -> Result<String, AbieosError> {
        let ctx = self.ctx();
        let account = self.string_to_name(account).unwrap();
        let datatype = CString::new(datatype).unwrap();
        let bin_data: *const c_char = bin.as_ptr() as *const c_char;
        let bin_size: usize = bin.len();
        unsafe {
            let p = abieos_bin_to_json(ctx, account, datatype.as_ptr(), bin_data, bin_size);
            if p.is_null() {
                Err(AbieosError::BinToJson(self.get_error()))
            } else {
                Ok(string_from_ptr(p))
            }
        }
    }

    /// Deserialize HEX string into JSON (u64 account name)
    pub fn hex_to_json_native(&self, account: u64, datatype: &str, hex: String) -> Result<String, AbieosError> {
        let ctx = self.ctx();
        let datatype = CString::new(datatype).unwrap();
        let hex = CString::new(hex).unwrap();
        unsafe {
            let p = abieos_hex_to_json(ctx, account, datatype.as_ptr(), hex.as_ptr());
            if p.is_null() {
                Err(AbieosError::HexToJson(self.get_error()))
            } else {
                Ok(string_from_ptr(p))
            }
        }
    }


    /// Get the type for an action (string names as input)
    pub fn get_type_for_action(&self, contract: &str, action: &str) -> Result<String, AbieosError> {
        let ctx = self.ctx();
        let contract = self.string_to_name(contract).unwrap();
        let action = self.string_to_name(action).unwrap();
        unsafe {
            let p = abieos_get_type_for_action(ctx, contract, action);
            if p.is_null() {
                Err(AbieosError::GetTypeForAction(self.get_error()))
            } else {
                Ok(string_from_ptr(p))
            }
        }
    }

    /// Get the type for an action (u64 names as input)
    pub fn get_type_for_action_native(&self, contract: u64, action: u64) -> Result<String, AbieosError> {
        let ctx = self.ctx();
        let p = unsafe { abieos_get_type_for_action(ctx, contract, action) };
        if p.is_null() {
            Err(AbieosError::GetTypeForAction(self.get_error()))
        } else {
            Ok(string_from_ptr(p))
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
                Err(AbieosError::GetTypeForTable(self.get_error()))
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
                Err(AbieosError::GetTypeForActionResult(self.get_error()))
            } else {
                Ok(string_from_ptr(p))
            }
        }
    }

    /// Convert ABI binary to JSON
    pub fn abi_bin_to_json(&self, abi: Vec<u8>) -> Result<String, AbieosError> {
        let ctx = self.ctx();
        let abi_bin_data: *const c_char = abi.as_ptr() as *const c_char;
        let abi_bin_size: usize = abi.len();
        unsafe {
            let p = abieos_abi_bin_to_json(
                ctx,
                abi_bin_data,
                abi_bin_size,
            );
            if p.is_null() {
                Err(AbieosError::AbiBinToJson(self.get_error()))
            } else {
                Ok(string_from_ptr(p))
            }
        }
    }

    /// Convert ABI JSON to binary
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
                _ => Err(AbieosError::AbiJsonToBin(self.get_error()))
            }
        }
    }
}

pub mod abieos {
    
    //! # Abieos
    //! 
    //!  Abieos is a Rust wrapper for the abieos C library
    
    use crate::{abieos_context, abieos_create};

    pub fn create() -> *mut abieos_context {
        unsafe {
            abieos_create()
        }
    }
}
