use std::fmt;
use std::io::Error as IoError;
use zip::result::ZipError;

/// An EU4 Error
#[derive(Debug)]
pub struct Eu4Error(Box<Eu4ErrorKind>);

impl Eu4Error {
    pub(crate) fn new(kind: Eu4ErrorKind) -> Eu4Error {
        Eu4Error(Box::new(kind))
    }

    /// Return the specific type of error
    pub fn kind(&self) -> &Eu4ErrorKind {
        &self.0
    }
}

/// Specific type of error
#[derive(Debug)]
pub enum Eu4ErrorKind {
    ZipCentralDirectory(ZipError),
    ZipMissingEntry(&'static str, ZipError),
    ZipExtraction(&'static str, IoError),
    ZipSize(&'static str),
    IoErr(IoError),
    UnknownHeader,
    UnknownToken {
        token_id: u16,
    },
    Deserialize {
        part: Option<String>,
        err: jomini::Error,
    },
}

impl fmt::Display for Eu4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind() {
            Eu4ErrorKind::ZipCentralDirectory(_) => {
                write!(f, "unable to read zip central directory")
            }
            Eu4ErrorKind::ZipMissingEntry(s, _) => write!(f, "unable to locate {} in zip", s),
            Eu4ErrorKind::ZipExtraction(s, _) => write!(f, "unable to extract {} in zip", s),
            Eu4ErrorKind::ZipSize(s) => write!(f, "{} in zip is too large", s),
            Eu4ErrorKind::IoErr(_) => write!(f, "io error"),
            Eu4ErrorKind::UnknownHeader => write!(f, "unknown header encountered in zip"),
            Eu4ErrorKind::UnknownToken { token_id } => {
                write!(f, "unknown binary token encountered (id: {})", token_id)
            }
            Eu4ErrorKind::Deserialize { ref part, ref err } => match part {
                Some(p) => write!(f, "error deserializing: {}: {}", p, err),
                None => err.fmt(f),
            },
        }
    }
}

impl std::error::Error for Eu4Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.kind() {
            Eu4ErrorKind::ZipCentralDirectory(e) => Some(e),
            Eu4ErrorKind::ZipMissingEntry(_, e) => Some(e),
            Eu4ErrorKind::ZipExtraction(_, e) => Some(e),
            Eu4ErrorKind::IoErr(e) => Some(e),
            Eu4ErrorKind::Deserialize { ref err, .. } => Some(err),
            _ => None,
        }
    }
}

impl From<jomini::Error> for Eu4Error {
    fn from(err: jomini::Error) -> Self {
        Eu4Error::new(Eu4ErrorKind::Deserialize { part: None, err })
    }
}

impl From<IoError> for Eu4Error {
    fn from(err: IoError) -> Self {
        Eu4Error::new(Eu4ErrorKind::IoErr(err))
    }
}

impl From<Eu4ErrorKind> for Eu4Error {
    fn from(err: Eu4ErrorKind) -> Self {
        Eu4Error::new(err)
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
