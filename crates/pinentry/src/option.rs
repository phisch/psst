#[derive(Debug, PartialEq, Eq)]
pub enum PinentryOption {
    FormattedPassphrase,
    TtyType(String),
    TtyName(String),
    LcCtype(String),
    LcMessages(String),
    DefaultOk(String),
    DefaultCancel(String),
    DefaultPrompt(String),
    AllowExternalPasswordCache,
    Other(String),
}

impl PinentryOption {
    pub fn parse(input: &str) -> PinentryOption {
        let (name, value) = match input.split_once('=') {
            Some((name, value)) => (name, value.to_string()),
            None => (input, String::new()),
        };

        match name {
            "formatted-passphrase" => PinentryOption::FormattedPassphrase,
            "ttytype" => PinentryOption::TtyType(value),
            "ttyname" => PinentryOption::TtyName(value),
            "lc-ctype" => PinentryOption::LcCtype(value),
            "lc-messages" => PinentryOption::LcMessages(value),
            "default-ok" => PinentryOption::DefaultOk(value),
            "default-cancel" => PinentryOption::DefaultCancel(value),
            "default-prompt" => PinentryOption::DefaultPrompt(value),
            "allow-external-password-cache" => PinentryOption::AllowExternalPasswordCache,
            _ => PinentryOption::Other(input.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flag_options_without_a_value() {
        assert_eq!(
            PinentryOption::parse("formatted-passphrase"),
            PinentryOption::FormattedPassphrase
        );
        assert_eq!(
            PinentryOption::parse("allow-external-password-cache"),
            PinentryOption::AllowExternalPasswordCache
        );
    }

    #[test]
    fn parses_key_value_options() {
        assert_eq!(
            PinentryOption::parse("ttyname=/dev/pts/1"),
            PinentryOption::TtyName("/dev/pts/1".into())
        );
        assert_eq!(
            PinentryOption::parse("lc-ctype=en_US.UTF-8"),
            PinentryOption::LcCtype("en_US.UTF-8".into())
        );
    }

    #[test]
    fn treats_a_missing_value_as_empty() {
        assert_eq!(
            PinentryOption::parse("ttyname"),
            PinentryOption::TtyName(String::new())
        );
        assert_eq!(
            PinentryOption::parse("ttyname="),
            PinentryOption::TtyName(String::new())
        );
    }

    #[test]
    fn keeps_unknown_options_verbatim() {
        assert_eq!(
            PinentryOption::parse("some-future-option=42"),
            PinentryOption::Other("some-future-option=42".into())
        );
    }
}
