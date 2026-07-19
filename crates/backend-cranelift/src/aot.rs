#![allow(unused)]
use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::{
    types, AbiParam, InstBuilder, MemFlags, StackSlotData, StackSlotKind,
};
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::settings;
use cranelift_codegen::settings::Configurable;
use cranelift_codegen::Context;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_object::{ObjectBuilder, ObjectModule};
use cranelift_module::{default_libcall_names, Linkage, Module, FuncId, DataId, DataDescription};
use meridian_ir::{Chunk, ConstValue, Opcode, ProgramIR, PrintType};
use std::collections::HashMap;

pub struct AOTCompiler {
    module: ObjectModule,
    ctx: Context,
    builder_context: FunctionBuilderContext,
    functions: HashMap<String, FuncId>,
    print_func_id: FuncId,
    print_i64_func_id: FuncId,
    print_str_func_id: FuncId,
    malloc_func_id: FuncId,
    string_counter: usize,
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

                let mut malloc_sig = module.make_signature();
        malloc_sig.params.push(AbiParam::new(types::I64));
        malloc_sig.returns.push(AbiParam::new(types::I64));
        let malloc_func_id = module
            .declare_function("malloc", Linkage::Import, &malloc_sig)
            .unwrap();
        let mut print_str_sig = module.make_signature();
        print_str_sig.params.push(AbiParam::new(types::I64)); // pointer to char
        let print_str_func_id = module
            .declare_function("print_str", Linkage::Import, &print_str_sig)
            .unwrap();

        let ctx = module.make_context();

