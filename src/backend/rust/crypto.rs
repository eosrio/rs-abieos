use super::stream::{Reader, Writer};

#[derive(Clone, Copy)]
pub(crate) enum KeyKind {
    Public,
    Private,
    Signature,
}

pub(crate) fn write_key_like(s: &str, w: &mut Writer, kind: KeyKind) -> Result<(), String> {
    let (idx, body, suffix, fixed_len, legacy_wif) = match kind {
        KeyKind::Public if let Some(rest) = s.strip_prefix("EOS") => (0, rest, "", Some(33), false),
        KeyKind::Public if let Some(rest) = s.strip_prefix("PUB_K1_") => {
            (0, rest, "K1", Some(33), false)
        }
        KeyKind::Public if let Some(rest) = s.strip_prefix("PUB_R1_") => {
            (1, rest, "R1", Some(33), false)
        }
        KeyKind::Public if let Some(rest) = s.strip_prefix("PUB_WA_") => {
            (2, rest, "WA", None, false)
        }
        KeyKind::Private if let Some(rest) = s.strip_prefix("PVT_K1_") => {
            (0, rest, "K1", Some(32), false)
        }
        KeyKind::Private if let Some(rest) = s.strip_prefix("PVT_R1_") => {
            (1, rest, "R1", Some(32), false)
        }
        KeyKind::Private if s.starts_with("PVT_") => return Err("expected private_key".into()),
        KeyKind::Private => (0, s, "", Some(32), true),
        KeyKind::Signature if let Some(rest) = s.strip_prefix("SIG_K1_") => {
            (0, rest, "K1", Some(65), false)
        }
        KeyKind::Signature if let Some(rest) = s.strip_prefix("SIG_R1_") => {
            (1, rest, "R1", Some(65), false)
        }
        KeyKind::Signature if let Some(rest) = s.strip_prefix("SIG_WA_") => {
            (2, rest, "WA", None, false)
        }
        _ => return Err("unrecognized key format".into()),
    };
    let mut decoded = base58_decode(body)?;
    if decoded.len() < 4 {
        return Err("expected key".into());
    }
    let checksum = decoded.split_off(decoded.len() - 4);
    if legacy_wif {
        if decoded.len() != 33 {
            return Err("key has invalid size".into());
        }
        decoded.remove(0);
    } else {
        if let Some(expected) = fixed_len {
            if decoded.len() != expected {
                return Err("key has invalid size".into());
            }
        }
        let digest = ripemd160_with_suffix(&decoded, suffix.as_bytes());
        if checksum != digest[..4] {
            return Err("expected key".into());
        }
    }
    w.varuint32(idx);
    w.write(&decoded);
    Ok(())
}

pub(crate) fn read_key_like(r: &mut Reader, kind: KeyKind) -> Result<String, String> {
    let idx = r.varuint32()?;
    let len = match (kind, idx) {
        (KeyKind::Public, 0 | 1) => 33,
        (KeyKind::Private, 0 | 1) => 32,
        (KeyKind::Signature, 0 | 1) => 65,
        (KeyKind::Public, 2) => {
            let key = r.read(33)?.to_vec();
            let presence = r.byte()?;
            let rpid = r.string()?;
            let mut body = key;
            body.push(presence);
            let mut buf = Vec::new();
            let mut tmp = Writer::new(&mut buf);
            tmp.string(&rpid);
            body.extend(buf);
            return Ok(format!(
                "PUB_WA_{}",
                base58_encode_with_checksum(&body, b"WA")
            ));
        }
        (KeyKind::Signature, 2) => {
            let sig = r.read(65)?.to_vec();
            let auth = r.bytes_vec()?;
            let client = r.string()?;
            let mut body = sig;
            let mut buf = Vec::new();
            let mut tmp = Writer::new(&mut buf);
            tmp.bytes_vec(&auth);
            tmp.string(&client);
            body.extend(buf);
            return Ok(format!(
                "SIG_WA_{}",
                base58_encode_with_checksum(&body, b"WA")
            ));
        }
        _ => return Err("bad variant index".into()),
    };
    let body = r.read(len)?.to_vec();
    let (prefix, suffix) = match (kind, idx) {
        (KeyKind::Public, 0) => ("PUB_K1_", b"K1".as_slice()),
        (KeyKind::Public, 1) => ("PUB_R1_", b"R1".as_slice()),
        (KeyKind::Private, 0) => ("PVT_K1_", b"K1".as_slice()),
        (KeyKind::Private, 1) => ("PVT_R1_", b"R1".as_slice()),
        (KeyKind::Signature, 0) => ("SIG_K1_", b"K1".as_slice()),
        (KeyKind::Signature, 1) => ("SIG_R1_", b"R1".as_slice()),
        _ => ("", b"".as_slice()),
    };
    Ok(format!(
        "{}{}",
        prefix,
        base58_encode_with_checksum(&body, suffix)
    ))
}

