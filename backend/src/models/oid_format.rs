use bson::oid::ObjectId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod opt_oid_as_hex {
    use super::*;

    pub fn serialize<S>(val: &Option<ObjectId>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match val {
            Some(oid) => {
                if serializer.is_human_readable() {
                    serializer.serialize_str(&oid.to_hex())
                } else {
                    Serialize::serialize(oid, serializer)
                }
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ObjectId>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OptObjectIdVisitor;

        impl<'de> serde::de::Visitor<'de> for OptObjectIdVisitor {
            type Value = Option<ObjectId>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a hex string, an ObjectId, or null")
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
                let val = bson::Bson::deserialize(deserializer)?;
                match val {
                    bson::Bson::ObjectId(oid) => Ok(Some(oid)),
                    bson::Bson::String(s) => ObjectId::parse_str(&s)
                        .map(Some)
                        .map_err(serde::de::Error::custom),
                    bson::Bson::Document(doc) => {
                        if let Some(bson::Bson::String(s)) = doc.get("$oid") {
                            ObjectId::parse_str(s)
                                .map(Some)
                                .map_err(serde::de::Error::custom)
                        } else {
                            Err(serde::de::Error::custom("Document $oid invalide"))
                        }
                    }
                    bson::Bson::Null => Ok(None),
                    other => Err(serde::de::Error::custom(format!(
                        "Type BSON inattendu pour ObjectId: {:?}",
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

        deserializer.deserialize_option(OptObjectIdVisitor)
    }
}

pub mod oid_as_hex {
    use super::*;

    pub fn serialize<S>(val: &ObjectId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&val.to_hex())
        } else {
            Serialize::serialize(val, serializer)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ObjectId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = bson::Bson::deserialize(deserializer)?;
        match val {
            bson::Bson::ObjectId(oid) => Ok(oid),
            bson::Bson::String(s) => ObjectId::parse_str(&s).map_err(serde::de::Error::custom),
            bson::Bson::Document(doc) => {
                if let Some(bson::Bson::String(s)) = doc.get("$oid") {
                    ObjectId::parse_str(s).map_err(serde::de::Error::custom)
                } else {
                    Err(serde::de::Error::custom("Document $oid invalide"))
                }
            }
            other => Err(serde::de::Error::custom(format!(
                "Type BSON inattendu pour ObjectId: {:?}",
                other
            ))),
        }
    }
}
