#![allow(unused)]
use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::{
    types, AbiParam, InstBuilder, MemFlags, StackSlotData, StackSlotKind,
};
use cranelift_codegen::settings;
use cranelift_codegen::settings::Configurable;
use cranelift_codegen::Context;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_object::{ObjectBuilder, ObjectModule};
use cranelift_module::{default_libcall_names, Linkage, Module, FuncId};
use meridian_ir::{Chunk, ConstValue, Opcode, ProgramIR};
use std::collections::HashMap;

pub struct AOTCompiler {
    module: ObjectModule,
    ctx: Context,
    builder_context: FunctionBuilderContext,
    functions: HashMap<String, FuncId>,
    print_func_id: FuncId,
    print_i64_func_id: FuncId,
}

impl Default for AOTCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl AOTCompiler {
    pub fn new() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("is_pic", "true").unwrap();
        flag_builder.set("opt_level", "speed_and_size").unwrap();
        
        let isa_builder = cranelift_native::builder().unwrap();
        let isa = isa_builder.finish(settings::Flags::new(flag_builder)).unwrap();
        
        let builder = ObjectBuilder::new(
            isa,
            "meridian_binary",
            default_libcall_names(),
        ).unwrap();
        
        let mut module = ObjectModule::new(builder);
        
        // print_f64 returns nothing (void)
        let mut print_sig = module.make_signature();
        print_sig.params.push(AbiParam::new(types::F64));
        
        // For AOT, we expect print_f64 to be linked statically or dynamically.
        // It's usually imported from our C wrapper or standard library.
        let print_func_id = module
            .declare_function("print_f64", Linkage::Import, &print_sig)
            .unwrap();

        let mut print_i64_sig = module.make_signature();
        print_i64_sig.params.push(AbiParam::new(types::I64));
        let print_i64_func_id = module
            .declare_function("print_i64", Linkage::Import, &print_i64_sig)
            .unwrap();

        let ctx = module.make_context();

