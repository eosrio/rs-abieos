use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

#[allow(non_camel_case_types)]
pub type abieos_bool = c_int;

#[allow(non_camel_case_types)]
pub type abieos_context = abieos_context_s;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct abieos_context_s {
    last_error: CString,
    result_str: CString,
    result_bin: Vec<u8>,
    contracts: BTreeMap<u64, Abi>,
}

impl Default for abieos_context_s {
    fn default() -> Self {
        Self {
            last_error: cstring_lossy(""),
            result_str: cstring_lossy(""),
            result_bin: Vec::new(),
            contracts: BTreeMap::new(),
        }
    }
}

fn cstring_lossy(s: &str) -> CString {
    CString::new(s.replace('\0', "")).expect("interior nul removed")
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

#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn as_object(&self) -> Result<&[(String, Json)], String> {
        match self {
            Json::Object(fields) => Ok(fields),
            _ => Err("expected object".into()),
        }
    }

    fn as_array(&self) -> Result<&[Json], String> {
        match self {
            Json::Array(values) => Ok(values),
            _ => Err("expected array".into()),
        }
    }

    fn as_str_like(&self) -> Result<&str, String> {
        match self {
            Json::String(s) => Ok(s),
            _ => Err("expected string".into()),
        }
    }
}

struct JsonParser<'a> {
    src: &'a [u8],
    pos: usize,
    depth: usize,
}

impl<'a> JsonParser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src: src.as_bytes(),
            pos: 0,
            depth: 0,
        }
    }

    fn parse(mut self) -> Result<Json, String> {
        let value = self.parse_value()?;
        self.skip_ws();
        if self.pos != self.src.len() {
            return Err("Expected end of json".into());
        }
        Ok(value)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn bump(&mut self) -> Result<u8, String> {
        let b = self
            .peek()
            .ok_or_else(|| "Unexpected end of json".to_string())?;
        self.pos += 1;
        Ok(b)
    }

    fn expect(&mut self, b: u8, msg: &str) -> Result<(), String> {
        self.skip_ws();
        if self.bump()? == b {
            Ok(())
        } else {
            Err(msg.into())
        }
    }

    fn parse_value(&mut self) -> Result<Json, String> {
        self.skip_ws();
        if self.depth > 128 {
            return Err("recursion limit reached".into());
        }
        match self.peek() {
            Some(b'n') => {
                self.consume_lit(b"null")?;
                Ok(Json::Null)
            }
            Some(b't') => {
                self.consume_lit(b"true")?;
                Ok(Json::Bool(true))
            }
            Some(b'f') => {
                self.consume_lit(b"false")?;
                Ok(Json::Bool(false))
            }
            Some(b'"') => self.parse_string().map(Json::String),
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_object(),
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(Json::String),
            _ => Err("json parse error".into()),
        }
    }

    fn consume_lit(&mut self, lit: &[u8]) -> Result<(), String> {
        if self.src.get(self.pos..self.pos + lit.len()) == Some(lit) {
            self.pos += lit.len();
            Ok(())
        } else {
            Err("json parse error".into())
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect(b'"', "Expected string")?;
        let mut out = Vec::new();
        while let Some(b) = self.peek() {
            self.pos += 1;
            match b {
                b'"' => {
                    return String::from_utf8(out).map_err(|_| "Invalid encoding in string".into())
                }
                b'\\' => {
                    let esc = self.bump()?;
                    match esc {
                        b'"' => out.push(b'"'),
                        b'\\' => out.push(b'\\'),
                        b'/' => out.push(b'/'),
                        b'b' => out.push(0x08),
                        b'f' => out.push(0x0c),
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'u' => {
                            let mut cp = 0u32;
                            for _ in 0..4 {
                                cp = (cp << 4)
                                    | match self.bump()? {
                                        b'0'..=b'9' => (self.src[self.pos - 1] - b'0') as u32,
                                        b'a'..=b'f' => (self.src[self.pos - 1] - b'a' + 10) as u32,
                                        b'A'..=b'F' => (self.src[self.pos - 1] - b'A' + 10) as u32,
                                        _ => {
                                            return Err("Invalid escape character in string".into())
                                        }
                                    };
                            }
                            let mut buf = [0u8; 4];
                            let ch = char::from_u32(cp).unwrap_or('?');
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        _ => return Err("Invalid escape character in string".into()),
                    }
                }
                0..=31 => return Err("Invalid encoding in string".into()),
                _ => out.push(b),
            }
        }
        Err("Missing a closing quotation mark in string".into())
    }

    fn parse_number(&mut self) -> Result<String, String> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if self.peek() == Some(b'.') {
            self.pos += 1;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        std::str::from_utf8(&self.src[start..self.pos])
            .map(str::to_owned)
            .map_err(|_| "json parse error".into())
    }

    fn parse_array(&mut self) -> Result<Json, String> {
        self.expect(b'[', "Expected [")?;
        self.depth += 1;
        let mut values = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            self.depth -= 1;
            return Ok(Json::Array(values));
        }
        loop {
            values.push(self.parse_value()?);
            self.skip_ws();
            match self.bump()? {
                b',' => {}
                b']' => {
                    self.depth -= 1;
                    return Ok(Json::Array(values));
                }
                _ => return Err("Missing a comma or ']' after an array element".into()),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Json, String> {
        self.expect(b'{', "Expected {")?;
        self.depth += 1;
        let mut fields = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            self.depth -= 1;
            return Ok(Json::Object(fields));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.expect(b':', "Missing a colon after a name of object member")?;
            let value = self.parse_value()?;
            fields.push((key, value));
            self.skip_ws();
            match self.bump()? {
                b',' => {}
                b'}' => {
                    self.depth -= 1;
                    return Ok(Json::Object(fields));
                }
                _ => return Err("Missing a comma or '}' after an object member".into()),
            }
        }
    }
}

fn parse_json(src: &str) -> Result<Json, String> {
    JsonParser::new(src).parse()
}

fn quote_json(s: &str, out: &mut String) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' || c == '\u{7f}' => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

const HEX: &[u8; 16] = b"0123456789ABCDEF";

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    fn nibble(b: u8) -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        }
    }
    let bytes = s.as_bytes();
    if bytes.len() % 2 != 0 {
        return Err("Expected string containing hex".into());
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let h = nibble(pair[0]).ok_or_else(|| "expected hex string".to_string())?;
        let l = nibble(pair[1]).ok_or_else(|| "expected hex string".to_string())?;
        out.push((h << 4) | l);
    }
    Ok(out)
}

struct Writer {
    data: Vec<u8>,
}

