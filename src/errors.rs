use std::fmt;
use std::io::Error as IoError;
use zip::result::ZipError;

#[derive(Debug)]
pub enum Eu4Error {
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
        match self {
            Eu4Error::ZipCentralDirectory(_) => write!(f, "unable to read zip central directory"),
            Eu4Error::ZipMissingEntry(s, _) => write!(f, "unable to locate {} in zip", s),
            Eu4Error::ZipExtraction(s, _) => write!(f, "unable to extract {} in zip", s),
            Eu4Error::ZipSize(s) => write!(f, "{} in zip is too large", s),
            Eu4Error::IoErr(_) => write!(f, "io error"),
            Eu4Error::UnknownHeader => write!(f, "unknown header encountered in zip"),
            Eu4Error::UnknownToken { token_id } => {
                write!(f, "unknown binary token encountered (id: {})", token_id)
            }
            Eu4Error::Deserialize { ref part, ref err } => match part {
                Some(p) => write!(f, "error deserializing: {}: {}", p, err),
                None => err.fmt(f),
            },
        }
    }
}

impl std::error::Error for Eu4Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Eu4Error::ZipCentralDirectory(e) => Some(e),
            Eu4Error::ZipMissingEntry(_, e) => Some(e),
            Eu4Error::ZipExtraction(_, e) => Some(e),
            Eu4Error::IoErr(e) => Some(e),
            Eu4Error::Deserialize { ref err, .. } => Some(err),
            _ => None,
        }
    }
}

impl From<jomini::Error> for Eu4Error {
    fn from(err: jomini::Error) -> Self {
        Eu4Error::Deserialize { part: None, err }
    }
}
