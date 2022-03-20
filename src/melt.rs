use crate::{flavor::Eu4Flavor, tokens::TokenLookup, Eu4Date, Eu4Error, Eu4ErrorKind, Extraction};
use jomini::{
    common::PdsDate, BinaryFlavor, BinaryTape, BinaryToken, FailedResolveStrategy,
    TextWriterBuilder, TokenResolver,
};
use std::{
    collections::HashSet,
    io::{Cursor, Read},
};

struct QuoteMode {
    kind: QuoteKind,
    idx: usize,
}

impl QuoteMode {
    fn new() -> Self {
        QuoteMode {
            kind: QuoteKind::Inactive,
            idx: 0,
        }
    }

    fn clear(&mut self) {
        self.kind = QuoteKind::Inactive;
    }
}

#[derive(Debug, Clone, Copy)]
enum QuoteKind {
    // Regular quoting rules
    Inactive,

    // Unquote scalar and containers
    UnquoteAll,

    // Unquote only a scalar value
    UnquoteScalar,

    // Quote only a scalar value
    QuoteScalar,

    // Quote object keys
    ForceQuote,
}

/// Convert ironman data to plaintext
#[derive(Debug)]
pub struct Melter {
    on_failed_resolve: FailedResolveStrategy,
    extraction: Extraction,
    rewrite: bool,
}

impl Default for Melter {
    fn default() -> Self {
        Melter {
            extraction: Extraction::InMemory,
            on_failed_resolve: FailedResolveStrategy::Ignore,
            rewrite: true,
        }
    }
}

impl Melter {
    /// Create a customized version to melt binary data
    pub fn new() -> Self {
        Melter::default()
    }

    /// Set the memory allocation extraction behavior for when a zip is encountered
    pub fn with_extraction(mut self, extraction: Extraction) -> Self {
        self.extraction = extraction;
        self
    }

    /// Set the behavior for when an unresolved binary token is encountered
    pub fn with_on_failed_resolve(mut self, strategy: FailedResolveStrategy) -> Self {
        self.on_failed_resolve = strategy;
        self
    }

    /// Set if the melter should rewrite properties to better match the plaintext format
    ///
    /// Setting to false will preserve binary fields and values even if they
    /// don't make any sense in the plaintext output.
    pub fn with_rewrite(mut self, rewrite: bool) -> Self {
        self.rewrite = rewrite;
        self
    }

    /// Convert ironman data to plaintext
    pub fn melt_with_tokens<Q>(
        &self,
        data: &[u8],
        resolver: &Q,
    ) -> Result<(Vec<u8>, HashSet<u16>), Eu4Error>
    where
        Q: TokenResolver,
    {
        let mut out: Vec<u8> = b"EU4txt\n".to_vec();
        let mut unknown_tokens = HashSet::new();

        let is_zip = data
            .get(..4)
            .map_or(false, |x| x == &[0x50, 0x4b, 0x03, 0x04][..]);

        if is_zip {
            self.melt_zip(&mut out, &mut unknown_tokens, data, resolver)?;
        } else {
            out.reserve(data.len() * 2);
            let cut_header_len = if data
                .get(..b"EU4bin".len())
                .map_or(false, |x| x == &b"EU4bin"[..])
            {
                "EU4bin".len()
            } else {
                0
            };
            let tape = BinaryTape::from_slice(&data[cut_header_len..])?;
            self.convert(&mut out, &mut unknown_tokens, &tape, true, resolver)?;
        }

        Ok((out, unknown_tokens))
    }

    /// Convert ironman data to plaintext
    pub fn melt(&self, data: &[u8]) -> Result<(Vec<u8>, HashSet<u16>), Eu4Error> {
        self.melt_with_tokens(data, &TokenLookup)
    }

