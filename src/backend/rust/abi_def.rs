use super::istr::IStr;
use super::json::{Json, parse_json, quote_json};
use super::name::{name_to_string_value, string_to_name_value};
use super::stream::{Reader, Writer};

#[derive(Default, Clone)]
pub(crate) struct TypeDef {
    pub(crate) new_type_name: IStr,
    pub(crate) type_name: IStr,
}
#[derive(Default, Clone)]
pub(crate) struct FieldDef {
    pub(crate) name: IStr,
    pub(crate) type_name: IStr,
}
#[derive(Default, Clone)]
pub(crate) struct StructDef {
    pub(crate) name: IStr,
    pub(crate) base: IStr,
    pub(crate) fields: std::sync::Arc<[FieldDef]>,
}
#[derive(Default, Clone)]
pub(crate) struct ActionDef {
    pub(crate) name: u64,
    pub(crate) type_name: IStr,
    pub(crate) ricardian_contract: IStr,
}
#[derive(Default, Clone)]
pub(crate) struct IndexDef {
    pub(crate) name: IStr,
    pub(crate) key_type: IStr,
    pub(crate) table_id: u16,
}
#[derive(Default, Clone)]
pub(crate) struct TableDef {
    pub(crate) name: u64,
    pub(crate) name_str: IStr,
    pub(crate) index_type: IStr,
    pub(crate) key_names: std::sync::Arc<[IStr]>,
    pub(crate) key_types: std::sync::Arc<[IStr]>,
    pub(crate) type_name: IStr,
    pub(crate) table_id: u16,
    pub(crate) secondary_indexes: std::sync::Arc<[IndexDef]>,
}
#[derive(Default, Clone)]
pub(crate) struct ClausePair {
    pub(crate) id: IStr,
    pub(crate) body: IStr,
}
#[derive(Default, Clone)]
pub(crate) struct ErrorMessage {
    pub(crate) error_code: u64,
    pub(crate) error_msg: IStr,
}
#[derive(Default, Clone)]
pub(crate) struct VariantDef {
    pub(crate) name: IStr,
    pub(crate) types: std::sync::Arc<[IStr]>,
}
#[derive(Default, Clone)]
pub(crate) struct ActionResultDef {
    pub(crate) name: u64,
    pub(crate) result_type: IStr,
}
#[derive(Default, Clone)]
pub(crate) struct EnumValueDef {
    pub(crate) name: IStr,
    pub(crate) value: i64,
}
#[derive(Default, Clone)]
pub(crate) struct EnumDef {
    pub(crate) name: IStr,
    pub(crate) type_name: IStr,
    pub(crate) values: std::sync::Arc<[EnumValueDef]>,
}
#[derive(Default, Clone)]
pub(crate) struct AbiDef {
    pub(crate) version: IStr,
    pub(crate) types: Vec<TypeDef>,
    pub(crate) structs: Vec<StructDef>,
    pub(crate) actions: Vec<ActionDef>,
    pub(crate) tables: Vec<TableDef>,
    pub(crate) ricardian_clauses: Vec<ClausePair>,
    pub(crate) error_messages: Vec<ErrorMessage>,
    pub(crate) abi_extensions: Vec<(u16, Vec<u8>)>,
    pub(crate) variants: Vec<VariantDef>,
    pub(crate) action_results: Vec<ActionResultDef>,
    pub(crate) enums: Vec<EnumDef>,
    pub(crate) protobuf_types: IStr,
}

impl AbiDef {
    pub(crate) fn from_json_str(json: &str) -> Result<Self, String> {
        super::abi_json::parse_abi_def(json)
    }

    pub(crate) fn is_sysio(&self) -> bool {
        self.version.starts_with("sysio::abi/")
    }

    pub(crate) fn check_version(&self) -> Result<(), String> {
        if self.version.starts_with("eosio::abi/1.")
            || self.version.starts_with("eosio::abi/2.")
            || self.version.starts_with("sysio::abi/1.")
        {
            Ok(())
        } else {
            Err("unsupported abi version".into())
        }
    }

