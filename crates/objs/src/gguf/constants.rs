use crate::gguf::GGUFMetadataError;
use strum::Display;

pub const GGUF_MAGIC: u32 = 0x46554747;
pub const GGUF_VERSION: u32 = 3;

pub const GGUF_LE: u32 = 0;
pub const GGUF_BE: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum GGUFValueType {
  UINT8 = 0,
  INT8 = 1,
  UINT16 = 2,
  INT16 = 3,
  UINT32 = 4,
  INT32 = 5,
  FLOAT32 = 6,
  BOOL = 7,
  STRING = 8,
  ARRAY = 9,
  UINT64 = 10,
  INT64 = 11,
  FLOAT64 = 12,
}

impl TryFrom<u32> for GGUFValueType {
  type Error = GGUFMetadataError;

  fn try_from(value: u32) -> Result<Self, Self::Error> {
    let value_type = match value {
      0 => GGUFValueType::UINT8,
      1 => GGUFValueType::INT8,
      2 => GGUFValueType::UINT16,
      3 => GGUFValueType::INT16,
      4 => GGUFValueType::UINT32,
      5 => GGUFValueType::INT32,
      6 => GGUFValueType::FLOAT32,
      7 => GGUFValueType::BOOL,
      8 => GGUFValueType::STRING,
      9 => GGUFValueType::ARRAY,
      10 => GGUFValueType::UINT64,
      11 => GGUFValueType::INT64,
      12 => GGUFValueType::FLOAT64,
      _ => return Err(GGUFMetadataError::InvalidValueType(value)),
    };
    Ok(value_type)
  }
}

#[derive(Debug, Clone, PartialEq, Display)]
pub enum GGUFValue {
  // Unsigned integers
  U8(u8),
  U16(u16),
  U32(u32),
  U64(u64),

  // Signed integers
  I8(i8),
  I16(i16),
  I32(i32),
  I64(i64),

  // Floating point
  F32(f32),
  F64(f64),

  // Bool
  Bool(bool),

  // String (using String to own the data)
  String(String),

  // Array types
  Array(Vec<GGUFValue>),
}

impl GGUFValue {
  pub fn as_str(&self) -> Result<&str, GGUFMetadataError> {
    match self {
      GGUFValue::String(s) => Ok(s.as_str()),
      _ => Err(GGUFMetadataError::TypeMismatch {
        expected: "String".to_string(),
        actual: self.to_string(),
      }),
    }
  }

  pub fn as_u8(&self) -> Result<u8, GGUFMetadataError> {
    match self {
      GGUFValue::U8(v) => Ok(*v),
      _ => Err(GGUFMetadataError::TypeMismatch {
        expected: "U8".to_string(),
        actual: self.to_string(),
      }),
    }
  }

  pub fn as_u16(&self) -> Result<u16, GGUFMetadataError> {
    match self {
      GGUFValue::U16(v) => Ok(*v),
      _ => Err(GGUFMetadataError::TypeMismatch {
        expected: "U16".to_string(),
        actual: self.to_string(),
      }),
    }
  }

  pub fn as_u32(&self) -> Result<u32, GGUFMetadataError> {
    match self {
      GGUFValue::U32(v) => Ok(*v),
      _ => Err(GGUFMetadataError::TypeMismatch {
        expected: "U32".to_string(),
        actual: self.to_string(),
      }),
    }
  }

  pub fn as_u64(&self) -> Result<u64, GGUFMetadataError> {
    match self {
      GGUFValue::U64(v) => Ok(*v),
      _ => Err(GGUFMetadataError::TypeMismatch {
        expected: "U64".to_string(),
        actual: self.to_string(),
      }),
    }
  }

  pub fn as_i8(&self) -> Result<i8, GGUFMetadataError> {
    match self {
      GGUFValue::I8(v) => Ok(*v),
      _ => Err(GGUFMetadataError::TypeMismatch {
        expected: "I8".to_string(),
        actual: self.to_string(),
      }),
    }
  }

  pub fn as_i16(&self) -> Result<i16, GGUFMetadataError> {
    match self {
      GGUFValue::I16(v) => Ok(*v),
      _ => Err(GGUFMetadataError::TypeMismatch {
        expected: "I16".to_string(),
        actual: self.to_string(),
      }),
    }
  }

  pub fn as_i32(&self) -> Result<i32, GGUFMetadataError> {
    match self {
      GGUFValue::I32(v) => Ok(*v),
      _ => Err(GGUFMetadataError::TypeMismatch {
        expected: "I32".to_string(),
        actual: self.to_string(),
      }),
    }
  }

  pub fn as_i64(&self) -> Result<i64, GGUFMetadataError> {
    match self {
      GGUFValue::I64(v) => Ok(*v),
      _ => Err(GGUFMetadataError::TypeMismatch {
        expected: "I64".to_string(),
        actual: self.to_string(),
      }),
    }
  }

  pub fn as_f32(&self) -> Result<f32, GGUFMetadataError> {
    match self {
      GGUFValue::F32(v) => Ok(*v),
      _ => Err(GGUFMetadataError::TypeMismatch {
        expected: "F32".to_string(),
        actual: self.to_string(),
      }),
    }
  }

  pub fn as_f64(&self) -> Result<f64, GGUFMetadataError> {
    match self {
      GGUFValue::F64(v) => Ok(*v),
      _ => Err(GGUFMetadataError::TypeMismatch {
        expected: "F64".to_string(),
        actual: self.to_string(),
      }),
    }
  }

  pub fn as_bool(&self) -> Result<bool, GGUFMetadataError> {
    match self {
      GGUFValue::Bool(v) => Ok(*v),
      _ => Err(GGUFMetadataError::TypeMismatch {
        expected: "Bool".to_string(),
        actual: self.to_string(),
      }),
    }
  }

  pub fn as_array(&self) -> Result<&Vec<GGUFValue>, GGUFMetadataError> {
    match self {
      GGUFValue::Array(v) => Ok(v),
      _ => Err(GGUFMetadataError::TypeMismatch {
        expected: "Array".to_string(),
        actual: self.to_string(),
      }),
    }
  }
}
