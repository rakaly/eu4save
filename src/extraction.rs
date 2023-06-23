use std::fmt;

/// Describes the format of the save before decoding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    /// Plaintext
    Text,

    /// Plaintext documents within a zip file
    TextZip,

    /// Binary documents within a zip file
    BinaryZip,

    /// Binary
    Binary,
}

impl Encoding {
    pub fn as_str(&self) -> &'static str {
        match self {
            Encoding::Text => "text",
            Encoding::TextZip => "textzip",
            Encoding::BinaryZip => "binzip",
            Encoding::Binary => "binary",
        }
    }

    pub fn is_binary(&self) -> bool {
        matches!(self, Encoding::BinaryZip | Encoding::Binary)
    }

    pub fn is_text(&self) -> bool {
        matches!(self, Encoding::TextZip | Encoding::Text)
    }

    pub fn is_zip(&self) -> bool {
        matches!(self, Encoding::BinaryZip | Encoding::TextZip)
    }
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
