//! 日期时间序列化/反序列化工具
//!
//! 提供自定义 Serde 序列化/反序列化支持：
//! - 序列化: `DateTime`<Utc> -> RFC3339 字符串
//! - 反序列化: RFC3339 字符串 或 Unix 时间戳 -> `DateTime`<Utc>

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serializer};

/// 序列化 `DateTime`<Utc> 为 RFC3339 字符串
pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&dt.to_rfc3339())
}

/// 反序列化：支持 RFC3339 字符串或 Unix 时间戳（秒/毫秒自动识别）
pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TimestampOrString {
        String(String),
        I64(i64),
        U64(u64),
    }

    match TimestampOrString::deserialize(deserializer)? {
        TimestampOrString::String(s) => {
            // 尝试解析 RFC3339
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| Error::custom(format!("Invalid RFC3339 timestamp: {e}")))
        }
        TimestampOrString::I64(ts) => {
            // Unix 时间戳（自动判断秒/毫秒）
            parse_unix_timestamp(ts).ok_or_else(|| Error::custom("Invalid Unix timestamp"))
        }
        TimestampOrString::U64(ts) => {
            parse_unix_timestamp(ts as i64).ok_or_else(|| Error::custom("Invalid Unix timestamp"))
        }
    }
}

/// Option 版本：序列化和反序列化 Option<`DateTime`<Utc>>
pub mod option {
    use super::{parse_unix_timestamp, DateTime, Deserialize, Deserializer, Serializer, Utc};

    pub fn serialize<S>(dt: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match dt {
            Some(dt) => serializer.serialize_some(&dt.to_rfc3339()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum OptionalTimestamp {
            String(String),
            I64(i64),
            U64(u64),
        }

        match Option::<OptionalTimestamp>::deserialize(deserializer)? {
            Some(OptionalTimestamp::String(s)) => DateTime::parse_from_rfc3339(&s)
                .map(|dt| Some(dt.with_timezone(&Utc)))
                .map_err(|e| Error::custom(format!("Invalid RFC3339 timestamp: {e}"))),
            Some(OptionalTimestamp::I64(ts)) => parse_unix_timestamp(ts)
                .map(Some)
                .ok_or_else(|| Error::custom("Invalid Unix timestamp")),
            Some(OptionalTimestamp::U64(ts)) => parse_unix_timestamp(ts as i64)
                .map(Some)
                .ok_or_else(|| Error::custom("Invalid Unix timestamp")),
            None => Ok(None),
        }
    }
}

/// 解析 Unix 时间戳（自动判断秒/毫秒）
fn parse_unix_timestamp(ts: i64) -> Option<DateTime<Utc>> {
    // 如果时间戳 > 10^11，认为是毫秒（阿里云使用毫秒时间戳）
    if ts > 100_000_000_000 {
        DateTime::from_timestamp_millis(ts)
    } else {
        // 否则认为是秒
        DateTime::from_timestamp(ts, 0)
    }
}
