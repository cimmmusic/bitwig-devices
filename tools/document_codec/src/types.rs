use std::fmt;

use indexmap::IndexMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::names::ramona_field_name;

#[derive(Debug, thiserror::Error)]
pub enum BitwigPatchError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JAR error: {0}")]
    Jar(String),
    #[error("Class parse error: {0}")]
    ClassParse(String),
    #[error("Assembly error: {0}")]
    Assembly(String),
    #[error("Init class not found in JAR")]
    InitClassNotFound,
    #[error("Whitelist class not found in JAR")]
    WhitelistClassNotFound,
    #[error("Integrity pattern (sipush 5000) not found")]
    IntegrityPatternNotFound,
    #[error("Metadata field missing: {0}")]
    MetadataFieldMissing(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleIdentity {
    pub uuid: Uuid,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub creator: String,
    pub filename: String,
}

/// Binary v2 type tags (wire format identifiers).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypeTag {
    Byte, Int16, Int32, Int64, Bool, Float, Double, String,
    Object, NullObject, ObjectRef, ExtObjectRef,
    ByteArray, Int16Array, Int32Array, Int64Array, BoolArray,
    ObjectList, StringToObjectMap, Uuid, Color,
    FloatArray, DoubleArray, StringArray, RelativeRef,
}

impl TypeTag {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            1 => Some(Self::Byte), 2 => Some(Self::Int16), 3 => Some(Self::Int32),
            4 => Some(Self::Int64), 5 => Some(Self::Bool), 6 => Some(Self::Float),
            7 => Some(Self::Double), 8 => Some(Self::String), 9 => Some(Self::Object),
            10 => Some(Self::NullObject), 11 => Some(Self::ObjectRef),
            12 => Some(Self::ExtObjectRef), 13 => Some(Self::ByteArray),
            14 => Some(Self::Int16Array), 15 => Some(Self::Int32Array),
            16 => Some(Self::Int64Array), 17 => Some(Self::BoolArray),
            18 => Some(Self::ObjectList), 20 => Some(Self::StringToObjectMap),
            21 => Some(Self::Uuid), 22 => Some(Self::Color),
            23 => Some(Self::FloatArray), 24 => Some(Self::DoubleArray),
            25 => Some(Self::StringArray), 26 => Some(Self::RelativeRef),
            _ => None,
        }
    }

    pub fn to_byte(self) -> u8 {
        match self {
            Self::Byte => 1, Self::Int16 => 2, Self::Int32 => 3,
            Self::Int64 => 4, Self::Bool => 5, Self::Float => 6,
            Self::Double => 7, Self::String => 8, Self::Object => 9,
            Self::NullObject => 10, Self::ObjectRef => 11,
            Self::ExtObjectRef => 12, Self::ByteArray => 13,
            Self::Int16Array => 14, Self::Int32Array => 15,
            Self::Int64Array => 16, Self::BoolArray => 17,
            Self::ObjectList => 18, Self::StringToObjectMap => 20,
            Self::Uuid => 21, Self::Color => 22,
            Self::FloatArray => 23, Self::DoubleArray => 24,
            Self::StringArray => 25, Self::RelativeRef => 26,
        }
    }
}

/// A value in the Ramona binary v2 object tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum RamonaValue {
    Byte(u8), Int16(i16), Int32(i32), Int64(i64),
    Bool(bool), Float(f32), Double(f64),
    String(String),
    WideString(String),
    Uuid(Uuid), Color([f32; 4]),
    NullObject,
    ObjectRef(i32), ExtObjectRef(i32),
    Object(Box<RamonaObject>),
    ObjectList(Vec<RamonaListItem>),
    ByteArray(Vec<u8>), Int16Array(Vec<i16>), Int32Array(Vec<i32>),
    Int64Array(Vec<i64>), FloatArray(Vec<f32>), DoubleArray(Vec<f64>),
    BoolArray(Vec<bool>), StringArray(Vec<String>),
    StringToObjectMap(Vec<(String, RamonaValue)>),
    /// RelativeRef: a discriminated reference + path string.
    /// Target is either an inline object or an ObjectRef id.
    /// Path is a field-id path string (e.g. "10493").
    RelativeRef(Box<RelativeRefData>),
}

/// Data for a RelativeRef wire type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelativeRefData {
    pub target: RelativeRefTarget,
    pub path: String,
}

/// The target of a RelativeRef — either an inline object or an ObjectRef.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RelativeRefTarget {
    Object(RamonaObject),
    ObjectRef { id: i32 },
}

