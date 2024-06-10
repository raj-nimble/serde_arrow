use serde::{
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serialize, Serializer,
};

use crate::internal::{
    error::{fail, Error, Result},
    utils::{Mut, Offset},
};

use super::ArrayBuilder;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct MutableBitBuffer {
    pub(crate) buffer: Vec<u8>,
    pub(crate) len: usize,
    pub(crate) capacity: usize,
}

impl MutableBitBuffer {
    pub fn as_bool(&self) -> Vec<bool> {
        (0..self.len())
            .map(|i| {
                let flag = 1 << i;
                (self.buffer[i / 8] & flag) == flag
            })
            .collect()
    }

    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, value: bool) {
        while self.len >= self.capacity {
            for _ in 0..64 {
                self.buffer.push(0);
                self.capacity += 8;
            }
        }

        if value {
            self.buffer[self.len / 8] |= 1 << (self.len % 8);
        }
        self.len += 1;
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone)]
pub struct MutableOffsetBuffer<O> {
    pub(crate) offsets: Vec<O>,
    pub(crate) current_items: O,
}

impl<O: Offset> std::default::Default for MutableOffsetBuffer<O> {
    fn default() -> Self {
        Self {
            offsets: vec![O::default()],
            current_items: O::default(),
        }
    }
}

