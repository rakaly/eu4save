//! Parsing and deserializing EU4 save files
use crate::{flavor::Eu4Flavor, models::Eu4Save, Encoding, Eu4Error, Eu4ErrorKind, Eu4Melter};
use jomini::{
    binary::{FailedResolveStrategy, TokenResolver},
    text::ObjectReader,
    BinaryDeserializer, BinaryTape, TextDeserializer, TextTape, Windows1252Encoding,
};
use serde::Deserialize;
use std::{fmt::Display, io::Cursor};
use zip::result::ZipError;

const TXT_HEADER: &[u8] = b"EU4txt";
const BIN_HEADER: &[u8] = b"EU4bin";

fn is_text(data: &[u8]) -> Option<&[u8]> {
    let sentry = TXT_HEADER;
    if data.get(..sentry.len()).map_or(false, |x| x == sentry) {
        Some(&data[sentry.len()..])
    } else {
        None
    }
}

fn is_bin(data: &[u8]) -> Option<&[u8]> {
    let sentry = BIN_HEADER;
    if data.get(..sentry.len()).map_or(false, |x| x == sentry) {
        Some(&data[sentry.len()..])
    } else {
        None
    }
}

#[derive(Debug)]
struct Eu4ZipFilesIter {
    meta_index: Option<VerifiedIndex>,
    gamestate_index: Option<VerifiedIndex>,
    ai_index: Option<VerifiedIndex>,
}

impl Iterator for Eu4ZipFilesIter {
    type Item = VerifiedIndex;

    fn next(&mut self) -> Option<Self::Item> {
        self.meta_index
            .take()
            .or_else(|| self.gamestate_index.take())
            .or_else(|| self.ai_index.take())
    }
}

#[derive(Debug, Clone)]
struct Eu4Zip<'a> {
    archive: Eu4ZipFiles<'a>,
    is_text: bool,
    inflated_size: usize,
}

