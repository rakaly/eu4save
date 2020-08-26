use crate::{
    Eu4Error, Eu4ErrorKind, Eu4Save, Eu4SaveMeta, FailedResolveStrategy, GameState, Meta,
    TokenLookup,
};
use jomini::{BinaryDeserializerBuilder, TextDeserializer, TextTape};
use serde::de::DeserializeOwned;
use std::fmt;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Encoding {
    Text,
    TextZip,
    BinZip,
}

impl Encoding {
    pub fn as_str(&self) -> &'static str {
        match self {
            Encoding::Text => "text",
            Encoding::TextZip => "textzip",
            Encoding::BinZip => "binzip",
        }
    }
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Extraction {
    InMemory,
    #[cfg(feature = "mmap")]
    MmapTemporaries,
}

#[derive(Debug, Clone)]
pub struct Eu4ExtractorBuilder {
    extraction: Extraction,
    on_failed_resolve: FailedResolveStrategy,
}

impl Default for Eu4ExtractorBuilder {
    fn default() -> Self {
        Eu4ExtractorBuilder::new()
    }
}

impl Eu4ExtractorBuilder {
    pub fn new() -> Self {
        Eu4ExtractorBuilder {
            extraction: Extraction::InMemory,
            on_failed_resolve: FailedResolveStrategy::Ignore,
        }
    }

    pub fn with_extraction(mut self, extraction: Extraction) -> Self {
        self.extraction = extraction;
        self
    }

    pub fn with_on_failed_resolve(mut self, strategy: FailedResolveStrategy) -> Self {
        self.on_failed_resolve = strategy;
        self
    }

