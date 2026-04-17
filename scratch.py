import os
import re

roadmaps_dir = "/win/linux/Code/rust/flatfekt/docs/roadmaps"

for filename in os.listdir(roadmaps_dir):
    if not filename.endswith(".md"):
        continue
    filepath = os.path.join(roadmaps_dir, filename)
    with open(filepath, "r") as f:
        content = f.read()
    
    # Count checkboxes
    x_count = len(re.findall(r'- \[x\]', content, re.IGNORECASE))
    empty_count = len(re.findall(r'- \[\s\]', content))
    
    total = x_count + empty_count
    if total == 0:
        percent = 100.0 # Or 0 depending on interpretation, let's just print 0 if no tasks
        if "README" not in filename:
            print(f"{filename}: 0 tasks")
    else:
        percent = (x_count / total) * 100
        print(f"{filename}: {percent:.1f}% ({x_count}/{total})")