impl<'a> Eu4Zip<'a> {
    fn read_to_end(&self, zip_sink: &'a mut Vec<u8>) -> Result<(), Eu4Error> {
        for index in self.archive.files() {
            let file = self.archive.retrieve_file(index);
            file.read_to_end(zip_sink)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct VerifiedIndex {
    data_start: usize,
    data_end: usize,
    name: Eu4FileEntryName,
    size: usize,
}

impl VerifiedIndex {
    fn is_text(&self, data: &[u8]) -> Result<bool, Eu4Error> {
        let mut header = [0; TXT_HEADER.len()];
        let raw = &data[self.data_start..self.data_end];
        crate::deflate::inflate_exact(raw, &mut header).map_err(Eu4ErrorKind::from)?;

        Ok(is_text(&header).is_some())
    }
}

#[derive(Debug, Clone)]
struct Eu4ZipFiles<'a> {
    archive: &'a [u8],
    meta_index: Option<VerifiedIndex>,
    gamestate_index: Option<VerifiedIndex>,
    ai_index: Option<VerifiedIndex>,
}

impl<'a> Eu4ZipFiles<'a> {
    pub fn new(archive: &mut zip::ZipArchive<Cursor<&'a [u8]>>, data: &'a [u8]) -> Self {
        let mut meta_index = None;
        let mut gamestate_index = None;
        let mut ai_index = None;

        for index in 0..archive.len() {
            if let Ok(file) = archive.by_index_raw(index) {
                let size = file.size() as usize;
                let data_start = file.data_start() as usize;
                let data_end = data_start + file.compressed_size() as usize;
                let index = Eu4ZipFiles::strong_name(file.name()).map(|name| VerifiedIndex {
                    name,
                    data_start,
                    data_end,
                    size,
                });

                match index {
                    Some(x) if x.name == Eu4FileEntryName::Meta => {
                        meta_index = Some(x);
                    }
                    Some(x) if x.name == Eu4FileEntryName::Gamestate => {
                        gamestate_index = Some(x);
                    }
                    Some(x) if x.name == Eu4FileEntryName::Ai => {
                        ai_index = Some(x);
                    }
                    _ => {}
                }
            }
        }

        Self {
            archive: data,
            meta_index,
            gamestate_index,
            ai_index,
        }
    }

    fn strong_name(s: &str) -> Option<Eu4FileEntryName> {
        match s {
            "meta" => Some(Eu4FileEntryName::Meta),
            "gamestate" => Some(Eu4FileEntryName::Gamestate),
            "ai" => Some(Eu4FileEntryName::Ai),
            _ => None,
        }
    }

    pub fn retrieve_file(&self, index: VerifiedIndex) -> Eu4ZipFile {
        let raw = &self.archive[index.data_start..index.data_end];
        Eu4ZipFile {
            raw,
            size: index.size,
        }
    }

    fn files(&self) -> Eu4ZipFilesIter {
        Eu4ZipFilesIter {
            meta_index: self.meta_index,
            gamestate_index: self.gamestate_index,
            ai_index: self.ai_index,
        }
    }
}

struct Eu4ZipFile<'a> {
    raw: &'a [u8],
    size: usize,
}

impl<'a> Eu4ZipFile<'a> {
    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> Result<(), Eu4Error> {
        let start_len = buf.len();
        buf.resize(start_len + self.size(), 0);
        let body = &mut buf[start_len..];
        crate::deflate::inflate_exact(self.raw, body).map_err(Eu4ErrorKind::from)?;

        body.copy_within(TXT_HEADER.len().., 0);
        buf.truncate(start_len + self.size() - TXT_HEADER.len());
        Ok(())
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

enum FileKind<'a> {
    Text(&'a [u8]),
    Binary(&'a [u8]),
    Zip(Eu4Zip<'a>),
}

/// Entrypoint for parsing EU4 saves
///
/// EU4 saves are files that contain a "EU4txt" or "EU4bin" header or contains
/// files within a zip.
///
/// Only consumes enough data to determine encoding of the file
pub struct Eu4File<'a> {
    kind: FileKind<'a>,
}

impl<'a> Eu4File<'a> {
    /// Creates an EU4 file from a slice of data
    pub fn from_slice(data: &[u8]) -> Result<Eu4File, Eu4Error> {
        if let Some(text_data) = is_text(data) {
            Ok(Eu4File {
                kind: FileKind::Text(text_data),
            })
        } else if let Some(bin_data) = is_bin(data) {
            Ok(Eu4File {
                kind: FileKind::Binary(bin_data),
            })
        } else {
            let cursor = Cursor::new(data);
            let zip_attempt = zip::ZipArchive::new(cursor);
            match zip_attempt {
                Ok(mut zip) => {
                    let mut inflated_size = 0;
                    let mut found_text = None;
                    let eu4_files = Eu4ZipFiles::new(&mut zip, data);
                    for file in eu4_files.files() {
                        inflated_size += file.size;

                        if found_text.is_none() {
                            found_text = Some(file.is_text(data)?);
                        }
                    }

                    match found_text {
                        None => Err(Eu4ErrorKind::ZipHeader.into()),
                        Some(is_text) => Ok(Eu4File {
                            kind: FileKind::Zip(Eu4Zip {
                                archive: eu4_files,
                                is_text,
                                inflated_size,
                            }),
                        }),
                    }
                }
                Err(ZipError::InvalidArchive(_)) => Err(Eu4ErrorKind::UnknownHeader.into()),
                Err(e) => Err(Eu4ErrorKind::ZipArchive(e).into()),
            }
        }
    }

    /// Returns the detected decoding of the file
    pub fn encoding(&self) -> Encoding {
        match &self.kind {
            FileKind::Text(_) => Encoding::Text,
            FileKind::Binary(_) => Encoding::Binary,
            FileKind::Zip(zip) if zip.is_text => Encoding::TextZip,
            FileKind::Zip(_) => Encoding::BinaryZip,
        }
    }

    /// Returns the size of the file
    ///
    /// The size includes the inflated size of the zip
    pub fn size(&self) -> usize {
        match &self.kind {
            FileKind::Text(x) | FileKind::Binary(x) => x.len(),
            FileKind::Zip(x) => x.inflated_size,
        }
    }

    /// A convenience method for creating [`Eu4Save`](crate::models::Eu4Save)
    pub fn parse_save<R>(&self, resolver: &R) -> Result<Eu4Save, Eu4Error>
    where
        R: TokenResolver,
    {
        let mut zip_sink = Vec::new();
        let parsed_file = self.parse(&mut zip_sink)?;
        let des = parsed_file.deserializer(resolver);
        Eu4Save::from_deserializer(&des)
    }

    /// Parses the entire file
    ///
    /// If the file is a zip, the zip contents will be inflated into the zip
    /// sink before being parsed
    pub fn parse(&self, zip_sink: &'a mut Vec<u8>) -> Result<Eu4ParsedFile<'a>, Eu4Error> {
        match &self.kind {
            FileKind::Text(x) => {
                let text = Eu4Text::from_raw(x)?;
                Ok(Eu4ParsedFile {
                    kind: Eu4ParsedFileKind::Text(text),
                })
            }
            FileKind::Binary(x) => {
                let binary = Eu4Binary::from_raw(x)?;
                Ok(Eu4ParsedFile {
                    kind: Eu4ParsedFileKind::Binary(binary),
                })
            }
            FileKind::Zip(zip) => {
                zip.read_to_end(zip_sink)?;

                if zip.is_text {
                    let text = Eu4Text::from_raw(zip_sink)?;
                    Ok(Eu4ParsedFile {
                        kind: Eu4ParsedFileKind::Text(text),
                    })
                } else {
                    let binary = Eu4Binary::from_raw(zip_sink)?;
                    Ok(Eu4ParsedFile {
                        kind: Eu4ParsedFileKind::Binary(binary),
                    })
                }
            }
        }
    }

    /// Iterates through the individual entries of the Eu4 file
    ///
    /// Non-zips will yield a single entry
    pub fn entries(&self) -> Eu4FileEntries {
        match &self.kind {
            FileKind::Text(x) => Eu4FileEntries {
                kind: Eu4FileEntriesKind::Text {
                    has_yielded: false,
                    data: x,
                },
            },
            FileKind::Binary(x) => Eu4FileEntries {
                kind: Eu4FileEntriesKind::Binary {
                    has_yielded: false,
                    data: x,
                },
            },
            FileKind::Zip(x) => Eu4FileEntries {
                kind: Eu4FileEntriesKind::Zip {
                    files: Box::new(x.clone()),
                    iter: x.archive.files(),
                    is_text: x.is_text,
                },
            },
        }
    }
}

/// Contains the parsed EU4 file
pub enum Eu4ParsedFileKind<'a> {
    /// The EU4 file as text
    Text(Eu4Text<'a>),

    /// The EU4 file as binary
    Binary(Eu4Binary<'a>),
}

/// An EU4 file that has been parsed
pub struct Eu4ParsedFile<'data> {
    kind: Eu4ParsedFileKind<'data>,
}

impl<'data> Eu4ParsedFile<'data> {
    /// Returns the file as text
    pub fn as_text(&self) -> Option<&Eu4Text> {
        match &self.kind {
            Eu4ParsedFileKind::Text(x) => Some(x),
            _ => None,
        }
    }

    /// Returns the file as binary
    pub fn as_binary(&self) -> Option<&Eu4Binary> {
        match &self.kind {
            Eu4ParsedFileKind::Binary(x) => Some(x),
            _ => None,
        }
    }

    /// Returns the kind of file (binary or text)
    pub fn kind(&self) -> &Eu4ParsedFileKind {
        &self.kind
    }

    /// Prepares the file for deserialization into a custom structure
    pub fn deserializer<'b, RES>(&'b self, resolver: &'b RES) -> Eu4Deserializer<'data, 'b, RES>
    where
        RES: TokenResolver,
    {
        match &self.kind {
            Eu4ParsedFileKind::Text(x) => Eu4Deserializer {
                kind: Eu4DeserializerKind::Text(x.deserializer()),
            },
            Eu4ParsedFileKind::Binary(x) => Eu4Deserializer {
                kind: Eu4DeserializerKind::Binary(x.deserializer(resolver)),
            },
        }
    }
}

pub(crate) enum Eu4DeserializerKind<'data, 'tape, RES> {
    Text(Eu4TextDeserializer<'data, 'tape>),
    Binary(Eu4BinaryDeserializer<'data, 'tape, RES>),
}

/// A deserializer for custom structures
pub struct Eu4Deserializer<'data, 'tape, RES> {
    pub(crate) kind: Eu4DeserializerKind<'data, 'tape, RES>,
}

impl<'data, 'tape, RES> Eu4Deserializer<'data, 'tape, RES>
where
    RES: TokenResolver,
{
    pub fn on_failed_resolve(&mut self, strategy: FailedResolveStrategy) -> &mut Self {
        if let Eu4DeserializerKind::Binary(x) = &mut self.kind {
            x.on_failed_resolve(strategy);
        }
        self
    }

    pub fn deserialize<T>(&self) -> Result<T, Eu4Error>
    where
        T: Deserialize<'data>,
    {
        T::deserialize(self)
    }
}

enum Eu4FileEntriesKind<'a> {
    Text {
        has_yielded: bool,
        data: &'a [u8],
    },
    Binary {
        has_yielded: bool,
        data: &'a [u8],
    },
    Zip {
        is_text: bool,
        files: Box<Eu4Zip<'a>>,
        iter: Eu4ZipFilesIter,
    },
}

/// File entries contained within EU4 file
pub struct Eu4FileEntries<'a> {
    kind: Eu4FileEntriesKind<'a>,
}

impl<'a> Eu4FileEntries<'a> {
    pub fn next_entry(&mut self) -> Option<Eu4FileEntry<'a>> {
        match &mut self.kind {
            Eu4FileEntriesKind::Text { has_yielded, data } if !*has_yielded => {
                *has_yielded = true;
                Some(Eu4FileEntry {
                    kind: Eu4FileEntryKind::Text(data),
                })
            }
            Eu4FileEntriesKind::Binary { has_yielded, data } if !*has_yielded => {
                *has_yielded = true;
                Some(Eu4FileEntry {
                    kind: Eu4FileEntryKind::Binary(data),
                })
            }
            Eu4FileEntriesKind::Zip {
                files,
                iter,
                is_text,
            } => iter.next().map(|index| Eu4FileEntry {
                kind: {
                    Eu4FileEntryKind::Zip {
                        files: files.clone(),
                        is_text: *is_text,
                        index,
                    }
                },
            }),
            _ => None,
        }
    }
}

impl<'data> From<Eu4Text<'data>> for Eu4ParsedFile<'data> {
    fn from(value: Eu4Text<'data>) -> Self {
        Eu4ParsedFile {
            kind: Eu4ParsedFileKind::Text(value),
        }
    }
}

impl<'data> From<Eu4Binary<'data>> for Eu4ParsedFile<'data> {
    fn from(value: Eu4Binary<'data>) -> Self {
        Eu4ParsedFile {
            kind: Eu4ParsedFileKind::Binary(value),
        }
    }
}

