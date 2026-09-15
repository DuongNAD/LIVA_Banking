import sys
import os
import re
import json
import pptx

sys.stdout.reconfigure(encoding='utf-8')

with open('index.html', 'r', encoding='utf-8') as f:
    content = f.read()

# Find all section tags
sections = re.findall(r'<section\s+[^>]*class="[^"]*slide[^"]*"[^>]*>', content)
print(f"Total <section> elements with slide class: {len(sections)}")
for s in sections:
    print(s)

slide_ids = [int(m) for m in re.findall(r'id="slide-(\d+)"', content)]
print(f"Slide IDs found: {slide_ids}")
missing = set(range(1, 26)) - set(slide_ids)
print(f"Missing IDs from 1 to 25: {missing}")
