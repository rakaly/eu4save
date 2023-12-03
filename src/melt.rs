use crate::{
    file::{Eu4Binary, Eu4Text, Eu4Zip},
    flavor::Eu4Flavor,
    Eu4Date, Eu4Error, Eu4ErrorKind,
};
use jomini::{
    binary::{self, BinaryFlavor, FailedResolveStrategy, TokenReader, TokenResolver},
    common::PdsDate,
    TextWriterBuilder,
};
use std::{
    collections::HashSet,
    io::{Read, Write},
};

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

#[derive(Debug, Default)]
struct Quoter {
    queued: Option<QuoteKind>,
    depth: Vec<QuoteKind>,
}

impl Quoter {
    #[inline]
    pub fn push(&mut self) {
        let next = match self.queued.take() {
            Some(x @ QuoteKind::ForceQuote | x @ QuoteKind::UnquoteAll) => x,
            _ => QuoteKind::Inactive,
        };

        self.depth.push(next);
    }

    #[inline]
    pub fn pop(&mut self) {
        let _ = self.depth.pop();
    }

    #[inline]
    pub fn take_scalar(&mut self) -> QuoteKind {
        match self.queued.take() {
            Some(x) => x,
            None => self.depth.last().copied().unwrap_or(QuoteKind::Inactive),
        }
    }

    #[inline]
    fn queue(&mut self, mode: QuoteKind) {
        self.queued = Some(mode);
    }

