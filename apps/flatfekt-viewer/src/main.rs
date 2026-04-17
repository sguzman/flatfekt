use std::path::PathBuf;

use flatfekt_config::{ConfigError, RootConfig};
use flatfekt_runtime::Runtime;
use tracing::{info, warn};

fn main() -> anyhow::Result<()> {
    init_tracing();

    let config_path = std::env::var_os("FLATFEKT_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("flatfekt.toml"));

    let cfg = load_config_or_fail_fast(&config_path)?;
    enforce_platform_and_render_defaults(&cfg)?;
    info!(?cfg, "loaded config");

    let scene_path = cfg
        .app
        .as_ref()
        .and_then(|a| a.scene_path.clone())
        .unwrap_or_else(|| PathBuf::from("scenes/demo.toml"));

    let scene_file = flatfekt_schema::SceneFile::load_from_path(&scene_path)?;
    info!(path = %scene_path.display(), "loaded scene");
    let _ = scene_file;

    let rt = Runtime::new();
    let _ = rt;

    Ok(())
}

fn init_tracing() {
    // Early subscriber: gets replaced by config-driven filter once config is loaded.
    let env_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_level(true)
        .init();
}

fn load_config_or_fail_fast(path: &PathBuf) -> anyhow::Result<RootConfig> {
    match RootConfig::load_from_path(path) {
        Ok(cfg) => Ok(cfg),
        Err(ConfigError::Read { .. }) if !path.exists() => {
            warn!(path = %path.display(), "config file not found; using built-in defaults");
            Ok(RootConfig {
                app: None,
                logging: None,
                platform: None,
                render: None,
            })
        }
        Err(err) => Err(err.into()),
    }
}

fn enforce_platform_and_render_defaults(cfg: &RootConfig) -> anyhow::Result<()> {
    if cfg.unix_backend() != "wayland" {
        anyhow::bail!(
            "unsupported unix backend {:?}; this project defaults to Wayland",
            cfg.unix_backend()
        );
    }
    if cfg.render_backend() != "vulkan" {
        anyhow::bail!(
            "unsupported render backend {:?}; this project requires Vulkan",
            cfg.render_backend()
        );
    }

    // Rust 2024: mutating process environment is `unsafe` because it can violate invariants
    // when other threads read environment variables concurrently. We do this at startup before
    // spinning up any engine threads.
    unsafe {
        std::env::set_var("WINIT_UNIX_BACKEND", "wayland");
        std::env::set_var("WGPU_BACKEND", "vulkan");
    }

    require_vulkan_adapter()
}

fn require_vulkan_adapter() -> anyhow::Result<()> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))?;

    let info = adapter.get_info();
    info!(?info, "Vulkan adapter selected");
    Ok(())
}
