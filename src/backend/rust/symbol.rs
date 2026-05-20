pub(crate) fn string_to_symbol_code(s: &str) -> Result<u64, String> {
    let bytes = s.as_bytes();
    let mut pos = 0;
    let end = bytes.len();

    while pos < end && bytes[pos] == b' ' {
        pos += 1;
    }
    let mut code = 0u64;
    let mut i = 0usize;
    while pos < end && bytes[pos] >= b'A' && bytes[pos] <= b'Z' {
        if i >= 7 {
            return Err("Expected symbol code".into());
        }
        code |= (bytes[pos] as u64) << (8 * i);
        i += 1;
        pos += 1;
    }
    if i == 0 || pos != end {
        return Err("Expected symbol code".into());
    }
    Ok(code)
}

pub(crate) fn symbol_code_to_string(mut v: u64) -> String {
    let mut out = String::new();
    while v > 0 {
        out.push((v & 0xff) as u8 as char);
        v >>= 8;
    }
    out
}

pub(crate) fn string_to_symbol(s: &str) -> Result<u64, String> {
    let bytes = s.as_bytes();
    let mut pos = 0;
    let end = bytes.len();

    let mut precision = 0u8;
    let mut found = false;
    while pos < end && bytes[pos] >= b'0' && bytes[pos] <= b'9' {
        precision = precision.wrapping_mul(10).wrapping_add(bytes[pos] - b'0');
        found = true;
        pos += 1;
    }
    if !found || pos >= end || bytes[pos] != b',' {
        return Err("Expected symbol".into());
    }
    pos += 1;

    while pos < end && bytes[pos] == b' ' {
        pos += 1;
    }
    let mut code = 0u64;
    let mut i = 0usize;
    while pos < end && bytes[pos] >= b'A' && bytes[pos] <= b'Z' {
        if i >= 7 {
            return Err("Expected symbol".into());
        }
        code |= (bytes[pos] as u64) << (8 * i);
        i += 1;
        pos += 1;
    }
    if i == 0 || pos != end {
        return Err("Expected symbol".into());
    }

    Ok((code << 8) | precision as u64)
}

pub(crate) fn symbol_to_string(v: u64) -> String {
    format!("{},{}", v & 0xff, symbol_code_to_string(v >> 8))
}

pub(crate) fn string_to_asset(s: &str) -> Result<(i64, u64), String> {
    let bytes = s.as_bytes();
    let mut pos = 0;
    let end = bytes.len();

    while pos < end && bytes[pos] == b' ' {
        pos += 1;
    }

    let mut uamount = 0u64;
    let mut precision = 0u8;
    let mut negative = false;

    if pos < end && bytes[pos] == b'-' {
        negative = true;
        pos += 1;
    }

    let mut found_digit = false;
    while pos < end && bytes[pos] >= b'0' && bytes[pos] <= b'9' {
        uamount = uamount
            .wrapping_mul(10)
            .wrapping_add((bytes[pos] - b'0') as u64);
        found_digit = true;
        pos += 1;
    }
    if !found_digit {
        return Err("Expected string containing asset".into());
    }

    if pos < end && bytes[pos] == b'.' {
        pos += 1;
        while pos < end && bytes[pos] >= b'0' && bytes[pos] <= b'9' {
            uamount = uamount
                .wrapping_mul(10)
                .wrapping_add((bytes[pos] - b'0') as u64);
            precision = precision
                .checked_add(1)
                .ok_or_else(|| "precision overflow".to_string())?;
            pos += 1;
        }
    }

    let amount = if negative {
        uamount.wrapping_neg() as i64
    } else {
        uamount as i64
    };

    while pos < end && bytes[pos] == b' ' {
        pos += 1;
    }

    let mut code = 0u64;
    let mut i = 0usize;
    while pos < end && bytes[pos] >= b'A' && bytes[pos] <= b'Z' {
        if i >= 7 {
            return Err("Expected string containing asset".into());
        }
        code |= (bytes[pos] as u64) << (8 * i);
        i += 1;
        pos += 1;
    }

    if i == 0 || pos != end {
        return Err("Expected string containing asset".into());
    }

    let symbol = (code << 8) | precision as u64;
    Ok((amount, symbol))
}

pub(crate) fn asset_to_string(amount: i64, symbol: u64) -> String {
    let precision = (symbol & 0xff) as usize;
    let mut uamount = amount.unsigned_abs();
    let mut chars = Vec::new();
    for _ in 0..precision {
        chars.push((b'0' + (uamount % 10) as u8) as char);
        uamount /= 10;
    }
    if precision > 0 {
        chars.push('.');
    }
    loop {
        chars.push((b'0' + (uamount % 10) as u8) as char);
        uamount /= 10;
        if uamount == 0 {
            break;
        }
    }
    if amount < 0 {
        chars.push('-');
    }
    chars.reverse();
    format!(
        "{} {}",
        chars.into_iter().collect::<String>(),
        symbol_code_to_string(symbol >> 8)
    )
}

pub(crate) fn bitset_from_string(s: &str) -> Result<Vec<u8>, String> {
    let mut bytes = vec![0u8; s.len().div_ceil(8)];
    for (i, ch) in s.bytes().enumerate() {
        match ch {
            b'0' => {}
            b'1' => {
                let bit = s.len() - i - 1;
                bytes[bit / 8] |= 1 << (bit % 8);
            }
            _ => return Err("unexpected character in bitset".into()),
        }
    }
    Ok(bytes)
}

pub(crate) fn bitset_to_string(bits: usize, bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bits);
    for bit in (0..bits).rev() {
        out.push(if bytes[bit / 8] & (1 << (bit % 8)) != 0 {
            '1'
        } else {
            '0'
        });
    }
    out
}
