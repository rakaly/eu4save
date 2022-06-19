#![allow(dead_code)]
use crate::{flavor::Eu4Flavor, Eu4Error, Eu4ErrorKind};
use jomini::{
    binary::{BinaryDeserializerBuilder, FailedResolveStrategy, TokenResolver},
    json::JsonOptions,
    BinaryDeserializer, BinaryTape, TextDeserializer, TextTape,
};
use serde::Deserialize;
use std::io::{Cursor, Read};
use zip::{read::ZipFile, result::ZipError};

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

fn is_zip(data: &[u8]) -> bool {
    let sentry = [0x50, 0x4b, 0x03, 0x04];
    data.get(..sentry.len()).map_or(false, |x| x == sentry)
}

struct Eu4FileBuilder {}

impl Eu4FileBuilder {
    pub fn from_slice(self, data: &[u8]) -> Result<Eu4File, Eu4Error> {
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
                Ok(zip) => {
                    let mut inflated_size = 0;
                    let mut header = [0; TXT_HEADER.len()];
                    let mut found_text = None;
                    let mut eu4_files = Eu4ZipFiles::new(zip);
                    while let Some(mut file) = eu4_files.next_file() {
                        inflated_size += file.file.size();

                        if found_text.is_none() {
                            file.file.read_exact(&mut header)?;
                            found_text = Some(is_text(&header).is_some())
                        }
                    }

                    match found_text {
                        None => Err(Eu4ErrorKind::UnknownHeader.into()),
                        Some(is_text) => Ok(Eu4File {
                            kind: FileKind::Zip(Eu4Zip {
                                archive: eu4_files.into_zip(),
                                is_text,
                                inflated_size,
                            }),
                        }),
                    }
                }
                Err(ZipError::InvalidArchive(_)) => Err(Eu4ErrorKind::UnknownHeader.into()),
                Err(e) => Err(Eu4Error::new(Eu4ErrorKind::ZipCentralDirectory(e))),
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Eu4Zip<'a> {
    archive: zip::ZipArchive<Cursor<&'a [u8]>>,
    is_text: bool,
    inflated_size: u64,
}

impl<'a> Eu4Zip<'a> {
    fn files(&self) -> Eu4ZipFiles<'a> {
        Eu4ZipFiles::new(self.archive.clone())
    }

    fn read_to_end(&self, zip_sink: &'a mut Vec<u8>) -> Result<(), std::io::Error> {
        let mut files = self.files();
        while let Some(mut file) = files.next_file() {
            file.read_to_end(zip_sink)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct VerifiedIndex {
    index: usize,
    name: Eu4FileEntryName,
}

#[derive(Debug, Clone)]
struct Eu4ZipFiles<'a> {
    archive: zip::ZipArchive<Cursor<&'a [u8]>>,
    meta_index: Option<usize>,
    gamestate_index: Option<usize>,
    ai_index: Option<usize>,
}

impl<'a> Eu4ZipFiles<'a> {
    pub fn new(mut archive: zip::ZipArchive<Cursor<&'a [u8]>>) -> Self {
        let mut meta_index = None;
        let mut gamestate_index = None;
        let mut ai_index = None;

        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i) {
                match file.name() {
                    "meta" => meta_index = Some(i),
                    "gamestate" => gamestate_index = Some(i),
                    "ai" => ai_index = Some(i),
                    _ => {}
                }
            }
        }

        Self {
            archive,
            meta_index,
            gamestate_index,
            ai_index,
        }
    }

    pub fn retrieve_file(&mut self, index: VerifiedIndex) -> Eu4ZipFile {
        let file = self.archive.by_index(index.index).unwrap();
        Eu4ZipFile { file }
    }

