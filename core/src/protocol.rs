use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientMessage {
    Exit,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireMessage {
    #[serde(rename = "type")]
    kind: WireMessageType,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WireMessageType {
    Exit,
}

pub fn parse_client_message(line: &str) -> Result<ClientMessage, serde_json::Error> {
    let wire: WireMessage = serde_json::from_str(line)?;
    let message = match wire.kind {
        WireMessageType::Exit => ClientMessage::Exit,
    };
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::{ClientMessage, parse_client_message};

    #[test]
    fn accepts_exact_exit_message() {
        let parsed = parse_client_message(r#"{"type":"exit"}"#).expect("exit message should parse");
        assert_eq!(parsed, ClientMessage::Exit);
    }

    #[test]
    fn rejects_plain_string_message() {
        assert!(parse_client_message(r#""exit""#).is_err());
    }

    #[test]
    fn rejects_unknown_message_type() {
        assert!(parse_client_message(r#"{"type":"ping"}"#).is_err());
    }

    #[test]
    fn rejects_unknown_fields() {
        assert!(parse_client_message(r#"{"type":"exit","extra":"value"}"#).is_err());
    }
}
