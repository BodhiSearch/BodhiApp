use crate::gguf::{GGUFMetadataError, GGUFValue, GGUFValueType, GGUF_MAGIC};
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
    let mmap = unsafe { Mmap::map(&file).unwrap() };
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

#[cfg(test)]
mod tests {
  use crate::{
    gguf::{GGUFMetadata, GGUFValue, GGUF_MAGIC},
    test_utils::generate_test_data_gguf_metadata,
  };
  use anyhow_trace::anyhow_trace;
  use rstest::rstest;
  use std::path::PathBuf;

  // Common test helper to verify basic metadata
  fn verify_basic_metadata(metadata: &GGUFMetadata) {
    assert_eq!(metadata.magic(), GGUF_MAGIC);
    assert_eq!(metadata.version(), 3);
    assert_eq!(
      metadata
        .metadata()
        .get("general.architecture")
        .unwrap()
        .as_str()
        .unwrap(),
      "llama"
    );
  }

  #[anyhow_trace]
  #[rstest]
  #[case::le("tests/data/gguf/sample0_le.gguf", 3)]
  #[case::be("tests/data/gguf/sample0_be.gguf", 3)]
  fn test_gguf_metadata_endian(
    #[from(generate_test_data_gguf_metadata)] _setup: &(),
    #[case] input: &str,
    #[case] version: u32,
  ) -> anyhow::Result<()> {
    let metadata = GGUFMetadata::new(PathBuf::from(input).as_path())?;
    assert_eq!(metadata.magic(), GGUF_MAGIC);
    assert_eq!(metadata.version(), version);
    assert_eq!(metadata.metadata().len(), 1);

    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  fn test_gguf_metadata_sample0_files(
    #[from(generate_test_data_gguf_metadata)] _setup: &(),
    #[files("tests/data/gguf/sample0_*.gguf")] input: PathBuf,
  ) -> anyhow::Result<()> {
    let metadata = GGUFMetadata::new(input.as_path())?;
    verify_basic_metadata(&metadata);
    // Sample0 should only have the architecture field
    assert_eq!(metadata.metadata().len(), 1);
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  fn test_gguf_metadatasample1_files(
    #[from(generate_test_data_gguf_metadata)] _setup: &(),
    #[files("tests/data/gguf/sample1_*.gguf")] input: PathBuf,
  ) -> anyhow::Result<()> {
    let metadata = GGUFMetadata::new(input.as_path())?;
    verify_basic_metadata(&metadata);

    let md = metadata.metadata();
    // Test all KV data types
    assert_eq!(md.get("test_uint8").unwrap().as_u8()?, 255);
    assert_eq!(md.get("test_int8").unwrap().as_i8()?, -128);
    assert_eq!(md.get("test_uint16").unwrap().as_u16()?, 65535);
    assert_eq!(md.get("test_int16").unwrap().as_i16()?, -32768);
    assert_eq!(md.get("test_uint32").unwrap().as_u32()?, 4294967295);
    assert_eq!(md.get("test_int32").unwrap().as_i32()?, -2147483648);
    assert_eq!(
      md.get("test_uint64").unwrap().as_u64()?,
      18446744073709551615
    );
    assert_eq!(
      md.get("test_int64").unwrap().as_i64()?,
      -9223372036854775808
    );
    assert!((md.get("test_float32").unwrap().as_f32()? - 3.14159).abs() < f32::EPSILON);
    assert!((md.get("test_float64").unwrap().as_f64()? - 2.718281828459045).abs() < f64::EPSILON);
    assert_eq!(md.get("test_bool").unwrap().as_bool()?, true);
    assert_eq!(md.get("test_string").unwrap().as_str()?, "Hello GGUF!");

    // Test arrays
    if let GGUFValue::Array(arr) = md.get("test_array_int").unwrap() {
      assert_eq!(arr.len(), 5);
      for (i, val) in arr.iter().enumerate() {
        assert_eq!(val.as_i32()?, (i + 1) as i32);
      }
    }

    if let GGUFValue::Array(arr) = md.get("test_array_str").unwrap() {
      assert_eq!(arr.len(), 3);
      assert_eq!(arr[0].as_str()?, "a");
      assert_eq!(arr[1].as_str()?, "b");
      assert_eq!(arr[2].as_str()?, "c");
    }

    // Test original KV data
    assert_eq!(md.get("context_length").unwrap().as_u32()?, 2048);
    assert!((md.get("rope_freq_base").unwrap().as_f32()? - 10000.0).abs() < f32::EPSILON);

    // Total number of KV pairs (including general.architecture)
    assert_eq!(metadata.metadata().len(), 17);
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  fn test_gguf_metadata_sample_tokens_files(
    #[from(generate_test_data_gguf_metadata)] _setup: &(),
    #[files("tests/data/gguf/sample_tokens*.gguf")] input: PathBuf,
  ) -> anyhow::Result<()> {
    let metadata = GGUFMetadata::new(input.as_path())?;
    verify_basic_metadata(&metadata);

    let md = metadata.metadata();
    // Basic token info
    assert_eq!(md.get("vocab_size").unwrap().as_u32()?, 100);

    // Special tokens
    assert_eq!(md.get("tokenizer.ggml.bos_token_id").unwrap().as_u32()?, 1);
    assert_eq!(md.get("tokenizer.ggml.eos_token_id").unwrap().as_u32()?, 2);
    assert_eq!(
      md.get("tokenizer.ggml.padding_token_id")
        .unwrap()
        .as_u32()?,
      3
    );
    assert_eq!(
      md.get("tokenizer.ggml.seperator_token_id")
        .unwrap()
        .as_u32()?,
      4
    );

    // Token list
    if let GGUFValue::Array(tokens) = md.get("tokenizer.ggml.tokens").unwrap() {
      assert_eq!(tokens.len(), 3);
      assert_eq!(tokens[0].as_str()?, "<s>");
      assert_eq!(tokens[1].as_str()?, "</s>");
      assert_eq!(tokens[2].as_str()?, "<pad>");
    }

    // Token settings
    assert_eq!(
      md.get("tokenizer.ggml.add_bos_token").unwrap().as_bool()?,
      true
    );
    assert_eq!(
      md.get("tokenizer.ggml.add_eos_token").unwrap().as_bool()?,
      true
    );
    assert_eq!(
      md.get("tokenizer.ggml.add_space_prefix")
        .unwrap()
        .as_bool()?,
      true
    );

    // Tokenizer settings
    assert_eq!(md.get("tokenizer.ggml.model").unwrap().as_str()?, "llama");
    assert_eq!(
      md.get("tokenizer.ggml.remove_extra_whitespaces")
        .unwrap()
        .as_bool()?,
      true
    );

    Ok(())
  }
}
