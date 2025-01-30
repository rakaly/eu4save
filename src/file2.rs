#![allow(dead_code)]

use crate::{
    flavor::Eu4Flavor,
    models::{Eu4Save, GameState, Meta},
    Encoding, Eu4Error, Eu4ErrorKind,
};
use jomini::{
    binary::{de::BinaryReaderDeserializer, TokenResolver},
    text::de::TextReaderDeserializer,
    BinaryDeserializer, TextDeserializer,
};
use serde::{de::DeserializeOwned, Deserialize};
use std::{fs::File, io::Read};

const TXT_HEADER: &[u8] = b"EU4txt";
const BIN_HEADER: &[u8] = b"EU4bin";

pub struct Eu4File {}

impl Eu4File {
    pub fn from_slice(data: &[u8]) -> Result<Eu4SliceFile, Eu4Error> {
        match data.get(..TXT_HEADER.len()) {
            Some(TXT_HEADER) => Ok(Eu4SliceFile {
                kind: Eu4SliceFileKind::Text(&data[TXT_HEADER.len()..]),
            }),
            Some(BIN_HEADER) => Ok(Eu4SliceFile {
                kind: Eu4SliceFileKind::Binary(&data[BIN_HEADER.len()..]),
            }),
            _ => {
                let archive = rawzip::ZipArchive::from_slice(data).map_err(Eu4ErrorKind::Zip)?;
                let mut meta = None;
                let mut gamestate = None;
                let mut ai = None;
                let mut entries = archive.entries();
                let mut is_text = true;

                while let Ok(Some(entry)) = entries.next_entry() {
                    let compression = entry.compression_method();
                    match entry.file_raw_path() {
                        b"meta" => {
                            let wayfinder = entry.wayfinder();
                            meta = Some(wayfinder);

                            let ent = archive.get_entry(wayfinder).map_err(Eu4ErrorKind::Zip)?;

                            let mut header = [0u8; TXT_HEADER.len()];
                            match compression {
                                rawzip::CompressionMethod::Deflate => {
                                    flate2::read::DeflateDecoder::new(ent.data())
                                        .read_exact(&mut header)?
                                }
                                rawzip::CompressionMethod::Zstd => {
                                    zstd::stream::Decoder::new(ent.data())?
                                        .read_exact(&mut header)?
                                }
                                _ => return Err(Eu4ErrorKind::UnknownCompression.into()),
                            }

                            match &header[..] {
                                TXT_HEADER => is_text = true,
                                BIN_HEADER => is_text = false,
                                _ => return Err(Eu4ErrorKind::UnknownHeader.into()),
                            }
                        }
                        b"gamestate" => gamestate = Some(entry.wayfinder()),
                        b"ai" => ai = Some(entry.wayfinder()),
                        _ => {}
                    }

                    match (meta, gamestate, ai) {
                        (Some(meta), Some(gamestate), Some(ai)) => {
                            return Ok(Eu4SliceFile {
                                kind: Eu4SliceFileKind::Zip(Eu4SliceZip {
                                    archive,
                                    meta,
                                    gamestate,
                                    ai,
                                    compression,
                                    is_text,
                                }),
                            })
                        }
                        _ => {}
                    }
                }

                if meta.is_none() {
                    return Err(
                        Eu4ErrorKind::MissingFile(crate::file::Eu4FileEntryName::Meta).into(),
                    );
                }

                if gamestate.is_none() {
                    return Err(
                        Eu4ErrorKind::MissingFile(crate::file::Eu4FileEntryName::Gamestate).into(),
                    );
                }

                return Err(Eu4ErrorKind::MissingFile(crate::file::Eu4FileEntryName::Ai).into());
            }
        }
    }

