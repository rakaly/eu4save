use jomini::{binary::BinaryFlavor, Encoding, Windows1252Encoding};

/// The eu4 binary flavor
#[derive(Debug, Default)]
pub struct Eu4Flavor(Windows1252Encoding);

impl Eu4Flavor {
    /// Creates a new eu4 flavor
    pub fn new() -> Self {
        Eu4Flavor(Windows1252Encoding::new())
    }
}

/// Converts a utf-16 encoded string to a utf-8 encoded string. This is for the
/// Chinese supplementary mod: https://github.com/matanki-saito/EU4dll
#[cold]
fn to_utf16(data: &[u8]) -> String {
    let pairs = data
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]));

    char::decode_utf16(pairs).flat_map(Result::ok).collect()
}

impl Encoding for Eu4Flavor {
    fn decode<'a>(&self, data: &'a [u8]) -> std::borrow::Cow<'a, str> {
        // Heuristic to detect utf-16 encoded strings
        if matches!(data.get(0), Some(&0x10) | Some(&0x12)) {
            std::borrow::Cow::Owned(to_utf16(data))
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