enum Eu4FileEntryKind<'a> {
    Text(&'a [u8]),
    Binary(&'a [u8]),
    Zip {
        files: Box<Eu4Zip<'a>>,
        index: VerifiedIndex,
        is_text: bool,
    },
}

/// Name of the EU4 entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Eu4FileEntryName {
    Gamestate,
    Meta,
    Ai,
}

impl Display for Eu4FileEntryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Eu4FileEntryName::Meta => write!(f, "meta"),
            Eu4FileEntryName::Gamestate => write!(f, "gamestate"),
            Eu4FileEntryName::Ai => write!(f, "ai"),
        }
    }
}

/// An individual entry of an EU4 file
///
/// An entry could be the entire file if the input was a plaintext or a binary
/// document. In the advent of a zip file, an entry would be an individual file
/// in the zip.
pub struct Eu4FileEntry<'a> {
    kind: Eu4FileEntryKind<'a>,
}

impl<'a> Eu4FileEntry<'a> {
    /// Name of the EU4 entry
    pub fn name(&self) -> Option<Eu4FileEntryName> {
        if let Eu4FileEntryKind::Zip { index, .. } = &self.kind {
            Some(index.name)
        } else {
            None
        }
    }

    /// Size of the entry (eg: inflated size)
    pub fn size(&self) -> usize {
        match &self.kind {
            Eu4FileEntryKind::Text(x) | Eu4FileEntryKind::Binary(x) => x.len(),
            Eu4FileEntryKind::Zip { index, .. } => index.size,
        }
    }

