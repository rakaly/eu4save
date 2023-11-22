use crate::{flavor::Eu4Flavor, Eu4Date, Eu4Error, Eu4ErrorKind};
use jomini::{
    binary::{BinaryFlavor, FailedResolveStrategy, TokenResolver},
    common::PdsDate,
    BinaryTape, BinaryToken, Scalar, TextWriterBuilder,
};
use std::{collections::HashSet, io::Cursor};

#[derive(Debug)]
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

/// Convert a binary save to plaintext
pub struct Eu4Melter<'a, 'b> {
    tape: &'b BinaryTape<'a>,
    verbatim: bool,
    on_failed_resolve: FailedResolveStrategy,
}

impl<'a, 'b> Eu4Melter<'a, 'b> {
    pub(crate) fn new(tape: &'b BinaryTape<'a>) -> Self {
        Eu4Melter {
            tape,
            verbatim: false,
            on_failed_resolve: FailedResolveStrategy::Ignore,
        }
    }

    pub fn verbatim(&mut self, verbatim: bool) -> &mut Self {
        self.verbatim = verbatim;
        self
    }

    pub fn on_failed_resolve(&mut self, strategy: FailedResolveStrategy) -> &mut Self {
        self.on_failed_resolve = strategy;
        self
    }

    pub(crate) fn tokens_len(&self) -> usize {
        self.tape.tokens().len()
    }

    pub(crate) fn get_token(&self, idx: usize) -> Option<&BinaryToken> {
        self.tape.tokens().get(idx)
    }

    pub(crate) fn skip_value_idx(&self, token_idx: usize) -> usize {
        self.get_token(token_idx + 1)
            .map(|next_token| match next_token {
                BinaryToken::Object(end) | BinaryToken::Array(end) => end + 1,
                _ => token_idx + 2,
            })
            .unwrap_or(token_idx + 1)
    }