    pub fn next_index(&mut self) -> Option<VerifiedIndex> {
        if let Some(index) = self.meta_index.take() {
            return Some(VerifiedIndex {
                index,
                name: Eu4FileEntryName::Meta,
            });
        }

        if let Some(index) = self.gamestate_index.take() {
            return Some(VerifiedIndex {
                index,
                name: Eu4FileEntryName::Gamestate,
            });
        }

        if let Some(index) = self.ai_index.take() {
            return Some(VerifiedIndex {
                index,
                name: Eu4FileEntryName::Ai,
            });
        }

        None
    }

    pub fn next_file(&mut self) -> Option<Eu4ZipFile> {
        self.next_index().map(move |i| self.retrieve_file(i))
    }

    pub fn into_zip(self) -> zip::ZipArchive<Cursor<&'a [u8]>> {
        self.archive
    }
}

struct Eu4ZipFile<'a> {
    file: ZipFile<'a>,
    // name: Eu4FileEntryName,
}

impl<'a> Eu4ZipFile<'a> {
    pub fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let mut header = [0; TXT_HEADER.len()];
        self.file.read_exact(&mut header)?;
        buf.reserve(self.size());
        self.file.read_to_end(buf)
    }

    pub fn size(&self) -> usize {
        self.file.size() as usize
    }

    // pub fn name(&self) -> Eu4FileEntryName {
    //     self.name
    // }
}

enum FileKind<'a> {
    Text(&'a [u8]),
    Binary(&'a [u8]),
    Zip(Eu4Zip<'a>),
}

enum FileEncoding<'a> {
    None(&'a [u8]),
    Zip(Eu4Zip<'a>),
}

enum Eu4Tokens<'a> {
    Text(TextTape<'a>),
    Binary(BinaryTape<'a>),
}

struct Eu4File<'a> {
    kind: FileKind<'a>,
}

impl<'a> Eu4File<'a> {
    pub fn builder() -> Eu4FileBuilder {
        Eu4FileBuilder {}
    }

    pub fn deserialize(&self) {
        match &self.kind {
            FileKind::Text(x) => {}

            FileKind::Binary(x) => {}

            FileKind::Zip(zip) => {}
        }
    }

    pub fn parse(&self, zip_sink: &'a mut Vec<u8>) -> Result<Eu4ParsedFile<'a>, Eu4Error> {
        match &self.kind {
            FileKind::Text(x) => {
                let text = Eu4Text::from_slice(x)?;
                Ok(Eu4ParsedFile {
                    kind: Eu4ParsedFileKind::Text(text),
                })
            }
            FileKind::Binary(x) => {
                let binary = Eu4Binary::from_slice(x)?;
                Ok(Eu4ParsedFile {
                    kind: Eu4ParsedFileKind::Binary(binary),
                })
            }
            FileKind::Zip(zip) => {
                zip.read_to_end(zip_sink)?;

                if zip.is_text {
                    let text = Eu4Text::from_slice(zip_sink)?;
                    Ok(Eu4ParsedFile {
                        kind: Eu4ParsedFileKind::Text(text),
                    })
                } else {
                    let binary = Eu4Binary::from_slice(zip_sink)?;
                    Ok(Eu4ParsedFile {
                        kind: Eu4ParsedFileKind::Binary(binary),
                    })
                }
            }
        }
    }

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
                    files: x.files(),
                    is_text: x.is_text,
                },
            },
        }
    }

    // pub fn as_text(&self) -> Option<Eu4TextFile> {
    //     match &self.kind {
    //         FileKind::Text(x) => Some(Eu4TextFile {
    //             data: FileEncoding::None(x),
    //         }),
    //         FileKind::Zip(x) if x.is_text => Some(Eu4TextFile {
    //             data: FileEncoding::Zip(x.clone()),
    //         }),
    //         _ => None,
    //     }
    // }

    // pub fn as_binary(&self) -> Option<Eu4BinaryFile> {
    //     match &self.kind {
    //         FileKind::Binary(x) => Some(Eu4BinaryFile {
    //             data: FileEncoding::None(x),
    //         }),
    //         FileKind::Zip(x) if !x.is_text => Some(Eu4BinaryFile {
    //             data: FileEncoding::Zip(x.clone()),
    //         }),
    //         _ => None,
    //     }
    // }
}

