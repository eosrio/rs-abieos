mod abi;
mod abi_def;
mod abi_json;
mod builtins;
mod crypto;
mod fnv;
mod hex;
mod istr;
mod json;
mod name;
mod stream;
mod swar;
mod symbol;
mod time;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

use abi::Abi;
use abi_def::AbiDef;
use fnv::FnvMap;
use hex::{hex_decode, hex_encode_into};
use name::{bytes_to_name_value, name_to_string_value, name_to_string_value_into};
use stream::Reader;

#[allow(non_camel_case_types)]
pub type abieos_bool = c_int;

#[allow(non_camel_case_types)]
pub type abieos_context = abieos_context_s;

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Default)]
pub struct abieos_context_s {
    last_error: CString,
    result_str: Vec<u8>,
    result_bin: Vec<u8>,
    contracts: FnvMap<u64, Abi>,
}

fn cstring_lossy(s: &str) -> CString {
    CString::new(s.replace('\0', "")).expect("interior nul removed")
}

unsafe fn cstr_arg_borrowed<'a>(ptr: *const c_char) -> Result<&'a str, String> {
    if ptr.is_null() {
        Ok("")
    } else {
        CStr::from_ptr(ptr)
            .to_str()
            .map_err(|e| format!("Invalid UTF-8 in C-string: {}", e))
    }
}

fn set_result_str(ctx: &mut abieos_context_s, s: &str) -> *const c_char {
    ctx.result_str.clear();
    ctx.result_str.extend_from_slice(s.as_bytes());
    ctx.result_str.push(0);
    ctx.result_str.as_ptr().cast()
}

unsafe fn cstr_arg(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

unsafe fn bytes_arg<'a>(ptr: *const c_char, len: usize) -> &'a [u8] {
    if ptr.is_null() || len == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(ptr.cast::<u8>(), len)
    }
}

fn set_error(ctx: &mut abieos_context_s, error: impl Into<String>) -> abieos_bool {
    ctx.last_error = cstring_lossy(&error.into());
    0
}

unsafe fn with_ctx<T>(
    context: *mut abieos_context,
    errval: T,
    f: impl FnOnce(&mut abieos_context_s) -> Result<T, String>,
) -> T {
    let Some(ctx) = context.as_mut() else {
        return errval;
    };
    match f(ctx) {
        Ok(value) => value,
        Err(error) => {
            ctx.last_error = cstring_lossy(&error);
            errval
        }
    }
}

pub unsafe extern "C" fn abieos_create() -> *mut abieos_context {
    Box::into_raw(Box::new(abieos_context_s::default()))
}

pub unsafe extern "C" fn abieos_destroy(context: *mut abieos_context) {
    if !context.is_null() {
        drop(Box::from_raw(context));
    }
}

pub unsafe extern "C" fn abieos_get_error(context: *mut abieos_context) -> *const c_char {
    context
        .as_ref()
        .map(|ctx| ctx.last_error.as_ptr())
        .unwrap_or(c"context is null".as_ptr())
}

pub unsafe extern "C" fn abieos_get_bin_size(context: *mut abieos_context) -> c_int {
    context
        .as_ref()
        .map(|ctx| ctx.result_bin.len() as c_int)
        .unwrap_or(0)
}

pub unsafe extern "C" fn abieos_get_bin_data(context: *mut abieos_context) -> *const c_char {
    context
        .as_ref()
        .map(|ctx| ctx.result_bin.as_ptr().cast::<c_char>())
        .unwrap_or(std::ptr::null())
}

pub unsafe extern "C" fn abieos_get_bin_hex(context: *mut abieos_context) -> *const c_char {
    with_ctx(context, std::ptr::null(), |ctx| {
        ctx.result_str.clear();
        hex_encode_into(&ctx.result_bin, &mut ctx.result_str);
        ctx.result_str.push(0);
        Ok(ctx.result_str.as_ptr().cast())
    })
}

pub unsafe extern "C" fn abieos_string_to_name(
    _context: *mut abieos_context,
    str_: *const c_char,
) -> u64 {
    if str_.is_null() {
        0
    } else {
        bytes_to_name_value(CStr::from_ptr(str_).to_bytes())
    }
}

pub unsafe extern "C" fn abieos_name_to_string(
    context: *mut abieos_context,
    name: u64,
) -> *const c_char {
    with_ctx(context, std::ptr::null(), |ctx| {
        ctx.result_str.clear();
        name_to_string_value_into(name, &mut ctx.result_str);
        ctx.result_str.push(0);
        Ok(ctx.result_str.as_ptr().cast())
    })
}

