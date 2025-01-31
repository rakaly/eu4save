//! Parsing and deserializing EU4 save files
use crate::{
    flavor::Eu4Flavor,
    melt,
    models::{Eu4Save, GameState, Meta},
    Encoding, Eu4Error, Eu4ErrorKind, MeltOptions, MeltedDocument,
};
use jomini::{
    binary::{de::BinaryReaderDeserializer, TokenResolver},
    text::{de::TextReaderDeserializer, ObjectReader},
    BinaryDeserializer, TextDeserializer, TextTape, Windows1252Encoding,
};
use rawzip::{CompressionMethod, ReaderAt, ZipVerifier};
use serde::{de::DeserializeOwned, Deserialize};
use std::{
    collections::HashSet,
    fmt::Display,
    fs::File,
    io::{Read, Write},
};

#[cfg(feature = "zstd")]
use std::io::BufReader;

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
                                #[cfg(feature = "zstd")]
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
                    return Err(Eu4ErrorKind::MissingFile(
                        crate::file::Eu4FileEntryName::Gamestate,
                    )
                    .into());
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
                kind: Eu4FsFileKind::Binary(BinaryFile(file)),
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
                                #[cfg(feature = "zstd")]
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
                    return Err(Eu4ErrorKind::MissingFile(
                        crate::file::Eu4FileEntryName::Gamestate,
                    )
                    .into());
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

    pub fn melt<Resolver, Writer>(
        &self,
        options: MeltOptions,
        resolver: Resolver,
        mut output: Writer,
    ) -> Result<MeltedDocument, Eu4Error>
    where
        Resolver: TokenResolver,
        Writer: Write,
    {
        match &self.kind {
            Eu4SliceFileKind::Text(data) => {
                output.write_all(b"EU4txt\n")?;
                output.write_all(data)?;
                Ok(MeltedDocument::new())
            }
            Eu4SliceFileKind::Binary(data) => {
                output.write_all(b"EU4txt\n")?;
                Ok(melt::melt(
                    *data,
                    output,
                    resolver,
                    options.check_header(false),
                )?)
            }
            Eu4SliceFileKind::Zip(zip) => {
                if zip.is_text {
                    let meta = zip.archive.get_entry(zip.meta).map_err(Eu4ErrorKind::Zip)?;
                    let mut reader =
                        CompressedFileReader::from_compressed(meta.data(), zip.compression)?;
                    std::io::copy(&mut reader, &mut output)?;

                    let mut header = [0u8; TXT_HEADER.len() + 1];
                    let gamestate = zip
                        .archive
                        .get_entry(zip.gamestate)
                        .map_err(Eu4ErrorKind::Zip)?;
                    let mut reader =
                        CompressedFileReader::from_compressed(gamestate.data(), zip.compression)?;
                    reader.read_exact(&mut header)?;
                    std::io::copy(&mut reader, &mut output)?;

                    let ai = zip.archive.get_entry(zip.ai).map_err(Eu4ErrorKind::Zip)?;
                    let mut reader =
                        CompressedFileReader::from_compressed(ai.data(), zip.compression)?;
                    reader.read_exact(&mut header)?;
                    std::io::copy(&mut reader, &mut output)?;

                    Ok(MeltedDocument::new())
                } else {
                    output.write_all(b"EU4txt\n")?;
                    let meta = zip.archive.get_entry(zip.meta).map_err(Eu4ErrorKind::Zip)?;
                    let reader =
                        CompressedFileReader::from_compressed(meta.data(), zip.compression)?;
                    let meta_result =
                        melt(reader, &mut output, &resolver, options.skip_checksum(true))?;

                    let gamestate = zip
                        .archive
                        .get_entry(zip.gamestate)
                        .map_err(Eu4ErrorKind::Zip)?;
                    let reader =
                        CompressedFileReader::from_compressed(gamestate.data(), zip.compression)?;
                    let gamestate_result =
                        melt(reader, &mut output, &resolver, options.skip_checksum(true))?;

                    let ai = zip.archive.get_entry(zip.ai).map_err(Eu4ErrorKind::Zip)?;
                    let reader = CompressedFileReader::from_compressed(ai.data(), zip.compression)?;
                    let ai_result =
                        melt(reader, &mut output, &resolver, options.skip_checksum(false))?;

                    let union = meta_result
                        .unknown_tokens
                        .iter()
                        .chain(gamestate_result.unknown_tokens.iter())
                        .chain(ai_result.unknown_tokens.iter())
                        .copied()
                        .collect::<HashSet<u16>>();

                    Ok(MeltedDocument {
                        unknown_tokens: union,
                    })
                }
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
                let inflater = flate2::bufread::DeflateDecoder::new(zip_entry.data());
                let verifier = zip_entry.verifier(inflater);

                let mut modeller = Eu4Modeller::from_reader(verifier, &resolver);
                let data: T = modeller.deserialize()?;
                Ok(data)
            }
            #[cfg(feature = "zstd")]
            rawzip::CompressionMethod::Zstd => {
                let inflater = zstd::Decoder::new(zip_entry.data())?;
                let verifier = zip_entry.verifier(inflater);

                let mut modeller = Eu4Modeller::from_reader(verifier, &resolver);
                let data: T = modeller.deserialize()?;
                Ok(data)
            }
            _ => return Err(Eu4ErrorKind::UnknownCompression.into()),
        }
    }
}

