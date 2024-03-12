//! Set request.
use crate::{error::RedisError, protocol::request::Array};

// Set request
#[derive(Eq, PartialEq, Debug)]
pub struct Set {
    pub key: String,
    pub value: String,
    pub expiration_timeout_ms: Option<u64>,
}

impl TryFrom<Array> for Set {
    type Error = RedisError;

    fn try_from(array: Array) -> Result<Self, Self::Error> {
        if array.args_count() < 3 {
            return Err(RedisError::DeserializationError {
                raw_redis_message: array.serialize(),
                details: "SET command requires at least 2 arguments".to_owned(),
            });
        }
        // Skip array.args[0]: it is a command name.
        let mut args = array.args.into_iter().skip(1);
        let key = args.next().unwrap();
        let value = args.next().unwrap();
        let mut expiration_timeout_ms = None;
        while let Some(arg) = args.next() {
            if arg.to_ascii_uppercase() == "EX" {
                if expiration_timeout_ms.is_some() {
                    return Err(RedisError::DeserializationError {
                        raw_redis_message: "N/A".to_string(),
                        details: "Expiration time cannot be set twice".to_owned(),
                    });
                }
                expiration_timeout_ms = Some(
                    args.next()
                        .ok_or(RedisError::DeserializationError {
                            raw_redis_message: "N/A".to_string(),
                            details: "EX argument is missing".to_owned(),
                        })?
                        .parse::<u64>()
                        .map_err(|err| RedisError::DeserializationError {
                            raw_redis_message: "N/A".to_string(),
                            details: err.to_string(),
                        })?
                        * 1000,
                );
            }
            if arg.to_ascii_uppercase() == "PX" {
                if expiration_timeout_ms.is_some() {
                    return Err(RedisError::DeserializationError {
                        raw_redis_message: "N/A".to_string(),
                        details: "Expiration time cannot be set twice".to_owned(),
                    });
                }
                expiration_timeout_ms = Some(
                    args.next()
                        .ok_or(RedisError::DeserializationError {
                            raw_redis_message: "N/A".to_string(),
                            details: "PX argument is missing".to_owned(),
                        })?
                        .parse::<u64>()
                        .map_err(|err| RedisError::DeserializationError {
                            raw_redis_message: "N/A".to_string(),
                            details: err.to_string(),
                        })?,
                );
            }
        }

        Ok(Set {
            key,
            value,
            expiration_timeout_ms,
        })
    }
}

#[cfg(test)]
mod try_from_array {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn simple() {
        // Set a b
        let array = Array::new(vec!["set".to_string(), "a".to_string(), "b".to_string()]);
        assert_eq!(
            Set::try_from(array).unwrap(),
            Set {
                key: "a".to_string(),
                value: "b".to_string(),
                expiration_timeout_ms: None
            }
        );
    }

    #[test]
    fn px_arg() {
        // Set expiration timeout in milliseconds
        let array = Array::new(vec![
            "set".to_string(),
            "a".to_string(),
            "b".to_string(),
            "px".to_string(),
            "1000".to_string(),
        ]);
        assert_eq!(
            Set::try_from(array).unwrap(),
            Set {
                key: "a".to_string(),
                value: "b".to_string(),
                expiration_timeout_ms: Some(1000),
            }
        );
    }

    #[test]
    fn ex_arg() {
        // Set expiration timeout in seconds
        let array = Array::new(vec![
            "set".to_string(),
            "a".to_string(),
            "b".to_string(),
            "ex".to_string(),
            "2".to_string(),
        ]);
        assert_eq!(
            Set::try_from(array).unwrap(),
            Set {
                key: "a".to_string(),
                value: "b".to_string(),
                expiration_timeout_ms: Some(2000),
            }
        );
    }

    #[test]
    fn variadic_case() {
        // Case of key and value should preserve.
        // Case of the command name and other args can be ignored.
        let array = Array::new(vec![
            "SeT".to_string(),
            "A".to_string(),
            "b".to_string(),
            "Ex".to_string(),
            "3".to_string(),
        ]);
        assert_eq!(
            Set::try_from(array).unwrap(),
            Set {
                key: "A".to_string(),
                value: "b".to_string(),
                expiration_timeout_ms: Some(3000)
            }
        );
    }

    #[test]
    fn neg_value_is_missing() {
        // Value is missing
        let array = Array::new(vec!["SET".to_string(), "a".to_string()]);
        assert_matches!(
            Set::try_from(array),
            Err(RedisError::DeserializationError { .. })
        );
    }

    #[test]
    fn neg_px_is_set_twice() {
        // PX is set twice
        let array = Array::new(vec![
            "SET".to_string(),
            "a".to_string(),
            "b".to_string(),
            "PX".to_string(),
            "1000".to_string(),
            "PX".to_string(),
            "1000".to_string(),
        ]);
        assert_matches!(
            Set::try_from(array),
            Err(RedisError::DeserializationError { .. })
        );
    }

    #[test]
    fn neg_ex_is_set_twice() {
        // PX is set twice
        let array = Array::new(vec![
            "SET".to_string(),
            "a".to_string(),
            "b".to_string(),
            "EX".to_string(),
            "1000".to_string(),
            "EX".to_string(),
            "1000".to_string(),
        ]);
        assert_matches!(
            Set::try_from(array),
            Err(RedisError::DeserializationError { .. })
        );
    }

    #[test]
    fn neg_ex_and_px() {
        // PX is set twice
        let array = Array::new(vec![
            "SET".to_string(),
            "a".to_string(),
            "b".to_string(),
            "EX".to_string(),
            "1000".to_string(),
            "PX".to_string(),
            "1000".to_string(),
        ]);
        assert_matches!(
            Set::try_from(array),
            Err(RedisError::DeserializationError { .. })
        );
    }
}
