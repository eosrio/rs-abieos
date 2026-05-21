use super::swar::{hasless, hasvalue, haszero, word_at, HI, LO};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Json<'a> {
    Null,
    Bool(bool),
    String(std::borrow::Cow<'a, str>),
    Array(Vec<Json<'a>>),
    Object(Vec<(std::borrow::Cow<'a, str>, Json<'a>)>),
}

impl<'a> Json<'a> {
    pub(crate) fn as_object(&self) -> Result<&[(std::borrow::Cow<'a, str>, Json<'a>)], String> {
        match self {
            Json::Object(fields) => Ok(fields),
            _ => Err("Expected {".into()),
        }
    }

    pub(crate) fn as_array(&self) -> Result<&[Json<'a>], String> {
        match self {
            Json::Array(values) => Ok(values),
            _ => Err("Expected [".into()),
        }
    }

    pub(crate) fn as_str_like(&self) -> Result<&str, String> {
        match self {
            Json::String(s) => Ok(s.as_ref()),
            _ => Err("Expected string".into()),
        }
    }
}

pub(crate) struct JsonParser<'a> {
    src: &'a [u8],
    pos: usize,
    depth: usize,
}

impl<'a> JsonParser<'a> {
    pub(crate) fn new(src: &'a str) -> Self {
        Self {
            src: src.as_bytes(),
            pos: 0,
            depth: 0,
        }
    }

