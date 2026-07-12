import re

with open('crates/stdlib/src/lib.rs', 'r') as f:
    code = f.read()

# Replace return Value:: with Value:: in the whole file
code = re.sub(r'return\s+(Value::[a-zA-Z0-9_]+.*?);', r'\1', code)
code = code.replace("&sym_c.as_bytes_with_nul()", "sym_c.as_bytes_with_nul()")

with open('crates/stdlib/src/lib.rs', 'w') as f:
    f.write(code)
