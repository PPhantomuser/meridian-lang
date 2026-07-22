import re

with open("crates/backend-cranelift/src/aot.rs", "r") as f:
    content = f.read()

# Fix function signatures
content = content.replace("sig.params.push(AbiParam::new(types::F64));", "sig.params.push(AbiParam::new(types::I64));")
content = content.replace("sig.returns.push(AbiParam::new(types::F64));", "sig.returns.push(AbiParam::new(types::I64));")

# Fix builder.func.signature
content = content.replace("builder.func.signature.params.push(AbiParam::new(types::F64));", "builder.func.signature.params.push(AbiParam::new(types::I64));")
content = content.replace("builder.func.signature.returns.push(AbiParam::new(types::F64));", "builder.func.signature.returns.push(AbiParam::new(types::I64));")

# Fix entry block loading (remove bitcast)
entry_load_old = """        for i in 0..arg_count {
            let val = builder.block_params(entry_block)[i];
            let val_i64 = builder.ins().bitcast(types::I64, MemFlags::new(), val);
            builder.ins().stack_store(val_i64, slots[i], 0);
        }"""
entry_load_new = """        for i in 0..arg_count {
            let val = builder.block_params(entry_block)[i];
            builder.ins().stack_store(val, slots[i], 0);
        }"""
content = content.replace(entry_load_old, entry_load_new)

# Fix Call
call_old = """                        for a in 0..*arg_count {
                            let arg_i64 = builder.ins().stack_load(types::I64, slots[*arg_start + a], 0);
                            let arg = builder.ins().bitcast(types::F64, MemFlags::new(), arg_i64);
                            call_args.push(arg);
                        }
                        let local_func = self.module.declare_func_in_func(*func_id, builder.func);
                        let call = builder.ins().call(local_func, &call_args);
                        let res = builder.inst_results(call)[0];
                        let res_i64 = builder.ins().bitcast(types::I64, MemFlags::new(), res);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);"""
call_new = """                        for a in 0..*arg_count {
                            let arg_i64 = builder.ins().stack_load(types::I64, slots[*arg_start + a], 0);
                            call_args.push(arg_i64);
                        }
                        let local_func = self.module.declare_func_in_func(*func_id, builder.func);
                        let call = builder.ins().call(local_func, &call_args);
                        let res = builder.inst_results(call)[0];
                        builder.ins().stack_store(res, slots[*dest], 0);"""
content = content.replace(call_old, call_new)

# Fix Return
ret_old = """                    if has_return {
                        let val_i64 = builder.ins().stack_load(types::I64, slots[*reg], 0);
                        let val = builder.ins().bitcast(types::F64, MemFlags::new(), val_i64);
                        builder.ins().return_(&[val]);
                    } else {
                        builder.ins().return_(&[]);
                    }"""
ret_new = """                    if has_return {
                        let val_i64 = builder.ins().stack_load(types::I64, slots[*reg], 0);
                        builder.ins().return_(&[val_i64]);
                    } else {
                        let zero = builder.ins().iconst(types::I32, 0);
                        builder.ins().return_(&[zero]);
                    }"""
content = content.replace(ret_old, ret_new)

# Fix ConstValue::Int back to iconst
const_old = """ConstValue::Int(n) => {
                            let f = builder.ins().f64const(*n as f64);
                            builder.ins().bitcast(types::I64, MemFlags::new(), f)
                        }"""
const_new = """ConstValue::Int(n) => builder.ins().iconst(types::I64, *n),"""
content = content.replace(const_old, const_new)

with open("crates/backend-cranelift/src/aot.rs", "w") as f:
    f.write(content)
