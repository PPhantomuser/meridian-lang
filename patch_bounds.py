import re

with open("crates/semantic/src/lib.rs", "r") as f:
    content = f.read()

bound_check = """                            type_bindings.insert(param.name.clone(), type_args[i].clone());
                            for bound in &param.bounds {
                                let arg_name = meridian_ast::type_to_string(&type_args[i]);
                                let cap_str = format!("{} implements {}", arg_name, bound);
                                if !self.capabilities.contains(&cap_str) {
                                    self.diagnostics.push(Diagnostic::new(
                                        format!("Type '{}' does not implement required trait '{}'", arg_name, bound),
                                        "MER0107".to_string(),
                                        *span,
                                        DiagnosticCategory::Type,
                                        None,
                                    ));
                                }
                            }"""

content = content.replace("type_bindings.insert(param.name.clone(), type_args[i].clone());", bound_check)

bound_check_func = """                                            type_bindings.insert(param.name.clone(), inferred_args[j].clone());
                                            for bound in &param.bounds {
                                                let arg_name = meridian_ast::type_to_string(&inferred_args[j]);
                                                let cap_str = format!("{} implements {}", arg_name, bound);
                                                if !self.capabilities.contains(&cap_str) {
                                                    self.diagnostics.push(Diagnostic::new(
                                                        format!("Type '{}' does not implement required trait '{}'", arg_name, bound),
                                                        "MER0107".to_string(),
                                                        *span,
                                                        DiagnosticCategory::Type,
                                                        None,
                                                    ));
                                                }
                                            }"""

content = content.replace("type_bindings.insert(param.name.clone(), inferred_args[j].clone());", bound_check_func)

with open("crates/semantic/src/lib.rs", "w") as f:
    f.write(content)