    fn write_bin(&self, w: &mut Writer) {
        let is_sysio = self.is_sysio();
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
            for f in s.fields.iter() {
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
            if is_sysio {
                if !t.name_str.is_empty() {
                    w.string(&t.name_str);
                } else {
                    w.string(&name_to_string_value(t.name));
                }
            } else {
                w.u64(t.name);
            }
            w.string(&t.index_type);
            write_string_arc_vec(w, &t.key_names);
            write_string_arc_vec(w, &t.key_types);
            w.string(&t.type_name);
            if is_sysio {
                w.u16(t.table_id);
                w.varuint32(t.secondary_indexes.len() as u32);
                for idx in t.secondary_indexes.iter() {
                    w.string(&idx.name);
                    w.string(&idx.key_type);
                    w.u16(idx.table_id);
                }
            }
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
            write_string_arc_vec(w, &v.types);
        }
        w.varuint32(self.action_results.len() as u32);
        for r in &self.action_results {
            w.u64(r.name);
            w.string(&r.result_type);
        }
        if is_sysio || !self.enums.is_empty() || !self.protobuf_types.is_empty() {
            w.varuint32(self.enums.len() as u32);
            for e in &self.enums {
                w.string(&e.name);
                w.string(&e.type_name);
                w.varuint32(e.values.len() as u32);
                for v in e.values.iter() {
                    w.string(&v.name);
                    w.i64(v.value);
                }
            }
            w.string(&self.protobuf_types);
        }
    }

    /// Serialize into a caller-owned buffer, reusing its capacity. Repeated
    /// calls on the same context buffer then incur no reallocation.
    pub(crate) fn to_bin_into(&self, out: &mut Vec<u8>) {
        out.clear();
        let mut w = Writer::new(out);
        self.write_bin(&mut w);
    }

    pub(crate) fn read_bin(r: &mut Reader) -> Result<Self, String> {
        let mut def = AbiDef {
            version: r.istr()?,
            ..Default::default()
        };
        let is_sysio = def.is_sysio();
        def.types = read_vec(r, |r| {
            Ok(TypeDef {
                new_type_name: r.istr()?,
                type_name: r.istr()?,
            })
        })?;
        def.structs = read_vec(r, |r| {
            Ok(StructDef {
                name: r.istr()?,
                base: r.istr()?,
                fields: read_vec(r, |r| {
                    Ok(FieldDef {
                        name: r.istr()?,
                        type_name: r.istr()?,
                    })
                })?
                .into(),
            })
        })?;
        def.actions = read_vec(r, |r| {
            Ok(ActionDef {
                name: r.u64()?,
                type_name: r.istr()?,
                ricardian_contract: r.istr()?,
            })
        })?;
        if is_sysio {
            def.tables = read_vec(r, |r| {
                let name_str = r.istr()?;
                let name = string_to_name_value(&name_str);
                let index_type = r.istr()?;
                let key_names = read_string_arc_vec(r)?;
                let key_types = read_string_arc_vec(r)?;
                let type_name = r.istr()?;
                let table_id = r.u16()?;
                let secondary_indexes = read_vec(r, |r| {
                    Ok(IndexDef {
                        name: r.istr()?,
                        key_type: r.istr()?,
                        table_id: r.u16()?,
                    })
                })?
                .into();
                Ok(TableDef {
                    name,
                    name_str,
                    index_type,
                    key_names,
                    key_types,
                    type_name,
                    table_id,
                    secondary_indexes,
                })
            })?;
        } else {
            def.tables = read_vec(r, |r| {
                Ok(TableDef {
                    name: r.u64()?,
                    name_str: IStr::default(),
                    index_type: r.istr()?,
                    key_names: read_string_arc_vec(r)?,
                    key_types: read_string_arc_vec(r)?,
                    type_name: r.istr()?,
                    table_id: 0,
                    secondary_indexes: Vec::new().into(),
                })
            })?;
        }
        def.ricardian_clauses = read_vec(r, |r| {
            Ok(ClausePair {
                id: r.istr()?,
                body: r.istr()?,
            })
        })?;
        def.error_messages = read_vec(r, |r| {
            Ok(ErrorMessage {
                error_code: r.u64()?,
                error_msg: r.istr()?,
            })
        })?;
        def.abi_extensions = read_vec(r, |r| Ok((r.u16()?, r.bytes_vec()?)))?;
        if r.remaining() > 0 {
            def.variants = read_vec(r, |r| {
                Ok(VariantDef {
                    name: r.istr()?,
                    types: read_string_arc_vec(r)?,
                })
            })?;
        }
        if r.remaining() > 0 {
            def.action_results = read_vec(r, |r| {
                Ok(ActionResultDef {
                    name: r.u64()?,
                    result_type: r.istr()?,
                })
            })?;
        }
        if r.remaining() > 0 {
            def.enums = read_vec(r, |r| {
                Ok(EnumDef {
                    name: r.istr()?,
                    type_name: r.istr()?,
                    values: read_vec(r, |r| {
                        Ok(EnumValueDef {
                            name: r.istr()?,
                            value: r.i64()?,
                        })
                    })?
                    .into(),
                })
            })?;
        }
        if r.remaining() > 0 {
            def.protobuf_types = r.istr()?;
        }
        Ok(def)
    }

