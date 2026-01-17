/// Custom serialization/deserialization for i64 as a string.
pub(crate) mod i64_as_string {
    use serde::{self, de, Deserialize, Deserializer, Serializer};

    /// Serializes an `i64` as a string.
    pub fn serialize<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    /// Deserializes a string into an `i64`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }

    /// Optional `i64` as string.
    pub(crate) mod optional {
        use serde::{self, de, Deserialize, Deserializer, Serializer};

        /// Serializes an `Option<i64>` as a string or `None`.
        pub fn serialize<S>(value: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match value {
                Some(v) => serializer.serialize_str(&v.to_string()),
                None => serializer.serialize_none(),
            }
        }

        /// Deserializes a string into an `Option<i64>`.
        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
        where
            D: Deserializer<'de>,
        {
            Option::<String>::deserialize(deserializer)?
                .map(|s| s.parse::<i64>().map_err(de::Error::custom))
                .transpose()
        }
    }
}

/// Custom serialization/deserialization for the request key.
pub(crate) mod key_as_string {
    use serde::{self, Deserialize, Deserializer, Serializer};

    /// Serializes a `usize` key as a string.
    pub fn serialize<S>(key: &usize, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&key.to_string())
    }

    /// Deserializes a string key into a `usize`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<usize, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<usize>().map_err(|e| {
            let error_message = format!(
                "Failed to parse key '{s}' as a number: {e}. This crate uses index-based keys for batch requests; custom string keys are not supported."
            );
            serde::de::Error::custom(error_message)
        })
    }
}

/// Deserializes a string into an `i64`.
pub fn deserialize_string_to_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    String::deserialize(deserializer)?
        .parse()
        .map_err(serde::de::Error::custom)
}

/// Deserializes an optional string into an `Option<i64>`.
pub fn deserialize_optional_string_to_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    Option::<String>::deserialize(deserializer)?
        .map(|s| s.parse::<i64>().map_err(serde::de::Error::custom))
        .transpose()
}

/// Custom serialization/deserialization for mime::Mime as a string.
pub(crate) mod mime_as_string {
    use mime::Mime;
    use serde::{self, de, Deserialize, Deserializer, Serializer};

    /// Serializes a `Mime` as a string.
    pub fn serialize<S>(value: &Mime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(value.to_string().as_str())
    }

    /// Deserializes a string into a `Mime`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Mime, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }

    /// Optional `Mime` as string.
    pub(crate) mod optional {
        use mime::Mime;
        use serde::{self, de, Deserialize, Deserializer, Serializer};

        /// Serializes an `Option<Mime>` as a string or `None`.
        pub fn serialize<S>(value: &Option<Mime>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match value {
                Some(v) => serializer.serialize_str(v.to_string().as_str()),
                None => serializer.serialize_none(),
            }
        }

        /// Deserializes a string into an `Option<Mime>`.
        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Mime>, D::Error>
        where
            D: Deserializer<'de>,
        {
            Option::<String>::deserialize(deserializer)?
                .map(|s| s.parse::<Mime>().map_err(de::Error::custom))
                .transpose()
        }
    }
}
