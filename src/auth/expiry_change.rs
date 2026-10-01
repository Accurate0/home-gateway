use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ExpiryChange {
    #[default]
    Keep,
    Clear,
    Set(DateTime<Utc>),
}

impl ExpiryChange {
    pub fn is_keep(&self) -> bool {
        matches!(self, Self::Keep)
    }
}

impl<'de> Deserialize<'de> for ExpiryChange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match Option::<DateTime<Utc>>::deserialize(deserializer)? {
            Some(at) => Self::Set(at),
            None => Self::Clear,
        })
    }
}

impl Serialize for ExpiryChange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Set(at) => at.serialize(serializer),
            Self::Keep | Self::Clear => serializer.serialize_none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::api_types::UpdateKeyPayload;

    #[test]
    fn an_absent_expiry_is_kept_a_null_one_clears_and_a_timestamp_sets() {
        let absent: UpdateKeyPayload = serde_json::from_str("{}").unwrap();
        assert_eq!(absent.expires_at, ExpiryChange::Keep);

        let cleared: UpdateKeyPayload = serde_json::from_str(r#"{"expires_at":null}"#).unwrap();
        assert_eq!(cleared.expires_at, ExpiryChange::Clear);

        let set: UpdateKeyPayload =
            serde_json::from_str(r#"{"expires_at":"2030-01-01T00:00:00Z"}"#).unwrap();
        assert!(matches!(set.expires_at, ExpiryChange::Set(_)));
    }

    #[test]
    fn a_kept_expiry_is_left_out_of_the_payload_and_a_cleared_one_is_null() {
        let absent: UpdateKeyPayload = serde_json::from_str("{}").unwrap();
        assert_eq!(serde_json::to_string(&absent).unwrap(), "{}");

        let cleared: UpdateKeyPayload = serde_json::from_str(r#"{"expires_at":null}"#).unwrap();
        assert_eq!(
            serde_json::to_string(&cleared).unwrap(),
            r#"{"expires_at":null}"#
        );
    }
}
