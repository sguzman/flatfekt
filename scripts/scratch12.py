import sys

with open('crates/flatfekt-config/src/lib.rs', 'r') as f:
    content = f.read()

content = content.replace("pub max_catchup_steps: Option<u32>", "pub max_catchup_steps: Option<u32>,\n  pub deterministic: Option<bool>")
content = content.replace(
"""  pub fn timeline_enabled(&self) -> bool {
    self
      .runtime
      .as_ref()
      .and_then(|r| r.timeline.as_ref())
      .and_then(|t| t.enabled)
      .unwrap_or(true)
  }""",
"""  pub fn timeline_enabled(&self) -> bool {
    self
      .runtime
      .as_ref()
      .and_then(|r| r.timeline.as_ref())
      .and_then(|t| t.enabled)
      .unwrap_or(true)
  }

  pub fn timeline_deterministic(&self) -> bool {
    self
      .runtime
      .as_ref()
      .and_then(|r| r.timeline.as_ref())
      .and_then(|t| t.deterministic)
      .unwrap_or(false)
  }""")

with open('crates/flatfekt-config/src/lib.rs', 'w') as f:
    f.write(content)
