use crate::models::{ApiAliasRepository, DownloadRepository, ModelMetadataRepository};

/// Combined model repository trait (backward compatibility supertrait).
/// Downstream code that uses `ModelRepository` still works since
/// any implementor of the 3 sub-traits automatically satisfies this.
pub trait ModelRepository:
  DownloadRepository + ApiAliasRepository + ModelMetadataRepository
{
}

impl<T> ModelRepository for T where
  T: DownloadRepository + ApiAliasRepository + ModelMetadataRepository
{
}
