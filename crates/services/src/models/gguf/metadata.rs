use crate::models::gguf::{GGUFMetadataError, GGUFValue, GGUFValueType, GGUF_MAGIC};
use byteorder::{ByteOrder, ReadBytesExt, BE, LE};
use memmap2::Mmap;
use std::{collections::BTreeMap, fs::File, marker::PhantomData, path::Path};

const KNOWN_VERSIONS: [u32; 3] = [1, 2, 3];

pub struct GGUFMetadata {
  version: u32,
  magic: u32,
  metadata: BTreeMap<String, GGUFValue>,
}

impl GGUFMetadata {
  pub fn new(path: &Path) -> Result<GGUFMetadata, GGUFMetadataError> {
    let file = File::open(path)?;
    // SAFETY: The file is opened read-only. We do not mutate the file while the mapping is live,
    // and the mapping is dropped before the function returns or any further I/O occurs on this file.
    let mmap = unsafe { Mmap::map(&file) }?;
    let mut cursor = 0;
    let magic = (&mmap[cursor..cursor + 4]).read_u32::<LE>()?;
    if magic != GGUF_MAGIC {
      return Err(GGUFMetadataError::InvalidMagic(magic));
    }
    cursor += 4;
    if cursor + 4 > mmap.len() {
      return Err(GGUFMetadataError::UnexpectedEOF);
    }
    let version = (&mmap[cursor..cursor + 4]).read_u32::<LE>()?;
    let known_versions_be = KNOWN_VERSIONS
      .iter()
      .map(|v| u32::from_le(v.to_be()))
      .collect::<Vec<u32>>();
    match version {
      v if KNOWN_VERSIONS.contains(&v) => GGUFReader::<LE>::parse(mmap, cursor, magic),
      v if known_versions_be.contains(&v) => GGUFReader::<BE>::parse(mmap, cursor, magic),
      _ => Err(GGUFMetadataError::MalformedVersion(version)),
    }
  }

  pub fn magic(&self) -> u32 {
    self.magic
  }

  pub fn version(&self) -> u32 {
    self.version
  }

  pub fn metadata(&self) -> &BTreeMap<String, GGUFValue> {
    &self.metadata
  }

  pub fn get(&self, key: &str) -> Option<&GGUFValue> {
    self.metadata.get(key)
  }

  pub fn contains_key(&self, key: &str) -> bool {
    self.metadata.contains_key(key)
  }
}

struct GGUFReader<T: ByteOrder> {
  mmap: Mmap,
  version: u32,
  magic: u32,
  cursor: usize,
  metadata: BTreeMap<String, GGUFValue>,
  _phantom: PhantomData<T>,
}

impl<T: ByteOrder> GGUFReader<T> {
  fn parse(mmap: Mmap, mut cursor: usize, magic: u32) -> Result<GGUFMetadata, GGUFMetadataError> {
    let version = (&mmap[cursor..cursor + 4]).read_u32::<T>()?;
    cursor += 4;
    if !Self::is_version_supported(version) {
      return Err(GGUFMetadataError::UnsupportedVersion(version));
    }
    let mut reader = Self {
      mmap,
      version,
      magic,
      cursor,
      metadata: BTreeMap::new(),
      _phantom: PhantomData,
    };
    let _num_tensors = reader.read_u64()? as usize; // num_tensors
    let num_kv = reader.read_u64()? as usize;

    for _ in 0..num_kv {
      let key = reader.read_string()?;
      let value = reader.read_value()?;
      reader.metadata.insert(key, value);
    }

    Ok(GGUFMetadata {
      version: reader.version,
      magic: reader.magic,
      metadata: reader.metadata,
    })
  }

  fn is_version_supported(version: u32) -> bool {
    (2..=3).contains(&version)
  }

  fn read_u64(&mut self) -> Result<u64, GGUFMetadataError> {
    if self.cursor + 8 > self.mmap.len() {
      return Err(GGUFMetadataError::UnexpectedEOF);
    }
    let value = T::read_u64(&self.mmap[self.cursor..self.cursor + 8]);
    self.cursor += 8;
    Ok(value)
  }

  fn read_string(&mut self) -> Result<String, GGUFMetadataError> {
    let len = self.read_u64()? as usize;
    if self.cursor + len > self.mmap.len() {
      return Err(GGUFMetadataError::UnexpectedEOF);
    }
    let bytes = &self.mmap[self.cursor..self.cursor + len];
    self.cursor += len;
    Ok(String::from_utf8(bytes.to_vec())?)
  }

  fn read_value(&mut self) -> Result<GGUFValue, GGUFMetadataError> {
    let type_id = self.read_u32()?;
    let value_type = GGUFValueType::try_from(type_id)?;

    match value_type {
      GGUFValueType::UINT8 => Ok(GGUFValue::U8(self.read_u8()?)),
      GGUFValueType::INT8 => Ok(GGUFValue::I8(self.read_i8()?)),
      GGUFValueType::UINT16 => Ok(GGUFValue::U16(self.read_u16()?)),
      GGUFValueType::INT16 => Ok(GGUFValue::I16(self.read_i16()?)),
      GGUFValueType::UINT32 => Ok(GGUFValue::U32(self.read_u32()?)),
      GGUFValueType::INT32 => Ok(GGUFValue::I32(self.read_i32()?)),
      GGUFValueType::UINT64 => Ok(GGUFValue::U64(self.read_u64()?)),
      GGUFValueType::INT64 => Ok(GGUFValue::I64(self.read_i64()?)),
      GGUFValueType::FLOAT32 => Ok(GGUFValue::F32(self.read_f32()?)),
      GGUFValueType::FLOAT64 => Ok(GGUFValue::F64(self.read_f64()?)),
      GGUFValueType::BOOL => Ok(GGUFValue::Bool(self.read_bool()?)),
      GGUFValueType::STRING => Ok(GGUFValue::String(self.read_string()?)),
      GGUFValueType::ARRAY => self.read_array(),
    }
  }

