pub fn positive_vec_f32<'de, D>(deserializer: D) -> Result<Vec<f32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct VecVisitor;

    impl<'de> serde::de::Visitor<'de> for VecVisitor {
        type Value = Vec<f32>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a seq of wrapping floats")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut values = if let Some(size) = seq.size_hint() {
                Vec::with_capacity(size)
            } else {
                Vec::new()
            };

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

    deserializer.deserialize_seq(VecVisitor)
}
