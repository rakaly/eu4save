use jomini::{
    binary::ng::{
        BinaryConfig, BinaryTokenFormat, BinaryValueFormat, FieldId, FieldResolver, ParserState,
        PdxVisitor, TokenResult, ValueResult,
    },
    binary::{BinaryFlavor, FailedResolveStrategy, LexemeId},
    Encoding, Error, Windows1252Encoding,
};
use serde::de::Error as _;
use std::borrow::Cow;

/// The eu4 binary flavor
#[derive(Debug, Default)]
pub struct Eu4Flavor(Windows1252Encoding);

impl Eu4Flavor {
    /// Creates a new eu4 flavor
    pub fn new() -> Self {
        Eu4Flavor(Windows1252Encoding::new())
    }
}

impl Encoding for Eu4Flavor {
    fn decode<'a>(&self, data: &'a [u8]) -> std::borrow::Cow<'a, str> {
        // Heuristic to detect chinese escaped strings
        if matches!(data.first(), Some(0x10..=0x13)) {
            std::borrow::Cow::Owned(decode_eu4_escaped_text(data))
        } else {
            self.0.decode(data)
        }
    }
}

impl BinaryFlavor for Eu4Flavor {
    fn visit_f32(&self, data: [u8; 4]) -> f32 {
        // First encoding is an i32 that has a fixed point offset of 3 decimal digits
        i32::from_le_bytes(data) as f32 / 1000.0
    }

    fn visit_f64(&self, data: [u8; 8]) -> f64 {
        // Second encoding is Q49.15 with 5 fractional digits
        // https://en.wikipedia.org/wiki/Q_(number_format)
        let val = i64::from_le_bytes(data) as f64 / 32768.0;
        (val * 10_0000.0).round() / 10_0000.0
    }
}

/// Converts the EU4 chinese encoding into a utf-8 string
///
/// This function was converted from the original C++ code:
/// https://github.com/matanki-saito/EU4dll/blob/4b5e5e16ec09c6977f1c96dabc7e6bab16590b02/Plugin64/escape_tool.cpp
///
/// The author describes the encoding as: "Escaped Text -> wide char (ucs2) -> UTF 8"
#[cold]
pub fn decode_eu4_escaped_text(mut input: &[u8]) -> String {
    const ELLIPSIS: u32 = '…' as u32;
    let mut wide_chars = Vec::with_capacity(input.len());

    while let Some((&cp, rest)) = input.split_first() {
        input = rest;
        let code_point = match cp {
            0x10..=0x13 => {
                match input.split_first_chunk::<2>() {
                    None => ELLIPSIS,
                    Some(([low, high], rest)) => {
                        input = rest;
                        let mut sp = (u32::from(*high) << 8) + u32::from(*low);

                        // Apply escape transformations
                        sp = match cp {
                            0x10 => sp,
                            0x11 => sp.saturating_sub(0xE),
                            0x12 => sp.saturating_add(0x900),
                            0x13 => sp.saturating_add(0x8F2),
                            _ => sp,
                        };

                        if sp > 0xFFFF {
                            ELLIPSIS
                        } else {
                            sp
                        }
                    }
                }
            }
            _ => cp1252_to_ucs2(cp),
        };

        wide_chars.push(code_point as u16);
    }

    String::from_utf16_lossy(&wide_chars)
}

/// Converts a CP1252 byte to its UCS-2 equivalent
fn cp1252_to_ucs2(cp: u8) -> u32 {
    match cp {
        0x80 => 0x20AC,     // Euro
        0x82 => 0x201A,     // Single low-9 quotation
        0x83 => 0x0192,     // Latin small f with hook
        0x84 => 0x201E,     // Double low-9 quotation
        0x85 => 0x2026,     // Horizontal ellipsis
        0x86 => 0x2020,     // Dagger
        0x87 => 0x2021,     // Double dagger
        0x88 => 0x02C6,     // Modifier letter circumflex
        0x89 => 0x2030,     // Per mille
        0x8A => 0x0160,     // Latin capital S with caron
        0x8B => 0x2039,     // Single left-pointing angle quotation
        0x8C => 0x0152,     // Latin capital ligature OE
        0x8E => 0x017D,     // Latin capital Z with caron
        0x91 => 0x2018,     // Left single quotation
        0x92 => 0x2019,     // Right single quotation
        0x93 => 0x201C,     // Left double quotation
        0x94 => 0x201D,     // Right double quotation
        0x95 => 0x2022,     // Bullet
        0x96 => 0x2013,     // En dash
        0x97 => 0x2014,     // Em dash
        0x98 => 0x02DC,     // Small tilde
        0x99 => 0x2122,     // Trade mark
        0x9A => 0x0161,     // Latin small s with caron
        0x9B => 0x203A,     // Single right-pointing angle quotation
        0x9C => 0x0153,     // Latin small ligature oe
        0x9E => 0x017E,     // Latin small z with caron
        0x9F => 0x0178,     // Latin capital Y with diaeresis
        _ => u32::from(cp), // Default: use the code point as-is
    }
}