enum Eu4ParsedFileKind<'a> {
    Text(Eu4Text<'a>),
    Binary(Eu4Binary<'a>),
}

struct Eu4ParsedFile<'a> {
    kind: Eu4ParsedFileKind<'a>,
}

impl<'a> Eu4ParsedFile<'a> {
    pub fn as_text(&self) -> Option<&Eu4Text> {
        match &self.kind {
            Eu4ParsedFileKind::Text(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&Eu4Binary> {
        match &self.kind {
            Eu4ParsedFileKind::Binary(x) => Some(x),
            _ => None,
        }
    }

    pub fn deserializer(&self) -> Eu4Deserializer {
        match &self.kind {
            Eu4ParsedFileKind::Text(x) => Eu4Deserializer {
                kind: Eu4DeserializerKind::Text(x),
            },
            Eu4ParsedFileKind::Binary(x) => Eu4Deserializer {
                kind: Eu4DeserializerKind::Binary(x.deserializer()),
            },
        }
    }
}

enum Eu4DeserializerKind<'a, 'b> {
    Text(&'b Eu4Text<'a>),
    Binary(Eu4BinaryDeserializer<'a, 'b>),
}

struct Eu4Deserializer<'a, 'b> {
    kind: Eu4DeserializerKind<'a, 'b>,
}

impl<'a, 'b> Eu4Deserializer<'a, 'b> {
    pub fn on_failed_resolve(&mut self, strategy: FailedResolveStrategy) -> &mut Self {
        if let Eu4DeserializerKind::Binary(x) = &mut self.kind {
            x.on_failed_resolve(strategy);
        }
        self
    }

    pub fn build<T, R>(&self, resolver: &'a R) -> Result<T, Eu4Error>
    where
        R: TokenResolver,
        T: Deserialize<'a>,
    {
        match &self.kind {
            Eu4DeserializerKind::Text(x) => x.deserialize(),
            Eu4DeserializerKind::Binary(x) => x.build(resolver),
        }
    }
}

struct Eu4TextFile<'a> {
    data: FileEncoding<'a>,
}

impl<'a> Eu4TextFile<'a> {
    pub fn parse(&mut self, zip_sink: &'a mut Vec<u8>) -> Result<Eu4Text<'a>, Eu4Error> {
        match &self.data {
            FileEncoding::None(x) => Eu4Text::from_slice(x),
            FileEncoding::Zip(zip) => {
                zip.read_to_end(zip_sink)?;
                Eu4Text::from_slice(zip_sink.as_slice())
            }
        }
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
        files: Eu4ZipFiles<'a>,
    },
}

struct Eu4FileEntries<'a> {
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
            Eu4FileEntriesKind::Zip { files, is_text } => {
                files.next_index().map(|index| Eu4FileEntry {
                    kind: {
                        Eu4FileEntryKind::Zip {
                            files: files.clone(),
                            is_text: *is_text,
                            index,
                        }
                    },
                })
            }
            _ => None,
        }
    }
}

enum Eu4FileEntryKind<'a> {
    Text(&'a [u8]),
    Binary(&'a [u8]),
    Zip {
        files: Eu4ZipFiles<'a>,
        index: VerifiedIndex,
        is_text: bool,
    },
}

#[derive(Debug, Clone, Copy)]
enum Eu4FileEntryName {
    Gamestate,
    Meta,
    Ai,
}

struct Eu4FileEntry<'a> {
    kind: Eu4FileEntryKind<'a>,
}

impl<'a> Eu4FileEntry<'a> {
    pub fn name(&self) -> Option<Eu4FileEntryName> {
        if let Eu4FileEntryKind::Zip { index, .. } = &self.kind {
            Some(index.name)
        } else {
            None
        }
    }

