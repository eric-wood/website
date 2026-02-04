use core::cmp::Ord;
use std::{convert, fmt, str};

use chrono::{self, FixedOffset, ParseError};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Visitor},
};

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct DateTime(chrono::DateTime<FixedOffset>);

impl DateTime {
    pub fn now() -> Self {
        Self(chrono::Utc::now().fixed_offset())
    }

    pub fn min_date() -> Self {
        Self(chrono::DateTime::<FixedOffset>::MIN_UTC.fixed_offset())
    }
}

impl convert::TryFrom<String> for DateTime {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, ParseError> {
        // attempt parsing a few formats
        let dt = chrono::DateTime::parse_from_rfc3339(&value).or_else(|_| {
            Ok(
                chrono::NaiveDateTime::parse_from_str(&value, "%m/%d/%y %H:%M")?
                    .and_local_timezone(chrono_tz::US::Mountain)
                    .unwrap()
                    .fixed_offset(),
            )
        })?;
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
        formatter.write_str("an ISO 8601 string, or one in the format m/d/y \"h:m\"")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match DateTime::try_from(value.to_string()) {
            Ok(dt) => Ok(dt),
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
