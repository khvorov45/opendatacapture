/// Utilities for json handling
use crate::{Error, Result};

/// Convert the given value into a string that can go into an insert statement
pub fn insert_format(value: &serde_json::Value) -> Result<String> {
    use serde_json::Value;
    match value {
        Value::String(v) => Ok(format!("\'{}\'", v)),
        Value::Number(n) => Ok(format!("\'{}\'", n)),
        other => Err(Error::InsertFormatUnimplemented(other.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_insert() {
        assert_eq!(
            insert_format(
                &serde_json::from_str("\"email@example.com\"").unwrap()
            )
            .unwrap(),
            "'email@example.com'"
        );
        assert_eq!(
            insert_format(&serde_json::from_str("1").unwrap()).unwrap(),
            "'1'"
        );
        let unsupported_val = serde_json::from_str("{\"id\": 1}").unwrap();
        assert!(matches!(
            insert_format(&unsupported_val)
                .unwrap_err(),
            Error::InsertFormatUnimplemented(v) if v == unsupported_val
        ));
    }
}
