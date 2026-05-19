use super::hex::hex_decode;
use super::json::{parse_json, quote_json, Json};
use super::name::{name_to_string_value, string_to_name_value};
use super::stream::{Reader, Writer};

#[derive(Default, Clone)]
pub(crate) struct TypeDef {
    pub(crate) new_type_name: String,
    pub(crate) type_name: String,
}
#[derive(Default, Clone)]
pub(crate) struct FieldDef {
    pub(crate) name: String,
    pub(crate) type_name: String,
}
#[derive(Default, Clone)]
pub(crate) struct StructDef {
    pub(crate) name: String,
    pub(crate) base: String,
    pub(crate) fields: Vec<FieldDef>,
}
#[derive(Default, Clone)]
pub(crate) struct ActionDef {
    pub(crate) name: u64,
    pub(crate) type_name: String,
    pub(crate) ricardian_contract: String,
}
#[derive(Default, Clone)]
pub(crate) struct TableDef {
    pub(crate) name: u64,
    pub(crate) index_type: String,
    pub(crate) key_names: Vec<String>,
    pub(crate) key_types: Vec<String>,
    pub(crate) type_name: String,
}
#[derive(Default, Clone)]
pub(crate) struct ClausePair {
    pub(crate) id: String,
    pub(crate) body: String,
}
#[derive(Default, Clone)]
pub(crate) struct ErrorMessage {
    pub(crate) error_code: u64,
    pub(crate) error_msg: String,
}
#[derive(Default, Clone)]
pub(crate) struct VariantDef {
    pub(crate) name: String,
    pub(crate) types: Vec<String>,
}
#[derive(Default, Clone)]
pub(crate) struct ActionResultDef {
    pub(crate) name: u64,
    pub(crate) result_type: String,
}
#[derive(Default, Clone)]
pub(crate) struct AbiDef {
    pub(crate) version: String,
    pub(crate) types: Vec<TypeDef>,
    pub(crate) structs: Vec<StructDef>,
    pub(crate) actions: Vec<ActionDef>,
    pub(crate) tables: Vec<TableDef>,
    pub(crate) ricardian_clauses: Vec<ClausePair>,
    pub(crate) error_messages: Vec<ErrorMessage>,
    pub(crate) abi_extensions: Vec<(u16, Vec<u8>)>,
    pub(crate) variants: Vec<VariantDef>,
    pub(crate) action_results: Vec<ActionResultDef>,
}

fn obj_field<'a, 'b>(obj: &'a [(std::borrow::Cow<'b, str>, Json<'b>)], name: &str) -> Option<&'a Json<'b>> {
    obj.iter().find(|(k, _)| k.as_ref() == name).map(|(_, v)| v)
}

fn json_string(obj: &[(std::borrow::Cow<'_, str>, Json<'_>)], name: &str) -> Result<String, String> {
    Ok(obj_field(obj, name)
        .map(Json::as_str_like)
        .transpose()?
        .unwrap_or_default()
        .to_string())
}

fn json_name(obj: &[(std::borrow::Cow<'_, str>, Json<'_>)], name: &str) -> Result<u64, String> {
    Ok(string_to_name_value(&json_string(obj, name)?))
}

fn json_vec<T>(
    obj: &[(std::borrow::Cow<'_, str>, Json<'_>)],
    name: &str,
    mut f: impl FnMut(&Json<'_>) -> Result<T, String>,
) -> Result<Vec<T>, String> {
    let Some(value) = obj_field(obj, name) else {
        return Ok(Vec::new());
    };
    value.as_array()?.iter().map(&mut f).collect()
}

fn strings_from_json(value: &Json<'_>) -> Result<Vec<String>, String> {
    value
        .as_array()?
        .iter()
        .map(|v| v.as_str_like().map(str::to_owned))
        .collect()
}

impl AbiDef {
    pub(crate) fn from_json_str(json: &str) -> Result<Self, String> {
        let root = parse_json(json)?;
        Self::from_json(&root)
    }

    fn from_json(root: &Json<'_>) -> Result<Self, String> {
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

    pub(crate) fn check_version(&self) -> Result<(), String> {
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

    pub(crate) fn to_bin(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut w = Writer::new(&mut buf);
        self.write_bin(&mut w);
        buf
    }

    pub(crate) fn read_bin(r: &mut Reader) -> Result<Self, String> {
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

    pub(crate) fn to_json_string(&self) -> String {
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
