use serde::{
    de::{self, SeqAccess},
    Deserializer,
};
use std::fmt;
use std::marker::PhantomData;

pub(crate) fn deserialize_vec_overflow_byte<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct VecPairVisitor {
        marker: PhantomData<Vec<u8>>,
    }

    impl<'de> de::Visitor<'de> for VecPairVisitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a seq of bytes allowed to overflow")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut v = if let Some(size) = seq.size_hint() {
                Vec::with_capacity(size)
            } else {
                Vec::new()
            };

            while let Some(x) = seq.next_element::<u16>()? {
                v.push(x as u8)
            }

            Ok(v)
        }
    }

    deserializer.deserialize_seq(VecPairVisitor {
        marker: PhantomData,
    })
}
