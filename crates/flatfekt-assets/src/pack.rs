use std::path::PathBuf;

use flatfekt_schema::AssetPackSpec;

#[derive(Debug, thiserror::Error)]
pub enum PackError {
  #[error(
    "failed to load pack manifest: {0}"
  )]
  Manifest(String),
  #[error("pack root not found: {0}")]
  NotFound(PathBuf)
}

pub fn load_pack_stub(
  spec: &AssetPackSpec
) -> Result<(), PackError> {
  if !spec.root.exists() {
    return Err(PackError::NotFound(
      spec.root.clone()
    ));
  }
  tracing::info!(
    name = %spec.name,
    root = %spec.root.display(),
    "STUB: Loading asset pack"
  );
  Ok(())
}
