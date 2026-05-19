const HEX: &[u8; 16] = b"0123456789ABCDEF";

pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

pub(crate) fn hex_encode_into(bytes: &[u8], out: &mut Vec<u8>) {
    out.reserve(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize]);
        out.push(HEX[(b & 0x0f) as usize]);
    }
}

pub(crate) fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
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
        let h = nibble(pair[0]).ok_or_else(|| "Expected string containing hex".to_string())?;
        let l = nibble(pair[1]).ok_or_else(|| "Expected string containing hex".to_string())?;
        out.push((h << 4) | l);
    }
    Ok(out)
}
