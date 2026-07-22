import re

with open('crates/ast/src/lib.rs', 'r') as f:
    content = f.read()

# We will locate the 'pub fn monomorphize' definitions and only replace inside them.
def process_monomorphize_block(block):
    # Update signature inside this block (which is the first line of the block)
    block = block.replace(
        '(&self, type_bindings: &std::collections::HashMap<String, Type>)',
        '(&self, type_bindings: &std::collections::HashMap<String, Type>, id: u32)'
    )
    
    # Replace recursive calls
    block = block.replace('.monomorphize(type_bindings)', '.monomorphize(type_bindings, id)')
    
    # Replace span: *span
    block = re.sub(r'span:\s*\*span\s*,', 'span: span.with_id(id),', block)
    block = re.sub(r'span:\s*\*span\s*}', 'span: span.with_id(id) }', block)
    
    # Replace Expr::XX(..., *span) and Stmt::XX(..., *span)
    block = re.sub(r'Expr::([A-Za-z0-9_]+)\((.*?),\s*\*span\)', r'Expr::\1(\2, span.with_id(id))', block)
    block = re.sub(r'Stmt::([A-Za-z0-9_]+)\((.*?),\s*\*span\)', r'Stmt::\1(\2, span.with_id(id))', block)
    
    return block

# Find all monomorphize functions
parts = content.split('pub fn monomorphize')
new_content = parts[0]

for i in range(1, len(parts)):
    # each part starts with the rest of the signature `(&self, ...`
    # We want to process this entire function until the end.
    # To be safe, we can just process the whole string after `pub fn monomorphize` until the next one.
    # But wait, there might be other things after the last one. It's fine, we are only replacing `.monomorphize(type_bindings)` and `span: *span` which are very specific.
    # Wait, if we just process everything after `pub fn monomorphize`, the last one will process the rest of the file.
    # Are there other functions after `monomorphize`?
    # Let's find the closing brace of the monomorphize function.
    
    part = parts[i]
    
    # Find matching brace
    depth = 0
    in_function = False
    end_idx = len(part)
    for j, char in enumerate(part):
        if char == '{':
            depth += 1
            in_function = True
        elif char == '}':
            depth -= 1
            if in_function and depth == 0:
                end_idx = j + 1
                break
                
    func_body = 'pub fn monomorphize' + part[:end_idx]
    rest = part[end_idx:]
    
    func_body = process_monomorphize_block(func_body)
    
    new_content += func_body + rest

with open('crates/ast/src/lib.rs', 'w') as f:
    f.write(new_content)
