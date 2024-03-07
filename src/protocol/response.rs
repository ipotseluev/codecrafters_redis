//! Responses to Redis requests.

pub(crate) enum Response {
    Ok,
    Ping,
    Echo(String),
    Get(Option<String>),
}

impl Response {
    pub(crate) fn serialize(self) -> String {
        match self {
            Response::Ok => "+OK\r\n".to_string(),
            Response::Ping => "+PONG\r\n".to_string(),
            Response::Echo(arg) => format!("+{}\r\n", arg),
            Response::Get(Some(arg)) => format!("${}\r\n{}\r\n", arg.chars().count(), arg),
            Response::Get(None) => "$-1\r\n".to_string(),
        }
    }
}

#[cfg(test)]
mod serialize {
    use super::*;

    #[test]
    fn all() {
        assert_eq!(Response::Ok.serialize(), "+OK\r\n");
        assert_eq!(Response::Ping.serialize(), "+PONG\r\n");
        assert_eq!(
            Response::Echo("hello".to_string()).serialize(),
            "+hello\r\n"
        );
        assert_eq!(
            Response::Get(Some("value".to_string())).serialize(),
            "$5\r\nvalue\r\n"
        );
        assert_eq!(Response::Get(None).serialize(), "$-1\r\n");
    }
}