    /// Parse the entry into a file, while inflating the contents into the zip sink
    pub fn parse(&self, zip_sink: &'a mut Vec<u8>) -> Result<Eu4ParsedFile<'a>, Eu4Error> {
        match &self.kind {
            Eu4FileEntryKind::Text(x) => Ok(Eu4ParsedFile {
                kind: Eu4ParsedFileKind::Text(Eu4Text::from_raw(x)?),
            }),
            Eu4FileEntryKind::Binary(x) => Ok(Eu4ParsedFile {
                kind: Eu4ParsedFileKind::Binary(Eu4Binary::from_raw(x)?),
            }),
            Eu4FileEntryKind::Zip {
                files,
                is_text,
                index,
            } => {
                let file = files.archive.retrieve_file(*index);
                file.read_to_end(zip_sink)?;
                if *is_text {
                    Ok(Eu4ParsedFile {
                        kind: Eu4ParsedFileKind::Text(Eu4Text::from_raw(zip_sink)?),
                    })
                } else {
                    Ok(Eu4ParsedFile {
                        kind: Eu4ParsedFileKind::Binary(Eu4Binary::from_raw(zip_sink)?),
                    })
                }
            }
        }
    }
}

/// A parsed EU4 text document
pub struct Eu4Text<'a> {
    tape: TextTape<'a>,
}