    pub fn melt<R>(&self, resolver: &R) -> Result<MeltedDocument, Eu4Error>
    where
        R: TokenResolver,
    {
        let out = melt(self, resolver).map_err(|e| match e {
            MelterError::Write(x) => Eu4ErrorKind::Writer(x),
            MelterError::UnknownToken { token_id } => Eu4ErrorKind::UnknownToken { token_id },
            MelterError::InvalidDate(x) => Eu4ErrorKind::InvalidDate(x),
        })?;
        Ok(out)
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum MelterError {
    #[error("{0}")]
    Write(#[from] jomini::Error),

    #[error("")]
    UnknownToken { token_id: u16 },

    #[error("")]
    InvalidDate(i32),
}

/// Output from melting a binary save to plaintext
pub struct MeltedDocument {
    data: Vec<u8>,
    unknown_tokens: HashSet<u16>,
}

impl MeltedDocument {
    /// The converted plaintext data
    pub fn into_data(self) -> Vec<u8> {
        self.data
    }

    /// The converted plaintext data
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// The list of unknown tokens that the provided resolver accumulated
    pub fn unknown_tokens(&self) -> &HashSet<u16> {
        &self.unknown_tokens
    }
}

pub(crate) fn melt<R>(melter: &Eu4Melter, resolver: &R) -> Result<MeltedDocument, MelterError>
where
    R: TokenResolver,
{
    let mut out = Vec::with_capacity(melter.tokens_len() * 10);
    out.extend_from_slice(b"EU4txt\n");
    let mut unknown_tokens = HashSet::new();
    let pos = out.len() as u64;
    let mut writer = Cursor::new(out);
    writer.set_position(pos);

    let mut wtr = TextWriterBuilder::new()
        .indent_char(b'\t')
        .indent_factor(1)
        .from_writer(writer);
    let mut token_idx = 0;
    let mut known_number = false;
    let mut known_date = false;
    let mut quote_mode = QuoteMode::new();
    let mut long_format = false;
    let mut queued_checksum: Option<Scalar> = None;
    let flavor = Eu4Flavor::new();

    while let Some(token) = melter.get_token(token_idx) {
        match token {
            BinaryToken::End(x) => {
                if *x == quote_mode.idx {
                    quote_mode.clear();
                }

                wtr.write_end()?;
            }
            BinaryToken::I32(x) => {
                if known_number {
                    wtr.write_i32(*x)?;
                    known_number = false;
                } else if known_date {
                    if let Some(date) = Eu4Date::from_binary(*x) {
                        wtr.write_date(date.game_fmt())?;
                    } else if melter.on_failed_resolve != FailedResolveStrategy::Error {
                        wtr.write_i32(*x)?;
                    } else {
                        return Err(MelterError::InvalidDate(*x));
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
                Some(id) => {
                    let skip = (id == "is_ironman" && !melter.verbatim) || id == "checksum";
                    if skip && wtr.expecting_key() {
                        let next = melter.get_token(token_idx + 1);
                        if id == "checksum" {
                            if let Some(BinaryToken::Quoted(s)) = next {
                                queued_checksum = Some(*s);
                            }
                        };

                        token_idx = melter.skip_value_idx(token_idx);
                        continue;
                    }

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

                    if wtr.depth() == 2
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

                    if wtr.depth() == 4 && id == "leader" {
                        quote_mode = QuoteMode {
                            kind: QuoteKind::UnquoteScalar,
                            idx: token_idx + 1,
                        }
                    }

                    if wtr.depth() == 0 && id == "ai" {
                        long_format = true;
                    }

                    wtr.write_unquoted(id.as_bytes())?;
                }
                None => match melter.on_failed_resolve {
                    FailedResolveStrategy::Error => {
                        return Err(MelterError::UnknownToken { token_id: *x });
                    }
                    FailedResolveStrategy::Ignore if wtr.expecting_key() => {
                        token_idx = melter.skip_value_idx(token_idx);
                        continue;
                    }
                    _ => {
                        unknown_tokens.insert(*x);
                        write!(wtr, "__unknown_0x{:x}", x)?;
                    }
                },
            },

            x => wtr.write_binary(x)?,
        }

        token_idx += 1;
    }

    if let Some(checksum) = queued_checksum.take() {
        wtr.write_unquoted(b"checksum")?;
        wtr.write_quoted(checksum.as_bytes())?;
    }

    Ok(MeltedDocument {
        data: wtr.into_inner().into_inner(),
        unknown_tokens,
    })
}

#[cfg(all(test, ironman))]
mod tests {
    use super::*;
    use crate::{EnvTokens, Eu4File};

    #[test]
    fn test_short_input_regression() {
        // Make sure it doesn't crash
        let tape = BinaryTape::from_slice(&[]).unwrap();
        let _ = Eu4Melter::new(&tape)
            .on_failed_resolve(FailedResolveStrategy::Error)
            .melt(&EnvTokens)
            .unwrap();
    }

    #[test]
    fn test_ironman_nonscalar() {
        let data = [137, 53, 3, 0, 4, 0];
        let tape = BinaryTape::from_slice(&data).unwrap();
        let expected = b"EU4txt\n";
        let out = Eu4Melter::new(&tape)
            .on_failed_resolve(FailedResolveStrategy::Error)
            .melt(&EnvTokens)
            .unwrap();
        assert_eq!(out.data(), &expected[..]);
    }

    #[test]
    fn test_melt_meta() {
        let meta = include_bytes!("../tests/it/fixtures/meta.bin");
        let expected = include_bytes!("../tests/it/fixtures/meta.bin.melted");
        let file = Eu4File::from_slice(&meta[..]).unwrap();
        let mut zip_sink = Vec::new();
        let parsed_file = file.parse(&mut zip_sink).unwrap();
        let binary = parsed_file.as_binary().unwrap();
        let out = binary
            .melter()
            .on_failed_resolve(FailedResolveStrategy::Error)
            .melt(&EnvTokens)
            .unwrap();
        assert_eq!(out.data(), &expected[..]);
    }

    #[test]
    fn test_melt_skip_ironman() {
        let data = [
            0x45, 0x55, 0x34, 0x62, 0x69, 0x6e, 0x4d, 0x28, 0x01, 0x00, 0x0c, 0x00, 0x70, 0x98,
            0x8d, 0x03, 0x89, 0x35, 0x01, 0x00, 0x0e, 0x00, 0x01, 0x38, 0x2a, 0x01, 0x00, 0x0f,
            0x00, 0x03, 0x00, 0x42, 0x48, 0x41,
        ];

        let expected = b"EU4txt\ndate=1804.12.9\nplayer=\"BHA\"";
        let file = Eu4File::from_slice(&data).unwrap();
        let mut zip_sink = Vec::new();
        let parsed_file = file.parse(&mut zip_sink).unwrap();
        let binary = parsed_file.as_binary().unwrap();
        let out = binary
            .melter()
            .on_failed_resolve(FailedResolveStrategy::Error)
            .melt(&EnvTokens)
            .unwrap();

        assert_eq!(out.data(), &expected[..]);
    }

    #[test]
    fn test_melt_skip_ironman_in_object() {
        let data = [
            0x45, 0x55, 0x34, 0x62, 0x69, 0x6e, 0x4d, 0x28, 0x01, 0x00, 0x0c, 0x00, 0x70, 0x98,
            0x8d, 0x03, 0x23, 0x2d, 0x01, 0x00, 0x03, 0x00, 0x89, 0x35, 0x01, 0x00, 0x0e, 0x00,
            0x01, 0x04, 0x00, 0x38, 0x2a, 0x01, 0x00, 0x0f, 0x00, 0x03, 0x00, 0x42, 0x48, 0x41,
        ];

        let expected = "EU4txt\ndate=1804.12.9\nimpassable={ }\nplayer=\"BHA\"";
        let file = Eu4File::from_slice(&data).unwrap();
        let mut zip_sink = Vec::new();
        let parsed_file = file.parse(&mut zip_sink).unwrap();
        let binary = parsed_file.as_binary().unwrap();
        let out = binary
            .melter()
            .on_failed_resolve(FailedResolveStrategy::Error)
            .melt(&EnvTokens)
            .unwrap();

        assert_eq!(std::str::from_utf8(&out.data()).unwrap(), &expected[..]);
    }

    #[test]
    fn test_skip_quoting_flags() {
        let mut data = vec![];
        data.extend_from_slice(b"EU4bin");
        data.extend_from_slice(&[0xcc, 0x29, 0x01, 0x00, 0x03, 0x00, 0x0f, 0x00, 0x11, 0x00]);
        data.extend_from_slice(b"schools_initiated");
        data.extend_from_slice(&[0x01, 0x00, 0x0f, 0x00, 0x0b, 0x00]);
        data.extend_from_slice(b"1444.11.11\n");
        data.extend_from_slice(&0x0004u16.to_le_bytes());

        let expected = "EU4txt\nflags={\n\tschools_initiated=1444.11.11\n\n}";

        let file = Eu4File::from_slice(&data).unwrap();
        let mut zip_sink = Vec::new();
        let parsed_file = file.parse(&mut zip_sink).unwrap();
        let binary = parsed_file.as_binary().unwrap();
        let out = binary
            .melter()
            .on_failed_resolve(FailedResolveStrategy::Error)
            .melt(&EnvTokens)
            .unwrap();

        assert_eq!(std::str::from_utf8(out.data()).unwrap(), &expected[..]);
    }

    #[test]
    fn test_melt_skip_unknown_key() {
        let data = [
            0x45, 0x55, 0x34, 0x62, 0x69, 0x6e, 0xff, 0xff, 0x01, 0x00, 0x0c, 0x00, 0x70, 0x98,
            0x8d, 0x03, 0x89, 0x35, 0x01, 0x00, 0x0e, 0x00, 0x01, 0x38, 0x2a, 0x01, 0x00, 0x0f,
            0x00, 0x03, 0x00, 0x42, 0x48, 0x41,
        ];

        let expected = "EU4txt\nplayer=\"BHA\"";
        let file = Eu4File::from_slice(&data).unwrap();
        let mut zip_sink = Vec::new();
        let parsed_file = file.parse(&mut zip_sink).unwrap();
        let binary = parsed_file.as_binary().unwrap();
        let out = binary
            .melter()
            .on_failed_resolve(FailedResolveStrategy::Ignore)
            .melt(&EnvTokens)
            .unwrap();
        assert_eq!(std::str::from_utf8(out.data()).unwrap(), &expected[..]);
    }

    #[test]
    fn test_melt_skip_unknown_value() {
        let data = [
            0x45, 0x55, 0x34, 0x62, 0x69, 0x6e, 0x4d, 0x28, 0x01, 0x00, 0xff, 0xff, 0x89, 0x35,
            0x01, 0x00, 0x0e, 0x00, 0x01, 0x38, 0x2a, 0x01, 0x00, 0x0f, 0x00, 0x03, 0x00, 0x42,
            0x48, 0x41,
        ];

        let expected = "EU4txt\ndate=__unknown_0xffff\nplayer=\"BHA\"";
        let file = Eu4File::from_slice(&data).unwrap();
        let mut zip_sink = Vec::new();
        let parsed_file = file.parse(&mut zip_sink).unwrap();
        let binary = parsed_file.as_binary().unwrap();
        let out = binary
            .melter()
            .on_failed_resolve(FailedResolveStrategy::Ignore)
            .melt(&EnvTokens)
            .unwrap();
        assert_eq!(std::str::from_utf8(out.data()).unwrap(), &expected[..]);
    }
}
