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
                platform: None,
                render: None,
            }
        }
    };

    enforce_platform_and_render_defaults(&cfg)?;

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