    pub fn from_file(mut file: File) -> Result<Eu4FsFile, Eu4Error> {
        let mut header = [0u8; TXT_HEADER.len()];
        file.read_exact(&mut header)?;
        match &header[..] {
            TXT_HEADER => Ok(Eu4FsFile {
                kind: Eu4FsFileKind::Text(file),
            }),
            BIN_HEADER => Ok(Eu4FsFile {
                kind: Eu4FsFileKind::Binary(file),
            }),
            _ => {
                let mut buf = vec![0u8; rawzip::RECOMMENDED_BUFFER_SIZE];
                let archive =
                    rawzip::ZipArchive::from_file(file, &mut buf).map_err(Eu4ErrorKind::Zip)?;

                let mut meta = None;
                let mut gamestate = None;
                let mut ai = None;
                let mut entries = archive.entries(&mut buf);
                let mut is_text = true;

                while let Ok(Some(entry)) = entries.next_entry() {
                    let compression = entry.compression_method();
                    match entry.file_raw_path() {
                        b"meta" => {
                            let wayfinder = entry.wayfinder();
                            meta = Some(wayfinder);

                            let ent = archive.get_entry(wayfinder).map_err(Eu4ErrorKind::Zip)?;

                            match compression {
                                rawzip::CompressionMethod::Deflate => {
                                    flate2::read::DeflateDecoder::new(ent.reader())
                                        .read_exact(&mut header)?
                                }
                                rawzip::CompressionMethod::Zstd => {
                                    zstd::stream::Decoder::new(ent.reader())?
                                        .read_exact(&mut header)?
                                }
                                _ => return Err(Eu4ErrorKind::UnknownCompression.into()),
                            }

                            match &header[..] {
                                TXT_HEADER => is_text = true,
                                BIN_HEADER => is_text = false,
                                _ => return Err(Eu4ErrorKind::UnknownHeader.into()),
                            }
                        }
                        b"gamestate" => gamestate = Some(entry.wayfinder()),
                        b"ai" => ai = Some(entry.wayfinder()),
                        _ => {}
                    }

                    match (meta, gamestate, ai) {
                        (Some(meta), Some(gamestate), Some(ai)) => {
                            return Ok(Eu4FsFile {
                                kind: Eu4FsFileKind::Zip(Eu4FsZip {
                                    archive,
                                    meta,
                                    gamestate,
                                    ai,
                                    compression,
                                    is_text,
                                }),
                            })
                        }
                        _ => {}
                    }
                }

                if meta.is_none() {
                    return Err(
                        Eu4ErrorKind::MissingFile(crate::file::Eu4FileEntryName::Meta).into(),
                    );
                }

                if gamestate.is_none() {
                    return Err(
                        Eu4ErrorKind::MissingFile(crate::file::Eu4FileEntryName::Gamestate).into(),
                    );
                }

                return Err(Eu4ErrorKind::MissingFile(crate::file::Eu4FileEntryName::Ai).into());
            }
        }
    }
}

