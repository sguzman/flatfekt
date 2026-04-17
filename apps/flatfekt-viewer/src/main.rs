use std::path::PathBuf;

use flatfekt_config::RootConfig;
use flatfekt_runtime::Runtime;
use tracing::{info, warn};

fn main() -> anyhow::Result<()> {
    init_tracing();

    let config_path = std::env::var_os("FLATFEKT_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("flatfekt.toml"));

    let cfg = match RootConfig::load_from_path(&config_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            warn!(error = %err, "config load failed (continuing with defaults)");
            RootConfig {
                app: None,
                logging: None,
            }
        }
    };

    info!(?cfg, "loaded config");

    let rt = Runtime::new();
    let _ = rt;

    Ok(())
}

fn init_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}

