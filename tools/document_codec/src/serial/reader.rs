use std::io;
use uuid::Uuid;

use crate::names::ramona_class_name;
use crate::types::*;

/// Reads Ramona binary v2 format from a byte buffer.
pub struct BinaryReader {
    data: Vec<u8>,
    pos: usize,
}

impl BinaryReader {
    pub fn new(data: Vec<u8>) -> Self {
        BinaryReader { data, pos: 0 }
    }

    fn read_bytes(&mut self, n: usize) -> io::Result<&[u8]> {
        if self.pos + n > self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof,
                format!("need {} bytes at offset {}, have {}", n, self.pos, self.data.len() - self.pos)));
        }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        Ok(self.read_bytes(1)?[0])
    }

    fn read_i16(&mut self) -> io::Result<i16> {
        Ok(i16::from_be_bytes(self.read_bytes(2)?.try_into().unwrap()))
    }

    fn read_i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_be_bytes(self.read_bytes(4)?.try_into().unwrap()))
    }

    fn read_i64(&mut self) -> io::Result<i64> {
        Ok(i64::from_be_bytes(self.read_bytes(8)?.try_into().unwrap()))
    }

    fn read_f32(&mut self) -> io::Result<f32> {
        Ok(f32::from_be_bytes(self.read_bytes(4)?.try_into().unwrap()))
    }

    fn read_f64(&mut self) -> io::Result<f64> {
        Ok(f64::from_be_bytes(self.read_bytes(8)?.try_into().unwrap()))
    }

    fn read_string(&mut self) -> io::Result<String> {
        let (s, _wide) = self.read_string_with_flag()?;
        Ok(s)
    }

    fn read_string_with_flag(&mut self) -> io::Result<(String, bool)> {
        let raw = self.read_i32()?;
        let len = (raw & 0x7FFFFFFF) as usize;
        let wide = (raw as u32 & 0x80000000) != 0;
        if len > 67108864 {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("string too long: {} at offset 0x{:x}", len, self.pos)));
        }
        if wide {
            let mut units = Vec::with_capacity(len);
            for _ in 0..len {
                units.push(self.read_i16()? as u16);
            }
            let s = String::from_utf16(&units).unwrap_or_else(|_| {
                String::from_utf16_lossy(&units)
            });
            Ok((s, true))
        } else {
            let mut s = String::with_capacity(len);
            for _ in 0..len {
                s.push(self.read_u8()? as char);
            }
            Ok((s, false))
        }
    }

    fn read_type_tag(&mut self) -> io::Result<TypeTag> {
        let b = self.read_u8()?;
        TypeTag::from_byte(b).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData,
                format!("unknown type tag {} at offset 0x{:x}", b, self.pos - 1))
        })
    }

    fn read_uuid(&mut self) -> io::Result<Uuid> {
        let hi = self.read_i64()? as u64;
        let lo = self.read_i64()? as u64;
        Ok(Uuid::from_u128(((hi as u128) << 64) | lo as u128))
    }

    /// Read a RelativeRef: discriminant + (objectRef or inline object) + path string.
    pub fn read_relative_ref(&mut self) -> io::Result<crate::types::RelativeRefData> {
        use crate::types::{RelativeRefData, RelativeRefTarget};
        let discriminant = self.read_i32()?;
        let target = if discriminant == 1 {
            // ObjectRef variant
            let ref_id = self.read_i32()?;
            RelativeRefTarget::ObjectRef { id: ref_id }
        } else {
            // Inline object — discriminant is the class_id
            let obj = self.read_object(Some(discriminant))?;
            RelativeRefTarget::Object(obj)
        };
        let path = self.read_string()?;
        Ok(RelativeRefData { target, path })
    }

    pub fn read_object(&mut self, class_id_override: Option<i32>) -> io::Result<RamonaObject> {
        let class_id = match class_id_override {
            Some(id) => id,
            None => self.read_i32()?,
        };
        let (class_name, wide_class_name) = if class_id == 4 {
            self.read_string_with_flag()?
        } else {
            (ramona_class_name(class_id).unwrap_or("").to_string(), false)
        };

        let mut fields = indexmap::IndexMap::new();
        let mut wide_names = Vec::new();
        let mut next_field = self.read_i32()?;

        while next_field != 0 {
            let field_key = if next_field == 1 {
                let (name, wide) = self.read_string_with_flag()?;
                wide_names.push(wide);
                FieldKey::Name(name)
            } else {
                FieldKey::Id(next_field)
            };

            let type_tag = self.read_type_tag()?;
            let value = self.read_value(type_tag)?;
            fields.insert(field_key, value);

            next_field = self.read_i32()?;
        }

        Ok(RamonaObject { class_id, class_name, wide_names, wide_class_name, fields, stable_id: 0 })
    }

    pub fn read_value(&mut self, tag: TypeTag) -> io::Result<RamonaValue> {
        match tag {
            TypeTag::Byte => Ok(RamonaValue::Byte(self.read_u8()?)),
            TypeTag::Int16 => Ok(RamonaValue::Int16(self.read_i16()?)),
            TypeTag::Int32 => Ok(RamonaValue::Int32(self.read_i32()?)),
            TypeTag::Int64 => Ok(RamonaValue::Int64(self.read_i64()?)),
            TypeTag::Bool => Ok(RamonaValue::Bool(self.read_u8()? != 0)),
            TypeTag::Float => Ok(RamonaValue::Float(self.read_f32()?)),
            TypeTag::Double => Ok(RamonaValue::Double(self.read_f64()?)),
            TypeTag::String => {
                let (s, wide) = self.read_string_with_flag()?;
                Ok(if wide { RamonaValue::WideString(s) } else { RamonaValue::String(s) })
            }
            TypeTag::Uuid => Ok(RamonaValue::Uuid(self.read_uuid()?)),
            TypeTag::Color => {
                Ok(RamonaValue::Color([self.read_f32()?, self.read_f32()?, self.read_f32()?, self.read_f32()?]))
            }
            TypeTag::NullObject => Ok(RamonaValue::NullObject),
            TypeTag::Object => Ok(RamonaValue::Object(Box::new(self.read_object(None)?))),
            TypeTag::ObjectRef => Ok(RamonaValue::ObjectRef(self.read_i32()?)),
            TypeTag::ExtObjectRef => Ok(RamonaValue::ExtObjectRef(self.read_i32()?)),
            TypeTag::RelativeRef => {
                let data = self.read_relative_ref()?;
                Ok(RamonaValue::RelativeRef(Box::new(data)))
            }
            TypeTag::ObjectList => {
                let mut items = Vec::new();
                let mut next = self.read_i32()?;
                loop {
                    match next {
                        3 => break,
                        0 => items.push(RamonaListItem::Null),
                        1 => items.push(RamonaListItem::ObjectRef(self.read_i32()?)),
                        2 => items.push(RamonaListItem::ExtObjectRef(self.read_i32()?)),
                        5 => {
                            let data = self.read_relative_ref()?;
                            items.push(RamonaListItem::RelativeRef(data));
                        }
                        id => {
                            let obj = self.read_object(Some(id))?;
                            items.push(RamonaListItem::Object(obj));
                        }
                    }
                    next = self.read_i32()?;
                }
                Ok(RamonaValue::ObjectList(items))
            }
            TypeTag::ByteArray => {
                let len = self.read_i32()? as usize;
                let bytes = self.read_bytes(len)?.to_vec();
                Ok(RamonaValue::ByteArray(bytes))
            }
            TypeTag::Int16Array => {
                let len = self.read_i32()? as usize;
                (0..len).map(|_| self.read_i16()).collect::<io::Result<Vec<_>>>().map(RamonaValue::Int16Array)
            }
            TypeTag::Int32Array => {
                let len = self.read_i32()? as usize;
                (0..len).map(|_| self.read_i32()).collect::<io::Result<Vec<_>>>().map(RamonaValue::Int32Array)
            }
            TypeTag::Int64Array => {
                let len = self.read_i32()? as usize;
                (0..len).map(|_| self.read_i64()).collect::<io::Result<Vec<_>>>().map(RamonaValue::Int64Array)
            }
            TypeTag::FloatArray => {
                let len = self.read_i32()? as usize;
                (0..len).map(|_| self.read_f32()).collect::<io::Result<Vec<_>>>().map(RamonaValue::FloatArray)
            }
            TypeTag::DoubleArray => {
                let len = self.read_i32()? as usize;
                (0..len).map(|_| self.read_f64()).collect::<io::Result<Vec<_>>>().map(RamonaValue::DoubleArray)
            }
            TypeTag::BoolArray => {
                let len = self.read_i32()? as usize;
                (0..len).map(|_| self.read_u8().map(|b| b != 0)).collect::<io::Result<Vec<_>>>().map(RamonaValue::BoolArray)
            }
            TypeTag::StringArray => {
                let len = self.read_i32()? as usize;
                (0..len).map(|_| self.read_string()).collect::<io::Result<Vec<_>>>().map(RamonaValue::StringArray)
            }
            TypeTag::StringToObjectMap => {
                let mut entries = Vec::new();
                let has_entries = self.read_u8()?;
                if has_entries != 0 {
                    loop {
                        let key = self.read_string()?;
                        let obj = self.read_object(None)?;
                        entries.push((key, RamonaValue::Object(Box::new(obj))));
                        let more = self.read_u8()?;
                        if more == 0 { break; }
                    }
                }
                Ok(RamonaValue::StringToObjectMap(entries))
            }
        }
    }
}
