use crate::{
    file::{Eu4BinaryDeserializer, Eu4Deserializer, Eu4TextDeserializer},
    Eu4Error, Eu4ErrorKind,
};
use jomini::{binary::TokenResolver, DeserializeError};
use serde::Deserializer;

impl<'de, 'tape, RES> Deserializer<'de> for &'_ Eu4Deserializer<'de, 'tape, RES>
where
    RES: TokenResolver,
{
    type Error = Eu4Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_any(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_any(visitor),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_bool(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_bool(visitor),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_i8(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_i8(visitor),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_i16(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_i16(visitor),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_i32(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_i32(visitor),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_i64(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_i64(visitor),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_u8(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_u8(visitor),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_u16(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_u16(visitor),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_u32(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_u32(visitor),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_u64(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_u64(visitor),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_f32(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_f32(visitor),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_f64(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_f64(visitor),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_char(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_char(visitor),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_str(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_str(visitor),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_string(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_string(visitor),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_bytes(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_bytes(visitor),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_byte_buf(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_byte_buf(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_option(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_option(visitor),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_unit(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_unit(visitor),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_unit_struct(name, visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_unit_struct(name, visitor),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => {
                x.deserialize_newtype_struct(name, visitor)
            }
            crate::file::Eu4DeserializerKind::Binary(x) => {
                x.deserialize_newtype_struct(name, visitor)
            }
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_seq(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_seq(visitor),
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_tuple(len, visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_tuple(len, visitor),
        }
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
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => {
                x.deserialize_tuple_struct(name, len, visitor)
            }
            crate::file::Eu4DeserializerKind::Binary(x) => {
                x.deserialize_tuple_struct(name, len, visitor)
            }
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_map(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_map(visitor),
        }
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
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => {
                x.deserialize_struct(name, fields, visitor)
            }
            crate::file::Eu4DeserializerKind::Binary(x) => {
                x.deserialize_struct(name, fields, visitor)
            }
        }
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
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => {
                x.deserialize_enum(name, variants, visitor)
            }
            crate::file::Eu4DeserializerKind::Binary(x) => {
                x.deserialize_enum(name, variants, visitor)
            }
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_identifier(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_identifier(visitor),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.kind {
            crate::file::Eu4DeserializerKind::Text(x) => x.deserialize_ignored_any(visitor),
            crate::file::Eu4DeserializerKind::Binary(x) => x.deserialize_ignored_any(visitor),
        }
    }
}

fn translate_deserialize_error(e: DeserializeError) -> Eu4Error {
    let kind = match e.kind() {
        &jomini::DeserializeErrorKind::UnknownToken { token_id } => {
            Eu4ErrorKind::UnknownToken { token_id }
        }
        _ => Eu4ErrorKind::Deserialize(e),
    };

    Eu4Error::new(kind)
}

impl<'de, 'tape, RES: TokenResolver> Deserializer<'de>
    for &'_ Eu4BinaryDeserializer<'de, 'tape, RES>
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

impl<'de, 'tape> Deserializer<'de> for &'_ Eu4TextDeserializer<'de, 'tape> {
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
