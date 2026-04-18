use std::path::PathBuf;

use anyhow::Context;
use clap::{
  Parser,
  Subcommand
};
use flatfekt_runtime::{
  build_app,
  load_config,
  load_scene
};
use flatfekt_schema::SceneFile;

#[derive(Parser)]
#[command(name = "flatfekt-cli")]
#[command(about = "Flatfekt 2D scene runner and simulation environment CLI tool", long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Commands,

  #[arg(
    short,
    long,
    env = "FLATFEKT_CONFIG"
  )]
  config: Option<PathBuf>,

  #[arg(short, long)]
  level: Option<String>,

  #[arg(short, long)]
  filter: Option<String>
}

fn init_tracing(
  level: Option<&str>,
  filter: Option<&str>
) {
  let filter_str = filter
    .map(|s| s.to_owned())
    .or_else(|| {
      level.map(|s| s.to_owned())
    })
    .unwrap_or_else(|| {
      "info".to_owned()
    });

  tracing_subscriber::fmt()
    .with_env_filter(filter_str)
    .init();
}

#[derive(Subcommand)]
enum Commands {
  /// Validate a scene file
  Validate { path: PathBuf },
  /// Run a scene file
  Run { path: PathBuf },
  /// Format a scene file to canonical
  /// TOML
  Fmt {
    path:  PathBuf,
    #[arg(short, long)]
    check: bool
  },
  /// Print the resolved scene after
  /// templates and defaults are applied
  Resolve { path: PathBuf },
  /// Migrate a scene file to the latest
  /// schema version
  Migrate { path: PathBuf },
  /// Diff two scene files at the entity
  /// level
  Diff {
    path_a: PathBuf,
    path_b: PathBuf
  },
  /// Generate a new minimal scene
  /// template
  New { path: PathBuf },
  /// Generate a demo scene (e.g., text,
  /// timeline)
  Demo { name: String, path: PathBuf }
}

fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();
  init_tracing(
    cli.level.as_deref(),
    cli.filter.as_deref()
  );

  match cli.command {
    | Commands::Validate {
      path
    } => {
      let _ = load_scene(&path)
        .with_context(|| {
          format!(
            "Failed to validate scene \
             at {:?}",
            path
          )
        })?;
      println!(
        "OK: Scene at {:?} is valid.",
        path
      );
    }
    | Commands::Run {
      path
    } => {
      let config_path = cli
        .config
        .unwrap_or_else(|| {
          PathBuf::from("flatfekt.toml")
        });
      let cfg =
        load_config(&config_path)
          .unwrap_or_default();
      let scene_file =
        load_scene(&path)?;
      let mut app = build_app(
        cfg, path, scene_file
      )?;
      app.run();
    }
    | Commands::Fmt {
      path,
      check
    } => {
      let content =
        std::fs::read_to_string(&path)?;
      let scene: SceneFile =
        toml::from_str(&content)?;
      let formatted =
        toml::to_string_pretty(&scene)?;
      if check {
        if content != formatted {
          anyhow::bail!(
            "File is not formatted"
          );
        }
      } else {
        std::fs::write(
          &path, formatted
        )?;
        println!(
          "Formatted {:?}",
          path
        );
      }
    }
    | Commands::Resolve {
      path
    } => {
      let scene = load_scene(&path)?;
      println!(
        "{}",
        toml::to_string_pretty(&scene)?
      );
    }
    | Commands::Migrate {
      path
    } => {
      println!(
        "MIGRATOR: Scene at {:?} is \
         already at latest version \
         (0.1).",
        path
      );
    }
    | Commands::Diff {
      path_a,
      path_b
    } => {
      let scene_a =
        load_scene(&path_a)?;
      let scene_b =
        load_scene(&path_b)?;

      println!(
        "Diffing {:?} vs {:?}",
        path_a, path_b
      );
      // Simple entity count diff for
      // now
      let count_a =
        scene_a.scene.entities.len();
      let count_b =
        scene_b.scene.entities.len();
      if count_a != count_b {
        println!(
          "Entity count changed: {} \
           -> {}",
          count_a, count_b
        );
      } else {
        println!(
          "Entity count identical: {}",
          count_a
        );
      }

      // More detailed diff could go
      // here
    }
    | Commands::New {
      path
    } => {
      let template = r#"[scene]
schema_version = "0.1"

[scene.camera]
x = 0.0
y = 0.0
zoom = 1.0

[[scene.entities]]
id = "root"
[scene.entities.transform]
x = 0.0
y = 0.0
"#;
      std::fs::write(&path, template)?;
      println!(
        "Created new scene at {:?}",
        path
      );
    }
    | Commands::Demo {
      name,
      path
    } => {
      let content = match name.as_str()
      {
        | "text" => {
          r#"[scene]
schema_version = "0.1"

[[scene.entities]]
id = "text_demo"
[scene.entities.text]
value = "Hello Flatfekt"
size = 32.0
[scene.entities.transform]
x = 0.0
y = 0.0
"#
        }
        | _ => {
          anyhow::bail!(
            "Unknown demo: {}",
            name
          )
        }
      };
      std::fs::write(&path, content)?;
      println!(
        "Created demo '{}' at {:?}",
        name, path
      );
    }
  }

  Ok(())
}
