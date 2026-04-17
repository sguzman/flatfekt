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

pub fn load_pack(
  spec: &AssetPackSpec
) -> Result<Vec<PathBuf>, PackError> {
  if !spec.root.exists() {
    return Err(PackError::NotFound(
      spec.root.clone()
    ));
  }
  let manifest_path =
    spec.root.join("manifest.toml");
  if !manifest_path.exists() {
    return Err(PackError::Manifest(format!(
      "manifest.toml not found in {}",
      spec.root.display()
    )));
  }
  tracing::info!(
    name = %spec.name,
    root = %spec.root.display(),
    "Loading asset pack"
  );
  let mut assets = Vec::new();
  if let Ok(entries) =
    std::fs::read_dir(&spec.root)
  {
    for entry in entries.flatten() {
      let path = entry.path();
      if path.is_file()
        && path.extension().and_then(|e| {
          e.to_str()
        }) != Some("toml")
      {
        assets.push(path);
      }
    }
  }
  Ok(assets)
}