fn eu4_scalar<'a>(data: &'a [u8]) -> Result<Cow<'a, str>, Error> {
    if matches!(data.first(), Some(0x10..=0x13)) {
        Ok(Cow::Owned(decode_eu4_escaped_text(data)))
    } else {
        Ok(Windows1252Encoding::decode(data))
    }
}

fn resolve_name<'de, V, RES>(
    field: FieldId,
    visitor: V,
    config: &BinaryConfig<RES>,
) -> Result<ValueResult<V::Value, V>, Error>
where
    V: PdxVisitor<'de>,
    RES: FieldResolver,
{
    match config.field_resolver().resolve_field(field) {
        Some(name) => Ok(ValueResult::Value(visitor.visit_str(name)?)),
        None => match config.failed_resolve_strategy() {
            FailedResolveStrategy::Error => Err(Error::custom(format!(
                "unknown field token 0x{:x}",
                field.value()
            ))),
            FailedResolveStrategy::Stringify => Ok(ValueResult::Value(
                visitor.visit_string(format!("0x{:x}", field.value()))?,
            )),
            FailedResolveStrategy::Ignore => Ok(ValueResult::Value(
                visitor.visit_str("__internal_identifier_ignore")?,
            )),
        },
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Eu4Token<'a> {
    Open,
    Close,
    Equal,
    Field(FieldId),
    Bool(bool),
    U32(u32),
    I32(i32),
    Quoted(&'a [u8]),
    Unquoted(&'a [u8]),
    F32([u8; 4]),
    F64([u8; 8]),
}

#[derive(Default)]
pub(crate) struct Eu4Format;

impl Eu4Format {
    fn decode_f32(raw: [u8; 4]) -> f32 {
        i32::from_le_bytes(raw) as f32 / 1000.0
    }

    fn decode_f64(raw: [u8; 8]) -> f64 {
        let val = i64::from_le_bytes(raw) as f64 / 32768.0;
        (val * 100_000.0).round() / 100_000.0
    }

    fn deserialize_str<'de, V: PdxVisitor<'de>, RES: FieldResolver>(
        &mut self,
        reader: &mut ParserState,
        visitor: V,
        config: &BinaryConfig<RES>,
    ) -> Result<ValueResult<<V as PdxVisitor<'de>>::Value, V>, Error> {
        let mut cursor = reader.token_cursor();
        let Some(id) = cursor.read_lexeme() else {
            return Ok(ValueResult::MoreData(visitor));
        };
        if !matches!(id, LexemeId::QUOTED | LexemeId::UNQUOTED) {
            return self.deserialize_any(reader, visitor, config);
        }
        let Some(data) = cursor.read_len_prefixed() else {
            return Ok(ValueResult::MoreData(visitor));
        };
        cursor.consume();
        let value = match self.decode_scalar(data)? {
            Cow::Borrowed(x) => visitor.visit_str(x)?,
            Cow::Owned(x) => visitor.visit_string(x)?,
        };
        Ok(ValueResult::Value(value))
    }
}

impl BinaryTokenFormat for Eu4Format {
    type Token<'a> = Eu4Token<'a>;

    fn next_token<'a>(
        &mut self,
        reader: &'a mut ParserState,
    ) -> Result<TokenResult<Self::Token<'a>>, Error> {
        let mut cursor = reader.token_cursor();
        let Some(id) = cursor.read_lexeme() else {
            return Ok(TokenResult::MoreData);
        };
        match id {
            LexemeId::OPEN => {
                cursor.consume();
                Ok(TokenResult::Token(Eu4Token::Open))
            }
            LexemeId::CLOSE => {
                cursor.consume();
                Ok(TokenResult::Token(Eu4Token::Close))
            }
            LexemeId::EQUAL => {
                cursor.consume();
                Ok(TokenResult::Token(Eu4Token::Equal))
            }
            LexemeId::BOOL => {
                let Some(bytes) = cursor.read_bytes::<1>().copied() else {
                    return Ok(TokenResult::MoreData);
                };
                cursor.consume();
                Ok(TokenResult::Token(Eu4Token::Bool(bytes[0] != 0)))
            }
            LexemeId::U32 => {
                let Some(bytes) = cursor.read_bytes::<4>().copied() else {
                    return Ok(TokenResult::MoreData);
                };
                cursor.consume();
                Ok(TokenResult::Token(Eu4Token::U32(u32::from_le_bytes(bytes))))
            }
            LexemeId::I32 => {
                let Some(bytes) = cursor.read_bytes::<4>().copied() else {
                    return Ok(TokenResult::MoreData);
                };
                cursor.consume();
                Ok(TokenResult::Token(Eu4Token::I32(i32::from_le_bytes(bytes))))
            }
            LexemeId::QUOTED | LexemeId::UNQUOTED => {
                let Some(data) = cursor.read_len_prefixed() else {
                    return Ok(TokenResult::MoreData);
                };

                cursor.consume();
                if id == LexemeId::QUOTED {
                    Ok(TokenResult::Token(Eu4Token::Quoted(data)))
                } else {
                    Ok(TokenResult::Token(Eu4Token::Unquoted(data)))
                }
            }
            LexemeId::F32 => {
                let Some(bytes) = cursor.read_bytes::<4>().copied() else {
                    return Ok(TokenResult::MoreData);
                };
                cursor.consume();
                Ok(TokenResult::Token(Eu4Token::F32(bytes)))
            }
            LexemeId::F64 => {
                let Some(bytes) = cursor.read_bytes::<8>().copied() else {
                    return Ok(TokenResult::MoreData);
                };
                cursor.consume();
                Ok(TokenResult::Token(Eu4Token::F64(bytes)))
            }
            id => {
                cursor.consume();
                Ok(TokenResult::Token(Eu4Token::Field(FieldId::new(id.0))))
            }
        }
    }

    fn skip_value(
        &mut self,
        state: &mut ParserState,
        fill: &mut impl FnMut(&mut ParserState) -> Result<usize, Error>,
    ) -> Result<(), Error> {
        // Phase 1: consume the first token. Scalars return immediately;
        // OPEN breaks into the container scan below.
        loop {
            let mut cursor = state.token_cursor();
            let Some(id) = cursor.read_lexeme() else {
                if fill(state)? == 0 {
                    return Err(Error::eof());
                }
                continue;
            };

            match id {
                LexemeId::OPEN => {
                    cursor.consume();
                    break;
                }
                LexemeId::CLOSE => {
                    cursor.consume();
                    return Ok(());
                }
                LexemeId::BOOL => {
                    if cursor.read_bytes::<1>().is_some() {
                        cursor.consume();
                        return Ok(());
                    }

                    if fill(state)? == 0 {
                        return Err(Error::eof());
                    }
                    continue;
                }
                LexemeId::I32 | LexemeId::F32 | LexemeId::U32 => {
                    if cursor.read_bytes::<4>().is_some() {
                        cursor.consume();
                        return Ok(());
                    }

                    if fill(state)? == 0 {
                        return Err(Error::eof());
                    }
                    continue;
                }
                LexemeId::F64 => {
                    if cursor.read_bytes::<8>().is_some() {
                        cursor.consume();
                        return Ok(());
                    }

                    if fill(state)? == 0 {
                        return Err(Error::eof());
                    }
                    continue;
                }
                LexemeId::QUOTED | LexemeId::UNQUOTED => {
                    if cursor.read_len_prefixed().is_none() {
                        if fill(state)? == 0 {
                            return Err(Error::eof());
                        }
                        continue;
                    }
                    cursor.consume();
                    return Ok(());
                }
                _ => {
                    // field ID or EQUAL: 2 bytes
                    cursor.consume();
                    return Ok(());
                }
            }
        }

        // Phase 2: container scan — scalars are skipped without any depth
        // check; only OPEN/CLOSE affect depth.
        let mut depth: usize = 1;
        loop {
            let mut cursor = state.token_cursor();
            let committed = loop {
                let committed = cursor.checkpoint();
                let Some(id) = cursor.read_lexeme() else {
                    break committed;
                };

                match id {
                    LexemeId::CLOSE => {
                        depth -= 1;
                        if depth == 0 {
                            cursor.consume();
                            return Ok(());
                        }
                    }
                    LexemeId::OPEN => {
                        depth += 1;
                    }
                    LexemeId::BOOL => {
                        if cursor.read_bytes::<1>().is_none() {
                            break committed;
                        }
                    }
                    LexemeId::I32 | LexemeId::F32 | LexemeId::U32 => {
                        if cursor.read_bytes::<4>().is_none() {
                            break committed;
                        }
                    }
                    LexemeId::F64 => {
                        if cursor.read_bytes::<8>().is_none() {
                            break committed;
                        }
                    }
                    LexemeId::QUOTED | LexemeId::UNQUOTED => {
                        if cursor.read_len_prefixed().is_none() {
                            break committed;
                        }
                    }
                    _ => {
                        // field ID or EQUAL: only the lexeme bytes are consumed.
                    }
                }
            };

            cursor.consume_to(committed);
            if fill(state)? == 0 {
                return Err(Error::eof());
            }
        }
    }
}

