import re

with open('crates/cli/src/main.rs', 'r') as f:
    code = f.read()

code = code.replace("use meridian_lsp;\n", "")
code = code.replace("resolver::registry::get_package_index(&name)", "resolver::registry::get_package_index(name)")
code = code.replace("resolver::registry::generate_publish_payload(&pkg_name, &version, &source, &commit, deps_opt)", "resolver::registry::generate_publish_payload(&pkg_name, &version, source, commit, deps_opt)")
code = code.replace("path.extension().map_or(false, |ext| ext == \"merid\" || ext == \"mr\")", "path.extension().is_some_and(|ext| ext == \"merid\" || ext == \"mr\")")
code = code.replace("for (_, (chunk, _, _)) in &program_ir.functions", "for (chunk, _, _) in program_ir.functions.values()")

with open('crates/cli/src/main.rs', 'w') as f:
    f.write(code)

