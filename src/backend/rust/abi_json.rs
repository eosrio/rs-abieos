//! Single-pass ABI-JSON → [`AbiDef`] parser.
//!
//! The generic DOM (`parse_json` + `from_json`) allocated a `Vec` per JSON
//! object/array — eosio.abi has thousands of tiny field objects, so that was
//! the dominant remaining cost versus C++ rapidjson's arena DOM. This parser
//! consumes the *same* token primitives (`parse_string`/`parse_number`/
//! `skip_value`/array+object iteration), so string/number/escape/error
//! behavior is byte-identical, but materializes `AbiDef` directly: no DOM, no
//! second walk, `IStr` keeps short strings allocation-free, and a reused
//! scratch buffer means one allocation per struct's field list (matching C++)
//! instead of a temporary `Vec` plus its `Arc`.
//!
//! Semantics mirror the old `AbiDef::from_json` precisely:
//! - top-level / element non-object → `"Expected {"`
//! - missing key → field default (empty string / `0` / empty list)
//! - first occurrence of a key wins (matches `obj_field`'s `find`)
//! - present-but-not-string scalar slot → `"expected string"`
//! - present-but-not-array list slot → `"expected array"`
//! - unknown keys are skipped structurally (same malformed-input errors)
//!
//! The rare, fiddly `abi_extensions` entries reuse the DOM `parse_value` so
//! their exact error ordering/semantics are preserved with zero risk.

use std::borrow::Cow;
use std::sync::Arc;

use super::abi_def::AbiDef;
use super::abi_def::{
    ActionDef, ActionResultDef, ClausePair, ErrorMessage, FieldDef, StructDef, TableDef, TypeDef,
    VariantDef,
};
use super::hex::hex_decode;
use super::istr::IStr;
use super::json::JsonParser;
use super::name::string_to_name_value;

/// Read a scalar slot the way `json_string`/`as_str_like` did: a JSON string
/// or number yields its text; any other value type → `"expected string"`.
fn scalar<'a>(p: &mut JsonParser<'a>) -> Result<Cow<'a, str>, String> {
    p.skip_ws();
    match p.peek() {
        Some(b'"') => p.parse_string(),
        Some(b'-' | b'0'..=b'9') => p.parse_number(),
        _ => Err("expected string".into()),
    }
}

/// Collect a JSON array of strings into `scratch` (reused across calls) and
/// freeze it into an `Arc<[IStr]>` (mirrors `strings_from_json_arc`).
fn istr_arc_vec(p: &mut JsonParser<'_>, scratch: &mut Vec<IStr>) -> Result<Arc<[IStr]>, String> {
    scratch.clear();
    if !p.array_open()? {
        loop {
            scratch.push(IStr::from(scalar(p)?.as_ref()));
            if p.array_step()? {
                break;
            }
        }
    }
    Ok(Arc::from(scratch.as_slice()))
}

/// `true` while there are more elements; drives `while next_elem(p, &mut st)?`.
struct ArrayIter {
    started: bool,
    done: bool,
}
impl ArrayIter {
    fn new() -> Self {
        Self {
            started: false,
            done: false,
        }
    }
    /// Advance to the next element. Returns `false` when the array ends.
    fn next(&mut self, p: &mut JsonParser<'_>) -> Result<bool, String> {
        if self.done {
            return Ok(false);
        }
        if !self.started {
            self.started = true;
            if p.array_open()? {
                self.done = true;
                return Ok(false);
            }
            return Ok(true);
        }
        if p.array_step()? {
            self.done = true;
            return Ok(false);
        }
        Ok(true)
    }
}

