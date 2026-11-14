use crate::types::*;
use uuid::Uuid;

/// Writes Ramona binary v2 format to a byte buffer.
pub struct BinaryWriter {
    pub data: Vec<u8>,
}

impl BinaryWriter {
    pub fn new() -> Self {
        BinaryWriter { data: Vec::new() }
    }

    fn write_u8(&mut self, v: u8) {
        self.data.push(v);
    }

    fn write_i16(&mut self, v: i16) {
        self.data.extend_from_slice(&v.to_be_bytes());
    }

    fn write_i32(&mut self, v: i32) {
        self.data.extend_from_slice(&v.to_be_bytes());
    }

    fn write_i64(&mut self, v: i64) {
        self.data.extend_from_slice(&v.to_be_bytes());
    }

    fn write_f32(&mut self, v: f32) {
        self.data.extend_from_slice(&v.to_be_bytes());
    }

    fn write_f64(&mut self, v: f64) {
        self.data.extend_from_slice(&v.to_be_bytes());
    }

    fn write_string(&mut self, s: &str) {
        let len = s.chars().count() as i32;
        self.write_i32(len);
        for ch in s.chars() {
            self.write_u8(ch as u8);
        }
    }

    fn write_wide_string(&mut self, s: &str) {
        let units: Vec<u16> = s.encode_utf16().collect();
        self.write_i32(units.len() as i32 | (1_i32 << 31));
        for &u in &units {
            self.write_i16(u as i16);
        }
    }

    fn write_uuid(&mut self, uuid: &Uuid) {
        let bits = uuid.as_u128();
        self.write_i64((bits >> 64) as i64);
        self.write_i64(bits as i64);
    }

    /// Write a RelativeRef: discriminant + (objectRef or inline object) + path string.
    pub fn write_relative_ref(&mut self, data: &RelativeRefData) {
        match &data.target {
            RelativeRefTarget::ObjectRef { id } => {
                self.write_i32(1); // discriminant = 1 for ObjectRef
                self.write_i32(*id);
            }
            RelativeRefTarget::Object(obj) => {
                self.write_object(obj); // writes class_id + fields (class_id IS the discriminant)
            }
        }
        self.write_string(&data.path);
    }

    pub fn write_object(&mut self, obj: &RamonaObject) {
        self.write_i32(obj.class_id);
        self.write_object_body(obj);
    }

    fn write_object_body(&mut self, obj: &RamonaObject) {
        if obj.class_id == 4 {
            if obj.wide_class_name {
                self.write_wide_string(&obj.class_name);
            } else {
                self.write_string(&obj.class_name);
            }
        }

        let mut name_idx = 0;
        for (field_key, val) in &obj.fields {
            match field_key {
                FieldKey::Name(name) => {
                    self.write_i32(1);
                    let wide = obj.wide_names.get(name_idx).copied().unwrap_or(false);
                    name_idx += 1;
                    if wide {
                        self.write_wide_string(name);
                    } else {
                        self.write_string(name);
                    }
                }
                FieldKey::Id(id) => {
                    self.write_i32(*id);
                }
            }
            self.write_u8(val.type_tag().to_byte());
            self.write_value(val);
        }

        self.write_i32(0); // end of object
    }

    fn write_value(&mut self, val: &RamonaValue) {
        match val {
            RamonaValue::Byte(v) => self.write_u8(*v),
            RamonaValue::Int16(v) => self.write_i16(*v),
            RamonaValue::Int32(v) => self.write_i32(*v),
            RamonaValue::Int64(v) => self.write_i64(*v),
            RamonaValue::Bool(v) => self.write_u8(if *v { 1 } else { 0 }),
            RamonaValue::Float(v) => self.write_f32(*v),
            RamonaValue::Double(v) => self.write_f64(*v),
            RamonaValue::String(s) => self.write_string(s),
            RamonaValue::WideString(s) => self.write_wide_string(s),
            RamonaValue::Uuid(u) => self.write_uuid(u),
            RamonaValue::Color([r, g, b, a]) => {
                self.write_f32(*r);
                self.write_f32(*g);
                self.write_f32(*b);
                self.write_f32(*a);
            }
            RamonaValue::NullObject => {}
            RamonaValue::Object(obj) => self.write_object(obj),
            RamonaValue::ObjectRef(id) => self.write_i32(*id),
            RamonaValue::ExtObjectRef(id) => self.write_i32(*id),
            RamonaValue::RelativeRef(data) => self.write_relative_ref(data),
            RamonaValue::ObjectList(items) => {
                for item in items {
                    match item {
                        RamonaListItem::Null => self.write_i32(0),
                        RamonaListItem::ObjectRef(id) => {
                            self.write_i32(1);
                            self.write_i32(*id);
                        }
                        RamonaListItem::ExtObjectRef(id) => {
                            self.write_i32(2);
                            self.write_i32(*id);
                        }
                        RamonaListItem::RelativeRef(data) => {
                            self.write_i32(5);
                            self.write_relative_ref(data);
                        }
                        RamonaListItem::Object(obj) => {
                            self.write_i32(obj.class_id);
                            self.write_object_body(obj);
                        }
                    }
                }
                self.write_i32(3); // end of list
            }
            RamonaValue::ByteArray(v) => {
                self.write_i32(v.len() as i32);
                self.data.extend_from_slice(v);
            }
            RamonaValue::Int16Array(v) => {
                self.write_i32(v.len() as i32);
                for &x in v { self.write_i16(x); }
            }
            RamonaValue::Int32Array(v) => {
                self.write_i32(v.len() as i32);
                for &x in v { self.write_i32(x); }
            }
            RamonaValue::Int64Array(v) => {
                self.write_i32(v.len() as i32);
                for &x in v { self.write_i64(x); }
            }
            RamonaValue::FloatArray(v) => {
                self.write_i32(v.len() as i32);
                for &x in v { self.write_f32(x); }
            }
            RamonaValue::DoubleArray(v) => {
                self.write_i32(v.len() as i32);
                for &x in v { self.write_f64(x); }
            }
            RamonaValue::BoolArray(v) => {
                self.write_i32(v.len() as i32);
                for &x in v { self.write_u8(if x { 1 } else { 0 }); }
            }
            RamonaValue::StringArray(v) => {
                self.write_i32(v.len() as i32);
                for s in v { self.write_string(s); }
            }
            RamonaValue::StringToObjectMap(entries) => {
                if entries.is_empty() {
                    self.write_u8(0);
                } else {
                    self.write_u8(1);
                    for (i, (key, val)) in entries.iter().enumerate() {
                        self.write_string(key);
                        if let RamonaValue::Object(obj) = val {
                            self.write_object(obj);
                        }
                        self.write_u8(if i + 1 < entries.len() { 1 } else { 0 });
                    }
                }
            }
        }
    }
}
