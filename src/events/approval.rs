use serde::{Deserialize, Deserializer, Serialize, Serializer};

mod approval_value {
    use super::*;

    pub fn serialize<S>(value: &i8, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Reproduce Gerrit's "+N" convention for positive values.
        if *value > 0 {
            serializer.serialize_str(&format!("+{value}"))
        } else {
            serializer.serialize_str(&value.to_string())
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i8, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.trim_start_matches('+')
            .parse::<i8>()
            .map_err(serde::de::Error::custom)
    }
}

mod approval_value_opt {
    use super::*;

    pub fn serialize<S>(value: &Option<i8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(v) if *v > 0 => serializer.serialize_some(&format!("+{v}")),
            Some(v) => serializer.serialize_some(&v.to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<i8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        opt.map(|s| {
            s.trim_start_matches('+')
                .parse::<i8>()
                .map_err(serde::de::Error::custom)
        })
        .transpose()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Approval {
    #[serde(rename = "type")]
    pub kind: String,
    pub description: Option<String>,
    #[serde(with = "approval_value")]
    pub value: i8,
    #[serde(default, with = "approval_value_opt")]
    pub old_value: Option<i8>,
    pub granted_on: Option<i64>,
    pub by: Option<Account>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub name: Option<String>,
    pub email: Option<String>,
    pub username: String,
}
