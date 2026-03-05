//! Datetime serialization/deserialization helpers.
//!
//! Provides custom Serde serialization/deserialization support:
//! - Serialization: `DateTime<Utc>` -> RFC3339 string
//! - Deserialization: RFC3339 string or Unix timestamp -> `DateTime<Utc>`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serializer};

/// Serializes `DateTime<Utc>` as an RFC3339 string.
pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&dt.to_rfc3339())
}

/// Deserializes `DateTime<Utc>` from RFC3339 or Unix timestamp.
///
/// Unix timestamps are auto-detected as seconds or milliseconds.
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
            // Try RFC3339 first.
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| Error::custom(format!("Invalid RFC3339 timestamp: {e}")))
        }
        TimestampOrString::I64(ts) => {
            // Unix timestamp with second/millisecond auto-detection.
            parse_unix_timestamp(ts).ok_or_else(|| Error::custom("Invalid Unix timestamp"))
        }
        TimestampOrString::U64(ts) => {
            let ts =
                i64::try_from(ts).map_err(|_| Error::custom("Unix timestamp out of i64 range"))?;
            parse_unix_timestamp(ts).ok_or_else(|| Error::custom("Invalid Unix timestamp"))
        }
    }
}

/// `Option<DateTime<Utc>>` serializer/deserializer helpers.
pub mod option {
    use super::{DateTime, Deserialize, Deserializer, Serializer, Utc, parse_unix_timestamp};

    /// Serializes `Option<DateTime<Utc>>` as RFC3339 or `null`.
    pub fn serialize<S>(dt: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match dt {
            Some(dt) => serializer.serialize_some(&dt.to_rfc3339()),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes `Option<DateTime<Utc>>` from RFC3339, Unix timestamp, or `null`.
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
            Some(OptionalTimestamp::U64(ts)) => {
                let ts = i64::try_from(ts)
                    .map_err(|_| Error::custom("Unix timestamp out of i64 range"))?;
                parse_unix_timestamp(ts)
                    .map(Some)
                    .ok_or_else(|| Error::custom("Invalid Unix timestamp"))
            }
            None => Ok(None),
        }
    }
}

/// Parses a Unix timestamp with second/millisecond auto-detection.
fn parse_unix_timestamp(ts: i64) -> Option<DateTime<Utc>> {
    // Use absolute value to correctly handle negative timestamps (pre-epoch dates).
    // Values whose magnitude exceeds 10^11 are interpreted as milliseconds.
    if ts.unsigned_abs() > 100_000_000_000 {
        DateTime::from_timestamp_millis(ts)
    } else {
        // Otherwise treat the value as seconds.
        DateTime::from_timestamp(ts, 0)
    }
}
