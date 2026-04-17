import re

with open('crates/flatfekt-schema/src/lib.rs', 'r') as f:
    content = f.read()

content = re.sub(r'derive\(([^)]*?)\bDeserialize\b([^)]*?)\)', r'derive(\1Serialize, Deserialize\2)', content)

with open('crates/flatfekt-schema/src/lib.rs', 'w') as f:
    f.write(content)
