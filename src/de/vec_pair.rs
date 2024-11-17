use serde::{de, Deserialize, Deserializer};
use std::fmt;
use std::marker::PhantomData;

pub fn deserialize_vec_pair<'de, D, K, V>(deserializer: D) -> Result<Vec<(K, V)>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    deserialize_vec_pair_n(deserializer, 0)
}

pub fn deserialize_vec_pair_1400<'de, D, K, V>(deserializer: D) -> Result<Vec<(K, V)>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    deserialize_vec_pair_n(deserializer, 1400)
}

pub fn deserialize_vec_pair_8<'de, D, K, V>(deserializer: D) -> Result<Vec<(K, V)>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    deserialize_vec_pair_n(deserializer, 8)
}

pub fn deserialize_vec_pair_24<'de, D, K, V>(deserializer: D) -> Result<Vec<(K, V)>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    deserialize_vec_pair_n(deserializer, 24)
}

pub fn deserialize_vec_pair_51<'de, D, K, V>(deserializer: D) -> Result<Vec<(K, V)>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    deserialize_vec_pair_n(deserializer, 51)
}

#[inline]
pub fn deserialize_vec_pair_n<'de, D, K, V>(deserializer: D, capacity: usize) -> Result<Vec<(K, V)>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    struct VecPairVisitor<K1, V1> {
        marker: PhantomData<Vec<(K1, V1)>>,
        capacity: usize,
    }

    impl<'de, K1, V1> de::Visitor<'de> for VecPairVisitor<K1, V1>
    where
        K1: Deserialize<'de>,
        V1: Deserialize<'de>,
    {
        type Value = Vec<(K1, V1)>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map containing key value tuples")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            let mut values = if let Some(size) = map.size_hint() {
                Vec::with_capacity(size)
            } else {
                Vec::with_capacity(self.capacity)
            };

            while let Some((key, value)) = map.next_entry()? {
                values.push((key, value));
            }

            Ok(values)
        }
    }

    deserializer.deserialize_map(VecPairVisitor {
        capacity,
        marker: PhantomData,
    })
}
