use crate::deflate::ZipInflationError;
use zip::result::ZipError;

/// An EU4 Error
#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct Eu4Error(#[from] Box<Eu4ErrorKind>);

impl Eu4Error {
    pub(crate) fn new(kind: Eu4ErrorKind) -> Eu4Error {
        Eu4Error(Box::new(kind))
    }

    /// Return the specific type of error
    pub fn kind(&self) -> &Eu4ErrorKind {
        &self.0
    }
}

impl From<Eu4ErrorKind> for Eu4Error {
    fn from(err: Eu4ErrorKind) -> Self {
        Eu4Error::new(err)
    }
}

/// Specific type of error
#[derive(thiserror::Error, Debug)]
pub enum Eu4ErrorKind {
    #[error("unable to parse as zip: {0}")]
    ZipArchive(#[from] ZipError),

    #[error("unable to inflate zip entry: {msg}")]
    ZipBadData { msg: String },

    #[error("early eof, only able to write {written} bytes")]
    ZipEarlyEof { written: usize },

    #[error("unknown header found in zip entry. Must be EU4txt or EU4bin")]
    ZipHeader,

    #[error("unknown header found in file. Must be EU4txt, EU4bin, or a zip file.")]
    UnknownHeader,

    #[error("unable to parse due to: {0}")]
    Parse(#[source] jomini::Error),

    #[error("unable to deserialize due to: {0}")]
    Deserialize(#[source] jomini::Error),

    #[error("error while writing output: {0}")]
    Writer(#[source] jomini::Error),

    #[error("unknown binary token encountered: {token_id:#x}")]
    UnknownToken { token_id: u16 },

    #[error("country tags must be 3 letters in length")]
    CountryTagIncorrectSize,

    #[error("country tags must contain only ascii letters")]
    CountryTagInvalidCharacters,

    #[error("expected the binary integer: {0} to be parsed as a date")]
    InvalidDate(i32),
}

impl From<ZipInflationError> for Eu4ErrorKind {
    fn from(x: ZipInflationError) -> Self {
        match x {
            ZipInflationError::BadData { msg } => Eu4ErrorKind::ZipBadData { msg },
            ZipInflationError::EarlyEof { written } => Eu4ErrorKind::ZipEarlyEof { written },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_of_error_test() {
        assert_eq!(std::mem::size_of::<Eu4Error>(), 8);
    }
}