    pub fn build(self) -> Eu4Extractor {
        Eu4Extractor {
            extraction: self.extraction,
            on_failed_resolve: self.on_failed_resolve,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Eu4Extractor {
    extraction: Extraction,
    on_failed_resolve: FailedResolveStrategy,
}

impl Default for Eu4Extractor {
    fn default() -> Self {
        Eu4ExtractorBuilder::new().build()
    }
}

impl Eu4Extractor {
    pub fn extract_meta<R>(&self, mut reader: R) -> Result<(Meta, Encoding), Eu4Error>
    where
        R: Read + Seek,
    {
        let mut header = [0; "EU4txt".len()];
        reader.read_exact(&mut header)?;

        let mut buffer = Vec::with_capacity(0);
        if is_text(&header).is_some() {
            reader.read_to_end(&mut buffer)?;
            let meta = TextDeserializer::from_slice(&buffer)?;
            Ok((meta, Encoding::Text))
        } else if is_zip(&header) {
            reader.seek(SeekFrom::Start(0))?;
            let mut zip =
                zip::ZipArchive::new(reader).map_err(Eu4ErrorKind::ZipCentralDirectory)?;
            match self.extraction {
                Extraction::InMemory => {
                    melt_in_memory(&mut buffer, "meta", &mut zip, self.on_failed_resolve)
                }

                #[cfg(feature = "mmap")]
                Extraction::MmapTemporaries => {
                    melt_with_temporary("meta", &mut zip, self.on_failed_resolve)
                }
            }
        } else {
            Err(Eu4ErrorKind::UnknownHeader.into())
        }
    }

    pub fn extract_save<R>(&self, mut reader: R) -> Result<(Eu4Save, Encoding), Eu4Error>
    where
        R: Read + Seek,
    {
        let mut header = [0; "EU4txt".len()];
        reader.read_exact(&mut header)?;

        let mut buffer = Vec::with_capacity(0);
        if is_text(&header).is_some() {
            reader.read_to_end(&mut buffer)?;
            let tape = TextTape::from_slice(&buffer)?;
            let meta: Meta = TextDeserializer::from_tape(&tape)?;
            let game: GameState = TextDeserializer::from_tape(&tape)?;
            Ok((Eu4Save { meta, game }, Encoding::Text))
        } else if is_zip(&header) {
            reader.seek(SeekFrom::Start(0))?;
            let mut zip =
                zip::ZipArchive::new(reader).map_err(Eu4ErrorKind::ZipCentralDirectory)?;
            let (meta, encoding) = match self.extraction {
                Extraction::InMemory => {
                    melt_in_memory(&mut buffer, "meta", &mut zip, self.on_failed_resolve)
                }

                #[cfg(feature = "mmap")]
                Extraction::MmapTemporaries => {
                    melt_with_temporary("meta", &mut zip, self.on_failed_resolve)
                }
            }?;

            let (game, _) = match self.extraction {
                Extraction::InMemory => {
                    melt_in_memory(&mut buffer, "gamestate", &mut zip, self.on_failed_resolve)
                }

                #[cfg(feature = "mmap")]
                Extraction::MmapTemporaries => {
                    melt_with_temporary("gamestate", &mut zip, self.on_failed_resolve)
                }
            }?;

            Ok((Eu4Save { meta, game }, encoding))
        } else {
            Err(Eu4ErrorKind::UnknownHeader.into())
        }
    }

    // For the times where all you want is the metadata but will accept the game state too save on
    // future needless double parsing.
    pub fn extract_meta_optimistic<R>(
        &self,
        mut reader: R,
    ) -> Result<(Eu4SaveMeta, Encoding), Eu4Error>
    where
        R: Read + Seek,
    {
        let mut header = [0; "EU4txt".len()];
        reader.read_exact(&mut header)?;

        let mut buffer = Vec::with_capacity(0);

        // If we're encountering text then since we have to read through the whole document anyways
        // to extract the metadata we might as well extract the game state too.
        if is_text(&header).is_some() {
            reader.read_to_end(&mut buffer)?;
            let tape = TextTape::from_slice(&buffer)?;
            let meta: Meta = TextDeserializer::from_tape(&tape)?;
            let game: Option<GameState> = TextDeserializer::from_tape(&tape).map(Some)?;
            Ok((Eu4SaveMeta { meta, game }, Encoding::Text))
        } else if is_zip(&header) {
            reader.seek(SeekFrom::Start(0))?;
            let mut zip =
                zip::ZipArchive::new(reader).map_err(Eu4ErrorKind::ZipCentralDirectory)?;
            let (meta, encoding) = match self.extraction {
                Extraction::InMemory => {
                    melt_in_memory(&mut buffer, "meta", &mut zip, self.on_failed_resolve)
                }

                #[cfg(feature = "mmap")]
                Extraction::MmapTemporaries => {
                    melt_with_temporary("meta", &mut zip, self.on_failed_resolve)
                }
            }?;

            Ok((Eu4SaveMeta { meta, game: None }, encoding))
        } else {
            Err(Eu4ErrorKind::UnknownHeader.into())
        }
    }
}

fn melt_in_memory<T, R>(
    mut buffer: &mut Vec<u8>,
    name: &'static str,
    zip: &mut zip::ZipArchive<R>,
    on_failed_resolve: FailedResolveStrategy,
) -> Result<(T, Encoding), Eu4Error>
where
    R: Read + Seek,
    T: DeserializeOwned,
{
    buffer.clear();
    let mut zip_file = zip
        .by_name(name)
        .map_err(|e| Eu4ErrorKind::ZipMissingEntry(name, e))?;

    // protect against excessively large uncompressed data
    if zip_file.size() > 1024 * 1024 * 200 {
        return Err(Eu4ErrorKind::ZipSize(name).into());
    }

    buffer.reserve(zip_file.size() as usize);
    zip_file
        .read_to_end(&mut buffer)
        .map_err(|e| Eu4ErrorKind::ZipExtraction(name, e))?;

    if let Some(data) = is_bin(&buffer) {
        let res = BinaryDeserializerBuilder::new()
            .on_failed_resolve(on_failed_resolve)
            .from_slice(data, TokenLookup)
            .map_err(|e| Eu4ErrorKind::Deserialize {
                part: Some(name.to_string()),
                err: e,
            })?;
        Ok((res, Encoding::BinZip))
    } else if let Some(data) = is_text(&buffer) {
        let res = TextDeserializer::from_slice(data)?;
        Ok((res, Encoding::TextZip))
    } else {
        Err(Eu4ErrorKind::UnknownHeader.into())
    }
}

#[cfg(feature = "mmap")]
fn melt_with_temporary<T, R>(
    name: &'static str,
    zip: &mut zip::ZipArchive<R>,
    on_failed_resolve: FailedResolveStrategy,
) -> Result<(T, Encoding), Eu4Error>
where
    R: Read + Seek,
    T: DeserializeOwned,
{
    use std::io::{BufWriter, Write};

    let mut zip_file = zip
        .by_name(name)
        .map_err(|e| Eu4Error::ZipMissingEntry(name, e))?;

    // protect against excessively large uncompressed data
    if zip_file.size() > 1024 * 1024 * 200 {
        return Err(Eu4Error::ZipSize(name));
    }

    let file = tempfile::tempfile()?;
    let mut writer = BufWriter::new(file);
    std::io::copy(&mut zip_file, &mut writer).map_err(|e| Eu4Error::ZipExtraction(name, e))?;
    writer.flush()?;
    let file = writer.into_inner().unwrap();
    let mmap = unsafe { memmap::MmapOptions::new().map(&file)? };
    let buffer = &mmap[..];

    if let Some(data) = is_bin(&buffer) {
        let res = BinaryDeserializerBuilder::new()
            .on_failed_resolve(on_failed_resolve)
            .from_slice(data, TokenLookup)
            .map_err(|e| Eu4Error::Deserialize {
                part: Some(name.to_string()),
                err: e,
            })?;
        Ok((res, Encoding::BinZip))
    } else if let Some(data) = is_text(&buffer) {
        let res = TextDeserializer::from_slice(data)?;
        Ok((res, Encoding::TextZip))
    } else {
        Err(Eu4Error::UnknownHeader)
    }
}

fn is_text(data: &[u8]) -> Option<&[u8]> {
    let sentry = b"EU4txt";
    if data.get(..sentry.len()).map_or(false, |x| x == sentry) {
        Some(&data[sentry.len()..])
    } else {
        None
    }
}

fn is_bin(data: &[u8]) -> Option<&[u8]> {
    let sentry = b"EU4bin";
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