    #[inline]
    fn clear_queued(&mut self) {
        self.queued = None;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeltOptions {
    skip_checksum: bool,
    verbatim: bool,
    on_failed_resolve: FailedResolveStrategy,
    check_header: bool,
}

impl Default for MeltOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl MeltOptions {
    pub fn new() -> Self {
        Self {
            skip_checksum: false,
            verbatim: false,
            on_failed_resolve: FailedResolveStrategy::Ignore,
            check_header: true,
        }
    }

    pub fn skip_checksum(self, skip_checksum: bool) -> Self {
        MeltOptions {
            skip_checksum,
            ..self
        }
    }

    pub fn verbatim(self, verbatim: bool) -> Self {
        MeltOptions { verbatim, ..self }
    }

    pub fn check_header(self, check_header: bool) -> Self {
        MeltOptions {
            check_header,
            ..self
        }
    }

    pub fn on_failed_resolve(self, on_failed_resolve: FailedResolveStrategy) -> Self {
        MeltOptions {
            on_failed_resolve,
            ..self
        }
    }
}

#[derive(Debug)]
enum MeltInput<'data> {
    Text(Eu4Text<'data>),
    Binary(Eu4Binary<'data>),
    TextStream(crate::DeflateReader<'data>),
    BinaryStream(crate::DeflateReader<'data>),
    Zip(Eu4Zip<'data>),
}

pub struct Eu4Melter<'data> {
    input: MeltInput<'data>,
    options: MeltOptions,
}

impl<'data> From<Eu4Zip<'data>> for Eu4Melter<'data> {
    fn from(value: Eu4Zip<'data>) -> Self {
        Eu4Melter {
            input: MeltInput::Zip(value),
            options: MeltOptions::new(),
        }
    }
}

impl<'data> From<Eu4Text<'data>> for Eu4Melter<'data> {
    fn from(value: Eu4Text<'data>) -> Self {
        Eu4Melter {
            input: MeltInput::Text(value),
            options: MeltOptions::new(),
        }
    }
}

impl<'data> From<Eu4Binary<'data>> for Eu4Melter<'data> {
    fn from(value: Eu4Binary<'data>) -> Self {
        Eu4Melter {
            input: MeltInput::Binary(value),
            options: MeltOptions::new(),
        }
    }
}

impl<'data> Eu4Melter<'data> {
    pub fn from_reader(stream: crate::DeflateReader<'data>, is_text: bool) -> Self {
        if is_text {
            Eu4Melter {
                input: MeltInput::TextStream(stream),
                options: MeltOptions::new(),
            }
        } else {
            Eu4Melter {
                input: MeltInput::BinaryStream(stream),
                options: MeltOptions::new(),
            }
        }
    }

    pub fn verbatim(&mut self, verbatim: bool) -> &mut Self {
        self.options = self.options.verbatim(verbatim);
        self
    }

    pub fn on_failed_resolve(&mut self, on_failed_resolve: FailedResolveStrategy) -> &mut Self {
        self.options = self.options.on_failed_resolve(on_failed_resolve);
        self
    }

    pub fn melt<Writer, Resolver>(
        &mut self,
        mut output: Writer,
        resolver: Resolver,
    ) -> Result<MeltedDocument, Eu4Error>
    where
        Writer: Write,
        Resolver: TokenResolver,
    {
        match &mut self.input {
            MeltInput::Text(x) => {
                output.write_all(b"EU4txt\n")?;
                output.write_all(x.data())?;
                Ok(MeltedDocument::new())
            }
            MeltInput::TextStream(ref mut x) => {
                std::io::copy(x, &mut output)?;
                Ok(MeltedDocument::new())
            }
            MeltInput::Binary(x) => {
                output.write_all(b"EU4txt\n")?;
                let result = melt(x.data(), output, resolver, self.options.check_header(false))?;
                Ok(result)
            }
            MeltInput::BinaryStream(x) => {
                output.write_all(b"EU4txt\n")?;
                let result = melt(x, output, resolver, self.options)?;
                Ok(result)
            }
            MeltInput::Zip(zip) => {
                if zip.is_text() {
                    let meta = zip.meta_file()?;
                    std::io::copy(&mut meta.reader(), &mut output)?;

                    let mut header = [0u8; 7];
                    let gamestate = zip.gamestate_file()?;
                    let mut reader = gamestate.reader();
                    reader.read_exact(&mut header[..])?;
                    std::io::copy(&mut reader, &mut output)?;

                    let ai = zip.ai_file()?;
                    let mut reader = ai.reader();
                    reader.read_exact(&mut header[..])?;
                    std::io::copy(&mut reader, &mut output)?;

                    Ok(MeltedDocument::new())
                } else {
                    output.write_all(b"EU4txt\n")?;
                    let meta = zip.meta_file()?;
                    let meta_result = melt(
                        meta.reader(),
                        &mut output,
                        &resolver,
                        self.options.skip_checksum(true),
                    )?;

                    let gamestate = zip.gamestate_file()?;
                    let gamestate_result = melt(
                        gamestate.reader(),
                        &mut output,
                        &resolver,
                        self.options.skip_checksum(true),
                    )?;

                    let ai = zip.ai_file()?;
                    let ai_result = melt(
                        ai.reader(),
                        &mut output,
                        &resolver,
                        self.options.skip_checksum(false),
                    )?;

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

#[derive(Debug, Default)]
pub struct MeltedDocument {
    unknown_tokens: HashSet<u16>,
}

impl MeltedDocument {
    pub fn new() -> Self {
        Self::default()
    }

    /// The list of unknown tokens that the provided resolver accumulated
    pub fn unknown_tokens(&self) -> &HashSet<u16> {
        &self.unknown_tokens
    }
}

fn melt<Reader, Writer, Resolver>(
    input: Reader,
    output: Writer,
    resolver: Resolver,
    options: MeltOptions,
) -> Result<MeltedDocument, Eu4Error>
where
    Reader: Read,
    Writer: Write,
    Resolver: TokenResolver,
{
    let mut reader = TokenReader::new(input);
    if options.check_header && reader.read_bytes(6)? != b"EU4bin" {
        return Err(Eu4Error::new(Eu4ErrorKind::UnknownHeader));
    }

    let mut wtr = TextWriterBuilder::new()
        .indent_char(b'\t')
        .indent_factor(1)
        .from_writer(output);
    let flavor = Eu4Flavor::new();
    let mut unknown_tokens: HashSet<u16> = HashSet::new();
    let skip_checksum = options.skip_checksum;
    let verbatim = options.verbatim;
    let on_failed_resolve = options.on_failed_resolve;

    let mut quoter = Quoter::default();
    let mut known_number = false;
    let mut known_date = false;
    let mut long_format = false;
    while let Some(token) = reader.next()? {
        match token {
            jomini::binary::Token::Open => {
                quoter.push();
                wtr.write_array_start()?
            }
            jomini::binary::Token::Close => {
                quoter.pop();
                wtr.write_end()?
            }
            jomini::binary::Token::Equal => wtr.write_operator(jomini::text::Operator::Equal)?,
            jomini::binary::Token::U32(x) => wtr.write_u32(x)?,
            jomini::binary::Token::U64(x) => wtr.write_u64(x)?,
            jomini::binary::Token::I32(x) => {
                if known_number {
                    wtr.write_i32(x)?;
                    known_number = false;
                } else if known_date {
                    if let Some(date) = Eu4Date::from_binary(x) {
                        wtr.write_date(date.game_fmt())?;
                    } else if on_failed_resolve != FailedResolveStrategy::Error {
                        wtr.write_i32(x)?;
                    } else {
                        return Err(Eu4Error::new(Eu4ErrorKind::InvalidDate(x)));
                    }
                    known_date = false;
                } else if let Some(date) = Eu4Date::from_binary_heuristic(x) {
                    wtr.write_date(date.game_fmt())?;
                } else {
                    wtr.write_i32(x)?;
                }
            }
            jomini::binary::Token::Bool(x) => wtr.write_bool(x)?,
            jomini::binary::Token::Unquoted(x) => wtr.write_unquoted(x.as_bytes())?,
            jomini::binary::Token::Quoted(x) => match quoter.take_scalar() {
                QuoteKind::Inactive if wtr.expecting_key() => wtr.write_unquoted(x.as_bytes())?,
                QuoteKind::Inactive => wtr.write_quoted(x.as_bytes())?,
                QuoteKind::ForceQuote => wtr.write_quoted(x.as_bytes())?,
                QuoteKind::UnquoteAll => wtr.write_unquoted(x.as_bytes())?,
                QuoteKind::UnquoteScalar => wtr.write_unquoted(x.as_bytes())?,
                QuoteKind::QuoteScalar => wtr.write_quoted(x.as_bytes())?,
            },
            jomini::binary::Token::F32(x) if long_format => {
                write!(&mut wtr, "{:.6}", flavor.visit_f32(x))?
            }
            jomini::binary::Token::F32(x) => write!(&mut wtr, "{:.3}", flavor.visit_f32(x))?,
            jomini::binary::Token::F64(x) => write!(&mut wtr, "{:.5}", flavor.visit_f64(x))?,
            jomini::binary::Token::Rgb(x) => wtr.write_rgb(&x)?,
            jomini::binary::Token::I64(x) => wtr.write_i64(x)?,
            jomini::binary::Token::Id(x) => match resolver.resolve(x) {
                Some(id) => {
                    if (id == "checksum" && skip_checksum) || (id == "is_ironman" && !verbatim) {
                        let mut next = reader.read()?;
                        if matches!(next, binary::Token::Equal) {
                            next = reader.read()?;
                        }

                        if matches!(next, binary::Token::Open) {
                            reader.skip_container()?;
                        }
                        continue;
                    }

                    quoter.clear_queued();

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
                        | "flags" => quoter.queue(QuoteKind::UnquoteAll),
                        _ => {}
                    }

                    if wtr.depth() == 2
                        && matches!(id, "discovered_by" | "tribal_owner" | "active_disaster")
                    {
                        quoter.queue(QuoteKind::UnquoteAll);
                    }

                    match id {
                        "culture_group" | "saved_names" | "tech_level_dates"
                        | "incident_variables" => {
                            quoter.queue(QuoteKind::ForceQuote);
                        }
                        "subjects" => {
                            quoter.queue(QuoteKind::QuoteScalar);
                        }
                        _ => {}
                    }

                    if wtr.depth() == 4 && id == "leader" {
                        quoter.queue(QuoteKind::UnquoteScalar);
                    }

                    if wtr.depth() == 0 && id == "ai" {
                        long_format = true;
                    }

                    wtr.write_unquoted(id.as_bytes())?;
                }
                None => match on_failed_resolve {
                    FailedResolveStrategy::Error => {
                        return Err(Eu4Error::new(Eu4ErrorKind::UnknownToken { token_id: x }))
                    }
                    FailedResolveStrategy::Ignore if wtr.expecting_key() => {
                        let mut next = reader.read()?;
                        if matches!(next, binary::Token::Equal) {
                            next = reader.read()?;
                        }

                        if matches!(next, binary::Token::Open) {
                            reader.skip_container()?;
                        }
                    }
                    _ => {
                        unknown_tokens.insert(x);
                        write!(wtr, "__unknown_0x{:x}", x)?;
                    }
                },
            },
        }
    }

    Ok(MeltedDocument { unknown_tokens })
}

#[cfg(all(test, ironman))]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::{EnvTokens, Eu4File};

    #[test]
    fn test_melt_meta() {
        let meta = include_bytes!("../tests/it/fixtures/meta.bin");
        let expected = include_bytes!("../tests/it/fixtures/meta.bin.melted");
        let file = Eu4File::from_slice(&meta[..]).unwrap();
        let mut out = Cursor::new(Vec::new());
        file.melter()
            .on_failed_resolve(FailedResolveStrategy::Error)
            .melt(&mut out, &EnvTokens)
            .unwrap();
        assert_eq!(out.into_inner().as_slice(), &expected[..]);
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
        let mut out = Cursor::new(Vec::new());
        file.melter()
            .on_failed_resolve(FailedResolveStrategy::Error)
            .melt(&mut out, &EnvTokens)
            .unwrap();
        assert_eq!(out.into_inner().as_slice(), &expected[..]);
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
        let mut out = Cursor::new(Vec::new());
        file.melter()
            .on_failed_resolve(FailedResolveStrategy::Error)
            .melt(&mut out, &EnvTokens)
            .unwrap();
        assert_eq!(
            std::str::from_utf8(out.into_inner().as_slice()).unwrap(),
            &expected[..]
        );
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
        let mut out = Cursor::new(Vec::new());
        file.melter()
            .on_failed_resolve(FailedResolveStrategy::Error)
            .melt(&mut out, &EnvTokens)
            .unwrap();
        assert_eq!(
            std::str::from_utf8(out.into_inner().as_slice()).unwrap(),
            &expected[..]
        );
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
        let mut out = Cursor::new(Vec::new());
        file.melter()
            .on_failed_resolve(FailedResolveStrategy::Ignore)
            .melt(&mut out, &EnvTokens)
            .unwrap();
        assert_eq!(
            std::str::from_utf8(out.into_inner().as_slice()).unwrap(),
            &expected[..]
        );
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
        let mut out = Cursor::new(Vec::new());
        file.melter()
            .on_failed_resolve(FailedResolveStrategy::Ignore)
            .melt(&mut out, &EnvTokens)
            .unwrap();
        assert_eq!(
            std::str::from_utf8(out.into_inner().as_slice()).unwrap(),
            &expected[..]
        );
    }
}
