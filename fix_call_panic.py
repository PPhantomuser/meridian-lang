import re

with open("crates/backend-cranelift/src/aot.rs", "r") as f:
    content = f.read()

content = content.replace(
    "if let Some(func_id) = self.functions.get(name) {",
    "if let Some(func_id) = self.functions.get(name) {"
)
# Wait, I can just replace the whole block
old_call = """                Opcode::Call(dest, name, arg_start, arg_count) => {
                    if let Some(func_id) = self.functions.get(name) {
                        let mut call_args = Vec::new();
                        for a in 0..*arg_count {
                            let arg_i64 = builder.ins().stack_load(types::I64, slots[*arg_start + a], 0);
                            call_args.push(arg_i64);
                        }
                        let local_func = self.module.declare_func_in_func(*func_id, builder.func);
                        let call = builder.ins().call(local_func, &call_args);
                        let res = builder.inst_results(call)[0];
                        builder.ins().stack_store(res, slots[*dest], 0);
                    }
                }"""
new_call = """                Opcode::Call(dest, name, arg_start, arg_count) => {
                    if let Some(func_id) = self.functions.get(name) {
                        let mut call_args = Vec::new();
                        for a in 0..*arg_count {
                            let arg_i64 = builder.ins().stack_load(types::I64, slots[*arg_start + a], 0);
                            call_args.push(arg_i64);
                        }
                        let local_func = self.module.declare_func_in_func(*func_id, builder.func);
                        let call = builder.ins().call(local_func, &call_args);
                        let res = builder.inst_results(call)[0];
                        builder.ins().stack_store(res, slots[*dest], 0);
                    } else {
                        panic!("Function not found in AOT: {}", name);
                    }
                }"""
content = content.replace(old_call, new_call)

with open("crates/backend-cranelift/src/aot.rs", "w") as f:
    f.write(content)