    fn convert<Q>(
        &self,
        writer: &mut Vec<u8>,
        unknown_tokens: &mut HashSet<u16>,
        tape: &BinaryTape,
        write_checksum: bool,
        resolver: &Q,
    ) -> Result<(), Eu4Error>
    where
        Q: TokenResolver,
    {
        let mut wtr = TextWriterBuilder::new()
            .indent_char(b'\t')
            .indent_factor(1)
            .from_writer(writer);
        let mut token_idx = 0;
        let mut known_number = false;
        let mut known_date = false;
        let mut quote_mode = QuoteMode::new();
        let mut long_format = false;
        let mut depth = 0;
        let tokens = tape.tokens();
        let flavor = Eu4Flavor::new();

        while let Some(token) = tokens.get(token_idx) {
            match token {
                BinaryToken::Object(_) => {
                    depth += 1;
                    wtr.write_object_start()?;
                }
                BinaryToken::HiddenObject(_) => {
                    depth += 1;
                    wtr.write_hidden_object_start()?;
                }
                BinaryToken::Array(_) => {
                    depth += 1;
                    wtr.write_array_start()?;
                }
                BinaryToken::End(x) => {
                    depth -= 1;

                    if *x == quote_mode.idx {
                        quote_mode.clear();
                    }

                    wtr.write_end()?;
                }
                BinaryToken::Bool(x) => wtr.write_bool(*x)?,
                BinaryToken::U32(x) => wtr.write_u32(*x)?,
                BinaryToken::U64(x) => wtr.write_u64(*x)?,
                BinaryToken::I32(x) => {
                    if known_number {
                        wtr.write_i32(*x)?;
                        known_number = false;
                    } else if known_date {
                        if let Some(date) = Eu4Date::from_binary(*x) {
                            wtr.write_date(date.game_fmt())?;
                        } else if self.on_failed_resolve != FailedResolveStrategy::Error {
                            wtr.write_i32(*x)?;
                        } else {
                            return Err(Eu4Error::new(Eu4ErrorKind::InvalidDate(*x)));
                        }
                        known_date = false;
                    } else if let Some(date) = Eu4Date::from_binary_heuristic(*x) {
                        wtr.write_date(date.game_fmt())?;
                    } else {
                        wtr.write_i32(*x)?;
                    }
                }
                BinaryToken::Quoted(x) => {
                    match quote_mode.kind {
                        QuoteKind::Inactive if wtr.expecting_key() => {
                            wtr.write_unquoted(x.as_bytes())?
                        }
                        QuoteKind::Inactive => wtr.write_quoted(x.as_bytes())?,
                        QuoteKind::ForceQuote => wtr.write_quoted(x.as_bytes())?,
                        QuoteKind::UnquoteAll => wtr.write_unquoted(x.as_bytes())?,
                        QuoteKind::UnquoteScalar if token_idx == quote_mode.idx => {
                            wtr.write_unquoted(x.as_bytes())?
                        }
                        QuoteKind::UnquoteScalar => wtr.write_quoted(x.as_bytes())?,
                        QuoteKind::QuoteScalar if token_idx == quote_mode.idx => {
                            wtr.write_quoted(x.as_bytes())?
                        }
                        QuoteKind::QuoteScalar => wtr.write_unquoted(x.as_bytes())?,
                    };

                    // Clear quote mode after encountering a scalar value
                    if token_idx == quote_mode.idx {
                        quote_mode.clear();
                    }
                }
                BinaryToken::Unquoted(x) => {
                    wtr.write_unquoted(x.as_bytes())?;
                }
                BinaryToken::F32(x) => {
                    let val = flavor.visit_f32(*x);
                    if long_format {
                        write!(&mut wtr, "{:.6}", val)?;
                    } else {
                        write!(&mut wtr, "{:.3}", val)?;
                    }
                }
                BinaryToken::F64(x) => write!(&mut wtr, "{:.5}", flavor.visit_f64(*x))?,
                BinaryToken::Token(x) => match resolver.resolve(*x) {
                    Some(id)
                        if ((self.rewrite && id == "is_ironman")
                            || (id == "checksum" && !write_checksum))
                            && wtr.expecting_key() =>
                    {
                        let skip = tokens
                            .get(token_idx + 1)
                            .map(|next_token| match next_token {
                                BinaryToken::Object(end) => end + 1,
                                BinaryToken::Array(end) => end + 1,
                                _ => token_idx + 2,
                            })
                            .unwrap_or(token_idx + 1);

                        token_idx = skip;
                        continue;
                    }
                    Some(id) => {
                        // There are certain tokens that we know are integers and will dupe the date heuristic
                        known_number = id == "random" || id.ends_with("seed") || id == "id";
                        known_date = id == "date_built";

                        match id {
                            "friend"
                            | "production_leader_tag"
                            | "dynamic_countries"
                            | "electors"
                            | "cores"
                            | "named_unrest"
                            | "claims"
                            | "country_of_origin"
                            | "granted_privileges"
                            | "attackers"
                            | "defenders"
                            | "persistent_attackers"
                            | "persistent_defenders"
                            | "mission_slot"
                            | "votes"
                            | "ruler_flags"
                            | "neighbours"
                            | "home_neighbours"
                            | "core_neighbours"
                            | "call_to_arms_friends"
                            | "allies"
                            | "extended_allies"
                            | "trade_embargoed_by"
                            | "trade_embargoes"
                            | "transfer_trade_power_from"
                            | "friend_tags"
                            | "hidden_flags"
                            | "members"
                            | "colony_claim"
                            | "harsh"
                            | "concilatory"
                            | "current_at_war_with"
                            | "current_war_allies"
                            | "participating_countries"
                            | "subjects"
                            | "support_independence"
                            | "transfer_trade_power_to"
                            | "guarantees"
                            | "warnings"
                            | "flags" => {
                                quote_mode = QuoteMode {
                                    kind: QuoteKind::UnquoteAll,
                                    idx: token_idx + 1,
                                };
                            }
                            _ => {}
                        }

                        if depth == 2
                            && matches!(id, "discovered_by" | "tribal_owner" | "active_disaster")
                        {
                            quote_mode = QuoteMode {
                                kind: QuoteKind::UnquoteAll,
                                idx: token_idx + 1,
                            };
                        }

                        match id {
                            "culture_group" | "saved_names" | "tech_level_dates"
                            | "incident_variables" => {
                                quote_mode = QuoteMode {
                                    kind: QuoteKind::ForceQuote,
                                    idx: token_idx + 1,
                                };
                            }
                            "subjects" => {
                                quote_mode = QuoteMode {
                                    kind: QuoteKind::QuoteScalar,
                                    idx: token_idx + 1,
                                };
                            }
                            _ => {}
                        }

                        if depth == 4 && id == "leader" {
                            quote_mode = QuoteMode {
                                kind: QuoteKind::UnquoteScalar,
                                idx: token_idx + 1,
                            }
                        }

                        if depth == 0 && id == "ai" {
                            long_format = true;
                        }

                        wtr.write_unquoted(id.as_bytes())?;
                    }
                    None => {
                        unknown_tokens.insert(*x);
                        match self.on_failed_resolve {
                            FailedResolveStrategy::Error => {
                                return Err(Eu4ErrorKind::UnknownToken { token_id: *x }.into());
                            }
                            FailedResolveStrategy::Ignore if wtr.expecting_key() => {
                                let skip = tokens
                                    .get(token_idx + 1)
                                    .map(|next_token| match next_token {
                                        BinaryToken::Object(end) => end + 1,
                                        BinaryToken::Array(end) => end + 1,
                                        _ => token_idx + 2,
                                    })
                                    .unwrap_or(token_idx + 1);

                                token_idx = skip;
                                continue;
                            }
                            _ => {
                                write!(wtr, "__unknown_0x{:x}", x)?;
                            }
                        }
                    }
                },
                BinaryToken::Rgb(color) => {
                    wtr.write_header(b"rgb")?;
                    wtr.write_array_start()?;
                    wtr.write_u32(color.r)?;
                    wtr.write_u32(color.g)?;
                    wtr.write_u32(color.b)?;
                    wtr.write_end()?;
                }
            }

            token_idx += 1;
        }

        Ok(())
    }

