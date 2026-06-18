use std::fmt::{Display, Formatter};

use crate::error::AssuanError;

#[derive(Debug, PartialEq, Eq)]
pub enum Response {
    Ok(Option<String>),
    Error(AssuanError),
    Data(String),
    Status(String),
}

fn percent_encode(data: &str) -> String {
    let mut encoded = String::with_capacity(data.len());
    for byte in data.bytes() {
        match byte {
            b'%' => encoded.push_str("%25"),
            b'\r' => encoded.push_str("%0D"),
            b'\n' => encoded.push_str("%0A"),
            _ => encoded.push(byte as char),
        }
    }
    encoded
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Response::Ok(Some(text)) => write!(f, "OK {text}"),
            Response::Ok(None) => write!(f, "OK"),
            Response::Error(error) => write!(f, "{error}"),
            Response::Data(data) => write!(f, "D {}", percent_encode(data)),
            Response::Status(status) => write!(f, "S {status}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_each_response_kind() {
        assert_eq!(Response::Ok(None).to_string(), "OK");
        assert_eq!(Response::Ok(Some("foo".into())).to_string(), "OK foo");
        assert_eq!(Response::Data("foo".into()).to_string(), "D foo");
        assert_eq!(
            Response::Status("PIN_REPEATED".into()).to_string(),
            "S PIN_REPEATED"
        );
        assert_eq!(
            Response::Error(AssuanError::UnknownIpcCommand).to_string(),
            "ERR 536871187 Unknown IPC command <User defined source 1>"
        );
    }

    #[test]
    fn percent_encodes_reserved_characters_in_data() {
        assert_eq!(
            Response::Data("100% pa\r\nss".into()).to_string(),
            "D 100%25 pa%0D%0Ass"
        );
    }
}
