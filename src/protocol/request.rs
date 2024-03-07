//! Contains the Redis request types and their serialization and deserialization.
use crate::error::RedisError;
use core::fmt;

/// Contains Redis requests. All requests are arrays.
#[derive(Eq, PartialEq, Debug)]
pub enum Request {
    Ping,
    Echo(String),
    Set(String, String),
    Get(String),
}

impl Request {
    /// Converts a string to a request. Consumes the buffer up to the end of the first request.
    pub fn deserialize(buffer: &mut String) -> Result<Self, RedisError> {
        Request::try_from(Array::deserialize(buffer)?)
    }
}

impl TryFrom<Array> for Request {
    type Error = RedisError;

    fn try_from(array: Array) -> Result<Self, RedisError> {
        match array.args[0].to_lowercase().as_str() {
            "ping" => Ok(Request::Ping),
            "echo" => Ok(Request::Echo(array.args[1].clone())),
            "set" => Ok(Request::Set(array.args[1].clone(), array.args[2].clone())),
            "get" => Ok(Request::Get(array.args[1].clone())),
            cmd => Err(RedisError::DeserializationError {
                raw_redis_message: array.serialize(),
                details: format!("Unknown command: {}", cmd),
            }),
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
struct Array {
    size: usize,
    args: Vec<String>,
}

impl Array {
    fn new(args_count: usize, args: Vec<String>) -> Self {
        Array {
            size: args_count,
            args,
        }
    }

    fn add_element(&mut self, arg: String) {
        self.args.push(arg);
    }

    fn is_complete(&self) -> bool {
        self.args.len() == self.size
    }

    fn serialize(self) -> String {
        assert!(
            self.size > 0,
            "Array size must be greater than 0 for serialization"
        );
        let mut result = std::format!("*{}\r\n", self.size);
        self.args.into_iter().for_each(|arg| {
            result.push_str(format!("${}\r\n{}\r\n", arg.chars().count(), arg).as_str());
        });
        result
    }

    fn deserialize(buffer: &mut String) -> Result<Self, RedisError> {
        let original_message = buffer.clone();

        // Find the beginning of an array or clear the buffer and return an error
        if let Some(asterisk_position) = buffer.find('*') {
            buffer.drain(0..asterisk_position);
        } else {
            buffer.clear();
            return Err(RedisError::DeserializationError {
                raw_redis_message: original_message,
                details: "Couldn't find asterisk in a buffer".to_owned(),
            });
        }

        let mut lines = buffer.lines();

        // Find out the number of arguments
        let number_of_args = {
            lines
                .next()
                .ok_or(RedisError::DeserializationError {
                    raw_redis_message: original_message,
                    details: "First line is absent".to_owned(),
                })?
                .chars()
                .skip(1)
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<usize>()
                .map_err(|err| RedisError::DeserializationError {
                    raw_redis_message: buffer.clone(),
                    details: err.to_string(),
                })?
        };

        let mut array = Array::new(number_of_args, vec![]);

        for _i in 0..number_of_args * 2 {
            if let Some(line) = lines.next() {
                if line.chars().nth(0) == Some('$') {
                    // This is a length line
                    continue;
                }
                array.add_element(line.to_string());
            } else {
                return Err(RedisError::DeserializationError {
                    raw_redis_message: buffer.clone(),
                    details: format!("Not enough args for array. Message: '{}'", buffer),
                });
            }
        }

        // Collect reminder of buffer. It may contain more messages.
        *buffer = lines
            .map(|line| line.to_string() + "\r\n")
            .collect::<String>();

        if array.is_complete() {
            Ok(array)
        } else {
            Err(RedisError::DeserializationError {
                raw_redis_message: buffer.clone(),
                details: format!("Array is incomplete {}", array),
            })
        }
    }
}

impl fmt::Display for Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Array(size: {}, args: {:?})", self.size, self.args)
    }
}

#[cfg(test)]
mod deserialize {
    use super::*;
    const PING_RAW: &str = "*1\r\n$4\r\nping\r\n";
    const ECHO_ARG: &str = "Hello, world!";
    const ECHO_RAW: &str = "*2\r\n$4\r\necho\r\n$13\r\nHello, world!\r\n";
    use assert_matches::assert_matches;

    #[test]
    fn empty_buffer() {
        let mut buffer = "".to_string();
        assert_matches!(
            Request::deserialize(&mut buffer).unwrap_err(),
            RedisError::DeserializationError { .. }
        );
        assert!(buffer.is_empty());
    }