        Self {
            module,
            ctx,
            builder_context: FunctionBuilderContext::new(),
            functions: HashMap::new(),
            print_func_id,
            print_i64_func_id,
        }
    }

    pub fn compile_to_object(mut self, program_ir: &ProgramIR) -> Result<Vec<u8>, String> {
        for (name, (_chunk, _is_async, arg_count)) in &program_ir.functions {
            let mut sig = self.module.make_signature();
            for _ in 0..*arg_count {
                sig.params.push(AbiParam::new(types::F64));
            }
            sig.returns.push(AbiParam::new(types::F64));
            let export_name = if name == "main" { "meridian_user_main" } else { name.as_str() };
            let func_id = self.module.declare_function(export_name, Linkage::Export, &sig).unwrap();
            self.functions.insert(name.clone(), func_id);
        }

        for (name, arg_count) in &program_ir.extern_functions {
            let mut sig = self.module.make_signature();
            for _ in 0..*arg_count {
                sig.params.push(AbiParam::new(types::F64));
            }
            sig.returns.push(AbiParam::new(types::F64));
            let func_id = self.module.declare_function(name, Linkage::Import, &sig).unwrap();
            self.functions.insert(name.clone(), func_id);
        }

        for (name, (chunk, _is_async, arg_count)) in &program_ir.functions {
            let func_id = *self.functions.get(name).unwrap();
            self.compile_chunk(chunk, *arg_count, true)?;
            self.module.define_function(func_id, &mut self.ctx).unwrap();
            self.module.clear_context(&mut self.ctx);
        }

        let mut main_sig = self.module.make_signature();
        main_sig.returns.push(AbiParam::new(types::I32));
        let main_func_id = self.module.declare_function("meridian_main", Linkage::Export, &main_sig).unwrap();
        
        self.compile_chunk(&program_ir.main_chunk, 0, false)?;
        self.module.define_function(main_func_id, &mut self.ctx).unwrap();
        self.module.clear_context(&mut self.ctx);

        let product = self.module.finish();
        Ok(product.emit().unwrap())
    }

    #[allow(clippy::needless_range_loop)]
    fn compile_chunk(&mut self, chunk: &Chunk, arg_count: usize, has_return: bool) -> Result<(), String> {
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        
        for _ in 0..arg_count {
            builder.func.signature.params.push(AbiParam::new(types::F64));
        }
        if has_return {
            builder.func.signature.returns.push(AbiParam::new(types::F64));
        } else {
            builder.func.signature.returns.push(AbiParam::new(types::I32));
        }

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        let mut slots = Vec::new();
        // Allocate 100 registers as StackSlots
        for _ in 0..100 {
            let slot = builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8));
            slots.push(slot);
            let zero = builder.ins().f64const(0.0);
            let zero_i64 = builder.ins().bitcast(types::I64, MemFlags::new(), zero);
            builder.ins().stack_store(zero_i64, slot, 0);
        }

        for i in 0..arg_count {
            let val = builder.block_params(entry_block)[i];
            let val_i64 = builder.ins().bitcast(types::I64, MemFlags::new(), val);
            builder.ins().stack_store(val_i64, slots[i], 0);
        }

        let mut blocks = vec![entry_block];
        for _ in 1..chunk.instructions.len() {
            blocks.push(builder.create_block());
        }
        let end_block = builder.create_block();
        blocks.push(end_block);

        let mut current_block = entry_block;
        let mut block_terminated = false;

        for (i, inst) in chunk.instructions.iter().enumerate() {
            if i > 0 {
                if !block_terminated {
                    builder.ins().jump(blocks[i], &[]);
                }
                current_block = blocks[i];
                builder.switch_to_block(current_block);
                block_terminated = false;
            }

            match inst {
                Opcode::LoadConst(dest, const_idx) => {
                    let val_i64 = match &chunk.constants[*const_idx] {
                        ConstValue::Number(n) => {
                            let f = builder.ins().f64const(*n);
                            builder.ins().bitcast(types::I64, MemFlags::new(), f)
                        }
                        ConstValue::Int(n) => builder.ins().iconst(types::I64, *n),
                        ConstValue::Bool(b) => {
                            let f = builder.ins().f64const(if *b { 1.0 } else { 0.0 });
                            builder.ins().bitcast(types::I64, MemFlags::new(), f)
                        }
                        _ => builder.ins().iconst(types::I64, 0),
                    };
                    builder.ins().stack_store(val_i64, slots[*dest], 0);
                }
                Opcode::Move(dest, src) => {
                    let val_i64 = builder.ins().stack_load(types::I64, slots[*src], 0);
                    builder.ins().stack_store(val_i64, slots[*dest], 0);
                }
                Opcode::Add(dest, left, right, is_float) => {
                    let l_i64 = builder.ins().stack_load(types::I64, slots[*left], 0);
                    let r_i64 = builder.ins().stack_load(types::I64, slots[*right], 0);
                    if *is_float {
                        let l = builder.ins().bitcast(types::F64, MemFlags::new(), l_i64);
                        let r = builder.ins().bitcast(types::F64, MemFlags::new(), r_i64);
                        let res = builder.ins().fadd(l, r);
                        let res_i64 = builder.ins().bitcast(types::I64, MemFlags::new(), res);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    } else {
                        let res_i64 = builder.ins().iadd(l_i64, r_i64);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    }
                }
                Opcode::Sub(dest, left, right, is_float) => {
                    let l_i64 = builder.ins().stack_load(types::I64, slots[*left], 0);
                    let r_i64 = builder.ins().stack_load(types::I64, slots[*right], 0);
                    if *is_float {
                        let l = builder.ins().bitcast(types::F64, MemFlags::new(), l_i64);
                        let r = builder.ins().bitcast(types::F64, MemFlags::new(), r_i64);
                        let res = builder.ins().fsub(l, r);
                        let res_i64 = builder.ins().bitcast(types::I64, MemFlags::new(), res);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    } else {
                        let res_i64 = builder.ins().isub(l_i64, r_i64);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    }
                }
                Opcode::Print(src, is_float) => {
                    let val_i64 = builder.ins().stack_load(types::I64, slots[*src], 0);
                    if *is_float {
                        let val = builder.ins().bitcast(types::F64, MemFlags::new(), val_i64);
                        let local_print = self.module.declare_func_in_func(self.print_func_id, builder.func);
                        builder.ins().call(local_print, &[val]);
                    } else {
                        let local_print_i64 = self.module.declare_func_in_func(self.print_i64_func_id, builder.func);
                        builder.ins().call(local_print_i64, &[val_i64]);
                    }
                }
                Opcode::JumpIfFalse(cond_reg, offset) => {
                    let cond_i64 = builder.ins().stack_load(types::I64, slots[*cond_reg], 0);
                    let cond = builder.ins().bitcast(types::F64, MemFlags::new(), cond_i64);
                    let zero = builder.ins().f64const(0.0);
                    let cmp = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::Equal, cond, zero);
                    let next_block = blocks[i + 1];
                    let is_false = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::Equal, cond, zero);
                    builder.ins().brif(is_false, blocks[*offset], &[], next_block, &[]);
                    block_terminated = true;
                }
                Opcode::Jump(offset) => {
                    builder.ins().jump(blocks[*offset], &[]);
                    block_terminated = true;
                }
                Opcode::Call(dest, name, arg_start, arg_count) => {
                    if let Some(func_id) = self.functions.get(name) {
                        let mut call_args = Vec::new();
                        for a in 0..*arg_count {
                            let arg_i64 = builder.ins().stack_load(types::I64, slots[*arg_start + a], 0);
                            let arg = builder.ins().bitcast(types::F64, MemFlags::new(), arg_i64);
                            call_args.push(arg);
                        }
                        let local_func = self.module.declare_func_in_func(*func_id, builder.func);
                        let call = builder.ins().call(local_func, &call_args);
                        let res = builder.inst_results(call)[0];
                        let res_i64 = builder.ins().bitcast(types::I64, MemFlags::new(), res);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    }
                }
                Opcode::Return(reg) => {
                    if has_return {
                        let val_i64 = builder.ins().stack_load(types::I64, slots[*reg], 0);
                        let val = builder.ins().bitcast(types::F64, MemFlags::new(), val_i64);
                        builder.ins().return_(&[val]);
                    } else {
                        builder.ins().return_(&[]);
                    }
                    block_terminated = true;
                }
                Opcode::Borrow(dest, src) => {
                    let ptr = builder.ins().stack_addr(types::I64, slots[*src], 0);
                    builder.ins().stack_store(ptr, slots[*dest], 0);
                }
                Opcode::Dereference(dest, src) => {
                    let ptr = builder.ins().stack_load(types::I64, slots[*src], 0);
                    let val_i64 = builder.ins().load(types::I64, MemFlags::new(), ptr, 0);
                    builder.ins().stack_store(val_i64, slots[*dest], 0);
                }
                Opcode::Eq(dest, left, right, is_float) => {
                    let l_i64 = builder.ins().stack_load(types::I64, slots[*left], 0);
                    let r_i64 = builder.ins().stack_load(types::I64, slots[*right], 0);
                    let bool_val = if *is_float {
                        let l = builder.ins().bitcast(types::F64, MemFlags::new(), l_i64);
                        let r = builder.ins().bitcast(types::F64, MemFlags::new(), r_i64);
                        let cmp = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::Equal, l, r);
                        builder.ins().uextend(types::I64, cmp)
                    } else {
                        let cmp = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, l_i64, r_i64);
                        builder.ins().uextend(types::I64, cmp)
                    };
                    let val_f64 = builder.ins().fcvt_from_uint(types::F64, bool_val);
                    let res_i64 = builder.ins().bitcast(types::I64, MemFlags::new(), val_f64);
                    builder.ins().stack_store(res_i64, slots[*dest], 0);
                }
                Opcode::Lt(dest, left, right, is_float) => {
                    let l_i64 = builder.ins().stack_load(types::I64, slots[*left], 0);
                    let r_i64 = builder.ins().stack_load(types::I64, slots[*right], 0);
                    let bool_val = if *is_float {
                        let l = builder.ins().bitcast(types::F64, MemFlags::new(), l_i64);
                        let r = builder.ins().bitcast(types::F64, MemFlags::new(), r_i64);
                        let cmp = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::LessThan, l, r);
                        builder.ins().uextend(types::I64, cmp)
                    } else {
                        let cmp = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::SignedLessThan, l_i64, r_i64);
                        builder.ins().uextend(types::I64, cmp)
                    };
                    let val_f64 = builder.ins().fcvt_from_uint(types::F64, bool_val);
                    let res_i64 = builder.ins().bitcast(types::I64, MemFlags::new(), val_f64);
                    builder.ins().stack_store(res_i64, slots[*dest], 0);
                }
                Opcode::Mul(..) | Opcode::Div(..) | Opcode::Ne(..) | Opcode::Le(..) | Opcode::Gt(..) | Opcode::Ge(..) | Opcode::MakeStruct(..) |
                Opcode::FieldAccess(..) |
                Opcode::FieldAssign(..) |
                Opcode::MakeEnum(..) |
                Opcode::CheckEnum(..) |
                Opcode::ExtractEnum(..) |
                Opcode::MakeArray(..) |
                Opcode::ArrayIndex(..) |
                Opcode::ArrayAssign(..) |
                Opcode::AsyncCall(..) |
                Opcode::Await(..) |
                Opcode::Spawn(..) => {
                    return Err("Struct/Enum/Array/Async opcodes are not supported in AOT MVP (fallback to VM recommended)".to_string());
                }
                _ => {
                    return Err(format!("Unsupported opcode in AOT prototype: {:?}", inst));
                }
            }
        }
        
        if !block_terminated {
            if has_return {
                // Return unit/zero if missing
                let zero = builder.ins().f64const(0.0);
                let zero_i64 = builder.ins().bitcast(types::I64, MemFlags::new(), zero);
                builder.ins().return_(&[zero_i64]);
            } else {
                builder.ins().jump(end_block, &[]);
            }
        }
        
        builder.switch_to_block(end_block);
        if !has_return {
            let zero = builder.ins().iconst(types::I32, 0);
            builder.ins().return_(&[zero]);
        }

        builder.seal_all_blocks();
        builder.finalize();
        Ok(())
    }
}
