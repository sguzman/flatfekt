# Platform defaults

## Graphics environment

The default target graphics environment for this project is **Wayland**.

## Render backend

This project **requires Vulkan** as the graphics backend and will fail fast at startup if Vulkan is not available.

This assumption informs:

- windowing and presentation defaults
- capture/export plumbing (where applicable)
- platform-specific notes and troubleshooting guidance

If/when other environments are supported (e.g., X11), they will be explicitly documented as alternatives rather than implied defaults.