    pub fn as_binary(&self) -> Option<Eu4BinaryEntry<'a>> {
        match &self.kind {
            Eu4FileEntryKind::Binary(x) => Some(Eu4BinaryEntry {
                data: EntryEncoding::None(x),
            }),
            Eu4FileEntryKind::Zip {
                files,
                is_text,
                index,
            } if !*is_text => Some(Eu4BinaryEntry {
                data: EntryEncoding::Zip {
                    files: files.clone(),
                    index: *index,
                },
            }),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<Eu4TextEntry<'a>> {
        match &self.kind {
            Eu4FileEntryKind::Text(x) => Some(Eu4TextEntry {
                data: EntryEncoding::None(x),
            }),
            Eu4FileEntryKind::Zip {
                files,
                is_text,
                index,
            } if *is_text => Some(Eu4TextEntry {
                data: EntryEncoding::Zip {
                    files: files.clone(),
                    index: *index,
                },
            }),
            _ => None,
        }
    }
}

enum EntryEncoding<'a> {
    None(&'a [u8]),
    Zip {
        files: Eu4ZipFiles<'a>,
        index: VerifiedIndex,
    },
}

struct Eu4TextEntry<'a> {
    data: EntryEncoding<'a>,
}

impl<'a> Eu4TextEntry<'a> {
    pub fn parse(&self, zip_sink: &'a mut Vec<u8>) -> Result<Eu4Text<'a>, Eu4Error> {
        match &self.data {
            EntryEncoding::None(x) => Eu4Text::from_slice(x),
            EntryEncoding::Zip { files, index } => {
                let mut files = files.clone();
                let mut file = files.retrieve_file(*index);
                file.read_to_end(zip_sink)?;
                Eu4Text::from_slice(zip_sink.as_slice())
            }
        }
    }
}

struct Eu4Text<'a> {
    tape: TextTape<'a>,
}

impl<'a> Eu4Text<'a> {
    pub fn from_slice(data: &'a [u8]) -> Result<Self, Eu4Error> {
        let tape = TextTape::from_slice(data)?;
        Ok(Eu4Text { tape })
    }

    pub fn to_json_string(&self, options: JsonOptions) -> String {
        self.tape
            .windows1252_reader()
            .json()
            .with_options(options)
            .to_string()
    }

    pub fn to_json_vec(&self, options: JsonOptions) -> Vec<u8> {
        self.tape
            .windows1252_reader()
            .json()
            .with_options(options)
            .to_vec()
    }

    pub fn to_json_writer<W>(&self, writer: W, options: JsonOptions) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        self.tape
            .windows1252_reader()
            .json()
            .with_options(options)
            .to_writer(writer)
    }

    pub fn deserialize<T>(&self) -> Result<T, Eu4Error>
    where
        T: Deserialize<'a>,
    {
        TextDeserializer::from_windows1252_tape(&self.tape).map_err(|e| e.into())
    }
}

struct Eu4BinaryFile<'a> {
    data: FileEncoding<'a>,
}

impl<'a> Eu4BinaryFile<'a> {
    // pub fn parse(&mut self, zip_sink: &'a mut Vec<u8>) -> Result<Eu4Binary<'a>, Eu4Error> {
    //     match &self.data {
    //         FileEncoding::None(x) => Ok(Eu4Binary {
    //             tape: BinaryTape::from_slice(x)?,
    //         }),
    //         FileEncoding::Zip(zip) => {
    //             zip.read_to_end(zip_sink)?;
    //             Ok(Eu4Binary {
    //                 tape: BinaryTape::from_slice(zip_sink.as_slice())?,
    //             })
    //         }
    //     }
    // }
}

struct Eu4BinaryEntry<'a> {
    data: EntryEncoding<'a>,
}

