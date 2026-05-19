fn char_to_name_digit(c: u8) -> u64 {
    match c {
        b'a'..=b'z' => (c - b'a' + 6) as u64,
        b'1'..=b'5' => (c - b'1' + 1) as u64,
        _ => 0,
    }
}

pub(crate) fn bytes_to_name_value(bytes: &[u8]) -> u64 {
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

pub(crate) fn string_to_name_value(s: &str) -> u64 {
    bytes_to_name_value(s.as_bytes())
}

pub(crate) fn name_to_string_value_into(name: u64, out: &mut Vec<u8>) {
    const CHARMAP: &[u8; 32] = b".12345abcdefghijklmnopqrstuvwxyz";
    let mut tmp = name;
    let mut chars = [b'.'; 13];
    for i in 0..=12 {
        let mask = if i == 0 { 0x0f } else { 0x1f };
        chars[12 - i] = CHARMAP[(tmp & mask) as usize];
        tmp >>= if i == 0 { 4 } else { 5 };
    }
    let last = chars.iter().rposition(|c| *c != b'.');
    if let Some(i) = last {
        out.extend_from_slice(&chars[..=i]);
    }
}

pub(crate) fn name_to_string_value(name: u64) -> String {
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
