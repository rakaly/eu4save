use crate::{Eu4Error, Eu4Save, Eu4SaveMeta, GameState, Meta, TokenLookup};
use jomini::{BinaryDeserializer, TextDeserializer, TextTape};
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
pub struct Eu4Extractor {
    extraction: Extraction,
}

impl Default for Eu4Extractor {
    fn default() -> Self {
        Eu4Extractor::new(Extraction::InMemory)
    }
}

impl Eu4Extractor {
    pub fn new(extraction: Extraction) -> Self {
        Eu4Extractor { extraction }
    }

    pub fn extract_meta<R>(&self, mut reader: R) -> Result<(Meta, Encoding), Eu4Error>
    where
        R: Read + Seek,
    {
        let mut header = [0; "EU4txt".len()];
        reader.read_exact(&mut header).map_err(Eu4Error::IoErr)?;

        let mut buffer = Vec::with_capacity(0);
        if is_text(&header).is_some() {
            reader.read_to_end(&mut buffer).map_err(Eu4Error::IoErr)?;
            let meta = TextDeserializer::from_slice(&buffer)?;
            Ok((meta, Encoding::Text))
        } else if is_zip(&header) {
            reader.seek(SeekFrom::Start(0)).map_err(Eu4Error::IoErr)?;
            let mut zip = zip::ZipArchive::new(reader).map_err(Eu4Error::ZipCentralDirectory)?;
            match self.extraction {
                Extraction::InMemory => melt_in_memory(&mut buffer, "meta", &mut zip),

                #[cfg(feature = "mmap")]
                Extraction::MmapTemporaries => melt_with_temporary("meta", &mut zip),
            }
        } else {
            Err(Eu4Error::UnknownHeader)
        }
    }

    pub fn extract_save<R>(&self, mut reader: R) -> Result<(Eu4Save, Encoding), Eu4Error>
    where
        R: Read + Seek,
    {
        let mut header = [0; "EU4txt".len()];
        reader.read_exact(&mut header).map_err(Eu4Error::IoErr)?;

        let mut buffer = Vec::with_capacity(0);
        if is_text(&header).is_some() {
            reader.read_to_end(&mut buffer).map_err(Eu4Error::IoErr)?;
            let tape = TextTape::from_slice(&buffer)?;
            let meta: Meta = TextDeserializer::from_tape(&tape)?;
            let game: GameState = TextDeserializer::from_tape(&tape)?;
            Ok((Eu4Save { meta, game }, Encoding::Text))
        } else if is_zip(&header) {
            reader.seek(SeekFrom::Start(0)).map_err(Eu4Error::IoErr)?;
            let mut zip = zip::ZipArchive::new(reader).map_err(Eu4Error::ZipCentralDirectory)?;
            let (meta, encoding) = match self.extraction {
                Extraction::InMemory => melt_in_memory(&mut buffer, "meta", &mut zip),

                #[cfg(feature = "mmap")]
                Extraction::MmapTemporaries => melt_with_temporary("meta", &mut zip),
            }?;

            let (game, _) = match self.extraction {
                Extraction::InMemory => melt_in_memory(&mut buffer, "gamestate", &mut zip),

                #[cfg(feature = "mmap")]
                Extraction::MmapTemporaries => melt_with_temporary("gamestate", &mut zip),
            }?;

            Ok((Eu4Save { meta, game }, encoding))
        } else {
            Err(Eu4Error::UnknownHeader)
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
        reader.read_exact(&mut header).map_err(Eu4Error::IoErr)?;

        let mut buffer = Vec::with_capacity(0);

        // If we're encountering text then since we have to read through the whole document anyways
        // to extract the metadata we might as well extract the game state too.
        if is_text(&header).is_some() {
            reader.read_to_end(&mut buffer).map_err(Eu4Error::IoErr)?;
            let tape = TextTape::from_slice(&buffer)?;
            let meta: Meta = TextDeserializer::from_tape(&tape)?;
            let game: Option<GameState> = TextDeserializer::from_tape(&tape).map(Some)?;
            Ok((Eu4SaveMeta { meta, game }, Encoding::Text))
        } else if is_zip(&header) {
            reader.seek(SeekFrom::Start(0)).map_err(Eu4Error::IoErr)?;
            let mut zip = zip::ZipArchive::new(reader).map_err(Eu4Error::ZipCentralDirectory)?;
            let (meta, encoding) = match self.extraction {
                Extraction::InMemory => melt_in_memory(&mut buffer, "meta", &mut zip),

                #[cfg(feature = "mmap")]
                Extraction::MmapTemporaries => melt_with_temporary("meta", &mut zip),
            }?;

            Ok((Eu4SaveMeta { meta, game: None }, encoding))
        } else {
            Err(Eu4Error::UnknownHeader)
        }
    }
}

fn melt_in_memory<T, R>(
    mut buffer: &mut Vec<u8>,
    name: &'static str,
    zip: &mut zip::ZipArchive<R>,
) -> Result<(T, Encoding), Eu4Error>
where
    R: Read + Seek,
    T: DeserializeOwned,
{
    buffer.clear();
    let mut zip_file = zip
        .by_name(name)
        .map_err(|e| Eu4Error::ZipMissingEntry(name, e))?;

    // protect against excessively large uncompressed data
    if zip_file.size() > 1024 * 1024 * 200 {
        return Err(Eu4Error::ZipSize(name));
    }

    buffer.reserve(zip_file.size() as usize);
    zip_file
        .read_to_end(&mut buffer)
        .map_err(|e| Eu4Error::ZipExtraction(name, e))?;

    if let Some(data) = is_bin(&buffer) {
        let res = BinaryDeserializer::from_slice(data, TokenLookup).map_err(|e| {
            Eu4Error::Deserialize {
                part: Some(name.to_string()),
                err: e,
            }
        })?;
        Ok((res, Encoding::BinZip))
    } else if let Some(data) = is_text(&buffer) {
        let res = TextDeserializer::from_slice(data)?;
        Ok((res, Encoding::TextZip))
    } else {
        Err(Eu4Error::UnknownHeader)
    }
}

#[cfg(feature = "mmap")]
fn melt_with_temporary<T, R>(
    name: &'static str,
    zip: &mut zip::ZipArchive<R>,
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

    let file = tempfile::tempfile().map_err(Eu4Error::IoErr)?;
    let mut writer = BufWriter::new(file);
    std::io::copy(&mut zip_file, &mut writer).map_err(|e| Eu4Error::ZipExtraction(name, e))?;
    writer.flush().map_err(Eu4Error::IoErr)?;
    let file = writer.into_inner().unwrap();
    let mmap = unsafe {
        memmap::MmapOptions::new()
            .map(&file)
            .map_err(Eu4Error::IoErr)?
    };
    let buffer = &mmap[..];

    if let Some(data) = is_bin(&buffer) {
        let res = BinaryDeserializer::from_slice(data, TokenLookup).map_err(|e| {
            Eu4Error::Deserialize {
                part: Some(name.to_string()),
                err: e,
            }
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
