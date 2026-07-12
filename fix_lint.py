import re
with open('crates/lint/src/lib.rs', 'r') as f:
    code = f.read()

code = re.sub(r'\bspan\.clone\(\)', '*span', code)

with open('crates/lint/src/lib.rs', 'w') as f:
    f.write(code)