    pub fn melt_entries(
        &self,
        metadata: &[u8],
        gamestate: &[u8],
        ai: &[u8],
    ) -> Result<(Vec<u8>, HashSet<u16>), Eu4Error> {
        self.melt_entries_with_tokens(metadata, gamestate, ai, &TokenLookup)
    }

    pub fn melt_entries_with_tokens<Q>(
        &self,
        metadata: &[u8],
        gamestate: &[u8],
        ai: &[u8],
        resolver: &Q,
    ) -> Result<(Vec<u8>, HashSet<u16>), Eu4Error>
    where
        Q: TokenResolver,
    {
        let mut out = Vec::with_capacity(gamestate.len());
        out.extend_from_slice(b"EU4txt\n");

        let mut unknown_tokens = HashSet::new();
        self.melt_entry(
            &mut out,
            &mut unknown_tokens,
            metadata,
            resolver,
            "metadata",
        )?;
        self.melt_entry(
            &mut out,
            &mut unknown_tokens,
            gamestate,
            resolver,
            "gamestate",
        )?;
        self.melt_entry(&mut out, &mut unknown_tokens, ai, resolver, "ai")?;
        Ok((out, unknown_tokens))
    }

    fn melt_entry<Q>(
        &self,
        out: &mut Vec<u8>,
        unknown_tokens: &mut HashSet<u16>,
        inflated_data: &[u8],
        resolver: &Q,
        file: &str,
    ) -> Result<(), Eu4Error>
    where
        Q: TokenResolver,
    {
        let tape = BinaryTape::from_slice(&inflated_data["EU4bin".len()..]).map_err(|e| {
            Eu4ErrorKind::Deserialize {
                part: Some(file.to_string()),
                err: e,
            }
        })?;

        let write_checksum = file == "ai";
        self.convert(out, unknown_tokens, &tape, write_checksum, resolver)
    }

