use half::f16;
use serde::de::{Deserialize, DeserializeSeed, VariantAccess, Visitor};

use crate::internal::{
    arrow::{ArrayView, FieldMeta, PrimitiveArrayView, TimeUnit},
    error::{fail, Context, Error, Result},
    schema::{Strategy, STRATEGY_KEY},
    utils::{ChildName, Mut},
};

use super::{
    binary_deserializer::BinaryDeserializer, bool_deserializer::BoolDeserializer,
    date32_deserializer::Date32Deserializer, date64_deserializer::Date64Deserializer,
    decimal_deserializer::DecimalDeserializer, dictionary_deserializer::DictionaryDeserializer,
    duration_deserializer::DurationDeserializer, enum_deserializer::EnumDeserializer,
    fixed_size_binary_deserializer::FixedSizeBinaryDeserializer,
    fixed_size_list_deserializer::FixedSizeListDeserializer, float_deserializer::FloatDeserializer,
    integer_deserializer::IntegerDeserializer, list_deserializer::ListDeserializer,
    map_deserializer::MapDeserializer, null_deserializer::NullDeserializer,
    simple_deserializer::SimpleDeserializer, string_deserializer::StringDeserializer,
    struct_deserializer::StructDeserializer, time_deserializer::TimeDeserializer,
};