impl Writer {
    fn new() -> Self {
        Self { data: Vec::new() }
    }
    fn bytes(self) -> Vec<u8> {
        self.data
    }
    fn push(&mut self, b: u8) {
        self.data.push(b);
    }
    fn write(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
    fn u16(&mut self, v: u16) {
        self.write(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.write(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.write(&v.to_le_bytes());
    }
    fn i16(&mut self, v: i16) {
        self.write(&v.to_le_bytes());
    }
    fn i32(&mut self, v: i32) {
        self.write(&v.to_le_bytes());
    }
    fn i64(&mut self, v: i64) {
        self.write(&v.to_le_bytes());
    }
    fn u128(&mut self, v: u128) {
        self.write(&v.to_le_bytes());
    }
    fn i128(&mut self, v: i128) {
        self.write(&v.to_le_bytes());
    }
    fn varuint32(&mut self, mut v: u32) {
        loop {
            let mut b = (v & 0x7f) as u8;
            v >>= 7;
            if v > 0 {
                b |= 0x80;
            }
            self.push(b);
            if v == 0 {
                break;
            }
        }
    }
    fn string(&mut self, s: &str) {
        self.varuint32(s.len() as u32);
        self.write(s.as_bytes());
    }
    fn bytes_vec(&mut self, bytes: &[u8]) {
        self.varuint32(bytes.len() as u32);
        self.write(bytes);
    }
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }
    fn read(&mut self, len: usize) -> Result<&'a [u8], String> {
        if self.remaining() < len {
            return Err("read datastream of length over by".into());
        }
        let start = self.pos;
        self.pos += len;
        Ok(&self.data[start..self.pos])
    }
    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.read(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.read(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.read(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.read(8)?.try_into().unwrap()))
    }
    fn i16(&mut self) -> Result<i16, String> {
        Ok(i16::from_le_bytes(self.read(2)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> Result<i32, String> {
        Ok(i32::from_le_bytes(self.read(4)?.try_into().unwrap()))
    }
    fn i64(&mut self) -> Result<i64, String> {
        Ok(i64::from_le_bytes(self.read(8)?.try_into().unwrap()))
    }
    fn u128(&mut self) -> Result<u128, String> {
        Ok(u128::from_le_bytes(self.read(16)?.try_into().unwrap()))
    }
    fn i128(&mut self) -> Result<i128, String> {
        Ok(i128::from_le_bytes(self.read(16)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_le_bytes(self.read(4)?.try_into().unwrap()))
    }
    fn f64(&mut self) -> Result<f64, String> {
        Ok(f64::from_le_bytes(self.read(8)?.try_into().unwrap()))
    }
    fn varuint32(&mut self) -> Result<u32, String> {
        let mut v = 0u32;
        let mut shift = 0;
        loop {
            if shift >= 35 {
                return Err("invalid variable-length unsigned integer".into());
            }
            let b = self.byte()?;
            v |= ((b & 0x7f) as u32) << shift;
            shift += 7;
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
    }
    fn string(&mut self) -> Result<String, String> {
        let len = self.varuint32()? as usize;
        let bytes = self.read(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| "Invalid encoding in string".into())
    }
    fn bytes_vec(&mut self) -> Result<Vec<u8>, String> {
        let len = self.varuint32()? as usize;
        Ok(self.read(len)?.to_vec())
    }
}

fn char_to_name_digit(c: u8) -> u64 {
    match c {
        b'a'..=b'z' => (c - b'a' + 6) as u64,
        b'1'..=b'5' => (c - b'1' + 1) as u64,
        _ => 0,
    }
}

fn string_to_name_value(s: &str) -> u64 {
    let bytes = s.as_bytes();
    let mut name = 0u64;
    let mut i = 0usize;
    while i < bytes.len() && i < 12 {
        name |= (char_to_name_digit(bytes[i]) & 0x1f) << (64 - 5 * (i + 1));
        i += 1;
    }
    if i < bytes.len() {
        name |= char_to_name_digit(bytes[i]) & 0x0f;
    }
    name
}

fn name_to_string_value(name: u64) -> String {
    const CHARMAP: &[u8; 32] = b".12345abcdefghijklmnopqrstuvwxyz";
    let mut tmp = name;
    let mut chars = [b'.'; 13];
    for i in 0..=12 {
        let mask = if i == 0 { 0x0f } else { 0x1f };
        chars[12 - i] = CHARMAP[(tmp & mask) as usize];
        tmp >>= if i == 0 { 4 } else { 5 };
    }
    let last = chars.iter().rposition(|c| *c != b'.');
    match last {
        Some(i) => String::from_utf8(chars[..=i].to_vec()).unwrap(),
        None => String::new(),
    }
}

#[derive(Default, Clone)]
struct TypeDef {
    new_type_name: String,
    type_name: String,
}
#[derive(Default, Clone)]
struct FieldDef {
    name: String,
    type_name: String,
}
#[derive(Default, Clone)]
struct StructDef {
    name: String,
    base: String,
    fields: Vec<FieldDef>,
}
#[derive(Default, Clone)]
struct ActionDef {
    name: u64,
    type_name: String,
    ricardian_contract: String,
}
#[derive(Default, Clone)]
struct TableDef {
    name: u64,
    index_type: String,
    key_names: Vec<String>,
    key_types: Vec<String>,
    type_name: String,
}
#[derive(Default, Clone)]
struct ClausePair {
    id: String,
    body: String,
}
#[derive(Default, Clone)]
struct ErrorMessage {
    error_code: u64,
    error_msg: String,
}
#[derive(Default, Clone)]
struct VariantDef {
    name: String,
    types: Vec<String>,
}
#[derive(Default, Clone)]
struct ActionResultDef {
    name: u64,
    result_type: String,
}
#[derive(Default, Clone)]
struct AbiDef {
    version: String,
    types: Vec<TypeDef>,
    structs: Vec<StructDef>,
    actions: Vec<ActionDef>,
    tables: Vec<TableDef>,
    ricardian_clauses: Vec<ClausePair>,
    error_messages: Vec<ErrorMessage>,
    abi_extensions: Vec<(u16, Vec<u8>)>,
    variants: Vec<VariantDef>,
    action_results: Vec<ActionResultDef>,
}

fn obj_field<'a>(obj: &'a [(String, Json)], name: &str) -> Option<&'a Json> {
    obj.iter().find(|(k, _)| k == name).map(|(_, v)| v)
}

fn json_string(obj: &[(String, Json)], name: &str) -> Result<String, String> {
    Ok(obj_field(obj, name)
        .map(Json::as_str_like)
        .transpose()?
        .unwrap_or_default()
        .to_string())
}

fn json_name(obj: &[(String, Json)], name: &str) -> Result<u64, String> {
    Ok(string_to_name_value(&json_string(obj, name)?))
}

fn json_vec<T>(
    obj: &[(String, Json)],
    name: &str,
    mut f: impl FnMut(&Json) -> Result<T, String>,
) -> Result<Vec<T>, String> {
    let Some(value) = obj_field(obj, name) else {
        return Ok(Vec::new());
    };
    value.as_array()?.iter().map(&mut f).collect()
}

fn strings_from_json(value: &Json) -> Result<Vec<String>, String> {
    value
        .as_array()?
        .iter()
        .map(|v| v.as_str_like().map(str::to_owned))
        .collect()
}

impl AbiDef {
    fn from_json_str(json: &str) -> Result<Self, String> {
        let root = parse_json(json)?;
        Self::from_json(&root)
    }

    fn from_json(root: &Json) -> Result<Self, String> {
        let obj = root.as_object()?;
        let mut def = AbiDef {
            version: json_string(obj, "version")?,
            ..Default::default()
        };
        def.types = json_vec(obj, "types", |v| {
            let o = v.as_object()?;
            Ok(TypeDef {
                new_type_name: json_string(o, "new_type_name")?,
                type_name: json_string(o, "type")?,
            })
        })?;
        def.structs = json_vec(obj, "structs", |v| {
            let o = v.as_object()?;
            Ok(StructDef {
                name: json_string(o, "name")?,
                base: json_string(o, "base")?,
                fields: json_vec(o, "fields", |field| {
                    let f = field.as_object()?;
                    Ok(FieldDef {
                        name: json_string(f, "name")?,
                        type_name: json_string(f, "type")?,
                    })
                })?,
            })
        })?;
        def.actions = json_vec(obj, "actions", |v| {
            let o = v.as_object()?;
            Ok(ActionDef {
                name: json_name(o, "name")?,
                type_name: json_string(o, "type")?,
                ricardian_contract: json_string(o, "ricardian_contract")?,
            })
        })?;
        def.tables = json_vec(obj, "tables", |v| {
            let o = v.as_object()?;
            Ok(TableDef {
                name: json_name(o, "name")?,
                index_type: json_string(o, "index_type")?,
                key_names: obj_field(o, "key_names")
                    .map(strings_from_json)
                    .transpose()?
                    .unwrap_or_default(),
                key_types: obj_field(o, "key_types")
                    .map(strings_from_json)
                    .transpose()?
                    .unwrap_or_default(),
                type_name: json_string(o, "type")?,
            })
        })?;
        def.ricardian_clauses = json_vec(obj, "ricardian_clauses", |v| {
            let o = v.as_object()?;
            Ok(ClausePair {
                id: json_string(o, "id")?,
                body: json_string(o, "body")?,
            })
        })?;
        def.error_messages = json_vec(obj, "error_messages", |v| {
            let o = v.as_object()?;
            Ok(ErrorMessage {
                error_code: json_string(o, "error_code")?.parse().unwrap_or(0),
                error_msg: json_string(o, "error_msg")?,
            })
        })?;
        def.abi_extensions = json_vec(obj, "abi_extensions", |v| {
            let arr = v.as_array()?;
            if arr.len() != 2 {
                return Err("expected pair".into());
            }
            Ok((
                arr[0].as_str_like()?.parse().unwrap_or(0),
                hex_decode(arr[1].as_str_like()?)?,
            ))
        })?;
        def.variants = json_vec(obj, "variants", |v| {
            let o = v.as_object()?;
            Ok(VariantDef {
                name: json_string(o, "name")?,
                types: obj_field(o, "types")
                    .map(strings_from_json)
                    .transpose()?
                    .unwrap_or_default(),
            })
        })?;
        def.action_results = json_vec(obj, "action_results", |v| {
            let o = v.as_object()?;
            Ok(ActionResultDef {
                name: json_name(o, "name")?,
                result_type: json_string(o, "result_type")?,
            })
        })?;
        Ok(def)
    }

    fn check_version(&self) -> Result<(), String> {
        if self.version.starts_with("eosio::abi/1.") || self.version.starts_with("eosio::abi/2.") {
            Ok(())
        } else {
            Err("unsupported abi version".into())
        }
    }

    fn write_bin(&self, w: &mut Writer) {
        w.string(&self.version);
        w.varuint32(self.types.len() as u32);
        for t in &self.types {
            w.string(&t.new_type_name);
            w.string(&t.type_name);
        }
        w.varuint32(self.structs.len() as u32);
        for s in &self.structs {
            w.string(&s.name);
            w.string(&s.base);
            w.varuint32(s.fields.len() as u32);
            for f in &s.fields {
                w.string(&f.name);
                w.string(&f.type_name);
            }
        }
        w.varuint32(self.actions.len() as u32);
        for a in &self.actions {
            w.u64(a.name);
            w.string(&a.type_name);
            w.string(&a.ricardian_contract);
        }
        w.varuint32(self.tables.len() as u32);
        for t in &self.tables {
            w.u64(t.name);
            w.string(&t.index_type);
            write_string_vec(w, &t.key_names);
            write_string_vec(w, &t.key_types);
            w.string(&t.type_name);
        }
        w.varuint32(self.ricardian_clauses.len() as u32);
        for c in &self.ricardian_clauses {
            w.string(&c.id);
            w.string(&c.body);
        }
        w.varuint32(self.error_messages.len() as u32);
        for e in &self.error_messages {
            w.u64(e.error_code);
            w.string(&e.error_msg);
        }
        w.varuint32(self.abi_extensions.len() as u32);
        for (ty, data) in &self.abi_extensions {
            w.u16(*ty);
            w.bytes_vec(data);
        }
        w.varuint32(self.variants.len() as u32);
        for v in &self.variants {
            w.string(&v.name);
            write_string_vec(w, &v.types);
        }
        w.varuint32(self.action_results.len() as u32);
        for r in &self.action_results {
            w.u64(r.name);
            w.string(&r.result_type);
        }
    }

    fn to_bin(&self) -> Vec<u8> {
        let mut w = Writer::new();
        self.write_bin(&mut w);
        w.bytes()
    }

    fn read_bin(r: &mut Reader) -> Result<Self, String> {
        let mut def = AbiDef {
            version: r.string()?,
            ..Default::default()
        };
        def.types = read_vec(r, |r| {
            Ok(TypeDef {
                new_type_name: r.string()?,
                type_name: r.string()?,
            })
        })?;
        def.structs = read_vec(r, |r| {
            Ok(StructDef {
                name: r.string()?,
                base: r.string()?,
                fields: read_vec(r, |r| {
                    Ok(FieldDef {
                        name: r.string()?,
                        type_name: r.string()?,
                    })
                })?,
            })
        })?;
        def.actions = read_vec(r, |r| {
            Ok(ActionDef {
                name: r.u64()?,
                type_name: r.string()?,
                ricardian_contract: r.string()?,
            })
        })?;
        def.tables = read_vec(r, |r| {
            Ok(TableDef {
                name: r.u64()?,
                index_type: r.string()?,
                key_names: read_string_vec(r)?,
                key_types: read_string_vec(r)?,
                type_name: r.string()?,
            })
        })?;
        def.ricardian_clauses = read_vec(r, |r| {
            Ok(ClausePair {
                id: r.string()?,
                body: r.string()?,
            })
        })?;
        def.error_messages = read_vec(r, |r| {
            Ok(ErrorMessage {
                error_code: r.u64()?,
                error_msg: r.string()?,
            })
        })?;
        def.abi_extensions = read_vec(r, |r| Ok((r.u16()?, r.bytes_vec()?)))?;
        if r.remaining() > 0 {
            def.variants = read_vec(r, |r| {
                Ok(VariantDef {
                    name: r.string()?,
                    types: read_string_vec(r)?,
                })
            })?;
        }
        if r.remaining() > 0 {
            def.action_results = read_vec(r, |r| {
                Ok(ActionResultDef {
                    name: r.u64()?,
                    result_type: r.string()?,
                })
            })?;
        }
        Ok(def)
    }

    fn to_json_string(&self) -> String {
        let mut out = String::new();
        out.push('{');
        json_kv(
            &mut out,
            "version",
            |out| quote_json(&self.version, out),
            false,
        );
        json_kv(
            &mut out,
            "types",
            |out| {
                json_array(out, self.types.iter(), |out, t| {
                    out.push('{');
                    json_kv(
                        out,
                        "new_type_name",
                        |out| quote_json(&t.new_type_name, out),
                        false,
                    );
                    json_kv(out, "type", |out| quote_json(&t.type_name, out), true);
                    out.push('}');
                })
            },
            true,
        );
        json_kv(
            &mut out,
            "structs",
            |out| {
                json_array(out, self.structs.iter(), |out, s| {
                    out.push('{');
                    json_kv(out, "name", |out| quote_json(&s.name, out), false);
                    json_kv(out, "base", |out| quote_json(&s.base, out), true);
                    json_kv(
                        out,
                        "fields",
                        |out| {
                            json_array(out, s.fields.iter(), |out, f| {
                                out.push('{');
                                json_kv(out, "name", |out| quote_json(&f.name, out), false);
                                json_kv(out, "type", |out| quote_json(&f.type_name, out), true);
                                out.push('}');
                            })
                        },
                        true,
                    );
                    out.push('}');
                })
            },
            true,
        );
        json_kv(
            &mut out,
            "actions",
            |out| {
                json_array(out, self.actions.iter(), |out, a| {
                    out.push('{');
                    json_kv(
                        out,
                        "name",
                        |out| quote_json(&name_to_string_value(a.name), out),
                        false,
                    );
                    json_kv(out, "type", |out| quote_json(&a.type_name, out), true);
                    json_kv(
                        out,
                        "ricardian_contract",
                        |out| quote_json(&a.ricardian_contract, out),
                        true,
                    );
                    out.push('}');
                })
            },
            true,
        );
        json_kv(
            &mut out,
            "tables",
            |out| {
                json_array(out, self.tables.iter(), |out, t| {
                    out.push('{');
                    json_kv(
                        out,
                        "name",
                        |out| quote_json(&name_to_string_value(t.name), out),
                        false,
                    );
                    json_kv(
                        out,
                        "index_type",
                        |out| quote_json(&t.index_type, out),
                        true,
                    );
                    json_kv(
                        out,
                        "key_names",
                        |out| json_string_array(out, &t.key_names),
                        true,
                    );
                    json_kv(
                        out,
                        "key_types",
                        |out| json_string_array(out, &t.key_types),
                        true,
                    );
                    json_kv(out, "type", |out| quote_json(&t.type_name, out), true);
                    out.push('}');
                })
            },
            true,
        );
        json_kv(
            &mut out,
            "ricardian_clauses",
            |out| {
                json_array(out, self.ricardian_clauses.iter(), |out, c| {
                    out.push('{');
                    json_kv(out, "id", |out| quote_json(&c.id, out), false);
                    json_kv(out, "body", |out| quote_json(&c.body, out), true);
                    out.push('}');
                })
            },
            true,
        );
        json_kv(
            &mut out,
            "error_messages",
            |out| {
                json_array(out, self.error_messages.iter(), |out, e| {
                    out.push('{');
                    json_kv(
                        out,
                        "error_code",
                        |out| out.push_str(&e.error_code.to_string()),
                        false,
                    );
                    json_kv(out, "error_msg", |out| quote_json(&e.error_msg, out), true);
                    out.push('}');
                })
            },
            true,
        );
        json_kv(
            &mut out,
            "variants",
            |out| {
                json_array(out, self.variants.iter(), |out, v| {
                    out.push('{');
                    json_kv(out, "name", |out| quote_json(&v.name, out), false);
                    json_kv(out, "types", |out| json_string_array(out, &v.types), true);
                    out.push('}');
                })
            },
            true,
        );
        json_kv(
            &mut out,
            "action_results",
            |out| {
                json_array(out, self.action_results.iter(), |out, r| {
                    out.push('{');
                    json_kv(
                        out,
                        "name",
                        |out| quote_json(&name_to_string_value(r.name), out),
                        false,
                    );
                    json_kv(
                        out,
                        "result_type",
                        |out| quote_json(&r.result_type, out),
                        true,
                    );
                    out.push('}');
                })
            },
            true,
        );
        out.push('}');
        out
    }
}

fn json_kv(out: &mut String, key: &str, f: impl FnOnce(&mut String), comma: bool) {
    if comma {
        out.push(',');
    }
    quote_json(key, out);
    out.push(':');
    f(out);
}

fn json_array<'a, T: 'a>(
    out: &mut String,
    iter: impl Iterator<Item = &'a T>,
    mut f: impl FnMut(&mut String, &'a T),
) {
    out.push('[');
    let mut first = true;
    for value in iter {
        if !first {
            out.push(',');
        }
        first = false;
        f(out, value);
    }
    out.push(']');
}

fn json_string_array(out: &mut String, values: &[String]) {
    json_array(out, values.iter(), |out, s| quote_json(s, out));
}

fn write_string_vec(w: &mut Writer, values: &[String]) {
    w.varuint32(values.len() as u32);
    for value in values {
        w.string(value);
    }
}

fn read_vec<T>(
    r: &mut Reader,
    mut f: impl FnMut(&mut Reader) -> Result<T, String>,
) -> Result<Vec<T>, String> {
    let len = r.varuint32()? as usize;
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        out.push(f(r)?);
    }
    Ok(out)
}

fn read_string_vec(r: &mut Reader) -> Result<Vec<String>, String> {
    read_vec(r, |r| r.string())
}

#[derive(Clone)]
enum TypeKind {
    Builtin,
    Alias(String),
    Optional(String),
    Extension(String),
    Array(String),
    FixedArray(String, usize),
    Struct(Vec<FieldDef>),
    Variant(Vec<String>),
}

#[derive(Clone)]
struct Abi {
    action_types: BTreeMap<u64, String>,
    table_types: BTreeMap<u64, String>,
    action_result_types: BTreeMap<u64, String>,
    types: BTreeMap<String, TypeKind>,
}

impl Abi {
    fn builtin_only() -> Self {
        let mut abi = Abi {
            action_types: BTreeMap::new(),
            table_types: BTreeMap::new(),
            action_result_types: BTreeMap::new(),
            types: BTreeMap::new(),
        };
        abi.install_builtin_types();
        abi
    }

    fn install_builtin_types(&mut self) {
        for name in BUILTINS {
            self.types.insert((*name).to_string(), TypeKind::Builtin);
        }
        self.types.insert(
            "extended_asset".into(),
            TypeKind::Struct(vec![
                FieldDef {
                    name: "quantity".into(),
                    type_name: "asset".into(),
                },
                FieldDef {
                    name: "contract".into(),
                    type_name: "name".into(),
                },
            ]),
        );
    }

    fn from_def(def: &AbiDef) -> Result<Self, String> {
        def.check_version()?;
        let mut abi = Abi {
            action_types: def
                .actions
                .iter()
                .map(|a| (a.name, a.type_name.clone()))
                .collect(),
            table_types: def
                .tables
                .iter()
                .map(|t| (t.name, t.type_name.clone()))
                .collect(),
            action_result_types: def
                .action_results
                .iter()
                .map(|r| (r.name, r.result_type.clone()))
                .collect(),
            types: BTreeMap::new(),
        };
        abi.install_builtin_types();
        for t in &def.types {
            if t.new_type_name.is_empty() {
                return Err("Missing name".into());
            }
            if abi
                .types
                .insert(
                    t.new_type_name.clone(),
                    TypeKind::Alias(t.type_name.clone()),
                )
                .is_some()
            {
                return Err("Redefined type".into());
            }
        }
        for s in &def.structs {
            if s.name.is_empty() {
                return Err("Missing name".into());
            }
            if abi
                .types
                .insert(s.name.clone(), TypeKind::Struct(Vec::new()))
                .is_some()
            {
                return Err("Redefined type".into());
            }
        }
        for v in &def.variants {
            if v.name.is_empty() {
                return Err("Missing name".into());
            }
            if abi
                .types
                .insert(v.name.clone(), TypeKind::Variant(v.types.clone()))
                .is_some()
            {
                return Err("Redefined type".into());
            }
        }
        let structs: BTreeMap<_, _> = def
            .structs
            .iter()
            .map(|s| (s.name.clone(), s.clone()))
            .collect();
        let names: Vec<String> = structs.keys().cloned().collect();
        for name in names {
            let fields = abi.resolve_struct_fields(&structs, &name, 0)?;
            abi.types.insert(name, TypeKind::Struct(fields));
        }
        for s in &def.structs {
            for f in &s.fields {
                abi.ensure_type(&f.type_name, 0)?;
            }
        }
        for v in &def.variants {
            for t in &v.types {
                abi.ensure_type(t, 0)?;
            }
        }
        let all_names: Vec<String> = abi.types.keys().cloned().collect();
        for name in all_names {
            abi.ensure_type(&name, 0)?;
        }
        Ok(abi)
    }

    fn resolve_struct_fields(
        &mut self,
        structs: &BTreeMap<String, StructDef>,
        name: &str,
        depth: usize,
    ) -> Result<Vec<FieldDef>, String> {
        if depth >= 32 {
            return Err("Recursion limit reached".into());
        }
        let s = structs
            .get(name)
            .ok_or_else(|| "Unknown type".to_string())?;
        let mut fields = Vec::new();
        if !s.base.is_empty() {
            if structs.contains_key(&s.base) {
                fields.extend(self.resolve_struct_fields(structs, &s.base, depth + 1)?);
            } else {
                match self.ensure_type(&s.base, depth + 1)? {
                    TypeKind::Struct(base_fields) => fields.extend(base_fields),
                    _ => return Err("Base not a struct".into()),
                }
            }
        }
        fields.extend(s.fields.clone());
        Ok(fields)
    }

    fn ensure_type(&mut self, name: &str, depth: usize) -> Result<TypeKind, String> {
        if depth >= 32 {
            return Err("Recursion limit reached".into());
        }
        if let Some(kind) = self.types.get(name).cloned() {
            if let TypeKind::Alias(target) = kind {
                let resolved = self.ensure_type(&target, depth + 1)?;
                if matches!(resolved, TypeKind::Extension(_)) {
                    return Err("Extension typedef not allowed".into());
                }
                return Ok(resolved);
            }
            return Ok(kind);
        }
        let kind = if let Some(base) = name.strip_suffix('?') {
            let base_kind = self.ensure_type(base, depth + 1)?;
            if matches!(base_kind, TypeKind::Optional(_) | TypeKind::Extension(_)) {
                return Err(format!("Invalid optional nesting for type: {}", name));
            }
            TypeKind::Optional(base.to_string())
        } else if let Some(base) = name.strip_suffix("[]") {
            let base_kind = self.ensure_type(base, depth + 1)?;
            if matches!(base_kind, TypeKind::Optional(_) | TypeKind::Extension(_)) {
                return Err(format!("Invalid array nesting for type: {}", name));
            }
            TypeKind::Array(base.to_string())
        } else if let Some(base) = name.strip_suffix('$') {
            let base_kind = self.ensure_type(base, depth + 1)?;
            if matches!(base_kind, TypeKind::Extension(_)) {
                return Err(format!("Invalid extension nesting for type: {}", name));
            }
            TypeKind::Extension(base.to_string())
        } else if name.ends_with(']') {
            let idx = name.rfind('[').ok_or_else(|| {
                "']' character found without matching '[' in type specification".to_string()
            })?;
            let size_text = &name[idx + 1..name.len() - 1];
            if size_text.starts_with('+') {
                return Err("Unexpected size specification for fixed array type".into());
            }
            if size_text.starts_with('0') && size_text.len() > 1 {
                return Err(
                    "Leading zeros not allowed for fixed array lengrh specification".into(),
                );
            }
            let size: isize = size_text
                .parse()
                .map_err(|_| "Unexpected size specification for fixed array type".to_string())?;
            if size == 0 {
                return Err("Zero size fixed arrays not allowed".into());
            }
            if size < 0 {
                return Err("Negative size fixed arrays not allowed".into());
            }
            let base = &name[..idx];
            let base_kind = self.ensure_type(base, depth + 1)?;
            if matches!(base_kind, TypeKind::Optional(_) | TypeKind::Extension(_)) {
                return Err(format!("Invalid array nesting for type: {}", name));
            }
            TypeKind::FixedArray(base.to_string(), size as usize)
        } else {
            return Err(format!("unknown type \"{}\"", name));
        };
        self.types.insert(name.to_string(), kind.clone());
        Ok(kind)
    }

    fn json_to_bin(
        &mut self,
        type_name: &str,
        json: &str,
        reorderable: bool,
    ) -> Result<Vec<u8>, String> {
        let value = parse_json(json)?;
        let mut w = Writer::new();
        let mut skipped_extension = false;
        self.write_json_value(
            type_name,
            &value,
            &mut w,
            true,
            reorderable,
            &mut skipped_extension,
        )?;
        Ok(w.bytes())
    }

    fn write_json_value(
        &mut self,
        type_name: &str,
        value: &Json,
        w: &mut Writer,
        allow_extensions: bool,
        reorderable: bool,
        skipped_extension: &mut bool,
    ) -> Result<(), String> {
        if let Some(TypeKind::Alias(target)) = self.types.get(type_name).cloned() {
            return self.write_json_value(
                &target,
                value,
                w,
                allow_extensions,
                reorderable,
                skipped_extension,
            );
        }
        match self.ensure_type(type_name, 0)? {
            TypeKind::Alias(target) => self.write_json_value(
                &target,
                value,
                w,
                allow_extensions,
                reorderable,
                skipped_extension,
            ),
            TypeKind::Optional(base) => {
                if matches!(value, Json::Null) {
                    w.push(0);
                } else {
                    w.push(1);
                    self.write_json_value(
                        &base,
                        value,
                        w,
                        allow_extensions,
                        reorderable,
                        skipped_extension,
                    )?;
                }
                Ok(())
            }
            TypeKind::Extension(base) => self.write_json_value(
                &base,
                value,
                w,
                allow_extensions,
                reorderable,
                skipped_extension,
            ),
            TypeKind::Array(base) => {
                let arr = value.as_array()?;
                w.varuint32(arr.len() as u32);
                for item in arr {
                    self.write_json_value(&base, item, w, false, reorderable, skipped_extension)?;
                }
                Ok(())
            }
            TypeKind::FixedArray(base, size) => {
                let arr = value.as_array()?;
                if arr.len() != size {
                    return Err("incorrect size for fixed array".into());
                }
                for item in arr {
                    self.write_json_value(&base, item, w, false, reorderable, skipped_extension)?;
                }
                Ok(())
            }
            TypeKind::Struct(fields) => {
                self.write_struct_json(&fields, value, w, allow_extensions, reorderable)
            }
            TypeKind::Variant(types) => {
                let arr = value.as_array()?;
                if arr.len() != 2 {
                    return Err(r#"Expected variant: ["type", value]"#.into());
                }
                let variant_type = arr[0].as_str_like()?;
                let idx = types
                    .iter()
                    .position(|t| t == variant_type)
                    .ok_or_else(|| "Invalid type for variant".to_string())?;
                w.varuint32(idx as u32);
                self.write_json_value(
                    &types[idx],
                    &arr[1],
                    w,
                    allow_extensions,
                    reorderable,
                    skipped_extension,
                )
            }
            TypeKind::Builtin => write_builtin(type_name, value, w),
        }
    }

    fn write_struct_json(
        &mut self,
        fields: &[FieldDef],
        value: &Json,
        w: &mut Writer,
        allow_extensions: bool,
        reorderable: bool,
    ) -> Result<(), String> {
        let obj = value.as_object()?;
        let mut skipped = false;
        let mut seen = BTreeSet::new();
        for (idx, field) in fields.iter().enumerate() {
            let found = if reorderable {
                obj.iter().find(|(k, _)| k == &field.name)
            } else {
                obj.get(idx).filter(|(k, _)| k == &field.name)
            };
            let Some((_, field_value)) = found else {
                if matches!(
                    self.ensure_type(&field.type_name, 0)?,
                    TypeKind::Extension(_)
                ) && allow_extensions
                {
                    skipped = true;
                    continue;
                }
                return Err(format!("expected field \"{}\"", field.name));
            };
            if skipped {
                return Err("Unexpected field".into());
            }
            seen.insert(field.name.as_str());
            let field_allow = allow_extensions && idx + 1 == fields.len();
            self.write_json_value(
                &field.type_name,
                field_value,
                w,
                field_allow,
                reorderable,
                &mut skipped,
            )?;
        }
        if obj.iter().any(|(k, _)| !seen.contains(k.as_str())) {
            return Err("Unexpected field".into());
        }
        Ok(())
    }

    fn bin_to_json(&mut self, type_name: &str, data: &[u8]) -> Result<String, String> {
        let mut r = Reader::new(data);
        let mut out = String::new();
        self.read_json_value(type_name, &mut r, &mut out, true)?;
        Ok(out)
    }

    fn read_json_value(
        &mut self,
        type_name: &str,
        r: &mut Reader,
        out: &mut String,
        allow_extensions: bool,
    ) -> Result<(), String> {
        if let Some(TypeKind::Alias(target)) = self.types.get(type_name).cloned() {
            return self.read_json_value(&target, r, out, allow_extensions);
        }
        match self.ensure_type(type_name, 0)? {
            TypeKind::Alias(target) => self.read_json_value(&target, r, out, allow_extensions),
            TypeKind::Optional(base) => {
                if r.byte()? == 0 {
                    out.push_str("null");
                } else {
                    self.read_json_value(&base, r, out, allow_extensions)?;
                }
                Ok(())
            }
            TypeKind::Extension(base) => self.read_json_value(&base, r, out, allow_extensions),
            TypeKind::Array(base) => {
                let len = r.varuint32()? as usize;
                out.push('[');
                for i in 0..len {
                    if i != 0 {
                        out.push(',');
                    }
                    self.read_json_value(&base, r, out, false)?;
                }
                out.push(']');
                Ok(())
            }
            TypeKind::FixedArray(base, size) => {
                out.push('[');
                for i in 0..size {
                    if i != 0 {
                        out.push(',');
                    }
                    self.read_json_value(&base, r, out, false)?;
                }
                out.push(']');
                Ok(())
            }
            TypeKind::Struct(fields) => {
                out.push('{');
                let mut wrote = false;
                for (idx, field) in fields.iter().enumerate() {
                    if r.remaining() == 0
                        && matches!(
                            self.ensure_type(&field.type_name, 0)?,
                            TypeKind::Extension(_)
                        )
                        && allow_extensions
                    {
                        continue;
                    }
                    if wrote {
                        out.push(',');
                    }
                    wrote = true;
                    quote_json(&field.name, out);
                    out.push(':');
                    self.read_json_value(
                        &field.type_name,
                        r,
                        out,
                        allow_extensions && idx + 1 == fields.len(),
                    )?;
                }
                out.push('}');
                Ok(())
            }
            TypeKind::Variant(types) => {
                let idx = r.varuint32()? as usize;
                let ty = types
                    .get(idx)
                    .ok_or_else(|| "bad variant index".to_string())?
                    .clone();
                out.push('[');
                quote_json(&ty, out);
                out.push(',');
                self.read_json_value(&ty, r, out, allow_extensions)?;
                out.push(']');
                Ok(())
            }
            TypeKind::Builtin => read_builtin(type_name, r, out),
        }
    }
}

const BUILTINS: &[&str] = &[
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

fn parse_int_range(value: &Json, min: i128, max: i128) -> Result<i128, String> {
    let v: i128 = parse_num(value, "Expected integer")?;
    if v < min || v > max {
        Err("number is out of range".into())
    } else {
        Ok(v)
    }
}

fn parse_uint_range(value: &Json, max: u128) -> Result<u128, String> {
    let v: u128 = parse_num(value, "Expected integer")?;
    if v > max {
        Err("number is out of range".into())
    } else {
        Ok(v)
    }
}

fn write_builtin(type_name: &str, value: &Json, w: &mut Writer) -> Result<(), String> {
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
            w.write(&parse_num::<f32>(value, "Expected number")?.to_le_bytes());
            Ok(())
        }
        "double" | "float64" => {
            w.write(&parse_num::<f64>(value, "Expected number")?.to_le_bytes());
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
            w.u64(string_to_symbol_code(value.as_str_like()?)?);
            Ok(())
        }
        "symbol" => {
            w.u64(string_to_symbol(value.as_str_like()?)?);
            Ok(())
        }
        "asset" => {
            let (amount, symbol) = string_to_asset(value.as_str_like()?)?;
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

fn read_builtin(type_name: &str, r: &mut Reader, out: &mut String) -> Result<(), String> {
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

fn string_to_symbol_code(s: &str) -> Result<u64, String> {
    if s.is_empty() || s.len() > 7 || !s.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err("Expected symbol code".into());
    }
    Ok(s.bytes()
        .enumerate()
        .fold(0u64, |acc, (i, b)| acc | ((b as u64) << (8 * i))))
}

fn symbol_code_to_string(mut v: u64) -> String {
    let mut out = String::new();
    while v > 0 {
        out.push((v & 0xff) as u8 as char);
        v >>= 8;
    }
    out
}

fn string_to_symbol(s: &str) -> Result<u64, String> {
    let (precision, code) = s
        .split_once(',')
        .ok_or_else(|| "Expected symbol".to_string())?;
    let precision: u8 = precision
        .parse()
        .map_err(|_| "Expected symbol".to_string())?;
    Ok((string_to_symbol_code(code)? << 8) | precision as u64)
}

fn symbol_to_string(v: u64) -> String {
    format!("{},{}", v & 0xff, symbol_code_to_string(v >> 8))
}

fn string_to_asset(s: &str) -> Result<(i64, u64), String> {
    let (amount_s, code) = s
        .trim()
        .split_once(' ')
        .ok_or_else(|| "Expected string containing asset".to_string())?;
    let negative = amount_s.starts_with('-');
    let digits = if negative { &amount_s[1..] } else { amount_s };
    let mut amount = 0i64;
    let mut precision = 0u8;
    let mut seen_dot = false;
    for b in digits.bytes() {
        match b {
            b'0'..=b'9' => {
                amount = amount
                    .checked_mul(10)
                    .and_then(|v| v.checked_add((b - b'0') as i64))
                    .ok_or_else(|| "number is out of range".to_string())?;
                if seen_dot {
                    precision += 1;
                }
            }
            b'.' if !seen_dot => seen_dot = true,
            _ => return Err("Expected string containing asset".into()),
        }
    }
    if negative {
        amount = -amount;
    }
    Ok((
        amount,
        (string_to_symbol_code(code)? << 8) | precision as u64,
    ))
}

fn asset_to_string(amount: i64, symbol: u64) -> String {
    let precision = (symbol & 0xff) as usize;
    let mut uamount = amount.unsigned_abs();
    let mut chars = Vec::new();
    for _ in 0..precision {
        chars.push((b'0' + (uamount % 10) as u8) as char);
        uamount /= 10;
    }
    if precision > 0 {
        chars.push('.');
    }
    loop {
        chars.push((b'0' + (uamount % 10) as u8) as char);
        uamount /= 10;
        if uamount == 0 {
            break;
        }
    }
    if amount < 0 {
        chars.push('-');
    }
    chars.reverse();
    format!(
        "{} {}",
        chars.into_iter().collect::<String>(),
        symbol_code_to_string(symbol >> 8)
    )
}

fn bitset_from_string(s: &str) -> Result<Vec<u8>, String> {
    let mut bytes = vec![0u8; (s.len() + 7) / 8];
    for (i, ch) in s.bytes().enumerate() {
        match ch {
            b'0' => {}
            b'1' => {
                let bit = s.len() - i - 1;
                bytes[bit / 8] |= 1 << (bit % 8);
            }
            _ => return Err("unexpected character in bitset".into()),
        }
    }
    Ok(bytes)
}

fn bitset_to_string(bits: usize, bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bits);
    for bit in (0..bits).rev() {
        out.push(if bytes[bit / 8] & (1 << (bit % 8)) != 0 {
            '1'
        } else {
            '0'
        });
    }
    out
}

fn parse_time_seconds(s: &str) -> Result<u32, String> {
    let (sec, _) = parse_time_parts(s)?;
    Ok(sec as u32)
}

fn parse_time_microseconds(s: &str) -> Result<u64, String> {
    let (sec, micros) = parse_time_parts(s)?;
    Ok(sec * 1_000_000 + micros)
}

fn parse_time_parts(s: &str) -> Result<(u64, u64), String> {
    if s.len() < 19 {
        return Err("Expected time point".into());
    }
    let y: i32 = s[0..4]
        .parse()
        .map_err(|_| "Expected time point".to_string())?;
    let m: u32 = s[5..7]
        .parse()
        .map_err(|_| "Expected time point".to_string())?;
    let d: u32 = s[8..10]
        .parse()
        .map_err(|_| "Expected time point".to_string())?;
    let h: u32 = s[11..13]
        .parse()
        .map_err(|_| "Expected time point".to_string())?;
    let min: u32 = s[14..16]
        .parse()
        .map_err(|_| "Expected time point".to_string())?;
    let sec: u32 = s[17..19]
        .parse()
        .map_err(|_| "Expected time point".to_string())?;
    if &s[4..5] != "-"
        || &s[7..8] != "-"
        || &s[10..11] != "T"
        || &s[13..14] != ":"
        || &s[16..17] != ":"
    {
        return Err("Expected time point".into());
    }
    let mut micros = 0u64;
    if s.as_bytes().get(19) == Some(&b'.') {
        let frac = &s[20..];
        let mut scale = 100_000u64;
        for b in frac.bytes().take(6) {
            if !b.is_ascii_digit() {
                break;
            }
            micros += (b - b'0') as u64 * scale;
            scale /= 10;
        }
    }
    let days = days_from_civil(y, m, d);
    Ok((
        days as u64 * 86_400 + h as u64 * 3600 + min as u64 * 60 + sec as u64,
        micros,
    ))
}

fn format_time_microseconds(us: u64) -> String {
    let secs = us / 1_000_000;
    let millis = (us / 1000) % 1000;
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}",
        y,
        m,
        d,
        rem / 3600,
        rem / 60 % 60,
        rem % 60,
        millis
    )
}

fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = y - (m <= 2) as i32;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era as i64 * 146_097 + doe as i64 - 719_468
}

fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (y + (m <= 2) as i32, m, d)
}

#[derive(Clone, Copy)]
enum KeyKind {
    Public,
    Private,
    Signature,
}

fn write_key_like(s: &str, w: &mut Writer, kind: KeyKind) -> Result<(), String> {
    let (idx, body, suffix, fixed_len, legacy_wif) = match kind {
        KeyKind::Public if let Some(rest) = s.strip_prefix("EOS") => (0, rest, "", Some(33), false),
        KeyKind::Public if let Some(rest) = s.strip_prefix("PUB_K1_") => {
            (0, rest, "K1", Some(33), false)
        }
        KeyKind::Public if let Some(rest) = s.strip_prefix("PUB_R1_") => {
            (1, rest, "R1", Some(33), false)
        }
        KeyKind::Public if let Some(rest) = s.strip_prefix("PUB_WA_") => {
            (2, rest, "WA", None, false)
        }
        KeyKind::Private if let Some(rest) = s.strip_prefix("PVT_K1_") => {
            (0, rest, "K1", Some(32), false)
        }
        KeyKind::Private if let Some(rest) = s.strip_prefix("PVT_R1_") => {
            (1, rest, "R1", Some(32), false)
        }
        KeyKind::Private if s.starts_with("PVT_") => return Err("expected private_key".into()),
        KeyKind::Private => (0, s, "", Some(32), true),
        KeyKind::Signature if let Some(rest) = s.strip_prefix("SIG_K1_") => {
            (0, rest, "K1", Some(65), false)
        }
        KeyKind::Signature if let Some(rest) = s.strip_prefix("SIG_R1_") => {
            (1, rest, "R1", Some(65), false)
        }
        KeyKind::Signature if let Some(rest) = s.strip_prefix("SIG_WA_") => {
            (2, rest, "WA", None, false)
        }
        _ => return Err("unrecognized key format".into()),
    };
    let mut decoded = base58_decode(body)?;
    if decoded.len() < 4 {
        return Err("expected key".into());
    }
    let checksum = decoded.split_off(decoded.len() - 4);
    if legacy_wif {
        if decoded.len() != 33 {
            return Err("key has invalid size".into());
        }
        decoded.remove(0);
    } else {
        if let Some(expected) = fixed_len {
            if decoded.len() != expected {
                return Err("key has invalid size".into());
            }
        }
        let digest = ripemd160_with_suffix(&decoded, suffix.as_bytes());
        if checksum != digest[..4] {
            return Err("expected key".into());
        }
    }
    w.varuint32(idx);
    w.write(&decoded);
    Ok(())
}

fn read_key_like(r: &mut Reader, kind: KeyKind) -> Result<String, String> {
    let idx = r.varuint32()?;
    let len = match (kind, idx) {
        (KeyKind::Public, 0 | 1) => 33,
        (KeyKind::Private, 0 | 1) => 32,
        (KeyKind::Signature, 0 | 1) => 65,
        (KeyKind::Public, 2) => {
            let key = r.read(33)?.to_vec();
            let presence = r.byte()?;
            let rpid = r.string()?;
            let mut body = key;
            body.push(presence);
            let mut tmp = Writer::new();
            tmp.string(&rpid);
            body.extend(tmp.bytes());
            return Ok(format!(
                "PUB_WA_{}",
                base58_encode_with_checksum(&body, b"WA")
            ));
        }
        (KeyKind::Signature, 2) => {
            let sig = r.read(65)?.to_vec();
            let auth = r.bytes_vec()?;
            let client = r.string()?;
            let mut body = sig;
            let mut tmp = Writer::new();
            tmp.bytes_vec(&auth);
            tmp.string(&client);
            body.extend(tmp.bytes());
            return Ok(format!(
                "SIG_WA_{}",
                base58_encode_with_checksum(&body, b"WA")
            ));
        }
        _ => return Err("bad variant index".into()),
    };
    let body = r.read(len)?.to_vec();
    let (prefix, suffix) = match (kind, idx) {
        (KeyKind::Public, 0) => ("PUB_K1_", b"K1".as_slice()),
        (KeyKind::Public, 1) => ("PUB_R1_", b"R1".as_slice()),
        (KeyKind::Private, 0) => ("PVT_K1_", b"K1".as_slice()),
        (KeyKind::Private, 1) => ("PVT_R1_", b"R1".as_slice()),
        (KeyKind::Signature, 0) => ("SIG_K1_", b"K1".as_slice()),
        (KeyKind::Signature, 1) => ("SIG_R1_", b"R1".as_slice()),
        _ => ("", b"".as_slice()),
    };
    Ok(format!(
        "{}{}",
        prefix,
        base58_encode_with_checksum(&body, suffix)
    ))
}

const BASE58: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

fn base58_value(b: u8) -> Option<u8> {
    BASE58.iter().position(|c| *c == b).map(|v| v as u8)
}

fn base58_decode(s: &str) -> Result<Vec<u8>, String> {
    let mut out: Vec<u8> = Vec::new();
    for ch in s.bytes() {
        let mut carry = base58_value(ch).ok_or_else(|| "expected key".to_string())? as u32;
        for byte in out.iter_mut().rev() {
            let x = (*byte as u32) * 58 + carry;
            *byte = (x & 0xff) as u8;
            carry = x >> 8;
        }
        while carry > 0 {
            out.insert(0, (carry & 0xff) as u8);
            carry >>= 8;
        }
    }
    for ch in s.bytes() {
        if ch == b'1' {
            out.insert(0, 0);
        } else {
            break;
        }
    }
    Ok(out)
}

fn base58_encode_with_checksum(data: &[u8], suffix: &[u8]) -> String {
    let mut whole = data.to_vec();
    let digest = ripemd160_with_suffix(data, suffix);
    whole.extend_from_slice(&digest[..4]);
    base58_encode(&whole)
}

fn base58_encode(data: &[u8]) -> String {
    let mut digits: Vec<u8> = Vec::new();
    for byte in data {
        let mut carry = *byte as u32;
        for digit in &mut digits {
            let x = (*digit as u32) * 256 + carry;
            *digit = (x % 58) as u8;
            carry = x / 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }
    for byte in data {
        if *byte == 0 {
            digits.push(0);
        } else {
            break;
        }
    }
    digits
        .iter()
        .rev()
        .map(|d| BASE58[*d as usize] as char)
        .collect()
}

fn ripemd160_with_suffix(data: &[u8], suffix: &[u8]) -> [u8; 20] {
    let mut input = Vec::with_capacity(data.len() + suffix.len());
    input.extend_from_slice(data);
    input.extend_from_slice(suffix);
    ripemd160(&input)
}

fn ripemd160(data: &[u8]) -> [u8; 20] {
    const R: [usize; 80] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 7, 4, 13, 1, 10, 6, 15, 3, 12, 0, 9,
        5, 2, 14, 11, 8, 3, 10, 14, 4, 9, 15, 8, 1, 2, 7, 0, 6, 13, 11, 5, 12, 1, 9, 11, 10, 0, 8,
        12, 4, 13, 3, 7, 15, 14, 5, 6, 2, 4, 0, 5, 9, 7, 12, 2, 10, 14, 1, 3, 8, 11, 6, 15, 13,
    ];
    const RP: [usize; 80] = [
        5, 14, 7, 0, 9, 2, 11, 4, 13, 6, 15, 8, 1, 10, 3, 12, 6, 11, 3, 7, 0, 13, 5, 10, 14, 15, 8,
        12, 4, 9, 1, 2, 15, 5, 1, 3, 7, 14, 6, 9, 11, 8, 12, 2, 10, 0, 4, 13, 8, 6, 4, 1, 3, 11,
        15, 0, 5, 12, 2, 13, 9, 7, 10, 14, 12, 15, 10, 4, 1, 5, 8, 7, 6, 2, 13, 14, 0, 3, 9, 11,
    ];
    const S: [u32; 80] = [
        11, 14, 15, 12, 5, 8, 7, 9, 11, 13, 14, 15, 6, 7, 9, 8, 7, 6, 8, 13, 11, 9, 7, 15, 7, 12,
        15, 9, 11, 7, 13, 12, 11, 13, 6, 7, 14, 9, 13, 15, 14, 8, 13, 6, 5, 12, 7, 5, 11, 12, 14,
        15, 14, 15, 9, 8, 9, 14, 5, 6, 8, 6, 5, 12, 9, 15, 5, 11, 6, 8, 13, 12, 5, 12, 13, 14, 11,
        8, 5, 6,
    ];
    const SP: [u32; 80] = [
        8, 9, 9, 11, 13, 15, 15, 5, 7, 7, 8, 11, 14, 14, 12, 6, 9, 13, 15, 7, 12, 8, 9, 11, 7, 7,
        12, 7, 6, 15, 13, 11, 9, 7, 15, 11, 8, 6, 6, 14, 12, 13, 5, 14, 13, 13, 7, 5, 15, 5, 8, 11,
        14, 14, 6, 14, 6, 9, 12, 9, 12, 5, 15, 8, 8, 5, 12, 9, 12, 5, 14, 6, 8, 13, 6, 5, 15, 13,
        11, 11,
    ];

    let mut msg = data.to_vec();
    let bit_len = (msg.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());

    let mut h0 = 0x6745_2301u32;
    let mut h1 = 0xefcd_ab89u32;
    let mut h2 = 0x98ba_dcfeu32;
    let mut h3 = 0x1032_5476u32;
    let mut h4 = 0xc3d2_e1f0u32;

    for chunk in msg.chunks_exact(64) {
        let mut x = [0u32; 16];
        for (i, word) in x.iter_mut().enumerate() {
            let start = i * 4;
            *word = u32::from_le_bytes([
                chunk[start],
                chunk[start + 1],
                chunk[start + 2],
                chunk[start + 3],
            ]);
        }

        let (mut al, mut bl, mut cl, mut dl, mut el) = (h0, h1, h2, h3, h4);
        let (mut ar, mut br, mut cr, mut dr, mut er) = (h0, h1, h2, h3, h4);

        for j in 0..80 {
            let tl = al
                .wrapping_add(ripemd160_f(j, bl, cl, dl))
                .wrapping_add(x[R[j]])
                .wrapping_add(ripemd160_kl(j))
                .rotate_left(S[j])
                .wrapping_add(el);
            al = el;
            el = dl;
            dl = cl.rotate_left(10);
            cl = bl;
            bl = tl;

            let tr = ar
                .wrapping_add(ripemd160_f(79 - j, br, cr, dr))
                .wrapping_add(x[RP[j]])
                .wrapping_add(ripemd160_kr(j))
                .rotate_left(SP[j])
                .wrapping_add(er);
            ar = er;
            er = dr;
            dr = cr.rotate_left(10);
            cr = br;
            br = tr;
        }

        let t = h1.wrapping_add(cl).wrapping_add(dr);
        h1 = h2.wrapping_add(dl).wrapping_add(er);
        h2 = h3.wrapping_add(el).wrapping_add(ar);
        h3 = h4.wrapping_add(al).wrapping_add(br);
        h4 = h0.wrapping_add(bl).wrapping_add(cr);
        h0 = t;
    }

    let mut out = [0u8; 20];
    for (i, word) in [h0, h1, h2, h3, h4].into_iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    out
}

fn ripemd160_f(j: usize, x: u32, y: u32, z: u32) -> u32 {
    match j {
        0..=15 => x ^ y ^ z,
        16..=31 => (x & y) | (!x & z),
        32..=47 => (x | !y) ^ z,
        48..=63 => (x & z) | (y & !z),
        _ => x ^ (y | !z),
    }
}

fn ripemd160_kl(j: usize) -> u32 {
    match j {
        0..=15 => 0x0000_0000,
        16..=31 => 0x5a82_7999,
        32..=47 => 0x6ed9_eba1,
        48..=63 => 0x8f1b_bcdc,
        _ => 0xa953_fd4e,
    }
}

fn ripemd160_kr(j: usize) -> u32 {
    match j {
        0..=15 => 0x50a2_8be6,
        16..=31 => 0x5c4d_d124,
        32..=47 => 0x6d70_3ef3,
        48..=63 => 0x7a6d_76e9,
        _ => 0x0000_0000,
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
        ctx.result_str = cstring_lossy(&hex_encode(&ctx.result_bin));
        Ok(ctx.result_str.as_ptr())
    })
}

pub unsafe extern "C" fn abieos_string_to_name(
    _context: *mut abieos_context,
    str_: *const c_char,
) -> u64 {
    string_to_name_value(&cstr_arg(str_))
}

pub unsafe extern "C" fn abieos_name_to_string(
    context: *mut abieos_context,
    name: u64,
) -> *const c_char {
    with_ctx(context, std::ptr::null(), |ctx| {
        ctx.result_str = cstring_lossy(&name_to_string_value(name));
        Ok(ctx.result_str.as_ptr())
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
        let ty = abi.action_types.get(&action).ok_or_else(|| {
            format!(
                "contract \"{}\" does not have action \"{}\"",
                name_to_string_value(contract),
                name_to_string_value(action)
            )
        })?;
        ctx.result_str = cstring_lossy(ty);
        Ok(ctx.result_str.as_ptr())
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
        let ty = abi.table_types.get(&table).ok_or_else(|| {
            format!(
                "contract \"{}\" does not have table \"{}\"",
                name_to_string_value(contract),
                name_to_string_value(table)
            )
        })?;
        ctx.result_str = cstring_lossy(ty);
        Ok(ctx.result_str.as_ptr())
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
        let ty = abi.action_result_types.get(&action_result).ok_or_else(|| {
            format!(
                "contract \"{}\" does not have action_result \"{}\"",
                name_to_string_value(contract),
                name_to_string_value(action_result)
            )
        })?;
        ctx.result_str = cstring_lossy(ty);
        Ok(ctx.result_str.as_ptr())
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
    let type_name = cstr_arg(type_);
    let json = cstr_arg(json);
    with_ctx(context, 0, |ctx| {
        ctx.result_bin = if let Some(abi) = ctx.contracts.get_mut(&contract) {
            abi.json_to_bin(&type_name, &json, reorderable)?
        } else if contract == 0 {
            Abi::builtin_only().json_to_bin(&type_name, &json, reorderable)?
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
    let type_name = cstr_arg(type_);
    let bytes = bytes_arg(data, size);
    with_ctx(context, std::ptr::null(), |ctx| {
        let json = if let Some(abi) = ctx.contracts.get_mut(&contract) {
            abi.bin_to_json(&type_name, bytes)?
        } else if contract == 0 {
            Abi::builtin_only().bin_to_json(&type_name, bytes)?
        } else {
            return Err(format!(
                "contract \"{}\" is not loaded",
                name_to_string_value(contract)
            ));
        };
        ctx.result_str = cstring_lossy(&json);
        Ok(ctx.result_str.as_ptr())
    })
}

pub unsafe extern "C" fn abieos_hex_to_json(
    context: *mut abieos_context,
    contract: u64,
    type_: *const c_char,
    hex: *const c_char,
) -> *const c_char {
    let hex = cstr_arg(hex);
    match hex_decode(&hex) {
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
    let json = cstr_arg(json);
    with_ctx(context, 0, |ctx| {
        let def = AbiDef::from_json_str(&json)?;
        def.check_version()?;
        ctx.result_bin = def.to_bin();
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
        ctx.result_str = cstring_lossy(&def.to_json_string());
        Ok(ctx.result_str.as_ptr())
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