impl BinaryValueFormat for Eu4Format {
    fn decode_scalar<'a>(&self, data: &'a [u8]) -> Result<Cow<'a, str>, Error> {
        eu4_scalar(data)
    }

    fn deserialize_bool<'de, V: PdxVisitor<'de>, RES: FieldResolver>(
        &mut self,
        reader: &mut ParserState,
        visitor: V,
        config: &BinaryConfig<RES>,
    ) -> Result<ValueResult<V::Value, V>, Error> {
        let mut cursor = reader.token_cursor();
        let Some(id) = cursor.read_lexeme() else {
            return Ok(ValueResult::MoreData(visitor));
        };
        if id != LexemeId::BOOL {
            return self.deserialize_any(reader, visitor, config);
        }
        let Some(bytes) = cursor.read_bytes::<1>().copied() else {
            return Ok(ValueResult::MoreData(visitor));
        };
        cursor.consume();
        Ok(ValueResult::Value(visitor.visit_bool(bytes[0] != 0)?))
    }

    fn deserialize_u32<'de, V: PdxVisitor<'de>, RES: FieldResolver>(
        &mut self,
        reader: &mut ParserState,
        visitor: V,
        config: &BinaryConfig<RES>,
    ) -> Result<ValueResult<V::Value, V>, Error> {
        let mut cursor = reader.token_cursor();
        let Some(id) = cursor.read_lexeme() else {
            return Ok(ValueResult::MoreData(visitor));
        };
        if id != LexemeId::U32 {
            return self.deserialize_any(reader, visitor, config);
        }
        let Some(bytes) = cursor.read_bytes::<4>().copied() else {
            return Ok(ValueResult::MoreData(visitor));
        };
        cursor.consume();
        Ok(ValueResult::Value(
            visitor.visit_u32(u32::from_le_bytes(bytes))?,
        ))
    }

    fn deserialize_i32<'de, V: PdxVisitor<'de>, RES: FieldResolver>(
        &mut self,
        reader: &mut ParserState,
        visitor: V,
        config: &BinaryConfig<RES>,
    ) -> Result<ValueResult<V::Value, V>, Error> {
        let mut cursor = reader.token_cursor();
        let Some(id) = cursor.read_lexeme() else {
            return Ok(ValueResult::MoreData(visitor));
        };
        if id != LexemeId::I32 {
            return self.deserialize_any(reader, visitor, config);
        }
        let Some(bytes) = cursor.read_bytes::<4>().copied() else {
            return Ok(ValueResult::MoreData(visitor));
        };
        cursor.consume();
        Ok(ValueResult::Value(
            visitor.visit_i32(i32::from_le_bytes(bytes))?,
        ))
    }

    fn deserialize_f32<'de, V: PdxVisitor<'de>, RES: FieldResolver>(
        &mut self,
        reader: &mut ParserState,
        visitor: V,
        config: &BinaryConfig<RES>,
    ) -> Result<ValueResult<V::Value, V>, Error> {
        let mut cursor = reader.token_cursor();
        let Some(id) = cursor.read_lexeme() else {
            return Ok(ValueResult::MoreData(visitor));
        };
        if id != LexemeId::F32 {
            return self.deserialize_any(reader, visitor, config);
        }
        let Some(bytes) = cursor.read_bytes::<4>().copied() else {
            return Ok(ValueResult::MoreData(visitor));
        };
        cursor.consume();
        Ok(ValueResult::Value(
            visitor.visit_f32(Self::decode_f32(bytes))?,
        ))
    }

    fn deserialize_f64<'de, V: PdxVisitor<'de>, RES: FieldResolver>(
        &mut self,
        reader: &mut ParserState,
        visitor: V,
        config: &BinaryConfig<RES>,
    ) -> Result<ValueResult<V::Value, V>, Error> {
        let mut cursor = reader.token_cursor();
        let Some(id) = cursor.read_lexeme() else {
            return Ok(ValueResult::MoreData(visitor));
        };
        if id != LexemeId::F64 {
            return self.deserialize_any(reader, visitor, config);
        }
        let Some(bytes) = cursor.read_bytes::<8>().copied() else {
            return Ok(ValueResult::MoreData(visitor));
        };
        cursor.consume();
        Ok(ValueResult::Value(
            visitor.visit_f64(Self::decode_f64(bytes))?,
        ))
    }

    fn deserialize_str<'de, V: PdxVisitor<'de>, RES: FieldResolver>(
        &mut self,
        reader: &mut ParserState,
        visitor: V,
        config: &BinaryConfig<RES>,
    ) -> Result<ValueResult<V::Value, V>, Error> {
        self.deserialize_str(reader, visitor, config)
    }

    fn deserialize_identifier<'de, V: PdxVisitor<'de>, RES: FieldResolver>(
        &mut self,
        reader: &mut ParserState,
        visitor: V,
        config: &BinaryConfig<RES>,
    ) -> Result<ValueResult<V::Value, V>, Error> {
        let mut cursor = reader.token_cursor();
        let Some(id) = cursor.read_lexeme() else {
            return Ok(ValueResult::MoreData(visitor));
        };
        let field = FieldId::new(id.0);
        if let Some(name) = config.field_resolver().resolve_field(field) {
            cursor.consume();
            return Ok(ValueResult::Value(visitor.visit_str(name)?));
        }
        if matches!(id, LexemeId::QUOTED | LexemeId::UNQUOTED) {
            self.deserialize_str(reader, visitor, config)
        } else {
            self.deserialize_any(reader, visitor, config)
        }
    }

    fn deserialize_any<'de, V: PdxVisitor<'de>, RES: FieldResolver>(
        &mut self,
        reader: &mut ParserState,
        visitor: V,
        config: &BinaryConfig<RES>,
    ) -> Result<ValueResult<V::Value, V>, Error> {
        let mut cursor = reader.token_cursor();
        let Some(id) = cursor.read_lexeme() else {
            return Ok(ValueResult::MoreData(visitor));
        };
        match id {
            LexemeId::OPEN => {
                cursor.consume();
                Ok(ValueResult::Open(visitor))
            }
            LexemeId::BOOL => {
                let Some(bytes) = cursor.read_bytes::<1>().copied() else {
                    return Ok(ValueResult::MoreData(visitor));
                };
                cursor.consume();
                Ok(ValueResult::Value(visitor.visit_bool(bytes[0] != 0)?))
            }
            LexemeId::U32 => {
                let Some(bytes) = cursor.read_bytes::<4>().copied() else {
                    return Ok(ValueResult::MoreData(visitor));
                };
                cursor.consume();
                Ok(ValueResult::Value(
                    visitor.visit_u32(u32::from_le_bytes(bytes))?,
                ))
            }
            LexemeId::I32 => {
                let Some(bytes) = cursor.read_bytes::<4>().copied() else {
                    return Ok(ValueResult::MoreData(visitor));
                };
                cursor.consume();
                Ok(ValueResult::Value(
                    visitor.visit_i32(i32::from_le_bytes(bytes))?,
                ))
            }
            LexemeId::QUOTED | LexemeId::UNQUOTED => self.deserialize_str(reader, visitor, config),
            LexemeId::F32 => {
                let Some(bytes) = cursor.read_bytes::<4>().copied() else {
                    return Ok(ValueResult::MoreData(visitor));
                };
                cursor.consume();
                Ok(ValueResult::Value(
                    visitor.visit_f32(Self::decode_f32(bytes))?,
                ))
            }
            LexemeId::F64 => {
                let Some(bytes) = cursor.read_bytes::<8>().copied() else {
                    return Ok(ValueResult::MoreData(visitor));
                };
                cursor.consume();
                Ok(ValueResult::Value(
                    visitor.visit_f64(Self::decode_f64(bytes))?,
                ))
            }
            id => {
                cursor.consume();
                resolve_name(FieldId::new(id.0), visitor, config)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eu4_flavor_f64_rounding() {
        // This test was taken by running an observer game (plaintext)
        // with cloud auto save (binary) and comparing the two and
        // noticing that truncation instead of rounding would yield
        // `2.49859` instead of the expected `2.49860`
        let flavor = Eu4Flavor(Windows1252Encoding::new());
        let data: [u8; 8] = [210, 63, 1, 0, 0, 0, 0, 0];
        let actual = flavor.visit_f64(data);
        assert_eq!(actual, 2.49860);
    }
}