impl<'a> Eu4Text<'a> {
    /// Parse EU4 text data that has the "EU4txt" header
    pub fn from_slice(data: &'a [u8]) -> Result<Self, Eu4Error> {
        is_text(data)
            .ok_or_else(|| Eu4ErrorKind::UnknownHeader.into())
            .and_then(Self::from_raw)
    }

    /// Parse headerless EU4 text data
    pub fn from_raw(data: &'a [u8]) -> Result<Self, Eu4Error> {
        let tape = TextTape::from_slice(data).map_err(Eu4ErrorKind::Parse)?;
        Ok(Eu4Text { tape })
    }

    pub fn reader(&self) -> ObjectReader<Windows1252Encoding> {
        self.tape.windows1252_reader()
    }

    pub fn deserializer(&self) -> Eu4TextDeserializer {
        Eu4TextDeserializer {
            deser: TextDeserializer::from_windows1252_tape(&self.tape),
        }
    }
}

/// Deserializes binary data into custom structures
pub struct Eu4TextDeserializer<'data, 'tape> {
    pub(crate) deser: TextDeserializer<'tape, 'data, Windows1252Encoding>,
}

impl<'de, 'tape> Eu4TextDeserializer<'de, 'tape> {
    pub fn deserialize<T>(&self) -> Result<T, Eu4Error>
    where
        T: Deserialize<'de>,
    {
        T::deserialize(self)
    }
}

/// A parsed EU4 binary document
pub struct Eu4Binary<'data> {
    tape: BinaryTape<'data>,
}

impl<'data> Eu4Binary<'data> {
    /// Parse EU4 binary data that has the "EU4bin" header
    pub fn from_slice(data: &'data [u8]) -> Result<Self, Eu4Error> {
        is_bin(data)
            .ok_or_else(|| Eu4ErrorKind::UnknownHeader.into())
            .and_then(Self::from_raw)
    }

    /// Parse headerless EU4 binary data
    pub fn from_raw(data: &'data [u8]) -> Result<Self, Eu4Error> {
        let tape = BinaryTape::from_slice(data).map_err(Eu4ErrorKind::Parse)?;
        Ok(Eu4Binary { tape })
    }

    pub(crate) fn tape(&self) -> &BinaryTape {
        &self.tape
    }

    pub fn deserializer<'b, RES>(
        &'b self,
        resolver: &'b RES,
    ) -> Eu4BinaryDeserializer<'data, 'b, RES>
    where
        RES: TokenResolver,
    {
        let deser =
            BinaryDeserializer::builder_flavor(Eu4Flavor::new()).from_tape(&self.tape, resolver);
        Eu4BinaryDeserializer { deser }
    }

    pub fn melter<'b>(&'b self) -> Eu4Melter<'data, 'b> {
        Eu4Melter::new(&self.tape)
    }
}

/// Deserializes binary data into custom structures
pub struct Eu4BinaryDeserializer<'data, 'tape, RES> {
    pub(crate) deser: BinaryDeserializer<'tape, 'data, 'tape, RES, Eu4Flavor>,
}

