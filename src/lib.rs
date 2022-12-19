#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::{CStr, CString};

include!("./bindings.rs");

pub struct Abieos {
    pub context: Option<*mut abieos_context>,
}

impl Abieos {
    pub fn new() -> Abieos {
        unsafe {
            let context = abieos_create();
            return Abieos {
                context: Some(context)
            };
        }
    }

    fn ctx(&self) -> *mut abieos_context {
        self.context.unwrap()
    }

    /// Convert a String into a u64 native name
    pub fn string_to_name(&self, name: String) -> u64 {
        let ctx = self.ctx();
        if name.len() > 12 {
            return 0;
        }
        let name = CString::new(name.as_bytes()).unwrap();
        let name_ptr = name.as_ptr();
        unsafe {
            abieos_string_to_name(ctx, name_ptr)
        }
    }

    /// Convert a u64 native name into a String
    pub fn name_to_string(&self, name: u64) -> String {
        let ctx = self.ctx();
        unsafe {
            let c_buf = abieos_name_to_string(ctx, name);
            CStr::from_ptr(c_buf).to_str().unwrap().to_owned()
        }
    }

    /// Load a contract ABI in memory
    pub fn load_abi(&self, contract: String, abi_content: String) -> i32 {
        let ctx = self.ctx();
        unsafe {
            let abi = CString::new(abi_content.as_bytes()).unwrap();
            let abi_ptr = abi.as_ptr();
            let contract = self.string_to_name(contract);
            abieos_set_abi(ctx, contract, abi_ptr)
        }
    }

    /// Serialize JSON into binary
    pub fn json_to_hex(&self, account: String, datatype: String, json: String) -> String {
        let ctx = self.ctx();
        unsafe {
            let datatype = CString::new(datatype.as_bytes()).unwrap();
            let json = CString::new(json.as_bytes()).unwrap();
            let account = self.string_to_name(account);
            let status = abieos_json_to_bin(
                ctx,
                account,
                datatype.as_ptr(),
                json.as_ptr(),
            );
            if status == 1 {
                let c_buf = abieos_get_bin_hex(ctx);
                CStr::from_ptr(c_buf).to_str().unwrap().to_owned()
            } else {
                String::from("")
            }
        }
    }

    /// Deserialize HEX string into JSON
    pub fn hex_to_json(&self, account: String, datatype: String, hex: String) -> String {
        let ctx = self.ctx();
        unsafe {
            let datatype = CString::new(datatype.as_bytes()).unwrap();
            let hex = CString::new(hex.as_bytes()).unwrap();
            let account = self.string_to_name(account);
            let c_buf = abieos_hex_to_json(
                ctx,
                account,
                datatype.as_ptr(),
                hex.as_ptr(),
            );
            CStr::from_ptr(c_buf).to_str().unwrap().to_owned()
        }
    }
}

pub mod abieos {
    use crate::{abieos_context, abieos_create};

    pub fn create() -> *mut abieos_context {
        unsafe {
            let context = abieos_create();
            println!("{:?}", context);
            context
        }
    }
}
