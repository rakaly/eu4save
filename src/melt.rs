use crate::{tokens::TokenLookup, Eu4Date, Eu4Error, Eu4ErrorKind};
use jomini::{BinaryTape, BinaryToken, FailedResolveStrategy, TokenResolver};
use std::{
    collections::HashSet,
    io::{Cursor, Read, Write},
};

fn melter(
    writer: &mut Vec<u8>,
    unknown_tokens: &mut HashSet<u16>,
    tape: &BinaryTape,
    failed_resolver: FailedResolveStrategy,
    write_checksum: bool,
) -> Result<(), Eu4Error> {
    let mut depth = 0;
    let mut in_objects = Vec::new();
    let mut in_object = 1;
    let mut token_idx = 0;
    let mut known_number = false;
    let mut known_date = false;
    let tokens = tape.tokens();

    while let Some(token) = tokens.get(token_idx) {
        let mut did_change = false;
        if in_object == 1 {
            let depth = match token {
                BinaryToken::End(_) => depth - 1,
                _ => depth,
            };

            for _ in 0..depth {
                writer.push(b' ');
            }
        }

        match token {
            BinaryToken::Object(_) => {
                did_change = true;
                writer.extend_from_slice(b"{\n");
                depth += 1;
                in_objects.push(in_object);
                in_object = 1;
            }
            BinaryToken::HiddenObject(_) => {
                did_change = true;
                depth += 1;
                in_objects.push(in_object);
                in_object = 1;
            }
            BinaryToken::Array(_) => {
                did_change = true;
                writer.push(b'{');
                depth += 1;
                in_objects.push(in_object);
                in_object = 0;
            }
            BinaryToken::End(x) => {
                if !matches!(tokens.get(*x), Some(BinaryToken::HiddenObject(_))) {
                    writer.push(b'}');
                }

                let obj = in_objects.pop();

                // The binary parser should already ensure that this will be something, but this is
                // just a sanity check
                debug_assert!(obj.is_some());
                in_object = obj.unwrap_or(1);
                depth -= 1;
            }
            BinaryToken::Bool(x) => match x {
                true => writer.extend_from_slice(b"yes"),
                false => writer.extend_from_slice(b"no"),
            },
            BinaryToken::U32(x) => writer.extend_from_slice(format!("{}", x).as_bytes()),
            BinaryToken::U64(x) => writer.extend_from_slice(format!("{}", x).as_bytes()),
            BinaryToken::I32(x) => {
                if known_number {
                    writer.extend_from_slice(format!("{}", x).as_bytes());
                    known_number = false;
                } else if known_date {
                    if let Some(date) = Eu4Date::from_binary(*x) {
                        writer.extend_from_slice(date.game_fmt().as_bytes());
                    } else {
                        return Err(Eu4Error::new(Eu4ErrorKind::InvalidDate(*x)));
                    }
                    known_date = false;
                } else if let Some(date) = Eu4Date::from_binary_heuristic(*x) {
                    writer.extend_from_slice(date.game_fmt().as_bytes());
                } else {
                    writer.extend_from_slice(format!("{}", x).as_bytes());
                }
            }
            BinaryToken::Quoted(x) => {
                let data = x.view_data();
                let end_idx = match data.last() {
                    Some(x) if *x == b'\n' => data.len() - 1,
                    Some(_x) => data.len(),
                    None => data.len(),
                };

                // quoted fields occuring as keys should remain unquoted
                if in_object == 1 {
                    writer.extend_from_slice(&data[..end_idx]);
                } else {
                    writer.push(b'"');
                    writer.extend_from_slice(&data[..end_idx]);
                    writer.push(b'"');
                }
            }
            BinaryToken::Unquoted(x) => {
                let data = x.view_data();
                writer.extend_from_slice(&data);
            }
            BinaryToken::F32(x) => write!(writer, "{:.3}", x).map_err(Eu4ErrorKind::IoErr)?,
            BinaryToken::F64(x) => write!(writer, "{:.5}", x).map_err(Eu4ErrorKind::IoErr)?,
            BinaryToken::Token(x) => match TokenLookup.resolve(*x) {
                Some(id)
                    if (id == "is_ironman" || (id == "checksum" && !write_checksum))
                        && in_object == 1 =>
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
                    known_number =
                        in_object == 1 && (id == "random" || id.ends_with("seed") || id == "id");
                    known_date = in_object == 1 && id == "date_built";
                    writer.extend_from_slice(&id.as_bytes())
                }
                None => {
                    unknown_tokens.insert(*x);
                    match failed_resolver {
                        FailedResolveStrategy::Error => {
                            return Err(Eu4ErrorKind::UnknownToken { token_id: *x }.into());
                        }
                        FailedResolveStrategy::Ignore if in_object == 1 => {
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
                            let unknown = format!("__unknown_0x{:x}", x);
                            writer.extend_from_slice(unknown.as_bytes());
                        }
                    }
                }
            },
            BinaryToken::Rgb(color) => {
                writer.extend_from_slice(b"rgb {");
                writer.extend_from_slice(format!("{} ", color.r).as_bytes());
                writer.extend_from_slice(format!("{} ", color.g).as_bytes());
                writer.extend_from_slice(format!("{}", color.b).as_bytes());
                writer.push(b'}');
            }
        }

        if !did_change && in_object == 1 {
            writer.push(b'=');
            in_object = 2;
        } else if in_object == 2 {
            in_object = 1;
            writer.push(b'\n');
        } else if in_object != 1 {
            writer.push(b' ');
        }

        token_idx += 1;
    }

    Ok(())
}

