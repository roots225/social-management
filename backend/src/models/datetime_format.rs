use bson::Bson;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod opt_chrono_datetime {
    use super::*;

    pub fn serialize<S>(val: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match val {
            Some(dt) => {
                if serializer.is_human_readable() {
                    serializer.serialize_str(&dt.to_rfc3339())
                } else {
                    let bdt = bson::DateTime::from_chrono(*dt);
                    Serialize::serialize(&bdt, serializer)
                }
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OptDateTimeVisitor;

        impl<'de> serde::de::Visitor<'de> for OptDateTimeVisitor {
            type Value = Option<DateTime<Utc>>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an RFC 3339 date-time string, a BSON datetime, or null")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(None)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                let val = Bson::deserialize(deserializer)?;
                match val {
                    Bson::DateTime(bdt) => Ok(Some(bdt.to_chrono())),
                    Bson::String(s) => match DateTime::parse_from_rfc3339(&s) {
                        Ok(dt) => Ok(Some(dt.with_timezone(&Utc))),
                        Err(e) => Err(serde::de::Error::custom(format!(
                            "Invalid RFC3339 datetime: {}",
                            e
                        ))),
                    },
                    Bson::Int64(ms) => Ok(Some(bson::DateTime::from_millis(ms).to_chrono())),
                    Bson::Document(doc) => {
                        if let Some(date_val) = doc.get("$date") {
                            match date_val {
                                Bson::String(s) => match DateTime::parse_from_rfc3339(s) {
                                    Ok(dt) => Ok(Some(dt.with_timezone(&Utc))),
                                    Err(e) => Err(serde::de::Error::custom(format!(
                                        "Invalid RFC3339 datetime in $date: {}",
                                        e
                                    ))),
                                },
                                Bson::Int64(ms) => {
                                    Ok(Some(bson::DateTime::from_millis(*ms).to_chrono()))
                                }
                                Bson::Document(inner) => {
                                    if let Some(Bson::String(ms_str)) = inner.get("$numberLong") {
                                        if let Ok(ms) = ms_str.parse::<i64>() {
                                            Ok(Some(bson::DateTime::from_millis(ms).to_chrono()))
                                        } else {
                                            Err(serde::de::Error::custom("Invalid $numberLong"))
                                        }
                                    } else {
                                        Err(serde::de::Error::custom("Unknown $date format"))
                                    }
                                }
                                _ => Err(serde::de::Error::custom("Unexpected $date format")),
                            }
                        } else {
                            Err(serde::de::Error::custom("Expected $date field"))
                        }
                    }
                    Bson::Null => Ok(None),
                    other => Err(serde::de::Error::custom(format!(
                        "Unexpected BSON type for DateTime: {:?}",
                        other
                    ))),
                }
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(None)
            }
        }

        deserializer.deserialize_option(OptDateTimeVisitor)
    }
}

pub mod chrono_datetime {
    use super::*;

    pub fn serialize<S>(val: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&val.to_rfc3339())
        } else {
            let bdt = bson::DateTime::from_chrono(*val);
            Serialize::serialize(&bdt, serializer)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = Bson::deserialize(deserializer)?;
        match val {
            Bson::DateTime(bdt) => Ok(bdt.to_chrono()),
            Bson::String(s) => match DateTime::parse_from_rfc3339(&s) {
                Ok(dt) => Ok(dt.with_timezone(&Utc)),
                Err(e) => Err(serde::de::Error::custom(format!(
                    "Invalid RFC3339 datetime: {}",
                    e
                ))),
            },
            Bson::Int64(ms) => Ok(bson::DateTime::from_millis(ms).to_chrono()),
            Bson::Document(doc) => {
                if let Some(date_val) = doc.get("$date") {
                    match date_val {
                        Bson::String(s) => match DateTime::parse_from_rfc3339(s) {
                            Ok(dt) => Ok(dt.with_timezone(&Utc)),
                            Err(e) => Err(serde::de::Error::custom(format!(
                                "Invalid RFC3339 datetime in $date: {}",
                                e
                            ))),
                        },
                        Bson::Int64(ms) => Ok(bson::DateTime::from_millis(*ms).to_chrono()),
                        Bson::Document(inner) => {
                            if let Some(Bson::String(ms_str)) = inner.get("$numberLong") {
                                if let Ok(ms) = ms_str.parse::<i64>() {
                                    Ok(bson::DateTime::from_millis(ms).to_chrono())
                                } else {
                                    Err(serde::de::Error::custom("Invalid $numberLong"))
                                }
                            } else {
                                Err(serde::de::Error::custom("Unknown $date format"))
                            }
                        }
                        _ => Err(serde::de::Error::custom("Unexpected $date format")),
                    }
                } else {
                    Err(serde::de::Error::custom("Expected $date field"))
                }
            }
            other => Err(serde::de::Error::custom(format!(
                "Unexpected BSON type for DateTime: {:?}",
                other
            ))),
        }
    }
}
