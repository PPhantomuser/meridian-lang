import re

with open('crates/semantic/src/lib.rs', 'r') as f:
    code = f.read()

# Default impl
code = code.replace(
    "pub capabilities: std::collections::HashSet<String>,\n}",
    "pub capabilities: std::collections::HashSet<String>,\n}\n\nimpl Default for SemanticAnalyzer {\n    fn default() -> Self {\n        Self::new()\n    }\n}"
)

# map_entry warning
code = code.replace(
    "    fn declare_variable(&mut self, name: String, ty: Type, mutable: bool, span: Span) {",
    "    #[allow(clippy::map_entry)]\n    fn declare_variable(&mut self, name: String, ty: Type, mutable: bool, span: Span) {"
)

# unwrap_or_default
code = code.replace(
    ".or_insert_with(HashMap::new);",
    ".or_default();"
)

# needless_borrow
code = code.replace(
    "if !self.types_compatible(&func_err, &err) {",
    "if !self.types_compatible(func_err, &err) {"
)

# collapsible_if
code = code.replace(
    """                        if self.macro_expansion_depth > 0 {
                            if unsafe_funcs.contains(&name.as_str()) {""",
    """                        if self.macro_expansion_depth > 0 && unsafe_funcs.contains(&name.as_str()) {"""
)
code = code.replace(
    """                            }
                        }
                        if arguments.len() != signature.parameters.len() {""",
    """                        }
                        if arguments.len() != signature.parameters.len() {"""
)

# allow collapsible match
code = code.replace(
    "    fn resolve_type(&mut self, ty: &Type) -> Type {",
    "    #[allow(clippy::collapsible_match)]\n    fn resolve_type(&mut self, ty: &Type) -> Type {"
)

# len_zero
code = code.replace(
    "if binding_names.len() > 0 && payload_tys.len() != binding_names.len()",
    "if !binding_names.is_empty() && payload_tys.len() != binding_names.len()"
)

with open('crates/semantic/src/lib.rs', 'w') as f:
    f.write(code)