impl RelativeRefData {
    /// Get a reference to the inline object, if this is an Object variant.
    pub fn as_object(&self) -> Option<&RamonaObject> {
        match &self.target {
            RelativeRefTarget::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Get a mutable reference to the inline object, if this is an Object variant.
    pub fn as_object_mut(&mut self) -> Option<&mut RamonaObject> {
        match &mut self.target {
            RelativeRefTarget::Object(obj) => Some(obj),
            _ => None,
        }
    }
}

impl RamonaValue {
    /// Returns the wire type tag for this value.
    pub fn type_tag(&self) -> TypeTag {
        match self {
            Self::Byte(_) => TypeTag::Byte,
            Self::Int16(_) => TypeTag::Int16,
            Self::Int32(_) => TypeTag::Int32,
            Self::Int64(_) => TypeTag::Int64,
            Self::Bool(_) => TypeTag::Bool,
            Self::Float(_) => TypeTag::Float,
            Self::Double(_) => TypeTag::Double,
            Self::String(_) | Self::WideString(_) => TypeTag::String,
            Self::Object(_) => TypeTag::Object,
            Self::NullObject => TypeTag::NullObject,
            Self::ObjectRef(_) => TypeTag::ObjectRef,
            Self::ExtObjectRef(_) => TypeTag::ExtObjectRef,
            Self::ByteArray(_) => TypeTag::ByteArray,
            Self::Int16Array(_) => TypeTag::Int16Array,
            Self::Int32Array(_) => TypeTag::Int32Array,
            Self::Int64Array(_) => TypeTag::Int64Array,
            Self::BoolArray(_) => TypeTag::BoolArray,
            Self::ObjectList(_) => TypeTag::ObjectList,
            Self::StringToObjectMap(_) => TypeTag::StringToObjectMap,
            Self::Uuid(_) => TypeTag::Uuid,
            Self::Color(_) => TypeTag::Color,
            Self::FloatArray(_) => TypeTag::FloatArray,
            Self::DoubleArray(_) => TypeTag::DoubleArray,
            Self::StringArray(_) => TypeTag::StringArray,
            Self::RelativeRef(_) => TypeTag::RelativeRef,
        }
    }
}

/// An item in a Ramona object list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RamonaListItem {
    Null,
    Object(RamonaObject),
    ObjectRef(i32),
    ExtObjectRef(i32),
    RelativeRef(RelativeRefData),
}

/// Key for a field in a RamonaObject — either a numeric ID or a string name.
///
/// Serializes as `"field_name#123"` for known IDs, `"#123"` for unknown IDs,
/// and plain `"name"` for string-keyed fields. Deserializer parses the ID
/// from the `#` suffix, so the human-readable prefix is purely cosmetic.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldKey {
    Id(i32),
    Name(String),
}

impl Serialize for FieldKey {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            FieldKey::Id(id) => {
                if let Some(name) = ramona_field_name(*id) {
                    s.serialize_str(&format!("{name}#{id}"))
                } else {
                    s.serialize_str(&format!("#{id}"))
                }
            }
            FieldKey::Name(name) => s.serialize_str(name),
        }
    }
}

impl<'de> Deserialize<'de> for FieldKey {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        // Find the last '#' — everything after it is the numeric ID.
        // "field_name#123" → Id(123), "#123" → Id(123), "foo" → Name("foo")
        if let Some(hash_pos) = s.rfind('#') {
            let after = &s[hash_pos + 1..];
            if let Ok(id) = after.parse::<i32>() {
                return Ok(FieldKey::Id(id));
            }
        }
        Ok(FieldKey::Name(s))
    }
}

impl fmt::Display for FieldKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FieldKey::Id(id) => {
                if let Some(name) = ramona_field_name(*id) {
                    write!(f, "{name}")
                } else {
                    write!(f, "#{id}")
                }
            }
            FieldKey::Name(name) => write!(f, "\"{name}\""),
        }
    }
}

/// A Ramona serialized object with class identity and ordered fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RamonaObject {
    pub class_id: i32,
    pub class_name: String,
    /// Tracks which string-keyed field names were wide-encoded in the original binary.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wide_names: Vec<bool>,
    /// True if the class_name was wide-encoded.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub wide_class_name: bool,
    pub fields: IndexMap<FieldKey, RamonaValue>,
    /// Stable identity for this object, used to resolve ObjectRef across structural
    /// edits.  Assigned on load, not part of the wire format or JSON export.
    #[serde(skip, default)]
    pub stable_id: i32,
}

impl RamonaObject {
    pub fn class_display_name(&self) -> String {
        if !self.class_name.is_empty() {
            if self.class_id == 4 {
                self.class_name.clone()
            } else {
                format!("{} /*{}*/", self.class_name, self.class_id)
            }
        } else {
            format!("class#{}", self.class_id)
        }
    }
}

/// BtWg file header (42-byte hex-ASCII prefix).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BtWgHeader {
    pub format_version: u32,
    pub serialization_format: u32,
    pub document_version: u32,
    pub object_offset: u64,
    pub resources_offset: u64,
    pub flags: u16,
}

/// Detected Bitwig JAR version from embedded version string.
#[derive(Debug, Clone)]
pub struct JarVersion {
    pub version: String,
    pub revision: String,
}

/// Top-level parsed representation of a .bwmodule file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub header: BtWgHeader,
    pub metadata: RamonaObject,
    pub body: RamonaObject,
    #[serde(with = "hex_bytes")]
    pub meta_nonce: [u8; 16],
    #[serde(with = "hex_bytes")]
    pub body_nonce: [u8; 16],
}

mod hex_bytes {
    use serde::{Serializer, Deserializer, Deserialize};

    pub fn serialize<S: Serializer>(bytes: &[u8; 16], s: S) -> Result<S::Ok, S::Error> {
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        s.serialize_str(&hex)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 16], D::Error> {
        let hex = String::deserialize(d)?;
        let bytes: Vec<u8> = (0..hex.len()).step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i+2], 16).map_err(serde::de::Error::custom))
            .collect::<Result<_, _>>()?;
        bytes.try_into().map_err(|_| serde::de::Error::custom("expected 16 bytes"))
    }
}
