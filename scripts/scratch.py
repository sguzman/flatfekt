import sys

with open('crates/flatfekt-schema/src/lib.rs', 'r') as f:
    lines = f.readlines()

new_lines = []
for line in lines:
    if line.startswith('#[derive('):
        if 'JsonSchema' not in line:
            line = line.replace('#[derive(', '#[derive(schemars::JsonSchema, ')
    new_lines.append(line)

with open('crates/flatfekt-schema/src/lib.rs', 'w') as f:
    f.writelines(new_lines)