enum Eu4SliceFileKind<'a> {
    Text(&'a [u8]),
    Binary(&'a [u8]),
    Zip(Eu4SliceZip<'a>),
}

pub struct Eu4SliceFile<'a> {
    kind: Eu4SliceFileKind<'a>,
}

impl<'a> Eu4SliceFile<'a> {
    pub fn encoding(&self) -> Encoding {
        match &self.kind {
            Eu4SliceFileKind::Text(_) => Encoding::Text,
            Eu4SliceFileKind::Binary(_) => Encoding::Binary,
            Eu4SliceFileKind::Zip(archive) => {
                if archive.is_text {
                    Encoding::TextZip
                } else {
                    Encoding::BinaryZip
                }
            }
        }
    }

    pub fn parse_save<R>(&self, resolver: R) -> Result<Eu4Save, Eu4Error>
    where
        R: TokenResolver,
    {
        match &self.kind {
            Eu4SliceFileKind::Text(data) => {
                let reader = jomini::text::TokenReader::new(*data);
                let deserializer = TextDeserializer::from_windows1252_reader(reader);
                let mut deserializer = Eu4TextDeserializer {
                    deser: deserializer,
                };
                deserializer.deserialize()
            }
            Eu4SliceFileKind::Binary(data) => {
                let mut deserializer = BinaryDeserializer::builder_flavor(Eu4Flavor::new())
                    .from_slice(data, &resolver);
                let result = deserializer.deserialize()?;
                Ok(result)
            }
            Eu4SliceFileKind::Zip(archive) => {
                let meta: Meta = archive.deserialize_entry(archive.meta, &resolver)?;
                let game: GameState = archive.deserialize_entry(archive.gamestate, &resolver)?;
                Ok(Eu4Save { meta, game })
            }
        }
    }
}

struct Eu4SliceZip<'a> {
    archive: rawzip::ZipSliceArchive<'a>,
    compression: rawzip::CompressionMethod,
    meta: rawzip::ZipArchiveEntryWayfinder,
    gamestate: rawzip::ZipArchiveEntryWayfinder,
    ai: rawzip::ZipArchiveEntryWayfinder,
    is_text: bool,
}
impl<'a> Eu4SliceZip<'a> {
    pub fn deserialize_entry<T, RES>(
        &self,
        entry: rawzip::ZipArchiveEntryWayfinder,
        resolver: RES,
    ) -> Result<T, Eu4Error>
    where
        T: DeserializeOwned,
        RES: TokenResolver,
    {
        let zip_entry = self.archive.get_entry(entry).map_err(Eu4ErrorKind::Zip)?;
        match self.compression {
            rawzip::CompressionMethod::Deflate => {
                let inflater = flate2::read::DeflateDecoder::new(zip_entry.data());
                let verifier = rawzip::ZipVerifier::new(inflater);

                let mut modeller = Eu4Modeller::from_reader(verifier, &resolver);
                let data: T = modeller.deserialize()?;
                let verifier = modeller.into_inner();

                let claim = verifier.verification_claim();
                zip_entry.verify_claim(claim).map_err(Eu4ErrorKind::Zip)?;
                Ok(data)
            }
            rawzip::CompressionMethod::Zstd => {
                let inflater = zstd::Decoder::new(zip_entry.data())?;
                let verifier = rawzip::ZipVerifier::new(inflater);

                let mut modeller = Eu4Modeller::from_reader(verifier, &resolver);
                let data: T = modeller.deserialize()?;
                let verifier = modeller.into_inner();

                let claim = verifier.verification_claim();
                zip_entry.verify_claim(claim).map_err(Eu4ErrorKind::Zip)?;
                Ok(data)
            }
            _ => return Err(Eu4ErrorKind::UnknownCompression.into()),
        }
    }
}

struct Eu4FsZip {
    archive: rawzip::ZipArchive<rawzip::FileReader>,
    compression: rawzip::CompressionMethod,
    meta: rawzip::ZipArchiveEntryWayfinder,
    gamestate: rawzip::ZipArchiveEntryWayfinder,
    ai: rawzip::ZipArchiveEntryWayfinder,
    is_text: bool,
}

impl Eu4FsZip {
    pub fn deserialize_entry<T, RES>(
        &self,
        entry: rawzip::ZipArchiveEntryWayfinder,
        resolver: RES,
    ) -> Result<T, Eu4Error>
    where
        T: DeserializeOwned,
        RES: TokenResolver,
    {
        let zip_entry = self.archive.get_entry(entry).map_err(Eu4ErrorKind::Zip)?;
        match self.compression {
            rawzip::CompressionMethod::Deflate => {
                let inflater = flate2::read::DeflateDecoder::new(zip_entry.reader());
                let verifier = rawzip::ZipVerifier::new(inflater);

                let mut modeller = Eu4Modeller::from_reader(verifier, &resolver);
                let data: T = modeller.deserialize()?;
                let verifier = modeller.into_inner();

                let claim = verifier.verification_claim();
                let reader = verifier.into_inner().into_inner();
                reader.verify_claim(claim).map_err(Eu4ErrorKind::Zip)?;
                Ok(data)
            }
            rawzip::CompressionMethod::Zstd => {
                let inflater = zstd::Decoder::new(zip_entry.reader())?;
                let verifier = rawzip::ZipVerifier::new(inflater);

                let mut modeller = Eu4Modeller::from_reader(verifier, &resolver);
                let data: T = modeller.deserialize()?;
                let verifier = modeller.into_inner();

                let claim = verifier.verification_claim();
                let reader = verifier.into_inner().finish().into_inner();
                reader.verify_claim(claim).map_err(Eu4ErrorKind::Zip)?;
                Ok(data)
            }
            _ => return Err(Eu4ErrorKind::UnknownCompression.into()),
        }
    }
}

enum Eu4FsFileKind {
    Text(File),
    Binary(File),
    Zip(Eu4FsZip),
}

pub struct Eu4FsFile {
    kind: Eu4FsFileKind,
}