pub struct Eu4FsZipEntry<'archive, R, ReadAt> {
    reader: ZipVerifier<'archive, CompressedFileReader<R>, ReadAt>,
}

impl<'archive, R, ReadAt> Eu4FsZipEntry<'_, R, ReadAt>
where
    R: Read,
    ReadAt: ReaderAt,
{
    pub fn deserialize<T, RES>(&mut self, resolver: RES) -> Result<T, Eu4Error>
    where
        T: DeserializeOwned,
        RES: TokenResolver,
    {
        let mut modeller = Eu4Modeller::from_reader(&mut self.reader, &resolver);
        let data: T = modeller.deserialize()?;
        Ok(data)
    }

    pub fn melt<Resolver, Writer>(
        &mut self,
        options: MeltOptions,
        resolver: Resolver,
        mut output: Writer,
    ) -> Result<MeltedDocument, Eu4Error>
    where
        Resolver: TokenResolver,
        Writer: Write,
    {
        output.write_all(b"EU4txt\n")?;
        melt(
            &mut self.reader,
            output,
            resolver,
            options.skip_checksum(false),
        )
    }
}

impl<'archive, R, ReadAt> Read for Eu4FsZipEntry<'_, R, ReadAt>
where
    R: Read,
    ReadAt: ReaderAt,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

pub struct Eu4FsZip {
    archive: rawzip::ZipArchive<rawzip::FileReader>,
    compression: rawzip::CompressionMethod,
    meta: rawzip::ZipArchiveEntryWayfinder,
    gamestate: rawzip::ZipArchiveEntryWayfinder,
    ai: rawzip::ZipArchiveEntryWayfinder,
    is_text: bool,
}

impl Eu4FsZip {
    pub fn get(
        &self,
        name: crate::file::Eu4FileEntryName,
    ) -> Result<Eu4FsZipEntry<'_, rawzip::ZipReader<'_, rawzip::FileReader>, rawzip::FileReader>, Eu4Error> {
        let entry = match name {
            crate::file::Eu4FileEntryName::Meta => self.meta,
            crate::file::Eu4FileEntryName::Gamestate => self.gamestate,
            crate::file::Eu4FileEntryName::Ai => self.ai,
        };

        let entry = self.archive.get_entry(entry).map_err(Eu4ErrorKind::Zip)?;
        let reader = CompressedFileReader::from_compressed(entry.reader(), self.compression)?;
        let reader = entry.verifier(reader);

        Ok(Eu4FsZipEntry { reader })
    }

    pub fn encoding(&self) -> Encoding {
        if self.is_text {
            Encoding::TextZip
        } else {
            Encoding::BinaryZip
        }
    }

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
        let reader = CompressedFileReader::from_compressed(zip_entry.reader(), self.compression)?;
        let data: T = Eu4Modeller::from_reader(reader, &resolver).deserialize()?;
        Ok(data)
    }
}

pub struct BinaryFile(File);

impl BinaryFile {
    pub fn melt<Resolver, Writer>(
        &mut self,
        options: MeltOptions,
        resolver: Resolver,
        mut output: Writer,
    ) -> Result<MeltedDocument, Eu4Error>
    where
        Resolver: TokenResolver,
        Writer: Write,
    {
        output.write_all(b"EU4txt\n")?;
        melt::melt(&mut self.0, output, resolver, options.check_header(false))
    }
}

pub enum Eu4FsFileKind {
    Text(File),
    Binary(BinaryFile),
    Zip(Eu4FsZip),
}

pub struct Eu4FsFile {
    kind: Eu4FsFileKind,
}

impl Eu4FsFile {
    pub fn kind(&self) -> &Eu4FsFileKind {
        &self.kind
    }

    pub fn kind_mut(&mut self) -> &mut Eu4FsFileKind {
        &mut self.kind
    }

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
                    .from_reader(&file.0, &resolver);
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

