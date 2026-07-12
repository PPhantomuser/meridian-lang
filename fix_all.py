with open('crates/lint/src/lib.rs', 'r') as f:
    code = f.read()

code = code.replace("span.clone()", "*span")

with open('crates/lint/src/lib.rs', 'w') as f:
    f.write(code)

with open('crates/lsp/src/lib.rs', 'r') as f:
    code = f.read()

code = code.replace(
    "pub struct Workspace {\n    documents: HashMap<String, String>,\n    semantics: HashMap<String, SemanticAnalyzer>,\n}\n\nimpl Workspace {",
    "pub struct Workspace {\n    documents: HashMap<String, String>,\n    semantics: HashMap<String, SemanticAnalyzer>,\n}\n\nimpl Default for Workspace {\n    fn default() -> Self {\n        Self::new()\n    }\n}\n\nimpl Workspace {"
)

with open('crates/lsp/src/lib.rs', 'w') as f:
    f.write(code)

with open('crates/ir/src/lib.rs', 'r') as f:
    code = f.read()

code = code.replace("self.compile_stmt(*stmt);", "self.compile_stmt(stmt);")

with open('crates/ir/src/lib.rs', 'w') as f:
    f.write(code)

