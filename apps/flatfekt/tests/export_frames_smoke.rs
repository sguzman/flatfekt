#[cfg(feature = "gpu_tests")]
mod gpu {
  use std::path::PathBuf;

  use assert_cmd::Command;
  use predicates::prelude::*;

  fn want_gpu_tests() -> bool {
    std::env::var("FLATFEKT_GPU_TESTS")
      .ok()
      .as_deref()
      == Some("1")
  }

  #[test]
  fn export_frames_smoke_to_temp_dir() {
    if !want_gpu_tests() {
      eprintln!(
        "skipping GPU export smoke; \
         set FLATFEKT_GPU_TESTS=1 to \
         enable"
      );
      return;
    }

    let temp = tempfile::tempdir()
      .expect("tempdir");
    let root = temp.path();

    let bake_root = root.join("bake");
    let export_root =
      root.join("export");

    let scene = PathBuf::from(
      "scenes/physics_test.toml"
    );

    let bake =
      Command::cargo_bin("flatfekt")
        .expect("flatfekt binary")
        .arg("--config")
        .arg(
          ".config/flatfekt/flatfekt.\
           toml"
        )
        .arg("bake")
        .arg(&scene)
        .arg("--output-root")
        .arg(&bake_root)
        .arg("--fps")
        .arg("10")
        .arg("--duration-secs")
        .arg("0.1")
        .output()
        .expect("run bake");

    assert!(
      bake.status.success(),
      "bake failed: {}",
      String::from_utf8_lossy(
        &bake.stderr
      )
    );

    let bake_dir =
      String::from_utf8_lossy(
        &bake.stdout
      )
      .lines()
      .next()
      .unwrap_or("")
      .trim()
      .to_owned();
    assert!(
      !bake_dir.is_empty(),
      "expected bake to print bake dir"
    );

    let export =
      Command::cargo_bin("flatfekt")
        .expect("flatfekt binary")
        .arg("--config")
        .arg(
          ".config/flatfekt/flatfekt.\
           toml"
        )
        .arg("export-frames")
        .arg(&bake_dir)
        .arg("--output-root")
        .arg(&export_root)
        .arg("--fps")
        .arg("10")
        .arg("--duration-secs")
        .arg("0.1")
        .arg("--width")
        .arg("64")
        .arg("--height")
        .arg("64")
        .arg("--overwrite")
        .output()
        .expect("run export-frames");

    assert!(
      export.status.success(),
      "export-frames failed: {}",
      String::from_utf8_lossy(
        &export.stderr
      )
    );

    let export_dir =
      String::from_utf8_lossy(
        &export.stdout
      )
      .lines()
      .next()
      .unwrap_or("")
      .trim()
      .to_owned();

    assert!(
      !export_dir.is_empty(),
      "expected export to print \
       export dir"
    );

    let export_dir =
      PathBuf::from(export_dir);
    let frames_dir =
      export_dir.join("frames");
    let manifest =
      export_dir.join("export.json");

    assert!(
      manifest.exists(),
      "expected export.json at {}",
      manifest.display()
    );

    let mut pngs: Vec<PathBuf> =
      Vec::new();
    if let Ok(rd) =
      std::fs::read_dir(&frames_dir)
    {
      for e in rd.flatten() {
        let p = e.path();
        if p
          .extension()
          .and_then(|s| s.to_str())
          == Some("png")
        {
          pngs.push(p);
        }
      }
    }
    assert!(
      !pngs.is_empty(),
      "expected at least one png in {}",
      frames_dir.display()
    );

    Command::cargo_bin("flatfekt")
      .expect("flatfekt binary")
      .arg("--config")
      .arg(
        ".config/flatfekt/flatfekt.\
         toml"
      )
      .arg("export-mp4")
      .arg(&bake_dir)
      .arg("--overwrite")
      .assert()
      .stdout(
        predicate::str::contains(
          ".mp4"
        )
      );
  }
}
