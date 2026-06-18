use crate::option::PinentryOption;

#[derive(Debug, PartialEq, Eq)]
pub enum ClientRequest {
    Bye,
    Reset,
    End,
    Help,
    Quit,
    Cancel,
    Nop,
    Auth,
    Option(PinentryOption),

    SetTimeout(u32),
    SetDescription(String),
    SetPrompt(String),
    SetTitle(String),
    SetOk(String),
    SetCancel(String),
    SetNotOk(String),
    SetError(String),
    SetRepeat(String),
    SetRepeatError(String),
    SetQualityBar(String),
    SetQualityBarTooltip(String),
    SetGenpin(String),
    SetGenpinTooltip(String),
    SetKeyInfo(String),

    GetPin,
    Confirm { one_button: bool },
    Message,
    GetInfo(String),
    ClearPassphrase(String),
}

impl ClientRequest {
    pub fn parse(line: &str) -> Option<ClientRequest> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        let (keyword, arg) = match line.split_once(char::is_whitespace) {
            Some((keyword, arg)) => (keyword, arg.trim_start()),
            None => (line, ""),
        };
        let arg = percent_decode(arg);

        let request = match keyword {
            "BYE" => ClientRequest::Bye,
            "RESET" => ClientRequest::Reset,
            "END" => ClientRequest::End,
            "HELP" => ClientRequest::Help,
            "QUIT" => ClientRequest::Quit,
            "CANCEL" => ClientRequest::Cancel,
            "NOP" => ClientRequest::Nop,
            "AUTH" => ClientRequest::Auth,
            "OPTION" => ClientRequest::Option(PinentryOption::parse(&arg)),

            "SETTIMEOUT" => ClientRequest::SetTimeout(arg.parse().unwrap_or(0)),
            "SETDESC" => ClientRequest::SetDescription(arg),
            "SETPROMPT" => ClientRequest::SetPrompt(arg),
            "SETTITLE" => ClientRequest::SetTitle(arg),
            "SETOK" => ClientRequest::SetOk(arg),
            "SETCANCEL" => ClientRequest::SetCancel(arg),
            "SETNOTOK" => ClientRequest::SetNotOk(arg),
            "SETERROR" => ClientRequest::SetError(arg),
            "SETREPEAT" => ClientRequest::SetRepeat(arg),
            "SETREPEATERROR" => ClientRequest::SetRepeatError(arg),
            "SETQUALITYBAR" => ClientRequest::SetQualityBar(arg),
            "SETQUALITYBAR_TT" => ClientRequest::SetQualityBarTooltip(arg),
            "SETGENPIN" => ClientRequest::SetGenpin(arg),
            "SETGENPIN_TT" => ClientRequest::SetGenpinTooltip(arg),
            "SETKEYINFO" => ClientRequest::SetKeyInfo(arg),

            "GETPIN" => ClientRequest::GetPin,
            "CONFIRM" => ClientRequest::Confirm {
                one_button: arg.split_whitespace().any(|a| a == "--one-button"),
            },
            "MESSAGE" => ClientRequest::Message,
            "GETINFO" => ClientRequest::GetInfo(arg),
            "CLEARPASSPHRASE" => ClientRequest::ClearPassphrase(arg),

            _ => return None,
        };

        Some(request)
    }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_value(bytes[i + 1]), hex_value(bytes[i + 2])) {
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_bare_command() {
        assert_eq!(ClientRequest::parse("GETPIN"), Some(ClientRequest::GetPin));
        assert_eq!(ClientRequest::parse("BYE"), Some(ClientRequest::Bye));
    }

    #[test]
    fn keeps_the_argument_verbatim_including_inner_spaces() {
        assert_eq!(
            ClientRequest::parse("SETDESC Enter the  passphrase for key 0xABCD"),
            Some(ClientRequest::SetDescription(
                "Enter the  passphrase for key 0xABCD".into()
            ))
        );
    }

    #[test]
    fn parses_settimeout_and_defaults_invalid_values_to_zero() {
        assert_eq!(
            ClientRequest::parse("SETTIMEOUT 30"),
            Some(ClientRequest::SetTimeout(30))
        );
        assert_eq!(
            ClientRequest::parse("SETTIMEOUT"),
            Some(ClientRequest::SetTimeout(0))
        );
        assert_eq!(
            ClientRequest::parse("SETTIMEOUT garbage"),
            Some(ClientRequest::SetTimeout(0))
        );
    }

    #[test]
    fn tooltip_commands_use_the_underscore_names() {
        assert_eq!(
            ClientRequest::parse("SETQUALITYBAR_TT Strength"),
            Some(ClientRequest::SetQualityBarTooltip("Strength".into()))
        );
        assert_eq!(
            ClientRequest::parse("SETGENPIN_TT Generate"),
            Some(ClientRequest::SetGenpinTooltip("Generate".into()))
        );
    }

    #[test]
    fn repeat_and_quality_bar_labels_are_optional() {
        assert_eq!(
            ClientRequest::parse("SETREPEAT"),
            Some(ClientRequest::SetRepeat(String::new()))
        );
        assert_eq!(
            ClientRequest::parse("SETREPEAT Repeat:"),
            Some(ClientRequest::SetRepeat("Repeat:".into()))
        );
    }

    #[test]
    fn confirm_detects_the_one_button_flag() {
        assert_eq!(
            ClientRequest::parse("CONFIRM"),
            Some(ClientRequest::Confirm { one_button: false })
        );
        assert_eq!(
            ClientRequest::parse("CONFIRM --one-button"),
            Some(ClientRequest::Confirm { one_button: true })
        );
    }

    #[test]
    fn parses_option_lines() {
        assert_eq!(
            ClientRequest::parse("OPTION ttyname=/dev/pts/3"),
            Some(ClientRequest::Option(PinentryOption::TtyName(
                "/dev/pts/3".into()
            )))
        );
    }

    #[test]
    fn parses_getinfo_subcommand() {
        assert_eq!(
            ClientRequest::parse("GETINFO version"),
            Some(ClientRequest::GetInfo("version".into()))
        );
    }

    #[test]
    fn percent_decodes_escapes_in_arguments() {
        assert_eq!(
            ClientRequest::parse("SETDESC Unlock the card%0A%0ANumber: 1 2 3%25"),
            Some(ClientRequest::SetDescription(
                "Unlock the card\n\nNumber: 1 2 3%".into()
            ))
        );
    }

    #[test]
    fn leaves_a_lone_percent_untouched() {
        assert_eq!(
            ClientRequest::parse("SETPROMPT 50% done"),
            Some(ClientRequest::SetPrompt("50% done".into()))
        );
    }

    #[test]
    fn returns_none_for_unknown_or_empty_lines() {
        assert_eq!(ClientRequest::parse("FROBNICATE now"), None);
        assert_eq!(ClientRequest::parse(""), None);
        assert_eq!(ClientRequest::parse("   "), None);
    }
}