  fn read_u8(&mut self) -> Result<u8, GGUFMetadataError> {
    if self.cursor + 1 > self.mmap.len() {
      return Err(GGUFMetadataError::UnexpectedEOF);
    }
    let value = self.mmap[self.cursor];
    self.cursor += 1;
    Ok(value)
  }

  fn read_i8(&mut self) -> Result<i8, GGUFMetadataError> {
    Ok(self.read_u8()? as i8)
  }

  fn read_u16(&mut self) -> Result<u16, GGUFMetadataError> {
    if self.cursor + 2 > self.mmap.len() {
      return Err(GGUFMetadataError::UnexpectedEOF);
    }
    let value = T::read_u16(&self.mmap[self.cursor..self.cursor + 2]);
    self.cursor += 2;
    Ok(value)
  }

  fn read_i16(&mut self) -> Result<i16, GGUFMetadataError> {
    if self.cursor + 2 > self.mmap.len() {
      return Err(GGUFMetadataError::UnexpectedEOF);
    }
    let value = T::read_i16(&self.mmap[self.cursor..self.cursor + 2]);
    self.cursor += 2;
    Ok(value)
  }

  fn read_u32(&mut self) -> Result<u32, GGUFMetadataError> {
    if self.cursor + 4 > self.mmap.len() {
      return Err(GGUFMetadataError::UnexpectedEOF);
    }
    let value = T::read_u32(&self.mmap[self.cursor..self.cursor + 4]);
    self.cursor += 4;
    Ok(value)
  }

  fn read_i32(&mut self) -> Result<i32, GGUFMetadataError> {
    if self.cursor + 4 > self.mmap.len() {
      return Err(GGUFMetadataError::UnexpectedEOF);
    }
    let value = T::read_i32(&self.mmap[self.cursor..self.cursor + 4]);
    self.cursor += 4;
    Ok(value)
  }

  fn read_i64(&mut self) -> Result<i64, GGUFMetadataError> {
    if self.cursor + 8 > self.mmap.len() {
      return Err(GGUFMetadataError::UnexpectedEOF);
    }
    let value = T::read_i64(&self.mmap[self.cursor..self.cursor + 8]);
    self.cursor += 8;
    Ok(value)
  }

  fn read_f32(&mut self) -> Result<f32, GGUFMetadataError> {
    if self.cursor + 4 > self.mmap.len() {
      return Err(GGUFMetadataError::UnexpectedEOF);
    }
    let value = T::read_f32(&self.mmap[self.cursor..self.cursor + 4]);
    self.cursor += 4;
    Ok(value)
  }

  fn read_f64(&mut self) -> Result<f64, GGUFMetadataError> {
    if self.cursor + 8 > self.mmap.len() {
      return Err(GGUFMetadataError::UnexpectedEOF);
    }
    let value = T::read_f64(&self.mmap[self.cursor..self.cursor + 8]);
    self.cursor += 8;
    Ok(value)
  }

  fn read_bool(&mut self) -> Result<bool, GGUFMetadataError> {
    Ok(self.read_u8()? != 0)
  }

  fn read_array(&mut self) -> Result<GGUFValue, GGUFMetadataError> {
    let item_type: GGUFValueType = self.read_u32()?.try_into()?;
    let len = self.read_u64()? as usize;
    let mut values = Vec::with_capacity(len);

    for _ in 0..len {
      let value = match item_type {
        GGUFValueType::UINT8 => GGUFValue::U8(self.read_u8()?),
        GGUFValueType::INT8 => GGUFValue::I8(self.read_i8()?),
        GGUFValueType::UINT16 => GGUFValue::U16(self.read_u16()?),
        GGUFValueType::INT16 => GGUFValue::I16(self.read_i16()?),
        GGUFValueType::UINT32 => GGUFValue::U32(self.read_u32()?),
        GGUFValueType::INT32 => GGUFValue::I32(self.read_i32()?),
        GGUFValueType::FLOAT32 => GGUFValue::F32(self.read_f32()?),
        GGUFValueType::BOOL => GGUFValue::Bool(self.read_bool()?),
        GGUFValueType::STRING => GGUFValue::String(self.read_string()?),
        GGUFValueType::UINT64 => GGUFValue::U64(self.read_u64()?),
        GGUFValueType::INT64 => GGUFValue::I64(self.read_i64()?),
        GGUFValueType::FLOAT64 => GGUFValue::F64(self.read_f64()?),
        _ => return Err(GGUFMetadataError::InvalidArrayValueType(item_type as u32)),
      };
      values.push(value);
    }
    Ok(GGUFValue::Array(values))
  }
}