pub enum ArrayDeserializer<'a> {
    Null(NullDeserializer),
    Bool(BoolDeserializer<'a>),
    U8(IntegerDeserializer<'a, u8>),
    U16(IntegerDeserializer<'a, u16>),
    U32(IntegerDeserializer<'a, u32>),
    U64(IntegerDeserializer<'a, u64>),
    I8(IntegerDeserializer<'a, i8>),
    I16(IntegerDeserializer<'a, i16>),
    I32(IntegerDeserializer<'a, i32>),
    I64(IntegerDeserializer<'a, i64>),
    F16(FloatDeserializer<'a, f16>),
    F32(FloatDeserializer<'a, f32>),
    F64(FloatDeserializer<'a, f64>),
    Decimal128(DecimalDeserializer<'a>),
    Duration(DurationDeserializer<'a>),
    Date32(Date32Deserializer<'a>),
    Date64(Date64Deserializer<'a>),
    Time32(TimeDeserializer<'a, i32>),
    Time64(TimeDeserializer<'a, i64>),
    Utf8(StringDeserializer<'a, i32>),
    LargeUtf8(StringDeserializer<'a, i64>),
    DictionaryU8I32(DictionaryDeserializer<'a, u8, i32>),
    DictionaryU16I32(DictionaryDeserializer<'a, u16, i32>),
    DictionaryU32I32(DictionaryDeserializer<'a, u32, i32>),
    DictionaryU64I32(DictionaryDeserializer<'a, u64, i32>),
    DictionaryI8I32(DictionaryDeserializer<'a, i8, i32>),
    DictionaryI16I32(DictionaryDeserializer<'a, i16, i32>),
    DictionaryI32I32(DictionaryDeserializer<'a, i32, i32>),
    DictionaryI64I32(DictionaryDeserializer<'a, i64, i32>),
    DictionaryU8I64(DictionaryDeserializer<'a, u8, i64>),
    DictionaryU16I64(DictionaryDeserializer<'a, u16, i64>),
    DictionaryU32I64(DictionaryDeserializer<'a, u32, i64>),
    DictionaryU64I64(DictionaryDeserializer<'a, u64, i64>),
    DictionaryI8I64(DictionaryDeserializer<'a, i8, i64>),
    DictionaryI16I64(DictionaryDeserializer<'a, i16, i64>),
    DictionaryI32I64(DictionaryDeserializer<'a, i32, i64>),
    DictionaryI64I64(DictionaryDeserializer<'a, i64, i64>),
    Struct(StructDeserializer<'a>),
    List(ListDeserializer<'a, i32>),
    LargeList(ListDeserializer<'a, i64>),
    FixedSizeList(FixedSizeListDeserializer<'a>),
    Binary(BinaryDeserializer<'a, i32>),
    LargeBinary(BinaryDeserializer<'a, i64>),
    FixedSizeBinary(FixedSizeBinaryDeserializer<'a>),
    Map(MapDeserializer<'a>),
    Enum(EnumDeserializer<'a>),
}

impl<'a> ArrayDeserializer<'a> {
    pub fn new(path: String, strategy: Option<&Strategy>, array: ArrayView<'a>) -> Result<Self> {
        use {ArrayDeserializer as D, ArrayView as V};
        match array {
            ArrayView::Null(_) => Ok(Self::Null(NullDeserializer::new(path))),
            V::Boolean(view) => Ok(D::Bool(BoolDeserializer::new(path, view))),
            V::Int8(view) => Ok(D::I8(IntegerDeserializer::new(path, view))),
            V::Int16(view) => Ok(D::I16(IntegerDeserializer::new(path, view))),
            V::Int32(view) => Ok(D::I32(IntegerDeserializer::new(path, view))),
            V::Int64(view) => Ok(D::I64(IntegerDeserializer::new(path, view))),
            V::UInt8(view) => Ok(D::U8(IntegerDeserializer::new(path, view))),
            V::UInt16(view) => Ok(D::U16(IntegerDeserializer::new(path, view))),
            V::UInt32(view) => Ok(D::U32(IntegerDeserializer::new(path, view))),
            V::UInt64(view) => Ok(D::U64(IntegerDeserializer::new(path, view))),
            V::Float16(view) => Ok(D::F16(FloatDeserializer::new(path, view))),
            V::Float32(view) => Ok(D::F32(FloatDeserializer::new(path, view))),
            V::Float64(view) => Ok(D::F64(FloatDeserializer::new(path, view))),
            V::Decimal128(view) => Ok(D::Decimal128(DecimalDeserializer::new(path, view))),
            ArrayView::Date32(view) => Ok(Self::Date32(Date32Deserializer::new(
                path,
                view.values,
                view.validity,
            ))),
            ArrayView::Date64(view) => Ok(Self::Date64(Date64Deserializer::new(
                path,
                view.values,
                view.validity,
                TimeUnit::Millisecond,
                is_utc_date64(strategy)?,
            ))),
            V::Time32(view) => Ok(D::Time32(TimeDeserializer::new(path, view))),
            V::Time64(view) => Ok(D::Time64(TimeDeserializer::new(path, view))),
            ArrayView::Timestamp(view) => match strategy {
                Some(Strategy::NaiveStrAsDate64 | Strategy::UtcStrAsDate64) => {
                    Ok(Self::Date64(Date64Deserializer::new(
                        path,
                        view.values,
                        view.validity,
                        view.unit,
                        is_utc_timestamp(view.timezone.as_deref())?,
                    )))
                }
                Some(strategy) => {
                    fail!("Invalid strategy: {strategy} is not supported for timestamp field")
                }
                None => Ok(Self::Date64(Date64Deserializer::new(
                    path,
                    view.values,
                    view.validity,
                    view.unit,
                    is_utc_timestamp(view.timezone.as_deref())?,
                ))),
            },
            V::Duration(view) => Ok(D::Duration(DurationDeserializer::new(
                path,
                view.unit,
                PrimitiveArrayView {
                    values: view.values,
                    validity: view.validity,
                },
            ))),
            V::Utf8(view) => Ok(D::Utf8(StringDeserializer::new(path, view))),
            V::LargeUtf8(view) => Ok(D::LargeUtf8(StringDeserializer::new(path, view))),
            V::Binary(view) => Ok(D::Binary(BinaryDeserializer::new(path, view))),
            V::LargeBinary(view) => Ok(D::LargeBinary(BinaryDeserializer::new(path, view))),
            V::FixedSizeBinary(view) => Ok(D::FixedSizeBinary(FixedSizeBinaryDeserializer::new(
                path, view,
            )?)),
            V::List(view) => {
                let child_path = format!("{path}.{child}", child = ChildName(&view.meta.name));
                Ok(D::List(ListDeserializer::new(
                    path,
                    ArrayDeserializer::new(
                        child_path,
                        get_strategy(&view.meta)?.as_ref(),
                        *view.element,
                    )?,
                    view.offsets,
                    view.validity,
                )?))
            }
            V::LargeList(view) => {
                let child_path = format!("{path}.{child}", child = ChildName(&view.meta.name));
                Ok(D::LargeList(ListDeserializer::new(
                    path,
                    ArrayDeserializer::new(
                        child_path,
                        get_strategy(&view.meta)?.as_ref(),
                        *view.element,
                    )?,
                    view.offsets,
                    view.validity,
                )?))
            }
            V::FixedSizeList(view) => {
                let child_path = format!("{path}.{child}", child = ChildName(&view.meta.name));
                Ok(D::FixedSizeList(FixedSizeListDeserializer::new(
                    path,
                    ArrayDeserializer::new(
                        child_path,
                        get_strategy(&view.meta)?.as_ref(),
                        *view.element,
                    )?,
                    view.validity,
                    view.n.try_into()?,
                    view.len,
                )))
            }
            V::Struct(view) => {
                let mut fields = Vec::new();
                for (field_view, field_meta) in view.fields {
                    let child_path = format!("{path}.{child}", child = ChildName(&field_meta.name));
                    let field_deserializer = ArrayDeserializer::new(
                        child_path,
                        get_strategy(&field_meta)?.as_ref(),
                        field_view,
                    )?;
                    let field_name = field_meta.name;

                    fields.push((field_name, field_deserializer));
                }

                Ok(D::Struct(StructDeserializer::new(
                    path,
                    fields,
                    view.validity,
                    view.len,
                )))
            }
            V::Map(view) => {
                let ArrayView::Struct(entries_view) = *view.element else {
                    fail!("Invalid entries field in map array");
                };
                let Ok(entries_fields) = <[_; 2]>::try_from(entries_view.fields) else {
                    fail!("Invalid entries field in map array")
                };
                let [(keys_view, keys_meta), (values_view, values_meta)] = entries_fields;
                let keys_path = format!("{path}.{child}", child = ChildName(&keys_meta.name));
                let keys = ArrayDeserializer::new(
                    keys_path,
                    get_strategy(&keys_meta)?.as_ref(),
                    keys_view,
                )?;

                let values_path = format!("{path}.{child}", child = ChildName(&values_meta.name));
                let values = ArrayDeserializer::new(
                    values_path,
                    get_strategy(&values_meta)?.as_ref(),
                    values_view,
                )?;

                Ok(D::Map(MapDeserializer::new(
                    path,
                    keys,
                    values,
                    view.offsets,
                    view.validity,
                )?))
            }
            V::Dictionary(view) => match (*view.indices, *view.values) {
                (V::Int8(keys), V::Utf8(values)) => Ok(D::DictionaryI8I32(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::Int16(keys), V::Utf8(values)) => Ok(D::DictionaryI16I32(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::Int32(keys), V::Utf8(values)) => Ok(D::DictionaryI32I32(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::Int64(keys), V::Utf8(values)) => Ok(D::DictionaryI64I32(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::UInt8(keys), V::Utf8(values)) => Ok(Self::DictionaryU8I32(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::UInt16(keys), V::Utf8(values)) => Ok(D::DictionaryU16I32(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::UInt32(keys), V::Utf8(values)) => Ok(D::DictionaryU32I32(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::UInt64(keys), V::Utf8(values)) => Ok(D::DictionaryU64I32(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::Int8(keys), V::LargeUtf8(values)) => Ok(D::DictionaryI8I64(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::Int16(keys), V::LargeUtf8(values)) => Ok(D::DictionaryI16I64(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::Int32(keys), V::LargeUtf8(values)) => Ok(D::DictionaryI32I64(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::Int64(keys), V::LargeUtf8(values)) => Ok(D::DictionaryI64I64(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::UInt8(keys), V::LargeUtf8(values)) => Ok(D::DictionaryU8I64(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::UInt16(keys), V::LargeUtf8(values)) => Ok(D::DictionaryU16I64(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::UInt32(keys), V::LargeUtf8(values)) => Ok(D::DictionaryU32I64(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::UInt64(keys), V::LargeUtf8(values)) => Ok(D::DictionaryU64I64(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                _ => fail!("Unsupported dictionary array type"),
            },
            ArrayView::DenseUnion(view) => {
                let mut fields = Vec::new();
                for (idx, (type_id, field_view, field_meta)) in view.fields.into_iter().enumerate()
                {
                    if usize::try_from(type_id) != Ok(idx) {
                        fail!("Only unions with consecutive type ids are currently supported");
                    }
                    let child_path = format!("{path}.{child}", child = ChildName(&field_meta.name));
                    let field_deserializer = ArrayDeserializer::new(
                        child_path,
                        get_strategy(&field_meta)?.as_ref(),
                        field_view,
                    )?;
                    fields.push((field_meta.name, field_deserializer))
                }

                Ok(Self::Enum(EnumDeserializer::new(
                    path,
                    view.types,
                    view.offsets,
                    fields,
                )?))
            }
        }
    }
}

fn is_utc_timestamp(timezone: Option<&str>) -> Result<bool> {
    match timezone {
        Some(tz) if tz.to_lowercase() == "utc" => Ok(true),
        Some(tz) => fail!("Unsupported timezone: {} is not supported", tz),
        None => Ok(false),
    }
}

fn is_utc_date64(strategy: Option<&Strategy>) -> Result<bool> {
    match strategy {
        None | Some(Strategy::UtcStrAsDate64) => Ok(true),
        Some(Strategy::NaiveStrAsDate64) => Ok(false),
        Some(strategy) => {
            fail!("Invalid strategy: {strategy} is not supported for date64 deserializer")
        }
    }
}

fn get_strategy(meta: &FieldMeta) -> Result<Option<Strategy>> {
    let Some(strategy) = meta.metadata.get(STRATEGY_KEY) else {
        return Ok(None);
    };
    Ok(Some(strategy.parse()?))
}

macro_rules! dispatch {
    ($obj:expr, $wrapper:ident($name:ident) => $expr:expr) => {
        match $obj {
            $wrapper::Null($name) => $expr,
            $wrapper::Bool($name) => $expr,
            $wrapper::U8($name) => $expr,
            $wrapper::U16($name) => $expr,
            $wrapper::U32($name) => $expr,
            $wrapper::U64($name) => $expr,
            $wrapper::I8($name) => $expr,
            $wrapper::I16($name) => $expr,
            $wrapper::I32($name) => $expr,
            $wrapper::I64($name) => $expr,
            $wrapper::F16($name) => $expr,
            $wrapper::F32($name) => $expr,
            $wrapper::F64($name) => $expr,
            $wrapper::Decimal128($name) => $expr,
            $wrapper::Duration($name) => $expr,
            $wrapper::Date32($name) => $expr,
            $wrapper::Date64($name) => $expr,
            $wrapper::Time32($name) => $expr,
            $wrapper::Time64($name) => $expr,
            $wrapper::Utf8($name) => $expr,
            $wrapper::LargeUtf8($name) => $expr,
            $wrapper::Struct($name) => $expr,
            $wrapper::List($name) => $expr,
            $wrapper::FixedSizeList($name) => $expr,
            $wrapper::LargeList($name) => $expr,
            $wrapper::Binary($name) => $expr,
            $wrapper::LargeBinary($name) => $expr,
            $wrapper::FixedSizeBinary($name) => $expr,
            $wrapper::Map($name) => $expr,
            $wrapper::Enum($name) => $expr,
            $wrapper::DictionaryU8I32($name) => $expr,
            $wrapper::DictionaryU16I32($name) => $expr,
            $wrapper::DictionaryU32I32($name) => $expr,
            $wrapper::DictionaryU64I32($name) => $expr,
            $wrapper::DictionaryI8I32($name) => $expr,
            $wrapper::DictionaryI16I32($name) => $expr,
            $wrapper::DictionaryI32I32($name) => $expr,
            $wrapper::DictionaryI64I32($name) => $expr,
            $wrapper::DictionaryU8I64($name) => $expr,
            $wrapper::DictionaryU16I64($name) => $expr,
            $wrapper::DictionaryU32I64($name) => $expr,
            $wrapper::DictionaryU64I64($name) => $expr,
            $wrapper::DictionaryI8I64($name) => $expr,
            $wrapper::DictionaryI16I64($name) => $expr,
            $wrapper::DictionaryI32I64($name) => $expr,
            $wrapper::DictionaryI64I64($name) => $expr,
        }
    };
}

impl<'de> Context for ArrayDeserializer<'de> {
    fn annotate(&self, annotations: &mut std::collections::BTreeMap<String, String>) {
        dispatch!(self, ArrayDeserializer(deser) => deser.annotate(annotations))
    }
}

impl<'de> SimpleDeserializer<'de> for ArrayDeserializer<'de> {
    fn deserialize_any<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_any(visitor))
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_ignored_any(visitor))
    }

    fn deserialize_option<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_option(visitor))
    }

    fn deserialize_unit<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_unit(visitor))
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        &mut self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_unit_struct(name, visitor))
    }

    fn deserialize_bool<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_bool(visitor))
    }

    fn deserialize_char<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_char(visitor))
    }

    fn deserialize_u8<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_u8(visitor))
    }

    fn deserialize_u16<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_u16(visitor))
    }

    fn deserialize_u32<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_u32(visitor))
    }

    fn deserialize_u64<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_u64(visitor))
    }

    fn deserialize_i8<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_i8(visitor))
    }

    fn deserialize_i16<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_i16(visitor))
    }

    fn deserialize_i32<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_i32(visitor))
    }

    fn deserialize_i64<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_i64(visitor))
    }

    fn deserialize_f32<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_f32(visitor))
    }

    fn deserialize_f64<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_f64(visitor))
    }

    fn deserialize_str<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_str(visitor))
    }

    fn deserialize_string<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_string(visitor))
    }

    fn deserialize_struct<V: Visitor<'de>>(
        &mut self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_struct(name, fields, visitor))
    }

    fn deserialize_map<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_map(visitor))
    }

    fn deserialize_seq<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_seq(visitor))
    }

    fn deserialize_tuple<V: Visitor<'de>>(&mut self, len: usize, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_tuple(len, visitor))
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        &mut self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_tuple_struct(name, len, visitor))
    }

    fn deserialize_identifier<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_identifier(visitor))
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        &mut self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_newtype_struct(name, visitor))
    }

    fn deserialize_enum<V: Visitor<'de>>(
        &mut self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_enum(name, variants, visitor))
    }

    fn deserialize_bytes<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_bytes(visitor))
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        dispatch!(self, ArrayDeserializer(deser) => deser.deserialize_byte_buf(visitor))
    }
}

impl<'a, 'de> VariantAccess<'de> for Mut<'a, ArrayDeserializer<'de>> {
    type Error = Error;

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        seed.deserialize(self)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.0
            .deserialize_struct("UNUSED_ENUM_STRUCT_NAME", fields, visitor)
    }

    fn tuple_variant<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        self.0.deserialize_tuple(len, visitor)
    }

    fn unit_variant(self) -> Result<()> {
        <()>::deserialize(self)
    }
}
