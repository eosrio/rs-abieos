//! Standalone, pre-parsed ABI handle (rust-backend only).

use super::abi::Abi;
use super::abi_def::AbiDef;
use super::hex::{hex_decode, hex_encode};
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

    /// The struct type that decodes rows of `table` (legacy EOSIO u64 name).
    /// Use [`type_for_table_name`](Self::type_for_table_name) for arbitrary Wire names.
    pub fn type_for_table(&self, table: u64) -> Option<&str> {
        self.abi.table_types.get(&table).map(|t| t.as_str())
    }

    /// The row type for a table name. Wire (`sysio`) names are matched exactly;
    /// EOSIO names retain their legacy name conversion semantics.
    pub fn type_for_table_name(&self, table: &str) -> Option<&str> {
        self.abi.type_for_table_name(table)
    }

    /// The struct type for `action` (u64 name), if the ABI declares it.
    pub fn type_for_action(&self, action: u64) -> Option<&str> {
        self.abi.action_types.get(&action).map(|t| t.as_str())
    }

    /// The result type for `action_result` (u64 name), if the ABI declares it.
    pub fn type_for_action_result(&self, action_result: u64) -> Option<&str> {
        self.abi
            .action_result_types
            .get(&action_result)
            .map(|t| t.as_str())
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

    /// Deserialize a hex-encoded binary payload as `datatype` into JSON.
    pub fn hex_to_json(&mut self, datatype: &str, hex: &str) -> Result<String, AbieosError> {
        let mut out = String::new();
        self.hex_to_json_into(datatype, hex, &mut out)?;
        Ok(out)
    }

    /// Like [`hex_to_json`](Self::hex_to_json) but writes into `out`, reusing its allocation.
    pub fn hex_to_json_into(
        &mut self,
        datatype: &str,
        hex: &str,
        out: &mut String,
    ) -> Result<(), AbieosError> {
        let bin = hex_decode(hex).map_err(AbieosError::HexToJson)?;
        self.abi
            .bin_to_json_into(datatype, &bin, out)
            .map_err(AbieosError::HexToJson)
    }

    /// Serialize `json` as `datatype` into binary (reorderable field order, matching
    /// [`Abieos::json_to_bin`](crate::Abieos::json_to_bin)).
    pub fn json_to_bin(&mut self, datatype: &str, json: &str) -> Result<Vec<u8>, AbieosError> {
        let mut out = Vec::new();
        self.json_to_bin_into(datatype, json, &mut out)?;
        Ok(out)
    }

    /// Like [`json_to_bin`](Self::json_to_bin) but writes into `out`, reusing its allocation.
    pub fn json_to_bin_into(
        &mut self,
        datatype: &str,
        json: &str,
        out: &mut Vec<u8>,
    ) -> Result<(), AbieosError> {
        self.abi
            .json_to_bin(datatype, json, true, out)
            .map_err(AbieosError::JsonToBin)
    }

    /// Serialize `json` as `datatype` into an uppercase hex string (matching
    /// [`Abieos::json_to_hex`](crate::Abieos::json_to_hex)).
    pub fn json_to_hex(&mut self, datatype: &str, json: &str) -> Result<String, AbieosError> {
        let mut bin = Vec::new();
        self.abi
            .json_to_bin(datatype, json, true, &mut bin)
            .map_err(AbieosError::JsonToHex)?;
        Ok(hex_encode(&bin))
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

    /// Decode an action payload for `action` (u64 name) in one call: resolve the action's
    /// struct type and deserialize `bin`. `Err(GetTypeForAction)` if the action isn't in the ABI.
    pub fn decode_action(&mut self, action: u64, bin: &[u8]) -> Result<String, AbieosError> {
        let ty = self
            .type_for_action(action)
            .ok_or_else(|| {
                AbieosError::GetTypeForAction(format!("action {action} not declared in ABI"))
            })?
            .to_owned();
        self.bin_to_json(&ty, bin)
    }

    /// Like [`decode_action`](Self::decode_action) but writes into `out`.
    pub fn decode_action_into(
        &mut self,
        action: u64,
        bin: &[u8],
        out: &mut String,
    ) -> Result<(), AbieosError> {
        let ty = self
            .type_for_action(action)
            .ok_or_else(|| {
                AbieosError::GetTypeForAction(format!("action {action} not declared in ABI"))
            })?
            .to_owned();
        self.bin_to_json_into(&ty, bin, out)
    }

    /// Decode an action result payload for `action_result` (u64 name) in one call: resolve the
    /// action result's type and deserialize `bin`. `Err(GetTypeForActionResult)` if the action
    /// result isn't in the ABI.
    pub fn decode_action_result(
        &mut self,
        action_result: u64,
        bin: &[u8],
    ) -> Result<String, AbieosError> {
        let ty = self
            .type_for_action_result(action_result)
            .ok_or_else(|| {
                AbieosError::GetTypeForActionResult(format!(
                    "action_result {action_result} not declared in ABI"
                ))
            })?
            .to_owned();
        self.bin_to_json(&ty, bin)
    }

    /// Like [`decode_action_result`](Self::decode_action_result) but writes into `out`.
    pub fn decode_action_result_into(
        &mut self,
        action_result: u64,
        bin: &[u8],
        out: &mut String,
    ) -> Result<(), AbieosError> {
        let ty = self
            .type_for_action_result(action_result)
            .ok_or_else(|| {
                AbieosError::GetTypeForActionResult(format!(
                    "action_result {action_result} not declared in ABI"
                ))
            })?
            .to_owned();
        self.bin_to_json_into(&ty, bin, out)
    }
}
