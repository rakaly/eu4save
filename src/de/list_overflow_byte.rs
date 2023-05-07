use serde::{
    de::{self, SeqAccess},
    Deserializer,
};
use std::fmt;

pub(crate) fn deserialize_list_overflow_byte<'de, D>(deserializer: D) -> Result<[u8; 3], D::Error>
where
    D: Deserializer<'de>,
{
    struct ListVisitor;

    impl<'de> de::Visitor<'de> for ListVisitor {
        type Value = [u8; 3];

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a seq of bytes allowed to overflow")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            seqqer(seq)
        }
    }

    deserializer.deserialize_seq(ListVisitor)
}

fn seqqer<'de, A>(mut seq: A) -> Result<[u8; 3], <A as SeqAccess<'de>>::Error>
where
    A: SeqAccess<'de>,
{
    let mut result = [0u8; 3];
    let mut seq_finished = false;
    for c in result.iter_mut() {
        if let Some(x) = seq.next_element::<u16>()? {
            *c = x as u8;
        } else {
            seq_finished = true;
            break;
        }
    }
    if !seq_finished {
        while let Some(_x) = seq.next_element::<de::IgnoredAny>()? {}
    }

    Ok(result)
}

pub(crate) fn deserialize_list_overflow_byte_opt<'de, D>(
    deserializer: D,
) -> Result<Option<[u8; 3]>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ListVisitor;

    impl<'de> de::Visitor<'de> for ListVisitor {
        type Value = Option<[u8; 3]>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a seq of bytes allowed to overflow")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            Ok(Some(seqqer(seq)?))
        }
    }

    deserializer.deserialize_seq(ListVisitor)
}