impl<'de, 'tape, RES: TokenResolver> Eu4BinaryDeserializer<'de, 'tape, RES> {
    pub fn on_failed_resolve(&mut self, strategy: FailedResolveStrategy) -> &mut Self {
        self.deser.on_failed_resolve(strategy);
        self
    }

    pub fn deserialize<T>(&self) -> Result<T, Eu4Error>
    where
        T: Deserialize<'de>,
    {
        T::deserialize(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, io::Write};
    use zip::{write::FileOptions, ZipWriter};

    fn create_zip(meta: &[u8], gamestate: &[u8], ai: &[u8]) -> Vec<u8> {
        let out = Vec::new();
        let writer = Cursor::new(out);
        let mut zip = ZipWriter::new(writer);

        if !meta.is_empty() {
            zip.start_file("meta", FileOptions::default()).unwrap();
            zip.write_all(b"EU4txt\n").unwrap();
            zip.write_all(meta).unwrap();
        }

        if !gamestate.is_empty() {
            zip.start_file("gamestate", FileOptions::default()).unwrap();
            zip.write_all(b"EU4txt\n").unwrap();
            zip.write_all(gamestate).unwrap();
        }

        if !ai.is_empty() {
            zip.start_file("ai", FileOptions::default()).unwrap();
            zip.write_all(b"EU4txt\n").unwrap();
            zip.write_all(ai).unwrap();
        }

        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn test_simple_file() {
        let file = Eu4File::from_slice(b"EU4txt\nhello=world").unwrap();
        let mut entries = file.entries();
        let entry = entries.next_entry().unwrap();
        assert!(entry.name().is_none());
        let mut sink = Vec::new();
        let parsed = entry.parse(&mut sink).unwrap();
        let text = parsed.as_text().unwrap();
        let json = text.reader().json().to_string();
        assert_eq!(&json, r#"{"hello":"world"}"#);
    }

    #[test]
    fn test_zip_meta_text_file() {
        #[derive(Deserialize)]
        struct MyMeta<'a> {
            date: &'a str,
        }

        let zip_data = create_zip(b"date=1463.5.28\n", b"speed=2", b"base=4636");

        let file = Eu4File::from_slice(&zip_data).unwrap();

        let mut found = false;
        let mut sink = Vec::new();

        let mut entries = file.entries();
        while let Some(entry) = entries.next_entry() {
            if let Some(Eu4FileEntryName::Meta) = entry.name() {
                let data = entry.parse(&mut sink).unwrap();
                let text = data.as_text().unwrap();
                let actual: MyMeta = text.deserializer().deserialize().unwrap();
                assert_eq!(actual.date, "1463.5.28");

                let out = text.reader().json().to_string();
                assert_eq!(&out, r#"{"date":"1463.5.28"}"#);
                found = true;
            }
        }

        assert!(found);
    }

    #[test]
    fn test_zip_meta_text_files() {
        #[derive(Deserialize)]
        struct MySave<'a> {
            date: &'a str,
            speed: u16,
            base: u16,
        }

        let zip_data = create_zip(b"date=1463.5.28\n", b"speed=2", b"base=4636");

        let file = Eu4File::from_slice(&zip_data).unwrap();
        let mut sink = Vec::new();
        let eu4 = file.parse(&mut sink).unwrap();
        let text = eu4.as_text().unwrap();
        let actual: MySave = text.deserializer().deserialize().unwrap();
        assert_eq!(actual.date, "1463.5.28");
        assert_eq!(actual.speed, 2);
        assert_eq!(actual.base, 4636);
    }

    #[test]
    fn test_zip_text_file() {
        #[derive(Deserialize)]
        struct MySave<'a> {
            date: &'a str,
            speed: u16,
            base: u16,
        }

        let zip_data = create_zip(b"date=1463.5.28\n", b"speed=2", b"base=4636");

        let file = Eu4File::from_slice(&zip_data).unwrap();
        let mut sink = Vec::new();
        let eu4 = file.parse(&mut sink).unwrap();
        let resolver: HashMap<u16, &str> = HashMap::new();
        let actual: MySave = eu4.deserializer(&resolver).deserialize().unwrap();
        assert_eq!(actual.date, "1463.5.28");
        assert_eq!(actual.speed, 2);
        assert_eq!(actual.base, 4636);
    }
}
