with open('crates/stdlib/src/lib.rs', 'r') as f:
    code = f.read()

code = "#![allow(clippy::needless_return)]\n" + code
code = code.replace("&sym_c.as_bytes_with_nul()", "sym_c.as_bytes_with_nul()")

with open('crates/stdlib/src/lib.rs', 'w') as f:
    f.write(code)

with open('crates/backend-cranelift/src/aot.rs', 'r') as f:
    code = f.read()

code = code.replace(
    "    pub fn compile_function(&mut self, name: &str, params: usize, stmts: &[crate::ir::IRStmt]) -> Vec<u8> {",
    "    #[allow(clippy::needless_range_loop)]\n    pub fn compile_function(&mut self, name: &str, params: usize, stmts: &[crate::ir::IRStmt]) -> Vec<u8> {"
)
if "impl Default for AOTCompiler" not in code:
    code = code.replace("impl AOTCompiler {", "impl Default for AOTCompiler {\n    fn default() -> Self {\n        Self::new()\n    }\n}\n\nimpl AOTCompiler {")

with open('crates/backend-cranelift/src/aot.rs', 'w') as f:
    f.write(code)


with open('crates/backend-cranelift/src/lib.rs', 'r') as f:
    code = f.read()

if "impl Default for JITCompiler" not in code:
    code = code.replace("impl JITCompiler {", "impl Default for JITCompiler {\n    fn default() -> Self {\n        Self::new()\n    }\n}\n\nimpl JITCompiler {")

with open('crates/backend-cranelift/src/lib.rs', 'w') as f:
    f.write(code)