const BASE58: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

fn base58_value(b: u8) -> Option<u8> {
    BASE58.iter().position(|c| *c == b).map(|v| v as u8)
}

fn base58_decode(s: &str) -> Result<Vec<u8>, String> {
    let mut out: Vec<u8> = Vec::new();
    for ch in s.bytes() {
        let mut carry = base58_value(ch).ok_or_else(|| "expected key".to_string())? as u32;
        for byte in out.iter_mut().rev() {
            let x = (*byte as u32) * 58 + carry;
            *byte = (x & 0xff) as u8;
            carry = x >> 8;
        }
        while carry > 0 {
            out.insert(0, (carry & 0xff) as u8);
            carry >>= 8;
        }
    }
    for ch in s.bytes() {
        if ch == b'1' {
            out.insert(0, 0);
        } else {
            break;
        }
    }
    Ok(out)
}

fn base58_encode_with_checksum(data: &[u8], suffix: &[u8]) -> String {
    let mut whole = data.to_vec();
    let digest = ripemd160_with_suffix(data, suffix);
    whole.extend_from_slice(&digest[..4]);
    base58_encode(&whole)
}

fn base58_encode(data: &[u8]) -> String {
    let mut digits: Vec<u8> = Vec::new();
    for byte in data {
        let mut carry = *byte as u32;
        for digit in &mut digits {
            let x = (*digit as u32) * 256 + carry;
            *digit = (x % 58) as u8;
            carry = x / 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }
    for byte in data {
        if *byte == 0 {
            digits.push(0);
        } else {
            break;
        }
    }
    digits
        .iter()
        .rev()
        .map(|d| BASE58[*d as usize] as char)
        .collect()
}

fn ripemd160_with_suffix(data: &[u8], suffix: &[u8]) -> [u8; 20] {
    let mut input = Vec::with_capacity(data.len() + suffix.len());
    input.extend_from_slice(data);
    input.extend_from_slice(suffix);
    ripemd160(&input)
}

fn ripemd160(data: &[u8]) -> [u8; 20] {
    const R: [usize; 80] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 7, 4, 13, 1, 10, 6, 15, 3, 12, 0, 9,
        5, 2, 14, 11, 8, 3, 10, 14, 4, 9, 15, 8, 1, 2, 7, 0, 6, 13, 11, 5, 12, 1, 9, 11, 10, 0, 8,
        12, 4, 13, 3, 7, 15, 14, 5, 6, 2, 4, 0, 5, 9, 7, 12, 2, 10, 14, 1, 3, 8, 11, 6, 15, 13,
    ];
    const RP: [usize; 80] = [
        5, 14, 7, 0, 9, 2, 11, 4, 13, 6, 15, 8, 1, 10, 3, 12, 6, 11, 3, 7, 0, 13, 5, 10, 14, 15, 8,
        12, 4, 9, 1, 2, 15, 5, 1, 3, 7, 14, 6, 9, 11, 8, 12, 2, 10, 0, 4, 13, 8, 6, 4, 1, 3, 11,
        15, 0, 5, 12, 2, 13, 9, 7, 10, 14, 12, 15, 10, 4, 1, 5, 8, 7, 6, 2, 13, 14, 0, 3, 9, 11,
    ];
    const S: [u32; 80] = [
        11, 14, 15, 12, 5, 8, 7, 9, 11, 13, 14, 15, 6, 7, 9, 8, 7, 6, 8, 13, 11, 9, 7, 15, 7, 12,
        15, 9, 11, 7, 13, 12, 11, 13, 6, 7, 14, 9, 13, 15, 14, 8, 13, 6, 5, 12, 7, 5, 11, 12, 14,
        15, 14, 15, 9, 8, 9, 14, 5, 6, 8, 6, 5, 12, 9, 15, 5, 11, 6, 8, 13, 12, 5, 12, 13, 14, 11,
        8, 5, 6,
    ];
    const SP: [u32; 80] = [
        8, 9, 9, 11, 13, 15, 15, 5, 7, 7, 8, 11, 14, 14, 12, 6, 9, 13, 15, 7, 12, 8, 9, 11, 7, 7,
        12, 7, 6, 15, 13, 11, 9, 7, 15, 11, 8, 6, 6, 14, 12, 13, 5, 14, 13, 13, 7, 5, 15, 5, 8, 11,
        14, 14, 6, 14, 6, 9, 12, 9, 12, 5, 15, 8, 8, 5, 12, 9, 12, 5, 14, 6, 8, 13, 6, 5, 15, 13,
        11, 11,
    ];

    let mut msg = data.to_vec();
    let bit_len = (msg.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());

    let mut h0 = 0x6745_2301u32;
    let mut h1 = 0xefcd_ab89u32;
    let mut h2 = 0x98ba_dcfeu32;
    let mut h3 = 0x1032_5476u32;
    let mut h4 = 0xc3d2_e1f0u32;

    for chunk in msg.chunks_exact(64) {
        let mut x = [0u32; 16];
        for (i, word) in x.iter_mut().enumerate() {
            let start = i * 4;
            *word = u32::from_le_bytes([
                chunk[start],
                chunk[start + 1],
                chunk[start + 2],
                chunk[start + 3],
            ]);
        }

        let (mut al, mut bl, mut cl, mut dl, mut el) = (h0, h1, h2, h3, h4);
        let (mut ar, mut br, mut cr, mut dr, mut er) = (h0, h1, h2, h3, h4);

        for j in 0..80 {
            let tl = al
                .wrapping_add(ripemd160_f(j, bl, cl, dl))
                .wrapping_add(x[R[j]])
                .wrapping_add(ripemd160_kl(j))
                .rotate_left(S[j])
                .wrapping_add(el);
            al = el;
            el = dl;
            dl = cl.rotate_left(10);
            cl = bl;
            bl = tl;

            let tr = ar
                .wrapping_add(ripemd160_f(79 - j, br, cr, dr))
                .wrapping_add(x[RP[j]])
                .wrapping_add(ripemd160_kr(j))
                .rotate_left(SP[j])
                .wrapping_add(er);
            ar = er;
            er = dr;
            dr = cr.rotate_left(10);
            cr = br;
            br = tr;
        }

        let t = h1.wrapping_add(cl).wrapping_add(dr);
        h1 = h2.wrapping_add(dl).wrapping_add(er);
        h2 = h3.wrapping_add(el).wrapping_add(ar);
        h3 = h4.wrapping_add(al).wrapping_add(br);
        h4 = h0.wrapping_add(bl).wrapping_add(cr);
        h0 = t;
    }

    let mut out = [0u8; 20];
    for (i, word) in [h0, h1, h2, h3, h4].into_iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    out
}

fn ripemd160_f(j: usize, x: u32, y: u32, z: u32) -> u32 {
    match j {
        0..=15 => x ^ y ^ z,
        16..=31 => (x & y) | (!x & z),
        32..=47 => (x | !y) ^ z,
        48..=63 => (x & z) | (y & !z),
        _ => x ^ (y | !z),
    }
}

fn ripemd160_kl(j: usize) -> u32 {
    match j {
        0..=15 => 0x0000_0000,
        16..=31 => 0x5a82_7999,
        32..=47 => 0x6ed9_eba1,
        48..=63 => 0x8f1b_bcdc,
        _ => 0xa953_fd4e,
    }
}

fn ripemd160_kr(j: usize) -> u32 {
    match j {
        0..=15 => 0x50a2_8be6,
        16..=31 => 0x5c4d_d124,
        32..=47 => 0x6d70_3ef3,
        48..=63 => 0x7a6d_76e9,
        _ => 0x0000_0000,
    }
}