pub(crate) fn parse_abi_def(json: &str) -> Result<AbiDef, String> {
    let mut p = JsonParser::new(json);
    let mut def = AbiDef::default();

    // First-occurrence-wins tracking for the top-level keys we consume.
    let mut seen_version = false;
    let mut seen: [bool; 9] = [false; 9];
    const K_TYPES: usize = 0;
    const K_STRUCTS: usize = 1;
    const K_ACTIONS: usize = 2;
    const K_TABLES: usize = 3;
    const K_RICARDIAN: usize = 4;
    const K_ERRMSG: usize = 5;
    const K_EXT: usize = 6;
    const K_VARIANTS: usize = 7;
    const K_ARESULTS: usize = 8;

    // Reused across all structs / string-lists: one allocation per frozen
    // `Arc<[..]>` instead of a temporary `Vec` plus its `Arc`.
    let mut fscratch: Vec<FieldDef> = Vec::new();
    let mut sscratch: Vec<IStr> = Vec::new();

    if !p.object_open()? {
        loop {
            let key = p.member_key()?;
            match key.as_ref() {
                "version" if !seen_version => {
                    seen_version = true;
                    def.version = IStr::from(scalar(&mut p)?.as_ref());
                }
                "types" if !seen[K_TYPES] => {
                    seen[K_TYPES] = true;
                    let mut it = ArrayIter::new();
                    while it.next(&mut p)? {
                        let mut t = TypeDef::default();
                        if !p.object_open()? {
                            loop {
                                let k = p.member_key()?;
                                match k.as_ref() {
                                    "new_type_name" => {
                                        t.new_type_name = IStr::from(scalar(&mut p)?.as_ref())
                                    }
                                    "type" => t.type_name = IStr::from(scalar(&mut p)?.as_ref()),
                                    _ => p.skip_value()?,
                                }
                                if p.object_step()? {
                                    break;
                                }
                            }
                        }
                        def.types.push(t);
                    }
                }
                "structs" if !seen[K_STRUCTS] => {
                    seen[K_STRUCTS] = true;
                    let mut it = ArrayIter::new();
                    while it.next(&mut p)? {
                        let mut name = IStr::default();
                        let mut base = IStr::default();
                        let mut got_name = false;
                        let mut got_base = false;
                        let mut got_fields = false;
                        fscratch.clear();
                        if !p.object_open()? {
                            loop {
                                let k = p.member_key()?;
                                match k.as_ref() {
                                    "name" if !got_name => {
                                        got_name = true;
                                        name = IStr::from(scalar(&mut p)?.as_ref());
                                    }
                                    "base" if !got_base => {
                                        got_base = true;
                                        base = IStr::from(scalar(&mut p)?.as_ref());
                                    }
                                    "fields" if !got_fields => {
                                        got_fields = true;
                                        let mut fit = ArrayIter::new();
                                        while fit.next(&mut p)? {
                                            let mut f = FieldDef::default();
                                            if !p.object_open()? {
                                                loop {
                                                    let fk = p.member_key()?;
                                                    match fk.as_ref() {
                                                        "name" => {
                                                            f.name =
                                                                IStr::from(scalar(&mut p)?.as_ref())
                                                        }
                                                        "type" => {
                                                            f.type_name =
                                                                IStr::from(scalar(&mut p)?.as_ref())
                                                        }
                                                        _ => p.skip_value()?,
                                                    }
                                                    if p.object_step()? {
                                                        break;
                                                    }
                                                }
                                            }
                                            fscratch.push(f);
                                        }
                                    }
                                    _ => p.skip_value()?,
                                }
                                if p.object_step()? {
                                    break;
                                }
                            }
                        }
                        def.structs.push(StructDef {
                            name,
                            base,
                            fields: Arc::from(fscratch.as_slice()),
                        });
                    }
                }
                "actions" if !seen[K_ACTIONS] => {
                    seen[K_ACTIONS] = true;
                    let mut it = ArrayIter::new();
                    while it.next(&mut p)? {
                        let mut a = ActionDef::default();
                        if !p.object_open()? {
                            loop {
                                let k = p.member_key()?;
                                match k.as_ref() {
                                    "name" => {
                                        a.name = string_to_name_value(scalar(&mut p)?.as_ref())
                                    }
                                    "type" => a.type_name = IStr::from(scalar(&mut p)?.as_ref()),
                                    "ricardian_contract" => {
                                        a.ricardian_contract = IStr::from(scalar(&mut p)?.as_ref())
                                    }
                                    _ => p.skip_value()?,
                                }
                                if p.object_step()? {
                                    break;
                                }
                            }
                        }
                        def.actions.push(a);
                    }
                }
                "tables" if !seen[K_TABLES] => {
                    seen[K_TABLES] = true;
                    let mut it = ArrayIter::new();
                    while it.next(&mut p)? {
                        let mut t = TableDef::default();
                        if !p.object_open()? {
                            loop {
                                let k = p.member_key()?;
                                match k.as_ref() {
                                    "name" => {
                                        t.name = string_to_name_value(scalar(&mut p)?.as_ref())
                                    }
                                    "index_type" => {
                                        t.index_type = IStr::from(scalar(&mut p)?.as_ref())
                                    }
                                    "key_names" => {
                                        t.key_names = istr_arc_vec(&mut p, &mut sscratch)?
                                    }
                                    "key_types" => {
                                        t.key_types = istr_arc_vec(&mut p, &mut sscratch)?
                                    }
                                    "type" => t.type_name = IStr::from(scalar(&mut p)?.as_ref()),
                                    _ => p.skip_value()?,
                                }
                                if p.object_step()? {
                                    break;
                                }
                            }
                        }
                        def.tables.push(t);
                    }
                }
                "ricardian_clauses" if !seen[K_RICARDIAN] => {
                    seen[K_RICARDIAN] = true;
                    let mut it = ArrayIter::new();
                    while it.next(&mut p)? {
                        let mut c = ClausePair::default();
                        if !p.object_open()? {
                            loop {
                                let k = p.member_key()?;
                                match k.as_ref() {
                                    "id" => c.id = IStr::from(scalar(&mut p)?.as_ref()),
                                    "body" => c.body = IStr::from(scalar(&mut p)?.as_ref()),
                                    _ => p.skip_value()?,
                                }
                                if p.object_step()? {
                                    break;
                                }
                            }
                        }
                        def.ricardian_clauses.push(c);
                    }
                }
                "error_messages" if !seen[K_ERRMSG] => {
                    seen[K_ERRMSG] = true;
                    let mut it = ArrayIter::new();
                    while it.next(&mut p)? {
                        let mut e = ErrorMessage::default();
                        if !p.object_open()? {
                            loop {
                                let k = p.member_key()?;
                                match k.as_ref() {
                                    "error_code" => {
                                        e.error_code = scalar(&mut p)?.parse().unwrap_or(0)
                                    }
                                    "error_msg" => {
                                        e.error_msg = IStr::from(scalar(&mut p)?.as_ref())
                                    }
                                    _ => p.skip_value()?,
                                }
                                if p.object_step()? {
                                    break;
                                }
                            }
                        }
                        def.error_messages.push(e);
                    }
                }
                "abi_extensions" if !seen[K_EXT] => {
                    seen[K_EXT] = true;
                    // DOM value parser for these rare entries → exact
                    // "expected pair"/element-type ordering as before.
                    let mut it = ArrayIter::new();
                    while it.next(&mut p)? {
                        let v = p.parse_value()?;
                        let arr = v.as_array()?;
                        if arr.len() != 2 {
                            return Err("expected pair".into());
                        }
                        let ty: u16 = arr[0].as_str_like()?.parse().unwrap_or(0);
                        let data = hex_decode(arr[1].as_str_like()?)?;
                        def.abi_extensions.push((ty, data));
                    }
                }
                "variants" if !seen[K_VARIANTS] => {
                    seen[K_VARIANTS] = true;
                    let mut it = ArrayIter::new();
                    while it.next(&mut p)? {
                        let mut name = IStr::default();
                        let mut types: Arc<[IStr]> = Vec::new().into();
                        let mut got_name = false;
                        let mut got_types = false;
                        if !p.object_open()? {
                            loop {
                                let k = p.member_key()?;
                                match k.as_ref() {
                                    "name" if !got_name => {
                                        got_name = true;
                                        name = IStr::from(scalar(&mut p)?.as_ref());
                                    }
                                    "types" if !got_types => {
                                        got_types = true;
                                        types = istr_arc_vec(&mut p, &mut sscratch)?;
                                    }
                                    _ => p.skip_value()?,
                                }
                                if p.object_step()? {
                                    break;
                                }
                            }
                        }
                        def.variants.push(VariantDef { name, types });
                    }
                }
                "action_results" if !seen[K_ARESULTS] => {
                    seen[K_ARESULTS] = true;
                    let mut it = ArrayIter::new();
                    while it.next(&mut p)? {
                        let mut r = ActionResultDef::default();
                        if !p.object_open()? {
                            loop {
                                let k = p.member_key()?;
                                match k.as_ref() {
                                    "name" => {
                                        r.name = string_to_name_value(scalar(&mut p)?.as_ref())
                                    }
                                    "result_type" => {
                                        r.result_type = IStr::from(scalar(&mut p)?.as_ref())
                                    }
                                    _ => p.skip_value()?,
                                }
                                if p.object_step()? {
                                    break;
                                }
                            }
                        }
                        def.action_results.push(r);
                    }
                }
                // Unknown top-level key, or a duplicate of one already taken:
                // structurally skip its value (same malformed-input errors).
                _ => p.skip_value()?,
            }
            if p.object_step()? {
                break;
            }
        }
    }

    if !p.at_end() {
        return Err("Expected end of json".into());
    }
    Ok(def)
}
