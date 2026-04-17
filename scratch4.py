import sys

with open('crates/flatfekt-schema/src/lib.rs', 'r') as f:
    lines = f.readlines()

new_lines = []
for idx, line in enumerate(lines):
    if line.startswith('#[derive(schemars::JsonSchema, Debug, thiserror::Error)]'):
        line = line.replace('schemars::JsonSchema, ', '')
    if 'pub payload: Option<toml::Value>' in line:
        new_lines.append('  #[schemars(skip)]\n')
    new_lines.append(line)

with open('crates/flatfekt-schema/src/lib.rs', 'w') as f:
    f.writelines(new_lines)