impl<O: Offset> MutableOffsetBuffer<O> {
    /// The number of items pushed (one less than the number of offsets)
    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.offsets.len() - 1
    }

    // push a new item with the given number of children
    pub fn push(&mut self, num_children: usize) -> Result<()> {
        self.current_items = self.current_items.clone() + O::try_form_usize(num_children)?;
        self.offsets.push(self.current_items.clone());

        Ok(())
    }

    pub fn push_current_items(&mut self) {
        self.offsets.push(self.current_items.clone());
    }

    pub fn inc_current_items(&mut self) -> Result<()> {
        self.current_items = self.current_items.clone() + O::try_form_usize(1)?;
        Ok(())
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

pub fn push_validity(buffer: &mut Option<MutableBitBuffer>, value: bool) -> Result<()> {
    if let Some(buffer) = buffer.as_mut() {
        buffer.push(value);
        Ok(())
    } else if value {
        Ok(())
    } else {
        fail!("cannot push null for non-nullable array");
    }
}

pub fn push_validity_default(buffer: &mut Option<MutableBitBuffer>) {
    if let Some(buffer) = buffer.as_mut() {
        buffer.push(false);
    }
}

/// A simplified serialization trait with default implementations raising an
/// error
///
/// This trait deviates from the serde model of returning new serializers for
/// (struct, tuple, seq, ..). The only exceptions are struct or tuple variants.
/// As these two cases can be sped up by returning the child serializer in the
/// start call.
///
#[allow(unused_variables)]
pub trait SimpleSerializer: Sized {
    fn name(&self) -> &str;

    fn serialize_default(&mut self) -> Result<()> {
        fail!("serialize_default is not supported for {}", self.name());
    }

    fn serialize_unit(&mut self) -> Result<()> {
        self.serialize_none()
    }

    fn serialize_none(&mut self) -> Result<()> {
        fail!(
            "serialize_unit/serialize_none is not supported for {}",
            self.name()
        );
    }

    fn serialize_some<V: serde::Serialize + ?Sized>(&mut self, value: &V) -> Result<()> {
        value.serialize(Mut(self))
    }

    fn serialize_bool(&mut self, v: bool) -> Result<()> {
        fail!("serialize_bool is not implemented for {}", self.name())
    }

    fn serialize_char(&mut self, v: char) -> Result<()> {
        fail!("serialize_char is not implemented for {}", self.name())
    }

    fn serialize_u8(&mut self, v: u8) -> Result<()> {
        fail!("serialize_u8 is not implemented for {}", self.name())
    }

    fn serialize_u16(&mut self, v: u16) -> Result<()> {
        fail!("serialize_u16 is not implemented for {}", self.name())
    }

    fn serialize_u32(&mut self, v: u32) -> Result<()> {
        fail!("serialize_u32 is not implemented for {}", self.name())
    }

    fn serialize_u64(&mut self, v: u64) -> Result<()> {
        fail!("serialize_u64 is not implemented for {}", self.name())
    }

    fn serialize_i8(&mut self, v: i8) -> Result<()> {
        fail!("serialize_i8 is not implemented for {}", self.name())
    }

    fn serialize_i16(&mut self, v: i16) -> Result<()> {
        fail!("serialize_i16 is not implemented for {}", self.name())
    }

    fn serialize_i32(&mut self, v: i32) -> Result<()> {
        fail!("serialize_i32 is not implemented for {}", self.name())
    }

    fn serialize_i64(&mut self, v: i64) -> Result<()> {
        fail!("serialize_i64 is not implemented for {}", self.name())
    }

    fn serialize_f32(&mut self, v: f32) -> Result<()> {
        fail!("serialize_f32 is not implemented for {}", self.name())
    }

    fn serialize_f64(&mut self, v: f64) -> Result<()> {
        fail!("serialize_f64 is not implemented for {}", self.name())
    }

    fn serialize_bytes(&mut self, v: &[u8]) -> Result<()> {
        fail!("serialize_bytes is not implemented for {}", self.name())
    }

    fn serialize_str(&mut self, v: &str) -> Result<()> {
        fail!("serialize_str is not implemented for {}", self.name())
    }

    fn serialize_newtype_struct<V: Serialize + ?Sized>(
        &mut self,
        name: &'static str,
        value: &V,
    ) -> Result<()> {
        value.serialize(Mut(self))
    }

    fn serialize_newtype_variant<V: Serialize + ?Sized>(
        &mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &V,
    ) -> Result<()> {
        fail!(
            "serialize_newtype_variant is not implemented for {}",
            self.name()
        )
    }

    fn serialize_unit_struct(&mut self, name: &'static str) -> Result<()> {
        fail!(
            "serialize_unit_struct is not implemented for {}",
            self.name()
        )
    }

    fn serialize_unit_variant(
        &mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        fail!(
            "serialize_unit_variant is not implemented for {}",
            self.name()
        )
    }

    fn serialize_map_start(&mut self, len: Option<usize>) -> Result<()> {
        fail!("serialize_map_start is not implemented for {}", self.name())
    }

    fn serialize_map_key<V: Serialize + ?Sized>(&mut self, key: &V) -> Result<()> {
        fail!("serialize_map_key is not implemented for {}", self.name());
    }

    fn serialize_map_value<V: Serialize + ?Sized>(&mut self, value: &V) -> Result<()> {
        fail!("serialize_map_value is not implemented for {}", self.name())
    }

    fn serialize_map_end(&mut self) -> Result<()> {
        fail!("serialize_map_end is not implemented for {}", self.name())
    }

    fn serialize_seq_start(&mut self, len: Option<usize>) -> Result<()> {
        fail!("serialize_seq_start is not implemented for {}", self.name())
    }

    fn serialize_seq_element<V: Serialize + ?Sized>(&mut self, value: &V) -> Result<()> {
        fail!(
            "serialize_seq_element is not implemented for {}",
            self.name()
        );
    }

    fn serialize_seq_end(&mut self) -> Result<()> {
        fail!("serialize_seq_end is not implemented for {}", self.name());
    }

    fn serialize_struct_start(&mut self, name: &'static str, len: usize) -> Result<()> {
        fail!(
            "serialize_start_start is not implemented for {}",
            self.name()
        )
    }

    fn serialize_struct_field<V: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &V,
    ) -> Result<()> {
        fail!(
            "serialize_struct_field is not implemented for {}",
            self.name()
        );
    }

    fn serialize_struct_end(&mut self) -> Result<()> {
        fail!(
            "serialize_struct_end is not implemented for {}",
            self.name()
        );
    }

    fn serialize_tuple_start(&mut self, len: usize) -> Result<()> {
        fail!(
            "serialize_tuple_start is not implemented for {}",
            self.name()
        )
    }

    fn serialize_tuple_element<V: Serialize + ?Sized>(&mut self, value: &V) -> Result<()> {
        fail!(
            "serialize_tuple_element is not implemented for {}",
            self.name()
        );
    }

    fn serialize_tuple_end(&mut self) -> Result<()> {
        fail!("serialize_tuple_end is not implemented for {}", self.name())
    }

    fn serialize_tuple_struct_start(&mut self, name: &'static str, len: usize) -> Result<()> {
        fail!(
            "serialize_tuple_struct_start is not implemented for {}",
            self.name()
        )
    }

    fn serialize_tuple_struct_field<V: Serialize + ?Sized>(&mut self, value: &V) -> Result<()> {
        fail!(
            "serialize_tuple_struct_field is not implemented for {}",
            self.name()
        );
    }

    fn serialize_tuple_struct_end(&mut self) -> Result<()> {
        fail!(
            "serialize_tuple_struct_end is not implemented for {}",
            self.name()
        );
    }

    fn serialize_struct_variant_start<'this>(
        &'this mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<&'this mut ArrayBuilder> {
        fail!(
            "serialize_struct_variant_start is not implemented for {}",
            self.name()
        )
    }

    fn serialize_tuple_variant_start<'this>(
        &'this mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<&'this mut ArrayBuilder> {
        fail!(
            "serialize_tuple_variant_start is not implemented for {}",
            self.name()
        )
    }
}

impl<'a, T: SimpleSerializer> Serializer for Mut<'a, T> {
    type Error = Error;
    type Ok = ();

