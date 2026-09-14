use serde::Deserialize;

pub const MAX_PUBLISH_TIMESTAMP: u64 = 253_402_300_799;

pub fn deserialize_optional_publish_timestamp<'de, D>(
    deserializer: D,
) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let timestamp = match value {
        serde_json::Value::Number(number) => number.as_u64(),
        serde_json::Value::String(legacy) => legacy.parse::<u64>().ok(),
        _ => None,
    }
    .filter(|timestamp| *timestamp <= MAX_PUBLISH_TIMESTAMP)
    .ok_or_else(|| serde::de::Error::custom("published_at must be whole UTC Unix seconds"))?;
    Ok(Some(timestamp))
}
