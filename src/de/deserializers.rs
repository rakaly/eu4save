use crate::{
    file::{Eu4BinaryDeserializer, Eu4TextDeserializer},
    Eu4Error, Eu4ErrorKind,
};
use jomini::binary::TokenResolver;
use serde::Deserializer;

fn translate_deserialize_error(e: jomini::Error) -> Eu4Error {
    let kind = match e.into_kind() {
        jomini::ErrorKind::Deserialize(x) => match x.kind() {
            &jomini::DeserializeErrorKind::UnknownToken { token_id } => {
                Eu4ErrorKind::UnknownToken { token_id }
            }
            _ => Eu4ErrorKind::Deserialize(x),
        },
        _ => Eu4ErrorKind::DeserializeImpl {
            msg: String::from("unexpected error"),
        },
    };

    Eu4Error::new(kind)
}

impl<'de, 'res: 'de, RES: TokenResolver, R> Deserializer<'de>
    for &'_ mut Eu4BinaryDeserializer<'res, RES, R>
where
    R: std::io::Read,
{
    type Error = Eu4Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_any(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_bool(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_i8(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_i16(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_i32(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_i64(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_u8(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_u16(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_u32(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_u64(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_f32(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_f64(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_char(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_str(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_string(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_bytes(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_byte_buf(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_option(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_unit(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_unit_struct(name, visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_newtype_struct(name, visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_seq(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_tuple(len, visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_tuple_struct(name, len, visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_map(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_struct(name, fields, visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_enum(name, variants, visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_identifier(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_ignored_any(visitor)
            .map_err(translate_deserialize_error)
    }
}

impl<'de, R: std::io::Read> Deserializer<'de> for &'_ mut Eu4TextDeserializer<R> {
    type Error = Eu4Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_any(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_bool(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_i8(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_i16(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_i32(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_i64(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_u8(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_u16(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_u32(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_u64(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_f32(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_f64(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_char(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_str(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_string(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_bytes(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_byte_buf(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_option(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_unit(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_unit_struct(name, visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_newtype_struct(name, visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_seq(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_tuple(len, visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_tuple_struct(name, len, visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_map(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_struct(name, fields, visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_enum(name, variants, visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_identifier(visitor)
            .map_err(translate_deserialize_error)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deser
            .deserialize_ignored_any(visitor)
            .map_err(translate_deserialize_error)
    }
}
