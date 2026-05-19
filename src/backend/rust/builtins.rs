use super::crypto::{read_key_like, write_key_like, KeyKind};
use super::hex::{hex_decode, hex_encode};
use super::json::{quote_json, Json};
use super::name::{name_to_string_value, string_to_name_value};
use super::stream::{Reader, Writer};
use super::symbol::{
    asset_to_string, bitset_from_string, bitset_to_string, string_to_asset, string_to_symbol,
    string_to_symbol_code, symbol_code_to_string, symbol_to_string,
};
use super::time::{format_time_microseconds, parse_time_microseconds, parse_time_seconds};

pub(crate) const BUILTINS: &[&str] = &[
    "bool",
    "int8",
    "uint8",
    "int16",
    "uint16",
    "int32",
    "uint32",
    "int64",
    "uint64",
    "int128",
    "uint128",
    "varuint32",
    "varint32",
    "float32",
    "float64",
    "float128",
    "float",
    "double",
    "time_point",
    "time_point_sec",
    "block_timestamp",
    "block_timestamp_type",
    "name",
    "bytes",
    "string",
    "checksum160",
    "checksum256",
    "checksum512",
    "public_key",
    "private_key",
    "signature",
    "symbol",
    "symbol_code",
    "asset",
    "bitset",
];

fn parse_num<T: std::str::FromStr>(value: &Json, msg: &str) -> Result<T, String> {
    value.as_str_like()?.parse().map_err(|_| msg.into())
}

fn parse_int_strict(s: &str) -> Result<i128, String> {
    if s.is_empty() {
        return Err("Expected integer".into());
    }
    let mut bytes = s.bytes();
    let mut negative = false;
    let mut first = true;
    let mut uval = 0u128;
    let mut found_digit = false;

    for b in bytes {
        if first {
            first = false;
            if b == b'-' {
                negative = true;
                continue;
            }
        }
        if b >= b'0' && b <= b'9' {
            let digit = (b - b'0') as u128;
            uval = uval.checked_mul(10)
                .and_then(|v| v.checked_add(digit))
                .ok_or_else(|| "number is out of range".to_string())?;
            found_digit = true;
        } else {
            return Err("Expected integer".into());
        }
    }
    if !found_digit {
        return Err("Expected integer".into());
    }

    if negative {
        const MIN_ABS: u128 = 170141183460469231731687303715884105728;
        if uval > MIN_ABS {
            return Err("number is out of range".into());
        }
        if uval == MIN_ABS {
            Ok(i128::MIN)
        } else {
            Ok(-(uval as i128))
        }
    } else {
        if uval > i128::MAX as u128 {
            return Err("number is out of range".into());
        }
        Ok(uval as i128)
    }
}

fn parse_uint_strict(s: &str) -> Result<u128, String> {
    if s.is_empty() {
        return Err("Expected integer".into());
    }
    let mut uval = 0u128;
    let mut found_digit = false;
    for b in s.bytes() {
        if b >= b'0' && b <= b'9' {
            let digit = (b - b'0') as u128;
            uval = uval.checked_mul(10)
                .and_then(|v| v.checked_add(digit))
                .ok_or_else(|| "number is out of range".to_string())?;
            found_digit = true;
        } else {
            return Err("Expected integer".into());
        }
    }
    if !found_digit {
        return Err("Expected integer".into());
    }
    Ok(uval)
}

fn parse_int_range(value: &Json, min: i128, max: i128) -> Result<i128, String> {
    let s = value.as_str_like()?;
    let v = parse_int_strict(s)?;
    if v < min || v > max {
        Err("number is out of range".into())
    } else {
        Ok(v)
    }
}

fn parse_uint_range(value: &Json, max: u128) -> Result<u128, String> {
    let s = value.as_str_like()?;
    let v = parse_uint_strict(s)?;
    if v > max {
        Err("number is out of range".into())
    } else {
        Ok(v)
    }
}

