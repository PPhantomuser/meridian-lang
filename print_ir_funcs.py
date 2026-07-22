import re

with open("crates/cli/src/main.rs", "r") as f:
    content = f.read()

content = content.replace(
    "let aot = meridian_backend_cranelift::AOTCompiler::new();",
    "println!(\"IR functions: {:?}\", program_ir.functions.keys().collect::<Vec<_>>());\n                let aot = meridian_backend_cranelift::AOTCompiler::new();"
)

with open("crates/cli/src/main.rs", "w") as f:
    f.write(content)