    #[test]
    fn junk() {
        let mut buffer = "junk".to_string();
        assert_matches!(
            Request::deserialize(&mut buffer).unwrap_err(),
            RedisError::DeserializationError { .. }
        );
        assert!(buffer.is_empty());
    }

    #[test]
    fn incomplete_request() {
        // If message is incomplete, the buffer must remain intact.
        let mut buffer = "*1\r\n$4\r\n".to_string();
        let old_buf_len = buffer.len();
        assert_matches!(
            Request::deserialize(&mut buffer).unwrap_err(),
            RedisError::DeserializationError { .. }
        );
        assert_eq!(buffer.len(), old_buf_len);
    }

    #[test]
    fn incomplete_request_with_prefix() {
        // If message is incomplete, buffer must remain intact starting from the asterisk.
        static PREFIX: &str = "\r\n$4s";
        let mut buffer = format!("{}*1\r\n$4\r\n", PREFIX);
        let old_buf_len = buffer.len();
        assert_matches!(
            Request::deserialize(&mut buffer).unwrap_err(),
            RedisError::DeserializationError { .. }
        );
        assert_eq!(buffer.len(), old_buf_len - PREFIX.len());
        assert_eq!(&buffer, "*1\r\n$4\r\n");
    }

    #[test]
    fn ping() {
        let mut buffer = PING_RAW.to_string();
        assert_eq!(Request::deserialize(&mut buffer).unwrap(), Request::Ping);
        assert!(buffer.is_empty());
    }

    #[test]
    fn ping_various_case() {
        let mut buffer = "*1\r\n$4\r\nPING\r\n".to_string();
        assert_eq!(Request::deserialize(&mut buffer).unwrap(), Request::Ping);
        let mut buffer = "*1\r\n$4\r\npiNG\r\n".to_string();
        assert_eq!(Request::deserialize(&mut buffer).unwrap(), Request::Ping);
    }

    #[test]
    fn ping_with_prefix() {
        // The deserialize function should skip unrelated characters
        let mut buffer = format!("\r\n$4s{}", PING_RAW);
        assert_eq!(Request::deserialize(&mut buffer).unwrap(), Request::Ping);
        assert!(buffer.is_empty());
    }

    #[test]
    fn ping_with_prefix_and_postfix() {
        // The deserialize function should skip unrelated characters and leave the part
        // of the message that wasn't parsed
        let mut buffer = format!("$4\r\n{}$4\r\n", PING_RAW);
        assert_eq!(Request::deserialize(&mut buffer).unwrap(), Request::Ping);
        assert_eq!(buffer, "$4\r\n");
    }

    #[test]
    fn ping_and_then_echo() {
        let mut buffer = format!("\r\n$4s{}\r\n$4s{}", PING_RAW, ECHO_RAW);
        assert_eq!(Request::deserialize(&mut buffer).unwrap(), Request::Ping);
        assert_eq!(
            Request::deserialize(&mut buffer).unwrap(),
            Request::Echo("Hello, world!".to_string())
        );
    }

    #[test]
    fn echo() {
        let mut buffer = ECHO_RAW.to_string();
        assert_eq!(
            Request::deserialize(&mut buffer).unwrap(),
            Request::Echo(ECHO_ARG.to_string())
        );
        assert!(buffer.is_empty());
    }

    #[test]
    fn set() {
        // set a b
        let mut buffer: String = "*3\r\n$3\r\nset\r\n$1\r\na\r\n$1\r\nb\r\n".to_owned();
        assert_eq!(
            Request::deserialize(&mut buffer).unwrap(),
            Request::Set("a".to_string(), "b".to_string())
        );
        assert!(buffer.is_empty());
    }

    /// Validates that case of the command arguments is preserved.
    #[test]
    fn set_case_sensitive_args() {
        // set a b
        let mut buffer: String = "*3\r\n$3\r\nset\r\n$1\r\nA\r\n$1\r\nb\r\n".to_owned();
        assert_eq!(
            Request::deserialize(&mut buffer).unwrap(),
            Request::Set("A".to_string(), "b".to_string())
        );
        assert!(buffer.is_empty());
    }

    #[test]
    fn get() {
        // get a
        let mut buffer: String = "*2\r\n$3\r\nget\r\n$1\r\na\r\n".to_owned();
        assert_eq!(
            Request::deserialize(&mut buffer).unwrap(),
            Request::Get("a".to_string())
        );
        assert!(buffer.is_empty());
    }
}