    pub(crate) fn to_json_string(&self) -> String {
        let is_sysio = self.is_sysio();
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
                        |out| {
                            if !t.name_str.is_empty() {
                                quote_json(&t.name_str, out);
                            } else {
                                quote_json(&name_to_string_value(t.name), out);
                            }
                        },
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
                        |out| json_string_arc_array(out, &t.key_names),
                        true,
                    );
                    json_kv(
                        out,
                        "key_types",
                        |out| json_string_arc_array(out, &t.key_types),
                        true,
                    );
                    json_kv(out, "type", |out| quote_json(&t.type_name, out), true);
                    if is_sysio || t.table_id != 0 || !t.secondary_indexes.is_empty() {
                        json_kv(
                            out,
                            "table_id",
                            |out| out.push_str(&t.table_id.to_string()),
                            true,
                        );
                        json_kv(
                            out,
                            "secondary_indexes",
                            |out| {
                                json_array(out, t.secondary_indexes.iter(), |out, idx| {
                                    out.push('{');
                                    json_kv(out, "name", |out| quote_json(&idx.name, out), false);
                                    json_kv(
                                        out,
                                        "key_type",
                                        |out| quote_json(&idx.key_type, out),
                                        true,
                                    );
                                    json_kv(
                                        out,
                                        "table_id",
                                        |out| out.push_str(&idx.table_id.to_string()),
                                        true,
                                    );
                                    out.push('}');
                                })
                            },
                            true,
                        );
                    }
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
                    json_kv(
                        out,
                        "types",
                        |out| json_string_arc_array(out, &v.types),
                        true,
                    );
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
        if is_sysio || !self.enums.is_empty() {
            json_kv(
                &mut out,
                "enums",
                |out| {
                    json_array(out, self.enums.iter(), |out, e| {
                        out.push('{');
                        json_kv(out, "name", |out| quote_json(&e.name, out), false);
                        json_kv(out, "type", |out| quote_json(&e.type_name, out), true);
                        json_kv(
                            out,
                            "values",
                            |out| {
                                json_array(out, e.values.iter(), |out, v| {
                                    out.push('{');
                                    json_kv(out, "name", |out| quote_json(&v.name, out), false);
                                    json_kv(
                                        out,
                                        "value",
                                        |out| out.push_str(&v.value.to_string()),
                                        true,
                                    );
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
        }
        if !self.protobuf_types.is_empty() {
            json_kv(
                &mut out,
                "protobuf_types",
                |out| {
                    // Retain the string form if outer whitespace would be lost
                    // when an object is read back into the binary string field.
                    if self.protobuf_types.starts_with('{')
                        && self.protobuf_types.ends_with('}')
                        && matches!(parse_json(&self.protobuf_types), Ok(Json::Object(_)))
                    {
                        out.push_str(&self.protobuf_types);
                    } else {
                        quote_json(&self.protobuf_types, out);
                    }
                },
                true,
            );
        }
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

fn json_string_arc_array(out: &mut String, values: &[IStr]) {
    json_array(out, values.iter(), |out, s| quote_json(s, out));
}

fn write_string_arc_vec(w: &mut Writer, values: &[IStr]) {
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
    // The element count is an untrusted varuint32 (up to ~4.29e9). Every
    // element consumes at least one byte, so a length exceeding the bytes
    // still available is malformed by construction. Bounding the initial
    // capacity by the remaining input prevents a crafted length field from
    // triggering a multi-GB pre-allocation (observed: a 182 GiB request that
    // aborted the process) before the data is ever validated. Reads past the
    // real input still fail fast via the reader's bounds check below.
    let mut out = Vec::with_capacity(len.min(r.remaining()));
    for _ in 0..len {
        out.push(f(r)?);
    }
    Ok(out)
}

fn read_string_arc_vec(r: &mut Reader) -> Result<std::sync::Arc<[IStr]>, String> {
    let list: Vec<IStr> = read_vec(r, |r| r.istr())?;
    Ok(list.into())
}
