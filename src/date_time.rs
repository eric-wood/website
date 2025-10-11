use std::{convert, fmt, str};

use chrono::{self, FixedOffset, ParseError};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Visitor},
};

pub struct DateTime(chrono::DateTime<FixedOffset>);

impl convert::TryFrom<String> for DateTime {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, ParseError> {
        let dt = chrono::DateTime::parse_from_rfc3339(&value)?;
        Ok(DateTime(dt))
    }
}

impl convert::From<DateTime> for String {
    fn from(value: DateTime) -> String {
        value.0.to_rfc3339()
    }
}

impl convert::From<&DateTime> for String {
    fn from(value: &DateTime) -> String {
        value.0.to_rfc3339()
    }
}

struct DateTimeVisitor;

impl<'de> Visitor<'de> for DateTimeVisitor {
    type Value = DateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an ISO 8601 string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match chrono::DateTime::parse_from_rfc3339(value) {
            Ok(dt) => Ok(DateTime(dt)),
            Err(_) => Err(E::custom(format!("unable to parse DateTime: {value}"))),
        }
    }
}

impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let result: String = self.into();
        serializer.serialize_str(&result)
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DateTimeVisitor)
    }
}