    fn melt_zip<Q>(
        &self,
        out: &mut Vec<u8>,
        unknown_tokens: &mut HashSet<u16>,
        zip_data: &[u8],
        resolver: &Q,
    ) -> Result<(), Eu4Error>
    where
        Q: TokenResolver,
    {
        let zip_reader = Cursor::new(&zip_data);
        let mut zip =
            zip::ZipArchive::new(zip_reader).map_err(Eu4ErrorKind::ZipCentralDirectory)?;

        // Pre-allocate enough data in the inflated data based on the uncompressed size of the ironman
        // data
        let size = zip
            .by_name("gamestate")
            .map_err(|e| Eu4ErrorKind::ZipMissingEntry("gamestate", e))
            .map(|x| x.size())?;
        out.reserve((size as usize) * 2);

        for file in &["meta", "gamestate", "ai"] {
            let mut zip_file = zip
                .by_name(file)
                .map_err(|e| Eu4ErrorKind::ZipMissingEntry(file, e))?;

            match self.extraction {
                Extraction::InMemory => {
                    let mut inflated_data: Vec<u8> = Vec::with_capacity(size as usize);
                    zip_file
                        .read_to_end(&mut inflated_data)
                        .map_err(|e| Eu4ErrorKind::ZipExtraction(file, e))?;
                    self.melt_entry(out, unknown_tokens, &inflated_data, resolver, file)?
                }

                #[cfg(feature = "mmap")]
                Extraction::MmapTemporaries => {
                    let mut mmap = memmap::MmapMut::map_anon(zip_file.size() as usize)?;
                    std::io::copy(&mut zip_file, &mut mmap.as_mut())
                        .map_err(|e| Eu4ErrorKind::ZipExtraction(file, e))?;
                    self.melt_entry(out, unknown_tokens, &mmap[..], resolver, file)?
                }
            }
        }

        Ok(())
    }
}

#[cfg(all(test, ironman))]
mod tests {
    use super::*;

    #[test]
    fn test_short_input_regression() {
        // Make sure it doesn't crash
        let _ = Melter::new()
            .with_on_failed_resolve(FailedResolveStrategy::Error)
            .melt(&[]);
    }

    #[test]
    fn test_rgb_regression() {
        let data = [
            45, 2, 1, 0, 1, 137, 1, 45, 1, 0, 67, 2, 0, 255, 255, 255, 255, 226, 2, 1, 0, 1, 137,
            1, 45, 1, 56, 226, 1, 255, 255, 255, 255, 255,
        ];
        let _ = Melter::new()
            .with_on_failed_resolve(FailedResolveStrategy::Ignore)
            .melt(&data);
    }

    #[test]
    fn test_ironman_nonscalar() {
        let data = [137, 53, 3, 0, 4, 0];
        let expected = b"EU4txt\n";
        let (out, _unknown) = Melter::new()
            .with_on_failed_resolve(FailedResolveStrategy::Error)
            .melt(&data)
            .unwrap();
        assert_eq!(out, &expected[..]);
    }

    #[test]
    fn test_melt_meta() {
        let meta = include_bytes!("../tests/it/fixtures/meta.bin");
        let expected = include_bytes!("../tests/it/fixtures/meta.bin.melted");
        let (out, _unknown) = Melter::new()
            .with_on_failed_resolve(FailedResolveStrategy::Error)
            .melt(&meta[..])
            .unwrap();
        assert_eq!(out, &expected[..]);
    }

