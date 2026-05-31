//! Standalone, pre-parsed ABI handle (rust-backend only).

use super::abi::Abi;
use super::abi_def::AbiDef;
use super::hex::hex_decode;
use super::stream::Reader;
use crate::AbieosError;

/// A pre-parsed ABI you can hold and decode against directly — no `abieos_context`, no
/// `set_abi` / `delete_contract` churn.
///
/// This is the ergonomic path for a **historical** delta/action decoder: parse each ABI
/// version once, keep a `BTreeMap<(account, valid_from_block), AbiHandle>`, and range-query
/// for "the version active at block N". Switching versions is then a map lookup, not a
/// re-parse, and there's no single-slot-per-account constraint to dance around.
///
/// `AbiHandle` is `Send` (move it between threads / store one map per worker). Decoding takes
/// `&mut self` because type resolution is cached lazily into the handle, so it is **not**
/// `Sync` — use one registry per thread, exactly as with [`Abieos`](crate::Abieos).
pub struct AbiHandle {
    abi: Abi,
}

impl AbiHandle {
    /// Parse an ABI from its JSON definition.
    pub fn from_json(abi_json: &str) -> Result<Self, AbieosError> {
        let def = AbiDef::from_json_str(abi_json).map_err(AbieosError::SetAbi)?;
        let abi = Abi::from_def(&def).map_err(AbieosError::SetAbi)?;
        Ok(Self { abi })
    }

    /// Parse an ABI from its serialized binary form.
    pub fn from_bin(abi_bin: &[u8]) -> Result<Self, AbieosError> {
        let def = AbiDef::read_bin(&mut Reader::new(abi_bin)).map_err(AbieosError::SetAbi)?;
        let abi = Abi::from_def(&def).map_err(AbieosError::SetAbi)?;
        Ok(Self { abi })
    }

    /// Parse an ABI from its serialized binary form, hex-encoded (the `setabi` payload as
    /// stored in the `account` table / the abi-index).
    pub fn from_hex(abi_hex: &str) -> Result<Self, AbieosError> {
        let bin = hex_decode(abi_hex).map_err(AbieosError::SetAbi)?;
        Self::from_bin(&bin)
    }

    /// The struct type that decodes rows of `table` (u64 name), if the ABI declares it.
    pub fn type_for_table(&self, table: u64) -> Option<&str> {
        self.abi.table_types.get(&table).map(|t| t.as_str())
    }

    /// The struct type for `action` (u64 name), if the ABI declares it.
    pub fn type_for_action(&self, action: u64) -> Option<&str> {
        self.abi.action_types.get(&action).map(|t| t.as_str())
    }

    /// Deserialize `bin` as `datatype` into JSON.
    pub fn bin_to_json(&mut self, datatype: &str, bin: &[u8]) -> Result<String, AbieosError> {
        self.abi
            .bin_to_json(datatype, bin)
            .map_err(AbieosError::BinToJson)
    }

    /// Deserialize into `out`, reusing its allocation (no per-call `String` once warm).
    pub fn bin_to_json_into(
        &mut self,
        datatype: &str,
        bin: &[u8],
        out: &mut String,
    ) -> Result<(), AbieosError> {
        self.abi
            .bin_to_json_into(datatype, bin, out)
            .map_err(AbieosError::BinToJson)
    }

    /// Decode a `contract_row` `value` for `table` (u64 name) in one call: resolve the table's
    /// struct type and deserialize. `Err(GetTypeForTable)` if the table isn't in the ABI.
    pub fn decode_table_row(&mut self, table: u64, bin: &[u8]) -> Result<String, AbieosError> {
        let ty = self
            .type_for_table(table)
            .ok_or_else(|| {
                AbieosError::GetTypeForTable(format!("table {table} not declared in ABI"))
            })?
            .to_owned();
        self.bin_to_json(&ty, bin)
    }

    /// Like [`decode_table_row`](Self::decode_table_row) but writes into `out`.
    pub fn decode_table_row_into(
        &mut self,
        table: u64,
        bin: &[u8],
        out: &mut String,
    ) -> Result<(), AbieosError> {
        let ty = self
            .type_for_table(table)
            .ok_or_else(|| {
                AbieosError::GetTypeForTable(format!("table {table} not declared in ABI"))
            })?
            .to_owned();
        self.bin_to_json_into(&ty, bin, out)
    }
}
