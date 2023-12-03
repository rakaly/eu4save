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
                zstd::stream::read::Decoder::with_buffer(data).expect("zstd to initialize"),
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

#[derive(thiserror::Error, Debug)]
pub(crate) enum ZipInflationError {
    #[error("unable to inflate: {msg}")]
    BadData { msg: String },

    #[error("early eof, only able to write {written} bytes")]
    EarlyEof { written: usize },
}

#[cfg(all(feature = "miniz", not(feature = "libdeflate")))]
pub(crate) fn inflate_exact(raw: &[u8], out: &mut [u8]) -> Result<(), ZipInflationError> {
    let inflation = miniz_oxide::inflate::decompress_slice_iter_to_slice(
        out,
        std::iter::once(raw),
        false,
        false,
    );

    match inflation {
        Ok(written) if written == out.len() => Ok(()),
        Ok(written) => Err(ZipInflationError::EarlyEof { written }),
        Err(miniz_oxide::inflate::TINFLStatus::HasMoreOutput) => Ok(()),
        Err(e) => Err(ZipInflationError::BadData {
            msg: format!("{:?}", e),
        }),
    }
}

#[cfg(feature = "libdeflate")]
pub(crate) fn inflate_exact(raw: &[u8], out: &mut [u8]) -> Result<(), ZipInflationError> {
    let inflation = libdeflater::Decompressor::new().deflate_decompress(raw, out);

    match inflation {
        Ok(written) if written == out.len() => Ok(()),
        Ok(written) => Err(ZipInflationError::EarlyEof { written }),
        Err(libdeflater::DecompressionError::InsufficientSpace) => Ok(()),
        Err(e) => Err(ZipInflationError::BadData { msg: e.to_string() }),
    }
}

#[cfg(feature = "zstd")]
pub(crate) fn zstd_inflate(raw: &[u8], out: &mut [u8]) -> Result<(), ZipInflationError> {
    let mut ctx = zstd::zstd_safe::DCtx::create();
    match ctx.decompress(out, raw) {
        Ok(written) if written == out.len() => Ok(()),
        Ok(written) => Err(ZipInflationError::EarlyEof { written }),
        Err(e) => Err(ZipInflationError::BadData {
            msg: zstd::zstd_safe::get_error_name(e).to_string(),
        }),
    }
}
