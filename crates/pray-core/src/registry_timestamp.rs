use serde::Deserialize;

pub const MAX_PUBLISH_TIMESTAMP: u64 = 253_402_300_799;

pub fn deserialize_optional_publish_timestamp<'de, D>(
    deserializer: D,
) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    if value.is_null() {
        return Ok(None);
    }
    unix_seconds_from_json(value)
        .filter(|timestamp| *timestamp <= MAX_PUBLISH_TIMESTAMP)
        .ok_or_else(|| serde::de::Error::custom("published_at must be whole UTC Unix seconds"))
        .map(Some)
}

fn unix_seconds_from_json(value: serde_json::Value) -> Option<u64> {
    match value {
        serde_json::Value::Number(number) => number.as_u64(),
        serde_json::Value::String(legacy) => unix_seconds_from_legacy_string(&legacy),
        _ => None,
    }
}

fn unix_seconds_from_legacy_string(value: &str) -> Option<u64> {
    value
        .parse::<u64>()
        .ok()
        .or_else(|| unix_seconds_from_rfc3339(value))
}

fn unix_seconds_from_rfc3339(value: &str) -> Option<u64> {
    let bytes = value.as_bytes();
    if bytes.len() < 20 {
        return None;
    }
    let year = digits(bytes, 0, 4)?;
    require_byte(bytes, 4, b'-')?;
    let month = digits(bytes, 5, 2)?;
    require_byte(bytes, 7, b'-')?;
    let day = digits(bytes, 8, 2)?;
    require_byte(bytes, 10, b'T')?;
    let hour = digits(bytes, 11, 2)?;
    require_byte(bytes, 13, b':')?;
    let minute = digits(bytes, 14, 2)?;
    require_byte(bytes, 16, b':')?;
    let second = digits(bytes, 17, 2)?;
    let mut index = 19;
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let fraction_start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index == fraction_start {
            return None;
        }
    }
    let (offset_seconds, end) = timezone_offset(bytes, index)?;
    if end != bytes.len()
        || !valid_date(year, month, day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }
    days_from_civil(year, month, day)
        .checked_mul(86_400)?
        .checked_add(i64::from(hour) * 3600 + i64::from(minute) * 60 + i64::from(second))?
        .checked_sub(offset_seconds)
        .and_then(|utc| u64::try_from(utc).ok())
}

fn timezone_offset(bytes: &[u8], index: usize) -> Option<(i64, usize)> {
    match bytes.get(index)? {
        b'Z' => Some((0, index + 1)),
        b'+' | b'-' => {
            let sign = if bytes[index] == b'+' { 1 } else { -1 };
            let hour = digits(bytes, index + 1, 2)?;
            require_byte(bytes, index + 3, b':')?;
            let minute = digits(bytes, index + 4, 2)?;
            if hour > 23 || minute > 59 {
                return None;
            }
            Some((
                sign * (i64::from(hour) * 3600 + i64::from(minute) * 60),
                index + 6,
            ))
        }
        _ => None,
    }
}

fn valid_date(year: u32, month: u32, day: u32) -> bool {
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) {
                29
            } else {
                28
            }
        }
        _ => return false,
    };
    day >= 1 && day <= days_in_month
}

fn days_from_civil(year: u32, month: u32, day: u32) -> i64 {
    let mut year = i64::from(year);
    if month <= 2 {
        year -= 1;
    }
    let era = year.div_euclid(400);
    let year_of_era = (year - era * 400) as u32;
    let shifted_month = i64::from(month) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + i64::from(day) - 1;
    let day_of_era =
        i64::from(year_of_era) * 365 + i64::from(year_of_era / 4 - year_of_era / 100) + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn digits(bytes: &[u8], start: usize, width: usize) -> Option<u32> {
    let slice = bytes.get(start..start + width)?;
    if !slice.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(slice).ok()?.parse().ok()
}

fn require_byte(bytes: &[u8], index: usize, expected: u8) -> Option<()> {
    (bytes.get(index) == Some(&expected)).then_some(())
}
