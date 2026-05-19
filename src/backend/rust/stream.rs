pub(crate) struct Writer<'a> {
    data: &'a mut Vec<u8>,
}

impl<'a> Writer<'a> {
    pub(crate) fn new(data: &'a mut Vec<u8>) -> Self {
        Self { data }
    }
    pub(crate) fn push(&mut self, b: u8) {
        self.data.push(b);
    }
    pub(crate) fn write(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
    pub(crate) fn u16(&mut self, v: u16) {
        self.write(&v.to_le_bytes());
    }
    pub(crate) fn u32(&mut self, v: u32) {
        self.write(&v.to_le_bytes());
    }
    pub(crate) fn u64(&mut self, v: u64) {
        self.write(&v.to_le_bytes());
    }
    pub(crate) fn i16(&mut self, v: i16) {
        self.write(&v.to_le_bytes());
    }
    pub(crate) fn i32(&mut self, v: i32) {
        self.write(&v.to_le_bytes());
    }
    pub(crate) fn i64(&mut self, v: i64) {
        self.write(&v.to_le_bytes());
    }
    pub(crate) fn u128(&mut self, v: u128) {
        self.write(&v.to_le_bytes());
    }
    pub(crate) fn i128(&mut self, v: i128) {
        self.write(&v.to_le_bytes());
    }
    pub(crate) fn varuint32(&mut self, mut v: u32) {
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
    pub(crate) fn string(&mut self, s: &str) {
        self.varuint32(s.len() as u32);
        self.write(s.as_bytes());
    }
    pub(crate) fn bytes_vec(&mut self, bytes: &[u8]) {
        self.varuint32(bytes.len() as u32);
        self.write(bytes);
    }
}

pub(crate) struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
    pub(crate) fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }
    pub(crate) fn read(&mut self, len: usize) -> Result<&'a [u8], String> {
        if self.remaining() < len {
            return Err("read datastream of length over by".into());
        }
        let start = self.pos;
        self.pos += len;
        Ok(&self.data[start..self.pos])
    }
    pub(crate) fn byte(&mut self) -> Result<u8, String> {
        Ok(self.read(1)?[0])
    }
    pub(crate) fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.read(2)?.try_into().unwrap()))
    }
    pub(crate) fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.read(4)?.try_into().unwrap()))
    }
    pub(crate) fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.read(8)?.try_into().unwrap()))
    }
    pub(crate) fn i16(&mut self) -> Result<i16, String> {
        Ok(i16::from_le_bytes(self.read(2)?.try_into().unwrap()))
    }
    pub(crate) fn i32(&mut self) -> Result<i32, String> {
        Ok(i32::from_le_bytes(self.read(4)?.try_into().unwrap()))
    }
    pub(crate) fn i64(&mut self) -> Result<i64, String> {
        Ok(i64::from_le_bytes(self.read(8)?.try_into().unwrap()))
    }
    pub(crate) fn u128(&mut self) -> Result<u128, String> {
        Ok(u128::from_le_bytes(self.read(16)?.try_into().unwrap()))
    }
    pub(crate) fn i128(&mut self) -> Result<i128, String> {
        Ok(i128::from_le_bytes(self.read(16)?.try_into().unwrap()))
    }
    pub(crate) fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_le_bytes(self.read(4)?.try_into().unwrap()))
    }
    pub(crate) fn f64(&mut self) -> Result<f64, String> {
        Ok(f64::from_le_bytes(self.read(8)?.try_into().unwrap()))
    }
    pub(crate) fn varuint32(&mut self) -> Result<u32, String> {
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
    pub(crate) fn string(&mut self) -> Result<String, String> {
        let len = self.varuint32()? as usize;
        let bytes = self.read(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| "Invalid encoding in string".into())
    }
    pub(crate) fn istr(&mut self) -> Result<super::istr::IStr, String> {
        let len = self.varuint32()? as usize;
        let bytes = self.read(len)?;
        let s = std::str::from_utf8(bytes).map_err(|_| "Invalid encoding in string".to_string())?;
        Ok(super::istr::IStr::from(s))
    }
    pub(crate) fn bytes_vec(&mut self) -> Result<Vec<u8>, String> {
        let len = self.varuint32()? as usize;
        Ok(self.read(len)?.to_vec())
    }
}