impl<'a> Eu4BinaryEntry<'a> {
    pub fn parse(&self, zip_sink: &'a mut Vec<u8>) -> Result<Eu4Binary<'a>, Eu4Error> {
        match &self.data {
            EntryEncoding::None(x) => Eu4Binary::from_slice(zip_sink.as_slice()),
            EntryEncoding::Zip { files, index } => {
                let mut files = files.clone();
                let mut file = files.retrieve_file(*index);
                file.read_to_end(zip_sink)?;
                Eu4Binary::from_slice(zip_sink.as_slice())
            }
        }
    }
}

struct Eu4Binary<'a> {
    tape: BinaryTape<'a>,
}

impl<'a> Eu4Binary<'a> {
    pub fn from_slice(data: &'a [u8]) -> Result<Self, Eu4Error> {
        let tape = BinaryTape::from_slice(data)?;
        Ok(Eu4Binary { tape })
    }

    pub fn deserializer<'b>(&'b self) -> Eu4BinaryDeserializer<'a, 'b> {
        Eu4BinaryDeserializer {
            builder: BinaryDeserializer::builder_flavor(Eu4Flavor::new()),
            tape: &self.tape,
        }
    }
}

struct Eu4BinaryDeserializer<'a, 'b> {
    builder: BinaryDeserializerBuilder<Eu4Flavor>,
    tape: &'b BinaryTape<'a>,
}

impl<'a, 'b> Eu4BinaryDeserializer<'a, 'b> {
    pub fn on_failed_resolve(&mut self, strategy: FailedResolveStrategy) -> &mut Self {
        self.builder.on_failed_resolve(strategy);
        self
    }

    pub fn build<T, R>(&self, resolver: &'a R) -> Result<T, Eu4Error>
    where
        R: TokenResolver,
        T: Deserialize<'a>,
    {
        self.builder
            .from_tape(self.tape, resolver)
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Write, collections::HashMap};
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
        let file = Eu4File::builder()
            .from_slice(b"EU4txt\nhello=world")
            .unwrap();
        let mut entries = file.entries();
        let entry = entries.next_entry().unwrap();
        assert!(entry.name().is_none());
        let text = entry.as_text().unwrap();
        let mut sink = Vec::new();
        let parsed = text.parse(&mut sink).unwrap();
        let json = parsed.to_json_string(JsonOptions::new());
        assert_eq!(&json, r#"{"hello":"world"}"#);
    }

    #[test]
    fn test_zip_meta_text_file() {
        #[derive(Deserialize)]
        struct MyMeta<'a> {
            date: &'a str,
        }

        let zip_data = create_zip(b"date=1463.5.28\n", b"speed=2", b"base=4636");

        let file = Eu4File::builder().from_slice(&zip_data).unwrap();

        let mut found = false;
        let mut sink = Vec::new();

        let mut entries = file.entries();
        while let Some(entry) = entries.next_entry() {
            if let Some(Eu4FileEntryName::Meta) = entry.name() {
                let data = entry.as_text().unwrap();
                let text = data.parse(&mut sink).unwrap();
                let actual: MyMeta = text.deserialize().unwrap();
                assert_eq!(actual.date, "1463.5.28");

                let out = text.to_json_string(JsonOptions::new());
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

        let file = Eu4File::builder().from_slice(&zip_data).unwrap();
        let mut sink = Vec::new();
        let eu4 = file.parse(&mut sink).unwrap();
        let text = eu4.as_text().unwrap();
        let actual: MySave = text.deserialize().unwrap();
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

        let file = Eu4File::builder().from_slice(&zip_data).unwrap();
        let mut sink = Vec::new();
        let eu4 = file.parse(&mut sink).unwrap();
        let resolver: HashMap<u16, &str> = HashMap::new();
        let actual: MySave = eu4.deserializer().build(&resolver).unwrap();
        assert_eq!(actual.date, "1463.5.28");
        assert_eq!(actual.speed, 2);
        assert_eq!(actual.base, 4636);
    }
}