    type SerializeMap = Mut<'a, T>;
    type SerializeSeq = Mut<'a, T>;
    type SerializeStruct = Mut<'a, T>;
    type SerializeTuple = Mut<'a, T>;
    type SerializeTupleStruct = Mut<'a, T>;
    type SerializeStructVariant = Mut<'a, ArrayBuilder>;
    type SerializeTupleVariant = Mut<'a, ArrayBuilder>;

    fn serialize_unit(self) -> Result<()> {
        self.0.serialize_unit()
    }

    fn serialize_none(self) -> Result<()> {
        self.0.serialize_none()
    }

    fn serialize_some<V: Serialize + ?Sized>(self, value: &V) -> Result<()> {
        self.0.serialize_some(value)
    }

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.0.serialize_bool(v)
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.0.serialize_char(v)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.0.serialize_u8(v)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.0.serialize_u16(v)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.0.serialize_u32(v)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.0.serialize_u64(v)
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.0.serialize_i8(v)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.0.serialize_i16(v)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.0.serialize_i32(v)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.0.serialize_i64(v)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.0.serialize_f32(v)
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.0.serialize_f64(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.0.serialize_bytes(v)
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.0.serialize_str(v)
    }

    fn serialize_newtype_struct<V: Serialize + ?Sized>(
        self,
        name: &'static str,
        value: &V,
    ) -> Result<()> {
        self.0.serialize_newtype_struct(name, value)
    }

    fn serialize_newtype_variant<V: Serialize + ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &V,
    ) -> Result<()> {
        self.0
            .serialize_newtype_variant(name, variant_index, variant, value)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        self.0.serialize_unit_struct(name)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.0.serialize_unit_variant(name, variant_index, variant)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.0.serialize_map_start(len)?;
        Ok(Mut(&mut *self.0))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.0.serialize_seq_start(len)?;
        Ok(Mut(&mut *self.0))
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.0.serialize_struct_start(name, len)?;
        Ok(Mut(&mut *self.0))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.0.serialize_tuple_start(len)?;
        Ok(Mut(&mut *self.0))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.0.serialize_tuple_struct_start(name, len)?;
        Ok(Mut(&mut *self.0))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        let variant_builder =
            self.0
                .serialize_struct_variant_start(name, variant_index, variant, len)?;
        Ok(Mut(variant_builder))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        let variant_builder =
            self.0
                .serialize_tuple_variant_start(name, variant_index, variant, len)?;
        Ok(Mut(variant_builder))
    }
}

impl<'a, T: SimpleSerializer> SerializeMap for Mut<'a, T> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<V: Serialize + ?Sized>(&mut self, key: &V) -> Result<()> {
        self.0.serialize_map_key(key)
    }

    fn serialize_value<V: Serialize + ?Sized>(&mut self, value: &V) -> Result<()> {
        self.0.serialize_map_value(value)
    }

    fn end(self) -> Result<()> {
        self.0.serialize_map_end()
    }
}

impl<'a, T: SimpleSerializer> SerializeSeq for Mut<'a, T> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<V: Serialize + ?Sized>(&mut self, value: &V) -> Result<()> {
        self.0.serialize_seq_element(value)
    }

    fn end(self) -> Result<()> {
        self.0.serialize_seq_end()
    }
}

impl<'a, T: SimpleSerializer> SerializeStruct for Mut<'a, T> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<V: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &V,
    ) -> Result<()> {
        self.0.serialize_struct_field(key, value)
    }

    fn end(self) -> Result<()> {
        self.0.serialize_struct_end()
    }
}

impl<'a, T: SimpleSerializer> SerializeTuple for Mut<'a, T> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<V: Serialize + ?Sized>(&mut self, value: &V) -> Result<()> {
        self.0.serialize_tuple_element(value)
    }

    fn end(self) -> Result<()> {
        self.0.serialize_tuple_end()
    }
}

impl<'a, T: SimpleSerializer> SerializeTupleStruct for Mut<'a, T> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<V: Serialize + ?Sized>(&mut self, value: &V) -> Result<()> {
        self.0.serialize_tuple_struct_field(value)
    }

    fn end(self) -> Result<()> {
        self.0.serialize_tuple_struct_end()
    }
}

impl<'a, T: SimpleSerializer> SerializeStructVariant for Mut<'a, T> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<V: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &V,
    ) -> Result<()> {
        self.0.serialize_struct_field(key, value)
    }

    fn end(self) -> Result<()> {
        self.0.serialize_struct_end()
    }
}

impl<'a, T: SimpleSerializer> SerializeTupleVariant for Mut<'a, T> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<V: Serialize + ?Sized>(&mut self, value: &V) -> Result<()> {
        self.0.serialize_tuple_struct_field(value)
    }

    fn end(self) -> Result<()> {
        self.0.serialize_tuple_struct_end()
    }
}
