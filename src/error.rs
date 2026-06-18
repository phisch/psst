use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AssuanError {
    UnknownIpcCommand,
    NotImplemented,
    Canceled,
    NotConfirmed,
}

impl AssuanError {
    pub fn code(self) -> u32 {
        match self {
            AssuanError::UnknownIpcCommand => 275,
            AssuanError::NotImplemented => 69,
            AssuanError::Canceled => 99,
            AssuanError::NotConfirmed => 114,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            AssuanError::UnknownIpcCommand => "Unknown IPC command",
            AssuanError::NotImplemented => "Not implemented",
            AssuanError::Canceled => "Operation cancelled",
            AssuanError::NotConfirmed => "Not confirmed",
        }
    }

    pub fn source(self) -> &'static str {
        "<User defined source 1>"
    }

    pub fn value(self) -> u32 {
        const SOURCE_USER_1: u32 = 32;
        const SOURCE_SHIFT: u32 = 24;
        (SOURCE_USER_1 << SOURCE_SHIFT) | self.code()
    }
}

impl Display for AssuanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ERR {} {} {}",
            self.value(),
            self.description(),
            self.source()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_source_and_code_into_a_single_value() {
        assert_eq!(AssuanError::UnknownIpcCommand.value(), 536870912 + 275);
        assert_eq!(AssuanError::NotImplemented.value(), 536870912 + 69);
        assert_eq!(AssuanError::Canceled.value(), 536870912 + 99);
        assert_eq!(AssuanError::NotConfirmed.value(), 536870912 + 114);
    }

    #[test]
    fn formats_a_full_err_line() {
        assert_eq!(
            AssuanError::UnknownIpcCommand.to_string(),
            "ERR 536871187 Unknown IPC command <User defined source 1>"
        );
        assert_eq!(
            AssuanError::Canceled.to_string(),
            "ERR 536871011 Operation cancelled <User defined source 1>"
        );
    }
}