pub(crate) fn write_builtin(type_name: &str, value: &Json, w: &mut Writer) -> Result<(), String> {
    match type_name {
        "bool" => match value {
            Json::Bool(v) => {
                w.push(*v as u8);
                Ok(())
            }
            _ => Err("Expected true or false".into()),
        },
        "int8" => {
            w.push(parse_int_range(value, i8::MIN as i128, i8::MAX as i128)? as i8 as u8);
            Ok(())
        }
        "uint8" => {
            w.push(parse_uint_range(value, u8::MAX as u128)? as u8);
            Ok(())
        }
        "int16" => {
            w.i16(parse_int_range(value, i16::MIN as i128, i16::MAX as i128)? as i16);
            Ok(())
        }
        "uint16" => {
            w.u16(parse_uint_range(value, u16::MAX as u128)? as u16);
            Ok(())
        }
        "int32" => {
            w.i32(parse_int_range(value, i32::MIN as i128, i32::MAX as i128)? as i32);
            Ok(())
        }
        "uint32" => {
            w.u32(parse_uint_range(value, u32::MAX as u128)? as u32);
            Ok(())
        }
        "int64" => {
            w.i64(parse_int_range(value, i64::MIN as i128, i64::MAX as i128)? as i64);
            Ok(())
        }
        "uint64" => {
            w.u64(parse_uint_range(value, u64::MAX as u128)? as u64);
            Ok(())
        }
        "int128" => {
            w.i128(parse_num(value, "Expected integer")?);
            Ok(())
        }
        "uint128" => {
            w.u128(parse_num(value, "Expected integer")?);
            Ok(())
        }
        "varuint32" => {
            w.varuint32(parse_uint_range(value, u32::MAX as u128)? as u32);
            Ok(())
        }
        "varint32" => {
            let v = parse_int_range(value, i32::MIN as i128, i32::MAX as i128)? as i32;
            w.varuint32(((v as u32) << 1) ^ ((v >> 31) as u32));
            Ok(())
        }
        "float" | "float32" => {
            w.write(&parse_num::<f32>(value, "Expected number or boolean")?.to_le_bytes());
            Ok(())
        }
        "double" | "float64" => {
            w.write(&parse_num::<f64>(value, "Expected number or boolean")?.to_le_bytes());
            Ok(())
        }
        "float128" => {
            let bytes = fixed_hex(value, 16)?;
            w.write(&bytes);
            Ok(())
        }
        "time_point" => {
            w.i64(parse_time_microseconds(value.as_str_like()?)? as i64);
            Ok(())
        }
        "time_point_sec" => {
            w.u32(parse_time_seconds(value.as_str_like()?)?);
            Ok(())
        }
        "block_timestamp" | "block_timestamp_type" => {
            let us = parse_time_microseconds(value.as_str_like()?)? as i64;
            let slot = ((us / 1000 - 946_684_800_000i64) / 500) as u32;
            w.u32(slot);
            Ok(())
        }
        "name" => {
            w.u64(string_to_name_value(value.as_str_like()?));
            Ok(())
        }
        "string" => {
            w.string(value.as_str_like()?);
            Ok(())
        }
        "bytes" => {
            w.bytes_vec(&hex_decode(value.as_str_like()?)?);
            Ok(())
        }
        "checksum160" => {
            w.write(&fixed_hex(value, 20)?);
            Ok(())
        }
        "checksum256" => {
            w.write(&fixed_hex(value, 32)?);
            Ok(())
        }
        "checksum512" => {
            w.write(&fixed_hex(value, 64)?);
            Ok(())
        }
        "symbol_code" => {
            w.u64(string_to_symbol_code(value.as_str_like()?).map_err(|_| "Expected symbol code".to_string())?);
            Ok(())
        }
        "symbol" => {
            w.u64(string_to_symbol(value.as_str_like()?).map_err(|_| "Expected symbol".to_string())?);
            Ok(())
        }
        "asset" => {
            let (amount, symbol) = string_to_asset(value.as_str_like()?).map_err(|_| "Expected symbol code".to_string())?;
            w.i64(amount);
            w.u64(symbol);
            Ok(())
        }
        "bitset" => {
            let bits = bitset_from_string(value.as_str_like()?)?;
            w.varuint32(value.as_str_like()?.len() as u32);
            w.write(&bits);
            Ok(())
        }
        "public_key" => write_key_like(value.as_str_like()?, w, KeyKind::Public),
        "private_key" => write_key_like(value.as_str_like()?, w, KeyKind::Private),
        "signature" => write_key_like(value.as_str_like()?, w, KeyKind::Signature),
        _ => Err(format!("unsupported builtin type \"{}\"", type_name)),
    }
}

fn fixed_hex(value: &Json, len: usize) -> Result<Vec<u8>, String> {
    let bytes = hex_decode(value.as_str_like()?)?;
    if bytes.len() != len {
        return Err("Hex string has incorrect length".into());
    }
    Ok(bytes)
}

fn push_float_json(value: f64, out: &mut String) {
    if value == f64::INFINITY {
        quote_json("Infinity", out);
    } else if value == f64::NEG_INFINITY {
        quote_json("-Infinity", out);
    } else if value.is_nan() {
        quote_json("NaN", out);
    } else {
        out.push_str(&format_finite_float_json(value));
    }
}

fn format_finite_float_json(value: f64) -> String {
    // C++ abieos tries std::to_chars(..., fixed) in a 25-byte buffer first.
    if value.fract() == 0.0 {
        let fixed = format!("{value:.0}");
        if fixed.len() <= 25 {
            return fixed;
        }
        return format_default_float_json(value);
    }

    let fixed = value.to_string();
    if fixed.contains('e') || fixed.contains('E') {
        if let Some(expanded) = expand_exponent_float(&fixed) {
            if expanded.len() <= 25 {
                return expanded;
            }
        }
    } else if fixed.len() <= 25 {
        return fixed;
    }

    format_default_float_json(value)
}

