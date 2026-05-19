pub(crate) fn parse_time_seconds(s: &str) -> Result<u32, String> {
    let bytes = s.as_bytes();
    let mut pos = 0;
    let end = bytes.len();

    let mut parse_uint = |pos: &mut usize, digits: usize| -> Option<u32> {
        let mut result = 0u32;
        for _ in 0..digits {
            if *pos < end && bytes[*pos] >= b'0' && bytes[*pos] <= b'9' {
                result = result * 10 + (bytes[*pos] - b'0') as u32;
                *pos += 1;
            } else {
                return None;
            }
        }
        Some(result)
    };

    let y = parse_uint(&mut pos, 4).ok_or_else(|| "Expected time point".to_string())?;
    if pos >= end || bytes[pos] != b'-' { return Err("Expected time point".into()); }
    pos += 1;

    let m = parse_uint(&mut pos, 2).ok_or_else(|| "Expected time point".to_string())?;
    if pos >= end || bytes[pos] != b'-' { return Err("Expected time point".into()); }
    pos += 1;

    let d = parse_uint(&mut pos, 2).ok_or_else(|| "Expected time point".to_string())?;
    if pos >= end || bytes[pos] != b'T' { return Err("Expected time point".into()); }
    pos += 1;

    let h = parse_uint(&mut pos, 2).ok_or_else(|| "Expected time point".to_string())?;
    if pos >= end || bytes[pos] != b':' { return Err("Expected time point".into()); }
    pos += 1;

    let min = parse_uint(&mut pos, 2).ok_or_else(|| "Expected time point".to_string())?;
    if pos >= end || bytes[pos] != b':' { return Err("Expected time point".into()); }
    pos += 1;

    let sec = parse_uint(&mut pos, 2).ok_or_else(|| "Expected time point".to_string())?;

    let days = days_from_civil(y as i32, m, d);
    let result_sec = (days as i32 as u32)
        .wrapping_mul(86400)
        .wrapping_add(h * 3600)
        .wrapping_add(min * 60)
        .wrapping_add(sec);

    if pos < end && bytes[pos] == b'.' {
        pos += 1;
        let mut parsed_digits = false;
        while pos < end && bytes[pos] >= b'0' && bytes[pos] <= b'9' {
            pos += 1;
            parsed_digits = true;
        }
        if !parsed_digits {
            return Err("Expected time point".into());
        }
    }

    if pos != end {
        return Err("Expected time point".into());
    }

    Ok(result_sec)
}

pub(crate) fn parse_time_microseconds(s: &str) -> Result<u64, String> {
    let bytes = s.as_bytes();
    let mut pos = 0;
    let end = bytes.len();

    let mut parse_uint = |pos: &mut usize, digits: usize| -> Option<u32> {
        let mut result = 0u32;
        for _ in 0..digits {
            if *pos < end && bytes[*pos] >= b'0' && bytes[*pos] <= b'9' {
                result = result * 10 + (bytes[*pos] - b'0') as u32;
                *pos += 1;
            } else {
                return None;
            }
        }
        Some(result)
    };

    let y = parse_uint(&mut pos, 4).ok_or_else(|| "Expected time point".to_string())?;
    if pos >= end || bytes[pos] != b'-' { return Err("Expected time point".into()); }
    pos += 1;

    let m = parse_uint(&mut pos, 2).ok_or_else(|| "Expected time point".to_string())?;
    if pos >= end || bytes[pos] != b'-' { return Err("Expected time point".into()); }
    pos += 1;

    let d = parse_uint(&mut pos, 2).ok_or_else(|| "Expected time point".to_string())?;
    if pos >= end || bytes[pos] != b'T' { return Err("Expected time point".into()); }
    pos += 1;

    let h = parse_uint(&mut pos, 2).ok_or_else(|| "Expected time point".to_string())?;
    if pos >= end || bytes[pos] != b':' { return Err("Expected time point".into()); }
    pos += 1;

    let min = parse_uint(&mut pos, 2).ok_or_else(|| "Expected time point".to_string())?;
    if pos >= end || bytes[pos] != b':' { return Err("Expected time point".into()); }
    pos += 1;

    let sec = parse_uint(&mut pos, 2).ok_or_else(|| "Expected time point".to_string())?;

    let days = days_from_civil(y as i32, m, d);
    let result_sec = (days as i32 as u32)
        .wrapping_mul(86400)
        .wrapping_add(h * 3600)
        .wrapping_add(min * 60)
        .wrapping_add(sec);

    let mut result_us = (result_sec as u64).wrapping_mul(1_000_000);

    if pos < end {
        if bytes[pos] != b'.' {
            return Err("Expected time point".into());
        }
        pos += 1;
        let mut scale = 100_000u64;
        let mut parsed_digits = false;
        while scale >= 1 && pos < end && bytes[pos] >= b'0' && bytes[pos] <= b'9' {
            result_us = result_us.wrapping_add((bytes[pos] - b'0') as u64 * scale);
            scale /= 10;
            pos += 1;
            parsed_digits = true;
        }
        if !parsed_digits {
            return Err("Expected time point".into());
        }
    }

    if pos != end {
        return Err("Expected time point".into());
    }

    Ok(result_us)
}

pub(crate) fn format_time_microseconds(us: u64) -> String {
    let secs = us / 1_000_000;
    let millis = (us / 1000) % 1000;
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}",
        y,
        m,
        d,
        rem / 3600,
        rem / 60 % 60,
        rem % 60,
        millis
    )
}

fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = y - (m <= 2) as i32;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era as i64 * 146_097 + doe as i64 - 719_468
}

fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (y + (m <= 2) as i32, m, d)
}
