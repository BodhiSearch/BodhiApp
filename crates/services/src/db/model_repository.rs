use crate::models::{ApiAliasRepository, DownloadRepository, ModelMetadataRepository};

pub trait ModelRepository:
  DownloadRepository + ApiAliasRepository + ModelMetadataRepository
{
}

impl<T> ModelRepository for T where
  T: DownloadRepository + ApiAliasRepository + ModelMetadataRepository
{
}