    fn parse(mut self) -> Result<Json<'a>, String> {
        let value = self.parse_value()?;
        self.skip_ws();
        if self.pos != self.src.len() {
            return Err("Expected end of json".into());
        }
        Ok(value)
    }

    pub(crate) fn skip_ws(&mut self) {
        let src = self.src;
        let mut i = self.pos;
        // Skip whole 8-byte runs of whitespace at once (pretty-printed ABIs
        // are mostly indentation).
        while i + 8 <= src.len() {
            let w = word_at(src, i);
            let is_ws =
                hasvalue(w, b' ') | hasvalue(w, b'\t') | hasvalue(w, b'\n') | hasvalue(w, b'\r');
            if is_ws & HI == HI {
                i += 8;
            } else {
                break;
            }
        }
        while i < src.len() && matches!(src[i], b' ' | b'\n' | b'\r' | b'\t') {
            i += 1;
        }
        self.pos = i;
    }

    pub(crate) fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    pub(crate) fn bump(&mut self) -> Result<u8, String> {
        let b = self
            .peek()
            .ok_or_else(|| "Unexpected end of json".to_string())?;
        self.pos += 1;
        Ok(b)
    }

    /// True once all input (modulo trailing whitespace) is consumed. Used by
    /// the streaming ABI parser to reject trailing content like `parse` does.
    pub(crate) fn at_end(&mut self) -> bool {
        self.skip_ws();
        self.pos == self.src.len()
    }

    pub(crate) fn expect(&mut self, b: u8, msg: &str) -> Result<(), String> {
        self.skip_ws();
        if self.bump()? == b {
            Ok(())
        } else {
            Err(msg.into())
        }
    }

    // --- Explicit array/object iteration primitives ---------------------
    //
    // These let the streaming ABI parser drive iteration with plain loops
    // (no per-element closures), so scratch buffers can be reused across
    // elements. Error strings match `parse_array`/`parse_object` exactly.

    /// Open a JSON array. Returns `true` if the array is empty (already
    /// fully consumed). A present non-array value errors `"Expected ["`,
    /// matching `as_array` on the DOM path.
    pub(crate) fn array_open(&mut self) -> Result<bool, String> {
        self.skip_ws();
        if self.peek() != Some(b'[') {
            return Err("Expected [".into());
        }
        self.pos += 1;
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(true);
        }
        Ok(false)
    }

    /// After an element, consume the separator. Returns `true` when the
    /// array is done (`]`), `false` to continue (`,`).
    pub(crate) fn array_step(&mut self) -> Result<bool, String> {
        self.skip_ws();
        match self.bump()? {
            b',' => Ok(false),
            b']' => Ok(true),
            _ => Err("Missing ',' or ']' after array element".into()),
        }
    }

    /// Open a JSON object. Returns `true` if empty. A present non-object
    /// value errors `"Expected {"` (matching `as_object`).
    pub(crate) fn object_open(&mut self) -> Result<bool, String> {
        self.expect(b'{', "Expected {")?;
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(true);
        }
        Ok(false)
    }

    /// Read the next object key and consume the following `:`.
    pub(crate) fn member_key(&mut self) -> Result<std::borrow::Cow<'a, str>, String> {
        self.skip_ws();
        let key = self.parse_string()?;
        self.expect(b':', "Missing ':' after object member name")?;
        Ok(key)
    }

    /// After a member, consume the separator. Returns `true` when the
    /// object is done (`}`), `false` to continue (`,`).
    pub(crate) fn object_step(&mut self) -> Result<bool, String> {
        self.skip_ws();
        match self.bump()? {
            b',' => Ok(false),
            b'}' => Ok(true),
            _ => Err("Missing a comma or '}' after an object member".into()),
        }
    }

    pub(crate) fn parse_value(&mut self) -> Result<Json<'a>, String> {
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

    /// Structurally consume one JSON value without materializing it, used to
    /// skip ABI keys the model does not read. Errors are byte-identical to
    /// `parse_value` so malformed-input parity with the DOM path is preserved.
    pub(crate) fn skip_value(&mut self) -> Result<(), String> {
        self.skip_ws();
        if self.depth > 128 {
            return Err("recursion limit reached".into());
        }
        match self.peek() {
            Some(b'n') => self.consume_lit(b"null"),
            Some(b't') => self.consume_lit(b"true"),
            Some(b'f') => self.consume_lit(b"false"),
            Some(b'"') => self.parse_string().map(|_| ()),
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(|_| ()),
            Some(b'[') => {
                self.expect(b'[', "Expected [")?;
                self.depth += 1;
                self.skip_ws();
                if self.peek() == Some(b']') {
                    self.pos += 1;
                    self.depth -= 1;
                    return Ok(());
                }
                loop {
                    self.skip_value()?;
                    self.skip_ws();
                    match self.bump()? {
                        b',' => {}
                        b']' => {
                            self.depth -= 1;
                            return Ok(());
                        }
                        _ => return Err("Missing ',' or ']' after array element".into()),
                    }
                }
            }
            Some(b'{') => {
                self.expect(b'{', "Expected {")?;
                self.depth += 1;
                self.skip_ws();
                if self.peek() == Some(b'}') {
                    self.pos += 1;
                    self.depth -= 1;
                    return Ok(());
                }
                loop {
                    self.skip_ws();
                    self.parse_string()?;
                    self.expect(b':', "Missing ':' after object member name")?;
                    self.skip_value()?;
                    self.skip_ws();
                    match self.bump()? {
                        b',' => {}
                        b'}' => {
                            self.depth -= 1;
                            return Ok(());
                        }
                        _ => return Err("Missing a comma or '}' after an object member".into()),
                    }
                }
            }
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

    pub(crate) fn parse_string(&mut self) -> Result<std::borrow::Cow<'a, str>, String> {
        self.expect(b'"', "Expected string")?;
        let start = self.pos;
        let src = self.src;
        let mut has_escape = false;
        let mut i = start;
        'outer: loop {
            // SWAR-skip ordinary bytes (not `"`, `\`, or a control char).
            while i + 8 <= src.len() {
                let w = word_at(src, i);
                let hit = haszero(w ^ LO.wrapping_mul(b'"' as u64))
                    | haszero(w ^ LO.wrapping_mul(b'\\' as u64))
                    | hasless(w, 0x20);
                if hit == 0 {
                    i += 8;
                } else {
                    break;
                }
            }
            while i < src.len() {
                match src[i] {
                    b'"' => {
                        self.pos = i + 1;
                        let s = std::str::from_utf8(&src[start..i])
                            .map_err(|_| "Invalid encoding in string".to_string())?;
                        if has_escape {
                            return self.parse_string_with_escapes(start);
                        } else {
                            return Ok(std::borrow::Cow::Borrowed(s));
                        }
                    }
                    b'\\' => {
                        // Skip the backslash and its escaped byte
                        // unconditionally (so an escaped quote does not
                        // terminate the string), then resume SWAR scanning.
                        has_escape = true;
                        i += 2;
                        continue 'outer;
                    }
                    0..=31 => {
                        self.pos = i + 1;
                        return Err("Invalid encoding in string".into());
                    }
                    _ => i += 1,
                }
            }
            break;
        }
        self.pos = i.min(src.len());
        Err("Missing closing quote in string".into())
    }

    fn parse_string_with_escapes(&self, start: usize) -> Result<std::borrow::Cow<'a, str>, String> {
        let mut out = Vec::new();
        let mut pos = start;
        while pos < self.src.len() {
            let b = self.src[pos];
            pos += 1;
            match b {
                b'"' => {
                    return String::from_utf8(out)
                        .map(std::borrow::Cow::Owned)
                        .map_err(|_| "Invalid encoding in string".into())
                }
                b'\\' => {
                    if pos >= self.src.len() {
                        return Err("Invalid escape character in string".into());
                    }
                    let esc = self.src[pos];
                    pos += 1;
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
                            if pos + 4 > self.src.len() {
                                return Err("Invalid escape character in string".into());
                            }
                            let mut cp = 0u32;
                            for _ in 0..4 {
                                cp = (cp << 4)
                                    | match self.src[pos] {
                                        b'0'..=b'9' => (self.src[pos] - b'0') as u32,
                                        b'a'..=b'f' => (self.src[pos] - b'a' + 10) as u32,
                                        b'A'..=b'F' => (self.src[pos] - b'A' + 10) as u32,
                                        _ => {
                                            return Err("Invalid escape character in string".into())
                                        }
                                    };
                                pos += 1;
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
        Err("Missing closing quote in string".into())
    }

    pub(crate) fn parse_number(&mut self) -> Result<std::borrow::Cow<'a, str>, String> {
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
            .map(std::borrow::Cow::Borrowed)
            .map_err(|_| "json parse error".into())
    }

    fn parse_array(&mut self) -> Result<Json<'a>, String> {
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
                _ => return Err("Missing ',' or ']' after array element".into()),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Json<'a>, String> {
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
            self.expect(b':', "Missing ':' after object member name")?;
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

pub(crate) fn parse_json(src: &str) -> Result<Json<'_>, String> {
    JsonParser::new(src).parse()
}

pub(crate) fn quote_json(s: &str, out: &mut String) {
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