pub unsafe extern "C" fn abieos_set_abi(
    context: *mut abieos_context,
    contract: u64,
    abi: *const c_char,
) -> abieos_bool {
    let abi_json = cstr_arg(abi);
    with_ctx(context, 0, |ctx| {
        let def = AbiDef::from_json_str(&abi_json)?;
        let abi = Abi::from_def(&def)?;
        ctx.contracts.insert(contract, abi);
        Ok(1)
    })
}

pub unsafe extern "C" fn abieos_set_abi_bin(
    context: *mut abieos_context,
    contract: u64,
    data: *const c_char,
    size: usize,
) -> abieos_bool {
    let bytes = bytes_arg(data, size);
    with_ctx(context, 0, |ctx| {
        if bytes.is_empty() {
            return Err("no data".into());
        }
        let def = AbiDef::read_bin(&mut Reader::new(bytes))?;
        let abi = Abi::from_def(&def)?;
        ctx.contracts.insert(contract, abi);
        Ok(1)
    })
}

pub unsafe extern "C" fn abieos_set_abi_hex(
    context: *mut abieos_context,
    contract: u64,
    hex: *const c_char,
) -> abieos_bool {
    let hex = cstr_arg(hex);
    with_ctx(context, 0, |ctx| {
        let data = hex_decode(&hex)?;
        if data.is_empty() {
            return Err("no data".into());
        }
        let def = AbiDef::read_bin(&mut Reader::new(&data))?;
        let abi = Abi::from_def(&def)?;
        ctx.contracts.insert(contract, abi);
        Ok(1)
    })
}

pub unsafe extern "C" fn abieos_get_type_for_action(
    context: *mut abieos_context,
    contract: u64,
    action: u64,
) -> *const c_char {
    with_ctx(context, std::ptr::null(), |ctx| {
        let abi = ctx.contracts.get(&contract).ok_or_else(|| {
            format!(
                "contract \"{}\" is not loaded",
                name_to_string_value(contract)
            )
        })?;
        let ty = abi
            .action_types
            .get(&action)
            .ok_or_else(|| {
                format!(
                    "contract \"{}\" does not have action \"{}\"",
                    name_to_string_value(contract),
                    name_to_string_value(action)
                )
            })?
            .clone();
        Ok(set_result_str(ctx, &ty))
    })
}

pub unsafe extern "C" fn abieos_get_type_for_table(
    context: *mut abieos_context,
    contract: u64,
    table: u64,
) -> *const c_char {
    with_ctx(context, std::ptr::null(), |ctx| {
        let abi = ctx.contracts.get(&contract).ok_or_else(|| {
            format!(
                "contract \"{}\" is not loaded",
                name_to_string_value(contract)
            )
        })?;
        let ty = abi
            .table_types
            .get(&table)
            .ok_or_else(|| {
                format!(
                    "contract \"{}\" does not have table \"{}\"",
                    name_to_string_value(contract),
                    name_to_string_value(table)
                )
            })?
            .clone();
        Ok(set_result_str(ctx, &ty))
    })
}

pub unsafe extern "C" fn abieos_get_type_for_action_result(
    context: *mut abieos_context,
    contract: u64,
    action_result: u64,
) -> *const c_char {
    with_ctx(context, std::ptr::null(), |ctx| {
        let abi = ctx.contracts.get(&contract).ok_or_else(|| {
            format!(
                "contract \"{}\" is not loaded",
                name_to_string_value(contract)
            )
        })?;
        let ty = abi
            .action_result_types
            .get(&action_result)
            .ok_or_else(|| {
                format!(
                    "contract \"{}\" does not have action_result \"{}\"",
                    name_to_string_value(contract),
                    name_to_string_value(action_result)
                )
            })?
            .clone();
        Ok(set_result_str(ctx, &ty))
    })
}

pub unsafe extern "C" fn abieos_json_to_bin(
    context: *mut abieos_context,
    contract: u64,
    type_: *const c_char,
    json: *const c_char,
) -> abieos_bool {
    json_to_bin_impl(context, contract, type_, json, false)
}

pub unsafe extern "C" fn abieos_json_to_bin_reorderable(
    context: *mut abieos_context,
    contract: u64,
    type_: *const c_char,
    json: *const c_char,
) -> abieos_bool {
    json_to_bin_impl(context, contract, type_, json, true)
}