fn melt_zip(
    mut out: &mut Vec<u8>,
    unknown_tokens: &mut HashSet<u16>,
    zip_data: &[u8],
    failed_resolver: FailedResolveStrategy,
) -> Result<(), Eu4Error> {
    let mut inflated_data = Vec::new();

    let zip_reader = Cursor::new(&zip_data);
    let mut zip = zip::ZipArchive::new(zip_reader).map_err(Eu4ErrorKind::ZipCentralDirectory)?;

    // Pre-allocate enough data in the inflated data based on the uncompressed size of the ironman
    // data
    let size = zip
        .by_name("gamestate")
        .map_err(|e| Eu4ErrorKind::ZipMissingEntry("gamestate", e))
        .map(|x| x.size())?;
    out.reserve((size as usize) * 2);

    for file in &["meta", "gamestate", "ai"] {
        inflated_data.clear();
        let mut zip_file = zip
            .by_name(file)
            .map_err(|e| Eu4ErrorKind::ZipMissingEntry(file, e))?;

        zip_file
            .read_to_end(&mut inflated_data)
            .map_err(|e| Eu4ErrorKind::ZipExtraction(file, e))?;

        let tape = BinaryTape::from_eu4(&inflated_data["EU4bin".len()..]).map_err(|e| {
            Eu4ErrorKind::Deserialize {
                part: Some(file.to_string()),
                err: e,
            }
        })?;

        let write_checksum = file == &"ai";
        melter(
            &mut out,
            unknown_tokens,
            &tape,
            failed_resolver,
            write_checksum,
        )?;
    }

    Ok(())
}

/// Convert ironman data to plaintext
pub fn melt(
    data: &[u8],
    failed_resolver: FailedResolveStrategy,
) -> Result<(Vec<u8>, HashSet<u16>), Eu4Error> {
    let mut out: Vec<u8> = b"EU4txt\n".to_vec();
    let mut unknown_tokens = HashSet::new();

    let is_zip = data
        .get(..4)
        .map_or(false, |x| x == &[0x50, 0x4b, 0x03, 0x04][..]);

    if is_zip {
        melt_zip(&mut out, &mut unknown_tokens, &data, failed_resolver)?;
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
        let tape = BinaryTape::from_eu4(&data[cut_header_len..])?;
        melter(&mut out, &mut unknown_tokens, &tape, failed_resolver, true)?;
    }

    Ok((out, unknown_tokens))
}

