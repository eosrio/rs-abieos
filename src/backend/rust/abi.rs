use std::collections::{BTreeMap, BTreeSet};

use super::abi_def::{AbiDef, FieldDef, StructDef};
use super::builtins::{read_builtin, write_builtin, BUILTINS};
use super::json::{parse_json, quote_json, Json};
use super::stream::{Reader, Writer};

#[derive(Clone)]
pub(crate) enum TypeKind {
    Builtin,
    Alias(String),
    Optional(String),
    Extension(String),
    Array(String),
    FixedArray(String, usize),
    Struct(std::sync::Arc<[FieldDef]>),
    Variant(std::sync::Arc<[String]>),
}

#[derive(Clone)]
pub(crate) struct Abi {
    pub(crate) action_types: BTreeMap<u64, String>,
    pub(crate) table_types: BTreeMap<u64, String>,
    pub(crate) action_result_types: BTreeMap<u64, String>,
    types: BTreeMap<String, TypeKind>,
}

impl Abi {
    pub(crate) fn builtin_only() -> Self {
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
            ].into()),
        );
    }

    pub(crate) fn from_def(def: &AbiDef) -> Result<Self, String> {
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
                .insert(s.name.clone(), TypeKind::Struct(Vec::new().into()))
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
                .insert(v.name.clone(), TypeKind::Variant(v.types.clone().into()))
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
            abi.types.insert(name, TypeKind::Struct(fields.into()));
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
                    TypeKind::Struct(base_fields) => fields.extend(base_fields.iter().cloned()),
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

    pub(crate) fn json_to_bin(
        &mut self,
        type_name: &str,
        json: &str,
        reorderable: bool,
        out: &mut Vec<u8>,
    ) -> Result<(), String> {
        let value = parse_json(json)?;
        out.clear();
        let mut w = Writer::new(out);
        let mut skipped_extension = false;
        self.write_json_value(
            type_name,
            &value,
            &mut w,
            true,
            reorderable,
            &mut skipped_extension,
        )?;
        Ok(())
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
                // Use reverse search to match C++ std::map last-wins behavior
                // for duplicate keys: RapidJSON feeds keys into std::map which
                // overwrites previous entries, effectively keeping the last value.
                obj.iter().rev().find(|(k, _)| k == &field.name)
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
        // In reorderable mode, C++ does not check for extra fields: the
        // jvalue_to_bin path iterates over struct fields and looks each up in
        // the std::map, silently ignoring any extra keys.  Only the ordered
        // (streaming SAX) path rejects unexpected fields.
        if !reorderable && obj.iter().any(|(k, _)| !seen.contains(k.as_ref())) {
            return Err("Unexpected field".into());
        }
        Ok(())
    }

    pub(crate) fn bin_to_json(&mut self, type_name: &str, data: &[u8]) -> Result<String, String> {
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