    #[test]
    fn test_melt_skip_ironman() {
        let data = [
            0x45, 0x55, 0x34, 0x62, 0x69, 0x6e, 0x4d, 0x28, 0x01, 0x00, 0x0c, 0x00, 0x70, 0x98,
            0x8d, 0x03, 0x89, 0x35, 0x01, 0x00, 0x0e, 0x00, 0x01, 0x38, 0x2a, 0x01, 0x00, 0x0f,
            0x00, 0x03, 0x00, 0x42, 0x48, 0x41,
        ];

        let expected = b"EU4txt\ndate=1804.12.9\nplayer=\"BHA\"\n";
        let (out, _unknown) = Melter::new()
            .with_on_failed_resolve(FailedResolveStrategy::Error)
            .melt(&data)
            .unwrap();
        assert_eq!(out, &expected[..]);
    }

    #[test]
    fn test_melt_skip_ironman_in_object() {
        let data = [
            0x45, 0x55, 0x34, 0x62, 0x69, 0x6e, 0x4d, 0x28, 0x01, 0x00, 0x0c, 0x00, 0x70, 0x98,
            0x8d, 0x03, 0x23, 0x2d, 0x01, 0x00, 0x03, 0x00, 0x89, 0x35, 0x01, 0x00, 0x0e, 0x00,
            0x01, 0x04, 0x00, 0x38, 0x2a, 0x01, 0x00, 0x0f, 0x00, 0x03, 0x00, 0x42, 0x48, 0x41,
        ];

        let expected = "EU4txt\ndate=1804.12.9\nimpassable={ }\nplayer=\"BHA\"\n";
        let (out, _unknown) = Melter::new()
            .with_on_failed_resolve(FailedResolveStrategy::Error)
            .melt(&data)
            .unwrap();
        assert_eq!(std::str::from_utf8(&out).unwrap(), &expected[..]);
    }

    #[test]
    fn test_skip_quoting_keys() {
        let mut data = vec![];
        data.extend_from_slice(b"EU4bin");
        data.extend_from_slice(&[0xcc, 0x29, 0x01, 0x00, 0x03, 0x00, 0x0f, 0x00, 0x11, 0x00]);
        data.extend_from_slice(b"schools_initiated");
        data.extend_from_slice(&[0x01, 0x00, 0x0f, 0x00, 0x0b, 0x00]);
        data.extend_from_slice(b"1444.11.11\n");
        data.extend_from_slice(&0x0004u16.to_le_bytes());

        let expected = b"EU4txt\nflags={\n\tschools_initiated=\"1444.11.11\"\n}\n";
        let (out, _unknown) = Melter::new()
            .with_on_failed_resolve(FailedResolveStrategy::Error)
            .melt(&data)
            .unwrap();
        assert_eq!(out, &expected[..]);
    }

    #[test]
    fn test_melt_skip_unknown_key() {
        let data = [
            0x45, 0x55, 0x34, 0x62, 0x69, 0x6e, 0xff, 0xff, 0x01, 0x00, 0x0c, 0x00, 0x70, 0x98,
            0x8d, 0x03, 0x89, 0x35, 0x01, 0x00, 0x0e, 0x00, 0x01, 0x38, 0x2a, 0x01, 0x00, 0x0f,
            0x00, 0x03, 0x00, 0x42, 0x48, 0x41,
        ];

        let expected = "EU4txt\nplayer=\"BHA\"\n";
        let (out, _unknown) = Melter::new()
            .with_on_failed_resolve(FailedResolveStrategy::Ignore)
            .melt(&data)
            .unwrap();
        assert_eq!(std::str::from_utf8(&out).unwrap(), &expected[..]);
    }

    #[test]
    fn test_melt_skip_unknown_value() {
        let data = [
            0x45, 0x55, 0x34, 0x62, 0x69, 0x6e, 0x4d, 0x28, 0x01, 0x00, 0xff, 0xff, 0x89, 0x35,
            0x01, 0x00, 0x0e, 0x00, 0x01, 0x38, 0x2a, 0x01, 0x00, 0x0f, 0x00, 0x03, 0x00, 0x42,
            0x48, 0x41,
        ];

        let expected = "EU4txt\ndate=__unknown_0xffff\nplayer=\"BHA\"\n";
        let (out, _unknown) = Melter::new()
            .with_on_failed_resolve(FailedResolveStrategy::Ignore)
            .melt(&data)
            .unwrap();
        assert_eq!(std::str::from_utf8(&out).unwrap(), &expected[..]);
    }
}
