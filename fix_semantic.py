import re

with open('crates/semantic/src/lib.rs', 'r') as f:
    content = f.read()

# Add next_monomorphize_id to SemanticAnalyzer struct
content = content.replace(
    'pub struct SemanticAnalyzer {',
    'pub struct SemanticAnalyzer {\n    next_monomorphize_id: u32,'
)

# Initialize it in SemanticAnalyzer::new
content = content.replace(
    '            monomorphized_stmts: Vec::new(),',
    '            monomorphized_stmts: Vec::new(),\n            next_monomorphize_id: 1,'
)

# Update monomorphize calls. 
# We need to insert `let mono_id = self.next_monomorphize_id; self.next_monomorphize_id += 1;` right before the monomorphize call.
def replace_mono_call(match):
    return (
        'let mono_id = self.next_monomorphize_id;\n'
        '                        self.next_monomorphize_id += 1;\n'
        '                        let mut mono_stmt = generic_stmt.monomorphize(&type_bindings, mono_id);'
    )

content = re.sub(
    r'let mut mono_stmt = generic_stmt\.monomorphize\(&type_bindings\);',
    replace_mono_call,
    content
)

with open('crates/semantic/src/lib.rs', 'w') as f:
    f.write(content)
