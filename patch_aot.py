import re

with open("crates/backend-cranelift/src/aot.rs", "r") as f:
    content = f.read()

content = content.replace("print_i64_func_id: FuncId,", "print_i64_func_id: FuncId,\n    malloc_func_id: FuncId,")

malloc_init = """        let mut malloc_sig = module.make_signature();
        malloc_sig.params.push(AbiParam::new(types::I64));
        malloc_sig.returns.push(AbiParam::new(types::I64));
        let malloc_func_id = module
            .declare_function("malloc", Linkage::Import, &malloc_sig)
            .unwrap();

        let ctx = module.make_context();"""

content = content.replace("let ctx = module.make_context();", malloc_init)

content = content.replace("print_i64_func_id,\n        }", "print_i64_func_id,\n            malloc_func_id,\n        }")

struct_impl = """
                Opcode::MakeStruct(dest, _name, fields, start_reg) => {
                    let size_val = builder.ins().iconst(types::I64, (fields.len() * 8) as i64);
                    let local_callee = self.module.declare_func_in_func(self.malloc_func_id, &mut builder.func);
                    let call = builder.ins().call(local_callee, &[size_val]);
                    let ptr = builder.inst_results(call)[0];
                    for (i, _) in fields.iter().enumerate() {
                        let field_val = builder.ins().stack_load(types::I64, slots[*start_reg + i], 0);
                        builder.ins().store(MemFlags::new(), field_val, ptr, (i * 8) as i32);
                    }
                    builder.ins().stack_store(ptr, slots[*dest], 0);
                }
                Opcode::GetField(dest, obj_reg, _struct_name, field_idx) => {
                    let ptr = builder.ins().stack_load(types::I64, slots[*obj_reg], 0);
                    let field_val = builder.ins().load(types::I64, MemFlags::new(), ptr, (*field_idx * 8) as i32);
                    builder.ins().stack_store(field_val, slots[*dest], 0);
                }
                Opcode::SetField(obj_reg, _struct_name, field_idx, val_reg) => {
                    let ptr = builder.ins().stack_load(types::I64, slots[*obj_reg], 0);
                    let field_val = builder.ins().stack_load(types::I64, slots[*val_reg], 0);
                    builder.ins().store(MemFlags::new(), field_val, ptr, (*field_idx * 8) as i32);
                }
                Opcode::MakeEnum(dest, _enum_name, _variant_name, start_reg, val_count) => {
                    let size_val = builder.ins().iconst(types::I64, ((*val_count + 1) * 8) as i64);
                    let local_callee = self.module.declare_func_in_func(self.malloc_func_id, &mut builder.func);
                    let call = builder.ins().call(local_callee, &[size_val]);
                    let ptr = builder.inst_results(call)[0];
                    
                    let tag = {
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut hasher = DefaultHasher::new();
                        _variant_name.hash(&mut hasher);
                        hasher.finish() as i64
                    };
                    let tag_val = builder.ins().iconst(types::I64, tag);
                    builder.ins().store(MemFlags::new(), tag_val, ptr, 0);
                    
                    for i in 0..*val_count {
                        let field_val = builder.ins().stack_load(types::I64, slots[*start_reg + i], 0);
                        builder.ins().store(MemFlags::new(), field_val, ptr, ((i + 1) * 8) as i32);
                    }
                    builder.ins().stack_store(ptr, slots[*dest], 0);
                }
                Opcode::GetEnumData(dest, obj_reg, _enum_name, _variant_name, field_idx) => {
                    let ptr = builder.ins().stack_load(types::I64, slots[*obj_reg], 0);
                    let field_val = builder.ins().load(types::I64, MemFlags::new(), ptr, ((*field_idx + 1) * 8) as i32);
                    builder.ins().stack_store(field_val, slots[*dest], 0);
                }
                Opcode::Borrow(dest, src) => {
                    let addr = builder.ins().stack_addr(types::I64, slots[*src], 0);
                    builder.ins().stack_store(addr, slots[*dest], 0);
                }
                Opcode::Dereference(dest, src) => {
                    let addr = builder.ins().stack_load(types::I64, slots[*src], 0);
                    let val = builder.ins().load(types::I64, MemFlags::new(), addr, 0);
                    builder.ins().stack_store(val, slots[*dest], 0);
                }
                Opcode::Print(reg) => {
                    let val_i64 = builder.ins().stack_load(types::I64, slots[*reg], 0);
                    let val_f64 = builder.ins().bitcast(types::F64, MemFlags::new(), val_i64);
                    let local_callee = self.module.declare_func_in_func(self.print_func_id, &mut builder.func);
                    builder.ins().call(local_callee, &[val_f64]);
                }
"""

content = re.sub(
    r"Opcode::Print\(reg\) => \{[^}]+\}",
    struct_impl.strip(),
    content
)

unsupported = """                Opcode::Mul(..) | Opcode::Div(..) | Opcode::Ne(..) | Opcode::Le(..) | Opcode::Gt(..) | Opcode::Ge(..) | Opcode::MakeStruct(..) |
                Opcode::GetField(..) | Opcode::SetField(..) | Opcode::MakeEnum(..) | Opcode::GetEnumDiscrim(..) | Opcode::GetEnumData(..) | Opcode::MakeArray(..) | Opcode::GetIndex(..) |
                Opcode::SetIndex(..) | Opcode::Borrow(..) | Opcode::Dereference(..) | Opcode::Spawn(..) | Opcode::Await(..) => {
                    return Err("Struct/Enum/Array/Async opcodes are not supported in AOT MVP (fallback to VM recommended)".to_string());
                }"""

supported = """                Opcode::Mul(..) | Opcode::Div(..) | Opcode::Ne(..) | Opcode::Le(..) | Opcode::Gt(..) | Opcode::Ge(..) | 
                Opcode::GetEnumDiscrim(..) | Opcode::MakeArray(..) | Opcode::GetIndex(..) |
                Opcode::SetIndex(..) | Opcode::Spawn(..) | Opcode::Await(..) => {
                    return Err("Array/Async opcodes are not supported in AOT MVP (fallback to VM recommended)".to_string());
                }"""

content = content.replace(unsupported, supported)

with open("crates/backend-cranelift/src/aot.rs", "w") as f:
    f.write(content)
