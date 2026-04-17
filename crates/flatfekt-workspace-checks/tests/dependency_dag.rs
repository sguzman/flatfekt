use std::collections::{
  BTreeMap,
  BTreeSet
};

use anyhow::Context;
use cargo_metadata::{
  CargoOpt,
  MetadataCommand,
  Package
};

#[test]
fn dependency_dag_rules_hold()
-> anyhow::Result<()> {
  let metadata = MetadataCommand::new()
    .features(CargoOpt::AllFeatures)
    .exec()
    .context(
      "failed to run cargo metadata"
    )?;

  let packages_by_name: BTreeMap<
    &str,
    &Package
  > = metadata
    .packages
    .iter()
    .map(|p| (p.name.as_str(), p))
    .collect();

  // Schema/config/tooling crates must
  // not pull Bevy (renderer) dependency
  // graph.
  let must_be_bevy_free: BTreeSet<
    &str
  > = BTreeSet::from_iter([
    "flatfekt-config",
    "flatfekt-schema"
  ]);

  for name in &must_be_bevy_free {
    let pkg = packages_by_name
      .get(name)
      .copied()
      .with_context(|| {
        format!(
          "missing package {name:?}"
        )
      })?;

    let deps: BTreeSet<&str> = pkg
      .dependencies
      .iter()
      .map(|d| d.name.as_str())
      .collect();

    if deps.contains("bevy") {
      anyhow::bail!(
        "{name} depends on bevy but \
         must remain Bevy-free"
      );
    }
    if deps.contains("wgpu") {
      anyhow::bail!(
        "{name} depends on wgpu but \
         must remain Bevy-free"
      );
    }
    if deps.contains("winit") {
      anyhow::bail!(
        "{name} depends on winit but \
         must remain Bevy-free"
      );
    }
  }

  // Engine crates must not depend on
  // apps.
  let apps: BTreeSet<&str> =
    BTreeSet::from_iter([
      "flatfekt-viewer"
    ]);

  let engine_crates: BTreeSet<&str> =
    BTreeSet::from_iter([
      "flatfekt-config",
      "flatfekt-schema",
      "flatfekt-runtime"
    ]);

  for name in &engine_crates {
    let pkg = packages_by_name
      .get(name)
      .copied()
      .with_context(|| {
        format!(
          "missing package {name:?}"
        )
      })?;

    let deps: BTreeSet<&str> = pkg
      .dependencies
      .iter()
      .map(|d| d.name.as_str())
      .collect();

    for app in &apps {
      if deps.contains(app) {
        anyhow::bail!(
          "engine crate {name} must \
           not depend on app {app}"
        );
      }
    }
  }

  Ok(())
}
