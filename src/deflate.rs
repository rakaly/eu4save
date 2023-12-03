use std::io::Read;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) enum CompressionMethod {
    Deflate,
    #[cfg(feature = "zstd")]
    Zstd,
}

#[derive(Debug)]
pub struct DeflateReader<'a> {
    kind: DeflateReaderKind<'a>,
}

impl<'a> DeflateReader<'a> {
    pub(crate) fn new(data: &'a [u8], compression: CompressionMethod) -> Self {
        let kind = match compression {
            CompressionMethod::Deflate => {
                DeflateReaderKind::Deflate(flate2::bufread::DeflateDecoder::new(data))
            }
            #[cfg(feature = "zstd")]
            CompressionMethod::Zstd => DeflateReaderKind::Zstd(
                zstd::stream::read::Decoder::with_buffer(data)
                    .expect("zstd stream decoder failed initialization"),
            ),
        };

        DeflateReader { kind }
    }
}

pub(crate) enum DeflateReaderKind<'a> {
    Deflate(flate2::bufread::DeflateDecoder<&'a [u8]>),
    #[cfg(feature = "zstd")]
    Zstd(zstd::stream::read::Decoder<'a, &'a [u8]>),
}

impl<'a> std::fmt::Debug for DeflateReaderKind<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deflate(_) => f.debug_tuple("Deflate").finish(),
            #[cfg(feature = "zstd")]
            Self::Zstd(_) => f.debug_tuple("Zstd").finish(),
        }
    }
}

impl<'a> Read for DeflateReader<'a> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.kind {
            DeflateReaderKind::Deflate(reader) => Ok(reader.read(buf)?),
            #[cfg(feature = "zstd")]
            DeflateReaderKind::Zstd(reader) => Ok(reader.read(buf)?),
        }
    }
}