unsafe fn json_to_bin_impl(
    context: *mut abieos_context,
    contract: u64,
    type_: *const c_char,
    json: *const c_char,
    reorderable: bool,
) -> abieos_bool {
    let type_name = match cstr_arg_borrowed(type_) {
        Ok(t) => t,
        Err(e) => return with_ctx(context, 0, |_| Err(e)),
    };
    let json_str = match cstr_arg_borrowed(json) {
        Ok(j) => j,
        Err(e) => return with_ctx(context, 0, |_| Err(e)),
    };
    with_ctx(context, 0, |ctx| {
        if let Some(abi) = ctx.contracts.get_mut(&contract) {
            abi.json_to_bin(type_name, json_str, reorderable, &mut ctx.result_bin)?;
        } else if contract == 0 {
            Abi::builtin_only().json_to_bin(
                type_name,
                json_str,
                reorderable,
                &mut ctx.result_bin,
            )?;
        } else {
            return Err(format!(
                "contract \"{}\" is not loaded",
                name_to_string_value(contract)
            ));
        };
        Ok(1)
    })
}

pub unsafe extern "C" fn abieos_bin_to_json(
    context: *mut abieos_context,
    contract: u64,
    type_: *const c_char,
    data: *const c_char,
    size: usize,
) -> *const c_char {
    let type_name = match cstr_arg_borrowed(type_) {
        Ok(t) => t,
        Err(e) => return with_ctx(context, std::ptr::null(), |_| Err(e)),
    };
    let bytes = bytes_arg(data, size);
    with_ctx(context, std::ptr::null(), |ctx| {
        let json = if let Some(abi) = ctx.contracts.get_mut(&contract) {
            abi.bin_to_json(type_name, bytes)?
        } else if contract == 0 {
            Abi::builtin_only().bin_to_json(type_name, bytes)?
        } else {
            return Err(format!(
                "contract \"{}\" is not loaded",
                name_to_string_value(contract)
            ));
        };
        Ok(set_result_str(ctx, &json))
    })
}

pub unsafe extern "C" fn abieos_hex_to_json(
    context: *mut abieos_context,
    contract: u64,
    type_: *const c_char,
    hex: *const c_char,
) -> *const c_char {
    let hex_str = match cstr_arg_borrowed(hex) {
        Ok(h) => h,
        Err(e) => {
            if let Some(ctx) = context.as_mut() {
                set_error(ctx, e);
            }
            return std::ptr::null();
        }
    };
    match hex_decode(hex_str) {
        Ok(data) => abieos_bin_to_json(
            context,
            contract,
            type_,
            data.as_ptr().cast::<c_char>(),
            data.len(),
        ),
        Err(e) => {
            if let Some(ctx) = context.as_mut() {
                set_error(ctx, e);
            }
            std::ptr::null()
        }
    }
}

pub unsafe extern "C" fn abieos_abi_json_to_bin(
    context: *mut abieos_context,
    json: *const c_char,
) -> abieos_bool {
    let json_str = match cstr_arg_borrowed(json) {
        Ok(j) => j,
        Err(e) => return with_ctx(context, 0, |_| Err(e)),
    };
    with_ctx(context, 0, |ctx| {
        let def = AbiDef::from_json_str(json_str)?;
        def.check_version()?;
        ctx.result_bin.reserve(json_str.len());
        def.to_bin_into(&mut ctx.result_bin);
        Ok(1)
    })
}

pub unsafe extern "C" fn abieos_abi_bin_to_json(
    context: *mut abieos_context,
    abi_bin_data: *const c_char,
    abi_bin_data_size: usize,
) -> *const c_char {
    let bytes = bytes_arg(abi_bin_data, abi_bin_data_size);
    with_ctx(context, std::ptr::null(), |ctx| {
        if bytes.is_empty() {
            return Err("no data".into());
        }
        let def = AbiDef::read_bin(&mut Reader::new(bytes))?;
        def.check_version()?;
        Ok(set_result_str(ctx, &def.to_json_string()))
    })
}

pub unsafe extern "C" fn abieos_delete_contract(
    context: *mut abieos_context,
    contract: u64,
) -> abieos_bool {
    context
        .as_mut()
        .map(|ctx| ctx.contracts.remove(&contract).is_some() as abieos_bool)
        .unwrap_or(0)
}

// Helper for building path-aware error messages (foundation for full parity)
pub(crate) fn with_field_path(base: &str, field: &str) -> String {
    if base.is_empty() {
        format!("in field '{}'", field)
    } else {
        format!("{} > {}", base, field)
    }
}