    pub fn melt<Resolver, Writer>(
        &mut self,
        options: MeltOptions,
        resolver: Resolver,
        mut output: Writer,
    ) -> Result<MeltedDocument, Eu4Error>
    where
        Resolver: TokenResolver,
        Writer: Write,
    {
        match &mut self.kind {
            Eu4FsFileKind::Text(file) => {
                output.write_all(b"EU4txt\n")?;
                std::io::copy(file, &mut output)?;
                Ok(MeltedDocument::new())
            }
            Eu4FsFileKind::Binary(file) => {
                output.write_all(b"EU4txt\n")?;
                Ok(melt::melt(
                    &file.0,
                    output,
                    resolver,
                    options.check_header(false),
                )?)
            }
            Eu4FsFileKind::Zip(zip) => {
                if zip.is_text {
                    let meta = zip.archive.get_entry(zip.meta).map_err(Eu4ErrorKind::Zip)?;
                    let mut reader =
                        meta.verifier(CompressedFileReader::from_compressed(meta.reader(), zip.compression)?);
                    std::io::copy(&mut reader, &mut output)?;

                    let mut header = [0u8; TXT_HEADER.len() + 1];
                    let gamestate = zip
                        .archive
                        .get_entry(zip.gamestate)
                        .map_err(Eu4ErrorKind::Zip)?;
                    let mut reader =
                        gamestate.verifier(CompressedFileReader::from_compressed(gamestate.reader(), zip.compression)?);
                    reader.read_exact(&mut header)?;
                    std::io::copy(&mut reader, &mut output)?;

                    let ai = zip.archive.get_entry(zip.ai).map_err(Eu4ErrorKind::Zip)?;
                    let mut reader =
                        ai.verifier(CompressedFileReader::from_compressed(ai.reader(), zip.compression)?);
                    reader.read_exact(&mut header)?;
                    std::io::copy(&mut reader, &mut output)?;

                    Ok(MeltedDocument::new())
                } else {
                    output.write_all(b"EU4txt\n")?;
                    let meta = zip.archive.get_entry(zip.meta).map_err(Eu4ErrorKind::Zip)?;
                    let reader =
                        meta.verifier(CompressedFileReader::from_compressed(meta.reader(), zip.compression)?);
                    let meta_result =
                        melt(reader, &mut output, &resolver, options.skip_checksum(true))?;

                    let gamestate = zip
                        .archive
                        .get_entry(zip.gamestate)
                        .map_err(Eu4ErrorKind::Zip)?;
                    let reader =
                        gamestate.verifier(CompressedFileReader::from_compressed(gamestate.reader(), zip.compression)?);
                    let gamestate_result =
                        melt(reader, &mut output, &resolver, options.skip_checksum(true))?;

                    let ai = zip.archive.get_entry(zip.ai).map_err(Eu4ErrorKind::Zip)?;
                    let reader =
                        ai.verifier(CompressedFileReader::from_compressed(ai.reader(), zip.compression)?);
                    let ai_result =
                        melt(reader, &mut output, &resolver, options.skip_checksum(false))?;

                    let union = meta_result
                        .unknown_tokens
                        .iter()
                        .chain(gamestate_result.unknown_tokens.iter())
                        .chain(ai_result.unknown_tokens.iter())
                        .copied()
                        .collect::<HashSet<u16>>();

                    Ok(MeltedDocument {
                        unknown_tokens: union,
                    })
                }
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

enum CompressedReaderKind<R> {
    Deflate(flate2::read::DeflateDecoder<R>),
    #[cfg(feature = "zstd")]
    Zstd(zstd::stream::Decoder<'static, BufReader<R>>),
}

struct CompressedFileReader<R> {
    reader: CompressedReaderKind<R>,
}

impl<R> CompressedFileReader<R> {
    pub fn from_compressed(reader: R, compression: CompressionMethod) -> Result<Self, Eu4Error>
    where
        R: Read,
    {
        match compression {
            CompressionMethod::Deflate => {
                let inflater = flate2::read::DeflateDecoder::new(reader);
                Ok(CompressedFileReader {
                    reader: CompressedReaderKind::Deflate(inflater),
                })
            }
            #[cfg(feature = "zstd")]
            CompressionMethod::Zstd => {
                let inflater = zstd::Decoder::new(reader)?;
                Ok(CompressedFileReader {
                    reader: CompressedReaderKind::Zstd(inflater),
                })
            }
            _ => Err(Eu4ErrorKind::UnknownCompression.into()),
        }
    }
}

impl<R> std::io::Read for CompressedFileReader<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.reader {
            CompressedReaderKind::Deflate(reader) => reader.read(buf),
            #[cfg(feature = "zstd")]
            CompressedReaderKind::Zstd(reader) => reader.read(buf),
        }
    }
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

fn is_text(data: &[u8]) -> Option<&[u8]> {
    let sentry = TXT_HEADER;
    if data.get(..sentry.len()).map_or(false, |x| x == sentry) {
        Some(&data[sentry.len()..])
    } else {
        None
    }
}

/// A parsed EU4 text document
pub struct Eu4ParsedText<'a> {
    tape: TextTape<'a>,
}

impl<'a> Eu4ParsedText<'a> {
    /// Parse EU4 text data that has the "EU4txt" header
    pub fn from_slice(data: &'a [u8]) -> Result<Self, Eu4Error> {
        is_text(data)
            .ok_or_else(|| Eu4ErrorKind::UnknownHeader.into())
            .and_then(Self::from_raw)
    }

    /// Parse headerless EU4 text data
    pub fn from_raw(data: &'a [u8]) -> Result<Self, Eu4Error> {
        let tape = TextTape::from_slice(data).map_err(Eu4ErrorKind::Parse)?;
        Ok(Eu4ParsedText { tape })
    }

    pub fn reader(&self) -> ObjectReader<Windows1252Encoding> {
        self.tape.windows1252_reader()
    }
}
