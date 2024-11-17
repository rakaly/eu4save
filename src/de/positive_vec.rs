use smallvec::SmallVec;

pub fn positive_vec_f32_38<'de, D>(deserializer: D) -> Result<SmallVec<[f32; 38]>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    positive_vec_f32::<_, 38>(deserializer)
}

pub fn positive_vec_f32_19<'de, D>(deserializer: D) -> Result<SmallVec<[f32; 19]>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    positive_vec_f32::<_, 19>(deserializer)
}

pub fn positive_vec_f32<'de, D, const SIZE: usize>(
    deserializer: D,
) -> Result<SmallVec<[f32; SIZE]>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct VecVisitor<const SIZE: usize>;

    impl<'de, const SIZE: usize> serde::de::Visitor<'de> for VecVisitor<SIZE> {
        type Value = SmallVec<[f32; SIZE]>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a seq of wrapping floats")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut values = SmallVec::with_capacity(seq.size_hint().unwrap_or(SIZE));
            let losses_max = (i32::MAX / 1000) as f32;
            let losses_min = -losses_max;

            while let Some(x) = seq.next_element::<f32>()? {
                let val = if x >= 0.0 {
                    x
                } else if x > losses_min {
                    x + 2.0 * losses_max
                } else {
                    x.abs()
                };
                values.push(val);
            }

            Ok(values)
        }
    }

    deserializer.deserialize_seq(VecVisitor::<SIZE>)
}