        Self {
            module,
            ctx,
            builder_context: FunctionBuilderContext::new(),
            functions: HashMap::new(),
            print_func_id,
            print_i64_func_id,
            print_str_func_id,
            malloc_func_id,
            string_counter: 0,
        }
    }

    pub fn compile_to_object(mut self, program_ir: &ProgramIR) -> Result<Vec<u8>, String> {
        for (name, (_chunk, _is_async, arg_count)) in &program_ir.functions {
            let mut sig = self.module.make_signature();
            for _ in 0..*arg_count {
                sig.params.push(AbiParam::new(types::I64));
            }
            sig.returns.push(AbiParam::new(types::I64));
            let export_name = if name == "main" { "meridian_user_main" } else { name.as_str() };
            let func_id = self.module.declare_function(export_name, Linkage::Export, &sig).unwrap();
            self.functions.insert(name.clone(), func_id);
        }

        for (name, arg_count) in &program_ir.extern_functions {
            let mut sig = self.module.make_signature();
            for _ in 0..*arg_count {
                sig.params.push(AbiParam::new(types::I64));
            }
            sig.returns.push(AbiParam::new(types::I64));
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
            builder.func.signature.params.push(AbiParam::new(types::I64));
        }
        if has_return {
            builder.func.signature.returns.push(AbiParam::new(types::I64));
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
            builder.ins().stack_store(val, slots[i], 0);
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
                        ConstValue::String(s) => {
                            let mut data_ctx = DataDescription::new();
                            let mut c_str = s.clone().into_bytes();
                            c_str.push(0); // null terminator
                            data_ctx.define(c_str.into_boxed_slice());
                            self.string_counter += 1;
                            let data_id = self.module.declare_data(
                                &format!("str_{}", self.string_counter),
                                Linkage::Local,
                                false,
                                false,
                            ).unwrap();
                            self.module.define_data(data_id, &data_ctx).unwrap();
                            let local_id = self.module.declare_data_in_func(data_id, builder.func);
                            builder.ins().symbol_value(types::I64, local_id)
                        }
                        _ => builder.ins().iconst(types::I64, 0),
                    };
                    builder.ins().stack_store(val_i64, slots[*dest], 0);
                }
                Opcode::Move(dest, src) => {
                    let val_i64 = builder.ins().stack_load(types::I64, slots[*src], 0);
                    builder.ins().stack_store(val_i64, slots[*dest], 0);
                }
                Opcode::Neg(dest, src, is_float) => {
                    let val_i64 = builder.ins().stack_load(types::I64, slots[*src], 0);
                    if *is_float {
                        let val = builder.ins().bitcast(types::F64, MemFlags::new(), val_i64);
                        let res = builder.ins().fneg(val);
                        let res_i64 = builder.ins().bitcast(types::I64, MemFlags::new(), res);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    } else {
                        let res_i64 = builder.ins().ineg(val_i64);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    }
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
                Opcode::Print(src, print_type) => {
                    let val_i64 = builder.ins().stack_load(types::I64, slots[*src], 0);
                    match print_type {
                        PrintType::Float => {
                            let val = builder.ins().bitcast(types::F64, MemFlags::new(), val_i64);
                            let local_print = self.module.declare_func_in_func(self.print_func_id, builder.func);
                            builder.ins().call(local_print, &[val]);
                        }
                        PrintType::Int | PrintType::Bool => {
                            let local_print_i64 = self.module.declare_func_in_func(self.print_i64_func_id, builder.func);
                            builder.ins().call(local_print_i64, &[val_i64]);
                        }
                        PrintType::String => {
                            let local_print_str = self.module.declare_func_in_func(self.print_str_func_id, builder.func);
                            builder.ins().call(local_print_str, &[val_i64]);
                        }
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
                            call_args.push(arg_i64);
                        }
                        let local_func = self.module.declare_func_in_func(*func_id, builder.func);
                        let call = builder.ins().call(local_func, &call_args);
                        let res = builder.inst_results(call)[0];
                        builder.ins().stack_store(res, slots[*dest], 0);
                    } else {
                        panic!("Function not found in AOT: {}", name);
                    }
                }
                Opcode::Return(reg) => {
                    if has_return {
                        let val_i64 = builder.ins().stack_load(types::I64, slots[*reg], 0);
                        builder.ins().return_(&[val_i64]);
                    } else {
                        let zero = builder.ins().iconst(types::I32, 0);
                        builder.ins().return_(&[zero]);
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
                Opcode::MakeStruct(dest, field_count, first_field_reg) => {
                    let local_malloc = self.module.declare_func_in_func(self.malloc_func_id, builder.func);
                    let size = builder.ins().iconst(types::I64, (*field_count * 8) as i64);
                    let call = builder.ins().call(local_malloc, &[size]);
                    let ptr = builder.inst_results(call)[0];
                    
                    for i in 0..*field_count {
                        let val_i64 = builder.ins().stack_load(types::I64, slots[*first_field_reg + i], 0);
                        builder.ins().store(MemFlags::new(), val_i64, ptr, (i * 8) as i32);
                    }
                    builder.ins().stack_store(ptr, slots[*dest], 0);
                }
                Opcode::FieldAccess(dest, obj_reg, field_idx) => {
                    let ptr = builder.ins().stack_load(types::I64, slots[*obj_reg], 0);
                    let val_i64 = builder.ins().load(types::I64, MemFlags::new(), ptr, (*field_idx * 8) as i32);
                    builder.ins().stack_store(val_i64, slots[*dest], 0);
                }
                Opcode::FieldAssign(obj_reg, field_idx, val_reg) => {
                    let ptr = builder.ins().stack_load(types::I64, slots[*obj_reg], 0);
                    let val_i64 = builder.ins().stack_load(types::I64, slots[*val_reg], 0);
                    builder.ins().store(MemFlags::new(), val_i64, ptr, (*field_idx * 8) as i32);
                }
                Opcode::Mul(dest, left, right, is_float) => {
                    let l_i64 = builder.ins().stack_load(types::I64, slots[*left], 0);
                    let r_i64 = builder.ins().stack_load(types::I64, slots[*right], 0);
                    if *is_float {
                        let l = builder.ins().bitcast(types::F64, MemFlags::new(), l_i64);
                        let r = builder.ins().bitcast(types::F64, MemFlags::new(), r_i64);
                        let res = builder.ins().fmul(l, r);
                        let res_i64 = builder.ins().bitcast(types::I64, MemFlags::new(), res);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    } else {
                        let res_i64 = builder.ins().imul(l_i64, r_i64);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    }
                }
                Opcode::Div(dest, left, right, is_float) => {
                    let l_i64 = builder.ins().stack_load(types::I64, slots[*left], 0);
                    let r_i64 = builder.ins().stack_load(types::I64, slots[*right], 0);
                    if *is_float {
                        let l = builder.ins().bitcast(types::F64, MemFlags::new(), l_i64);
                        let r = builder.ins().bitcast(types::F64, MemFlags::new(), r_i64);
                        let res = builder.ins().fdiv(l, r);
                        let res_i64 = builder.ins().bitcast(types::I64, MemFlags::new(), res);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    } else {
                        let res_i64 = builder.ins().sdiv(l_i64, r_i64);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    }
                }
                Opcode::Ne(dest, left, right, is_float) => {
                    let l_i64 = builder.ins().stack_load(types::I64, slots[*left], 0);
                    let r_i64 = builder.ins().stack_load(types::I64, slots[*right], 0);
                    if *is_float {
                        let l = builder.ins().bitcast(types::F64, MemFlags::new(), l_i64);
                        let r = builder.ins().bitcast(types::F64, MemFlags::new(), r_i64);
                        let cmp = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::NotEqual, l, r);
                        let res_i64 = builder.ins().uextend(types::I64, cmp);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    } else {
                        let cmp = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::NotEqual, l_i64, r_i64);
                        let res_i64 = builder.ins().uextend(types::I64, cmp);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    }
                }
                Opcode::Le(dest, left, right, is_float) => {
                    let l_i64 = builder.ins().stack_load(types::I64, slots[*left], 0);
                    let r_i64 = builder.ins().stack_load(types::I64, slots[*right], 0);
                    if *is_float {
                        let l = builder.ins().bitcast(types::F64, MemFlags::new(), l_i64);
                        let r = builder.ins().bitcast(types::F64, MemFlags::new(), r_i64);
                        let cmp = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::LessThanOrEqual, l, r);
                        let res_i64 = builder.ins().uextend(types::I64, cmp);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    } else {
                        let cmp = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual, l_i64, r_i64);
                        let res_i64 = builder.ins().uextend(types::I64, cmp);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    }
                }
                Opcode::Gt(dest, left, right, is_float) => {
                    let l_i64 = builder.ins().stack_load(types::I64, slots[*left], 0);
                    let r_i64 = builder.ins().stack_load(types::I64, slots[*right], 0);
                    if *is_float {
                        let l = builder.ins().bitcast(types::F64, MemFlags::new(), l_i64);
                        let r = builder.ins().bitcast(types::F64, MemFlags::new(), r_i64);
                        let cmp = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::GreaterThan, l, r);
                        let res_i64 = builder.ins().uextend(types::I64, cmp);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    } else {
                        let cmp = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThan, l_i64, r_i64);
                        let res_i64 = builder.ins().uextend(types::I64, cmp);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    }
                }
                Opcode::Ge(dest, left, right, is_float) => {
                    let l_i64 = builder.ins().stack_load(types::I64, slots[*left], 0);
                    let r_i64 = builder.ins().stack_load(types::I64, slots[*right], 0);
                    if *is_float {
                        let l = builder.ins().bitcast(types::F64, MemFlags::new(), l_i64);
                        let r = builder.ins().bitcast(types::F64, MemFlags::new(), r_i64);
                        let cmp = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::GreaterThanOrEqual, l, r);
                        let res_i64 = builder.ins().uextend(types::I64, cmp);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    } else {
                        let cmp = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThanOrEqual, l_i64, r_i64);
                        let res_i64 = builder.ins().uextend(types::I64, cmp);
                        builder.ins().stack_store(res_i64, slots[*dest], 0);
                    }
                }
                Opcode::MakeEnum(dest, variant_idx, start_reg, count) => {
                    let local_malloc = self.module.declare_func_in_func(self.malloc_func_id, builder.func);
                    let size = builder.ins().iconst(types::I64, ((*count + 1) * 8) as i64);
                    let call = builder.ins().call(local_malloc, &[size]);
                    let ptr = builder.inst_results(call)[0];
                    
                    let tag = builder.ins().iconst(types::I64, *variant_idx as i64);
                    builder.ins().store(MemFlags::new(), tag, ptr, 0);
                    
                    for i in 0..*count {
                        let val_i64 = builder.ins().stack_load(types::I64, slots[*start_reg + i], 0);
                        builder.ins().store(MemFlags::new(), val_i64, ptr, ((i + 1) * 8) as i32);
                    }
                    builder.ins().stack_store(ptr, slots[*dest], 0);
                }
                Opcode::CheckEnum(dest, obj_reg, expected_variant) => {
                    let ptr = builder.ins().stack_load(types::I64, slots[*obj_reg], 0);
                    let actual_variant = builder.ins().load(types::I64, MemFlags::new(), ptr, 0);
                    let expected_val = builder.ins().iconst(types::I64, *expected_variant as i64);
                    let cmp = builder.ins().icmp(IntCC::Equal, actual_variant, expected_val);
                    let cmp_i64 = builder.ins().uextend(types::I64, cmp);
                    builder.ins().stack_store(cmp_i64, slots[*dest], 0);
                }
                Opcode::ExtractEnum(dest_start, obj_reg, count) => {
                    let ptr = builder.ins().stack_load(types::I64, slots[*obj_reg], 0);
                    for i in 0..*count {
                        let val = builder.ins().load(types::I64, MemFlags::new(), ptr, ((i + 1) * 8) as i32);
                        builder.ins().stack_store(val, slots[*dest_start + i], 0);
                    }
                }
                Opcode::MakeArray(dest, first_elem_reg, count) => {
                    let local_malloc = self.module.declare_func_in_func(self.malloc_func_id, builder.func);
                    
                    let data_size = builder.ins().iconst(types::I64, (*count as i64) * 8);
                    let data_call = builder.ins().call(local_malloc, &[data_size]);
                    let data_ptr = builder.inst_results(data_call)[0];
                    
                    for i in 0..*count {
                        let val = builder.ins().stack_load(types::I64, slots[*first_elem_reg + i], 0);
                        builder.ins().store(MemFlags::new(), val, data_ptr, (i * 8) as i32);
                    }
                    
                    let struct_size = builder.ins().iconst(types::I64, 24);
                    let struct_call = builder.ins().call(local_malloc, &[struct_size]);
                    let struct_ptr = builder.inst_results(struct_call)[0];
                    
                    let len_val = builder.ins().iconst(types::I64, *count as i64);
                    builder.ins().store(MemFlags::new(), data_ptr, struct_ptr, 0);
                    builder.ins().store(MemFlags::new(), len_val, struct_ptr, 8);
                    builder.ins().store(MemFlags::new(), len_val, struct_ptr, 16);
                    
                    builder.ins().stack_store(struct_ptr, slots[*dest], 0);
                }
                Opcode::ArrayIndex(dest, array_reg, index_reg) => {
                    let struct_ptr = builder.ins().stack_load(types::I64, slots[*array_reg], 0);
                    let data_ptr = builder.ins().load(types::I64, MemFlags::new(), struct_ptr, 0);
                    let index_i64 = builder.ins().stack_load(types::I64, slots[*index_reg], 0);
                    
                    let eight = builder.ins().iconst(types::I64, 8);
                    let offset = builder.ins().imul(index_i64, eight);
                    let elem_addr = builder.ins().iadd(data_ptr, offset);
                    
                    let val = builder.ins().load(types::I64, MemFlags::new(), elem_addr, 0);
                    builder.ins().stack_store(val, slots[*dest], 0);
                }
                Opcode::ArrayAssign(array_reg, index_reg, value_reg) => {
                    let struct_ptr = builder.ins().stack_load(types::I64, slots[*array_reg], 0);
                    let data_ptr = builder.ins().load(types::I64, MemFlags::new(), struct_ptr, 0);
                    
                    let index_i64 = builder.ins().stack_load(types::I64, slots[*index_reg], 0);
                    
                    let val = builder.ins().stack_load(types::I64, slots[*value_reg], 0);
                    
                    let eight = builder.ins().iconst(types::I64, 8);
                    let offset = builder.ins().imul(index_i64, eight);
                    let elem_addr = builder.ins().iadd(data_ptr, offset);
                    
                    builder.ins().store(MemFlags::new(), val, elem_addr, 0);
                }
                Opcode::TryUnwrap(dest, src) => {
                    let ptr = builder.ins().stack_load(types::I64, slots[*src], 0);
                    let tag = builder.ins().load(types::I64, MemFlags::new(), ptr, 0);
                    let expected_ok = builder.ins().iconst(types::I64, 0);
                    let is_err = builder.ins().icmp(IntCC::NotEqual, tag, expected_ok);
                    
                    let err_block = builder.create_block();
                    let ok_block = builder.create_block();
                    let next_block = blocks[i + 1];
                    
                    builder.ins().brif(is_err, err_block, &[], ok_block, &[]);
                    
                    builder.switch_to_block(err_block);
                    builder.ins().return_(&[ptr]);
                    
                    builder.switch_to_block(ok_block);
                    let val = builder.ins().load(types::I64, MemFlags::new(), ptr, 8);
                    builder.ins().stack_store(val, slots[*dest], 0);
                    builder.ins().jump(next_block, &[]);
                    
                    block_terminated = true;
                }
                Opcode::AsyncCall(..) |
                Opcode::Await(..) |
                Opcode::Spawn(..) => {
                    return Err("Async opcodes are not supported in AOT MVP (fallback to VM recommended)".to_string());
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