impl Eu4FsFile {
    pub fn encoding(&self) -> Encoding {
        match &self.kind {
            Eu4FsFileKind::Text(_) => Encoding::Text,
            Eu4FsFileKind::Binary(_) => Encoding::Binary,
            Eu4FsFileKind::Zip(archive) => {
                if archive.is_text {
                    Encoding::TextZip
                } else {
                    Encoding::BinaryZip
                }
            }
        }
    }

    pub fn parse_save<R>(&self, resolver: R) -> Result<Eu4Save, Eu4Error>
    where
        R: TokenResolver,
    {
        match &self.kind {
            Eu4FsFileKind::Text(file) => {
                let reader = jomini::text::TokenReader::new(file);
                let deserializer = TextDeserializer::from_windows1252_reader(reader);
                let mut deserializer = Eu4TextDeserializer {
                    deser: deserializer,
                };
                deserializer.deserialize()
            }
            Eu4FsFileKind::Binary(file) => {
                let mut deserializer = BinaryDeserializer::builder_flavor(Eu4Flavor::new())
                    .from_reader(file, &resolver);
                let result = deserializer.deserialize()?;
                Ok(result)
            }
            Eu4FsFileKind::Zip(archive) => {
                let meta: Meta = archive.deserialize_entry(archive.meta, &resolver)?;
                let game: GameState = archive.deserialize_entry(archive.gamestate, &resolver)?;
                Ok(Eu4Save { meta, game })
            }
        }
    }
}

/// Deserializes binary data into custom structures
pub struct Eu4TextDeserializer<R> {
    pub(crate) deser: TextReaderDeserializer<R, jomini::Windows1252Encoding>,
}

impl<R: Read> Eu4TextDeserializer<R> {
    pub fn deserialize<'de, T>(&mut self) -> Result<T, Eu4Error>
    where
        T: Deserialize<'de>,
    {
        T::deserialize(self)
    }
}

pub struct Eu4BinaryDeserializer<'res, RES, R> {
    pub(crate) deser: BinaryReaderDeserializer<'res, RES, Eu4Flavor, R>,
}

impl<'de, 'res: 'de, RES: TokenResolver, R> Eu4BinaryDeserializer<'res, RES, R>
where
    R: Read,
{
    pub fn deserialize<T>(&mut self) -> Result<T, Eu4Error>
    where
        T: Deserialize<'de>,
    {
        T::deserialize(self)
    }
}

#[derive(Debug)]
pub struct Eu4Modeller<'res, Reader, Resolver> {
    reader: Reader,
    resolver: &'res Resolver,
}

impl<'res, Reader: Read, Resolver: TokenResolver> Eu4Modeller<'res, Reader, Resolver> {
    pub fn from_reader(reader: Reader, resolver: &'res Resolver) -> Self {
        Eu4Modeller { reader, resolver }
    }

    pub fn deserialize<T>(&mut self) -> Result<T, Eu4Error>
    where
        T: DeserializeOwned,
    {
        T::deserialize(self)
    }

    pub fn into_inner(self) -> Reader {
        self.reader
    }
}

impl<'de, 'res: 'de, Reader: Read, Resolver: TokenResolver> serde::de::Deserializer<'de>
    for &'_ mut Eu4Modeller<'res, Reader, Resolver>
{
    type Error = Eu4Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Eu4Error::new(Eu4ErrorKind::DeserializeImpl {
            msg: String::from("only struct supported"),
        }))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut header = [0u8; BIN_HEADER.len()];
        self.reader.read_exact(&mut header)?;
        if header == BIN_HEADER {
            use jomini::binary::BinaryFlavor;
            // self.encoding = Encoding::Binary;
            let flavor = Eu4Flavor::new();
            let mut deser = flavor
                .deserializer()
                .from_reader(&mut self.reader, self.resolver);
            Ok(deser.deserialize_struct(name, fields, visitor)?)
        } else if header == TXT_HEADER {
            // self.encoding = Encoding::Text;
            let reader = jomini::text::TokenReader::new(&mut self.reader);
            let mut deser = TextDeserializer::from_windows1252_reader(reader);
            Ok(deser.deserialize_struct(name, fields, visitor)?)
        } else {
            Err(Eu4ErrorKind::UnknownHeader.into())
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map enum identifier ignored_any
    }
}