fn format_default_float_json(value: f64) -> String {
    let mut s = format!("{value:?}");
    if let Some(exp) = s.find('e') {
        let next = s.as_bytes().get(exp + 1).copied();
        if !matches!(next, Some(b'+' | b'-')) {
            s.insert(exp + 1, '+');
        }
    }
    s
}

fn expand_exponent_float(s: &str) -> Option<String> {
    let exp_pos = s.find(['e', 'E'])?;
    let exp = s[exp_pos + 1..].parse::<i32>().ok()?;
    let mantissa = &s[..exp_pos];
    let sign_len = usize::from(mantissa.starts_with('-'));
    let unsigned = &mantissa[sign_len..];
    let point = unsigned.find('.').unwrap_or(unsigned.len());
    let mut digits = unsigned.replace('.', "");
    let decimal_pos = point as i32 + exp;
    let mut out = String::new();

    if sign_len != 0 {
        out.push('-');
    }
    if decimal_pos <= 0 {
        out.push_str("0.");
        out.extend(std::iter::repeat('0').take(decimal_pos.unsigned_abs() as usize));
        out.push_str(&digits);
    } else if decimal_pos as usize >= digits.len() {
        out.push_str(&digits);
        out.extend(std::iter::repeat('0').take(decimal_pos as usize - digits.len()));
    } else {
        let fractional = digits.split_off(decimal_pos as usize);
        out.push_str(&digits);
        out.push('.');
        out.push_str(&fractional);
    }
    if let Some(dot) = out.find('.') {
        while out.ends_with('0') {
            out.pop();
        }
        if out.len() == dot + 1 {
            out.pop();
        }
    }
    Some(out)
}

pub(crate) fn read_builtin(type_name: &str, r: &mut Reader, out: &mut String) -> Result<(), String> {
    match type_name {
        "bool" => out.push_str(if r.byte()? != 0 { "true" } else { "false" }),
        "int8" => out.push_str(&(r.byte()? as i8).to_string()),
        "uint8" => out.push_str(&r.byte()?.to_string()),
        "int16" => out.push_str(&r.i16()?.to_string()),
        "uint16" => out.push_str(&r.u16()?.to_string()),
        "int32" => out.push_str(&r.i32()?.to_string()),
        "uint32" => out.push_str(&r.u32()?.to_string()),
        "int64" => quote_json(&r.i64()?.to_string(), out),
        "uint64" => quote_json(&r.u64()?.to_string(), out),
        "int128" => quote_json(&r.i128()?.to_string(), out),
        "uint128" => quote_json(&r.u128()?.to_string(), out),
        "varuint32" => out.push_str(&r.varuint32()?.to_string()),
        "varint32" => {
            let v = r.varuint32()?;
            let n = ((v >> 1) as i32) ^ (-((v & 1) as i32));
            out.push_str(&n.to_string());
        }
        "float" | "float32" => push_float_json(r.f32()? as f64, out),
        "double" | "float64" => push_float_json(r.f64()?, out),
        "float128" => quote_json(&hex_encode(r.read(16)?), out),
        "time_point" => quote_json(&format_time_microseconds(r.i64()? as u64), out),
        "time_point_sec" => quote_json(&format_time_microseconds(r.u32()? as u64 * 1_000_000), out),
        "block_timestamp" | "block_timestamp_type" => {
            let ms = r.u32()? as u64 * 500 + 946_684_800_000u64;
            quote_json(&format_time_microseconds(ms * 1000), out);
        }
        "name" => quote_json(&name_to_string_value(r.u64()?), out),
        "string" => quote_json(&r.string()?, out),
        "bytes" => quote_json(&hex_encode(&r.bytes_vec()?), out),
        "checksum160" => quote_json(&hex_encode(r.read(20)?), out),
        "checksum256" => quote_json(&hex_encode(r.read(32)?), out),
        "checksum512" => quote_json(&hex_encode(r.read(64)?), out),
        "symbol_code" => quote_json(&symbol_code_to_string(r.u64()?), out),
        "symbol" => quote_json(&symbol_to_string(r.u64()?), out),
        "asset" => {
            let amount = r.i64()?;
            let symbol = r.u64()?;
            quote_json(&asset_to_string(amount, symbol), out);
        }
        "bitset" => {
            let bits = r.varuint32()? as usize;
            let byte_len = (bits + 7) / 8;
            quote_json(&bitset_to_string(bits, r.read(byte_len)?), out);
        }
        "public_key" => quote_json(&read_key_like(r, KeyKind::Public)?, out),
        "private_key" => quote_json(&read_key_like(r, KeyKind::Private)?, out),
        "signature" => quote_json(&read_key_like(r, KeyKind::Signature)?, out),
        _ => return Err(format!("unsupported builtin type \"{}\"", type_name)),
    }
    Ok(())
}