#[cfg(all(test, ironman_tokens))]
mod tests {
    use super::*;

    #[test]
    fn test_short_input_regression() {
        // Make sure it doesn't crash
        let _ = melt(&[], FailedResolveStrategy::Error);
    }

    #[test]
    fn test_rgb_regression() {
        let data = [
            45, 2, 1, 0, 1, 137, 1, 45, 1, 0, 67, 2, 0, 255, 255, 255, 255, 226, 2, 1, 0, 1, 137,
            1, 45, 1, 56, 226, 1, 255, 255, 255, 255, 255,
        ];
        let _ = melt(&data, FailedResolveStrategy::Ignore);
    }

    #[test]
    fn test_ironman_nonscalar() {
        let data = [137, 53, 3, 0, 4, 0];
        let expected = b"EU4txt\n";
        let out = melt(&data[..], FailedResolveStrategy::Error).unwrap();
        assert_eq!(out, &expected[..]);
    }

    #[test]
    fn test_melt_meta() {
        let meta = include_bytes!("../tests/fixtures/meta.bin");
        let expected = include_bytes!("../tests/fixtures/meta.bin.melted");
        let out = melt(&meta[..], FailedResolveStrategy::Error).unwrap();
        assert_eq!(out, &expected[..]);
    }

    #[test]
    fn test_melt_skip_ironman() {
        let data = [
            0x45, 0x55, 0x34, 0x62, 0x69, 0x6e, 0x4d, 0x28, 0x01, 0x00, 0x0c, 0x00, 0x70, 0x98,
            0x8d, 0x03, 0x89, 0x35, 0x01, 0x00, 0x0e, 0x00, 0x01, 0x38, 0x2a, 0x01, 0x00, 0x0f,
            0x00, 0x03, 0x00, 0x42, 0x48, 0x41,
        ];

        let expected = b"EU4txt\ndate=1804.12.09\nplayer=\"BHA\"\n";
        let out = melt(&data[..], FailedResolveStrategy::Error).unwrap();
        assert_eq!(out, &expected[..]);
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

        let expected = b"EU4txt\nflags={\n schools_initiated=\"1444.11.11\"\n}\n";
        let out = melt(&data[..], FailedResolveStrategy::Error).unwrap();
        assert_eq!(out, &expected[..]);
    }

    #[test]
    fn test_melt_skip_unknown_key() {
        let data = [
            0x45, 0x55, 0x34, 0x62, 0x69, 0x6e, 0xff, 0xff, 0x01, 0x00, 0x0c, 0x00, 0x70, 0x98,
            0x8d, 0x03, 0x89, 0x35, 0x01, 0x00, 0x0e, 0x00, 0x01, 0x38, 0x2a, 0x01, 0x00, 0x0f,
            0x00, 0x03, 0x00, 0x42, 0x48, 0x41,
        ];

        let expected = b"EU4txt\nplayer=\"BHA\"\n";
        let out = melt(&data[..], FailedResolveStrategy::Ignore).unwrap();
        assert_eq!(out, &expected[..]);
    }

    #[test]
    fn test_melt_skip_unknown_value() {
        let data = [
            0x45, 0x55, 0x34, 0x62, 0x69, 0x6e, 0x4d, 0x28, 0x01, 0x00, 0xff, 0xff, 0x89, 0x35,
            0x01, 0x00, 0x0e, 0x00, 0x01, 0x38, 0x2a, 0x01, 0x00, 0x0f, 0x00, 0x03, 0x00, 0x42,
            0x48, 0x41,
        ];

        let expected = b"EU4txt\ndate=__unknown_0xffff\nplayer=\"BHA\"\n";
        let out = melt(&data[..], FailedResolveStrategy::Ignore).unwrap();
        assert_eq!(out, &expected[..]);
    }
}
