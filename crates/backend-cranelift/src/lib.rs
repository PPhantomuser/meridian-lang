#![allow(unused)]
use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::{
    types, AbiParam, InstBuilder,
};
use cranelift_codegen::settings;
use cranelift_codegen::settings::Configurable;
use cranelift_codegen::Context;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module, FuncId};
use meridian_ir::{Chunk, ConstValue, Opcode, ProgramIR};
use std::collections::HashMap;

pub mod aot;
pub use aot::AOTCompiler;

pub struct JITCompiler {
    module: JITModule,
    ctx: Context,
    builder_context: FunctionBuilderContext,
    functions: HashMap<String, FuncId>,
    print_func_id: FuncId,
    print_i64_func_id: FuncId,
}

impl Default for JITCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl JITCompiler {
    pub fn new() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        
        let isa_builder = cranelift_native::builder().unwrap();
        let isa = isa_builder.finish(settings::Flags::new(flag_builder)).unwrap();
        
        let mut builder = JITBuilder::with_isa(isa, default_libcall_names());
        
        // Define native print symbol and bind it to our Rust function
        builder.symbol("print_f64", print_f64 as *const u8);
        builder.symbol("print_i64", print_i64 as *const u8);
        
        let mut module = JITModule::new(builder);
        
        // Declare the print_f64 function in the module
        let mut print_sig = module.make_signature();
        print_sig.params.push(AbiParam::new(types::F64));
        // print_f64 returns nothing (void)
        
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

    pub fn compile_and_run(&mut self, program_ir: &ProgramIR) {
        for (name, (_chunk, _is_async, arg_count)) in &program_ir.functions {
            let mut sig = self.module.make_signature();
            for _ in 0..*arg_count {
                sig.params.push(AbiParam::new(types::F64));
            }
            sig.returns.push(AbiParam::new(types::F64));
            let func_id = self.module.declare_function(name, Linkage::Export, &sig).unwrap();
            self.functions.insert(name.clone(), func_id);
        }

        // Declare external functions
        for (name, arg_count) in &program_ir.extern_functions {
            let mut sig = self.module.make_signature();
            for _ in 0..*arg_count {
                // In a robust implementation, we would map the exact C ABI types from the AST.
                // Since this is a minimal FFI foundation for V3.5, we will pass arguments as pointers/numbers (represented as F64 or I64 in Cranelift).
                // Assuming Cranelift pointer size is I64 on most platforms, but we use F64 for all Meridian values right now.
                // We'll pass them as F64 for now, but a true C-FFI would cast to I64 for pointers. Let's assume F64 works or we use I64 if it's a pointer.
                // Minimal approach: just use F64 for all.
                sig.params.push(AbiParam::new(types::F64));
            }
            sig.returns.push(AbiParam::new(types::F64));
            // Linkage::Import means it will be resolved by the system dynamic linker!
            let func_id = self.module.declare_function(name, Linkage::Import, &sig).unwrap();
            self.functions.insert(name.clone(), func_id);
        }

        // Compile all functions
        for (name, (chunk, _is_async, arg_count)) in &program_ir.functions {
            let func_id = *self.functions.get(name).unwrap();
            self.compile_chunk(chunk, *arg_count, true);
            self.module.define_function(func_id, &mut self.ctx).unwrap();
            self.module.clear_context(&mut self.ctx);
        }

        // Compile main
        let mut main_sig = self.module.make_signature();
        main_sig.returns.push(AbiParam::new(types::I32));
        // main has no args, returns void or 0
        let main_func_id = self.module.declare_function("main", Linkage::Export, &main_sig).unwrap();
        
        self.compile_chunk(&program_ir.main_chunk, 0, false);
        self.module.define_function(main_func_id, &mut self.ctx).unwrap();
        self.module.clear_context(&mut self.ctx);

        // Finalize all
        self.module.finalize_definitions().unwrap();

        // Get the main function pointer and run it
        let code = self.module.get_finalized_function(main_func_id);
        let main_fn: extern "C" fn() -> i32 = unsafe { std::mem::transmute(code) };
        main_fn();
    }

    fn compile_chunk(&mut self, chunk: &Chunk, arg_count: usize, has_return: bool) {
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        
        // Add arguments to signature
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

        // Declare variables for all registers (0 to 100 max for this prototype)
        for i in 0..100 {
            let var = Variable::new(i);
            builder.declare_var(var, types::F64);
            // Default initialize to 0.0
            let zero = builder.ins().f64const(0.0);
            builder.def_var(var, zero);
        }

        // Bind arguments to variables 0..arg_count
        for i in 0..arg_count {
            let val = builder.block_params(entry_block)[i];
            builder.def_var(Variable::new(i), val);
        }

        // We need blocks for jumps. We'll create blocks for every instruction index just to be safe.
        let mut blocks = vec![entry_block];
        for _ in 1..chunk.instructions.len() {
            blocks.push(builder.create_block());
        }
        // Create an end block
        let end_block = builder.create_block();
        blocks.push(end_block);

        for _i in 1..blocks.len() {
            // We can't seal blocks yet because of forward jumps
        }

        let mut current_block = entry_block;
        let mut block_terminated = false;

        for (i, inst) in chunk.instructions.iter().enumerate() {
            if i > 0 {
                // If the previous block didn't jump/return, jump to this block
                if !block_terminated {
                    builder.ins().jump(blocks[i], &[]);
                }
                current_block = blocks[i];
                builder.switch_to_block(current_block);
                block_terminated = false;
            }

            match inst {
                Opcode::LoadConst(dest, const_idx) => {
                    let val = match &chunk.constants[*const_idx] {
                        ConstValue::Number(n) => builder.ins().f64const(*n),
                        ConstValue::Int(n) => builder.ins().f64const(*n as f64), // temporary workaround until we support i64 native
                        ConstValue::Bool(b) => builder.ins().f64const(if *b { 1.0 } else { 0.0 }),
                        _ => builder.ins().f64const(0.0), // unsupported in primitive subset
                    };
                    builder.def_var(Variable::new(*dest), val);
                }
                Opcode::Move(dest, src) => {
                    let val = builder.use_var(Variable::new(*src));
                    builder.def_var(Variable::new(*dest), val);
                }
                Opcode::Add(dest, left, right, _) => {
                    let l = builder.use_var(Variable::new(*left));
                    let r = builder.use_var(Variable::new(*right));
                    let res = builder.ins().fadd(l, r);
                    builder.def_var(Variable::new(*dest), res);
                }
                Opcode::Sub(dest, left, right, _) => {
                    let l = builder.use_var(Variable::new(*left));
                    let r = builder.use_var(Variable::new(*right));
                    let res = builder.ins().fsub(l, r);
                    builder.def_var(Variable::new(*dest), res);
                }
                Opcode::Mul(dest, left, right, _) => {
                    let l = builder.use_var(Variable::new(*left));
                    let r = builder.use_var(Variable::new(*right));
                    let res = builder.ins().fmul(l, r);
                    builder.def_var(Variable::new(*dest), res);
                }
                Opcode::Div(dest, left, right, _) => {
                    let l = builder.use_var(Variable::new(*left));
                    let r = builder.use_var(Variable::new(*right));
                    let res = builder.ins().fdiv(l, r);
                    builder.def_var(Variable::new(*dest), res);
                }
                Opcode::Eq(dest, left, right, _) => {
                    let l = builder.use_var(Variable::new(*left));
                    let r = builder.use_var(Variable::new(*right));
                    let cmp = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::Equal, l, r);
                    
                    let true_val = builder.ins().f64const(1.0);
                    let false_val = builder.ins().f64const(0.0);
                    let res = builder.ins().select(cmp, true_val, false_val);
                    builder.def_var(Variable::new(*dest), res);
                }
                Opcode::Lt(dest, left, right, _) => {
                    let l = builder.use_var(Variable::new(*left));
                    let r = builder.use_var(Variable::new(*right));
                    let cmp = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::LessThan, l, r);
                    
                    let true_val = builder.ins().f64const(1.0);
                    let false_val = builder.ins().f64const(0.0);
                    let res = builder.ins().select(cmp, true_val, false_val);
                    builder.def_var(Variable::new(*dest), res);
                }
                Opcode::Le(dest, left, right, _) => {
                    let l = builder.use_var(Variable::new(*left));
                    let r = builder.use_var(Variable::new(*right));
                    let cmp = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::LessThanOrEqual, l, r);
                    
                    let true_val = builder.ins().f64const(1.0);
                    let false_val = builder.ins().f64const(0.0);
                    let res = builder.ins().select(cmp, true_val, false_val);
                    builder.def_var(Variable::new(*dest), res);
                }
                Opcode::Gt(dest, left, right, _) => {
                    let l = builder.use_var(Variable::new(*left));
                    let r = builder.use_var(Variable::new(*right));
                    let cmp = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::GreaterThan, l, r);
                    
                    let true_val = builder.ins().f64const(1.0);
                    let false_val = builder.ins().f64const(0.0);
                    let res = builder.ins().select(cmp, true_val, false_val);
                    builder.def_var(Variable::new(*dest), res);
                }
                Opcode::Ge(dest, left, right, _) => {
                    let l = builder.use_var(Variable::new(*left));
                    let r = builder.use_var(Variable::new(*right));
                    let cmp = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::GreaterThanOrEqual, l, r);
                    
                    let true_val = builder.ins().f64const(1.0);
                    let false_val = builder.ins().f64const(0.0);
                    let res = builder.ins().select(cmp, true_val, false_val);
                    builder.def_var(Variable::new(*dest), res);
                }
                Opcode::Ne(dest, left, right, _) => {
                    let l = builder.use_var(Variable::new(*left));
                    let r = builder.use_var(Variable::new(*right));
                    let cmp = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::NotEqual, l, r);
                    
                    let true_val = builder.ins().f64const(1.0);
                    let false_val = builder.ins().f64const(0.0);
                    let res = builder.ins().select(cmp, true_val, false_val);
                    builder.def_var(Variable::new(*dest), res);
                }
                Opcode::JumpIfFalse(cond, offset) => {
                    let cond_val = builder.use_var(Variable::new(*cond));
                    let zero = builder.ins().f64const(0.0);
                    let is_false = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::Equal, cond_val, zero);
                    let target_block = blocks[*offset];
                    let next_block = blocks[i + 1];
                    builder.ins().brif(is_false, target_block, &[], next_block, &[]);
                    block_terminated = true;
                }
                Opcode::Jump(offset) => {
                    let target_block = blocks[*offset];
                    builder.ins().jump(target_block, &[]);
                    block_terminated = true;
                }
                Opcode::Print(src, is_float) => {
                    let val = builder.use_var(Variable::new(*src));
                    if *is_float {
                        let local_print = self.module.declare_func_in_func(self.print_func_id, builder.func);
                        builder.ins().call(local_print, &[val]);
                    } else {
                        let local_print_i64 = self.module.declare_func_in_func(self.print_i64_func_id, builder.func);
                        builder.ins().call(local_print_i64, &[val]);
                    }
                }
                Opcode::Call(dest, func_name, arg_start, arg_count) => {
                    if let Some(func_id) = self.functions.get(func_name) {
                        let local_func = self.module.declare_func_in_func(*func_id, builder.func);
                        let mut args = Vec::new();
                        for j in 0..*arg_count {
                            args.push(builder.use_var(Variable::new(arg_start + j)));
                        }
                        let call_inst = builder.ins().call(local_func, &args);
                        let res = builder.inst_results(call_inst)[0];
                        builder.def_var(Variable::new(*dest), res);
                    }
                }
                Opcode::Return(src) => {
                    let val = builder.use_var(Variable::new(*src));
                    builder.ins().return_(&[val]);
                    block_terminated = true;
                }
                Opcode::Borrow(_, _) => {
                    unimplemented!("Borrow not yet supported in Cranelift backend. Requires StackSlots.");
                }
                Opcode::Dereference(_, _) => {
                    unimplemented!("Dereference not yet supported in Cranelift backend. Requires memory loads.");
                }
                Opcode::AsyncCall(_, _, _, _) | Opcode::Await(_, _) | Opcode::Spawn(_, _) => {
                    unimplemented!("Async not yet supported in Cranelift backend.");
                }
                Opcode::MakeStruct(..) | Opcode::FieldAccess(..) | Opcode::FieldAssign(..) => {
                    unimplemented!("Structs not yet supported in Cranelift backend (AOT Phase 2 fallback to VM Handles pending).");
                }
                Opcode::MakeArray(..) | Opcode::ArrayIndex(..) | Opcode::ArrayAssign(..) => {
                    unimplemented!("Arrays not yet supported in Cranelift backend (AOT Phase 2 fallback to VM Handles pending).");
                }
                Opcode::MakeEnum(..) | Opcode::CheckEnum(..) | Opcode::ExtractEnum(..) => {
                    unimplemented!("Enums not yet supported in Cranelift backend.");
                }
                Opcode::TryUnwrap(_, _) => {
                    unimplemented!("TryUnwrap is not supported in Cranelift AOT MVP");
                }
            }
        }

        if !block_terminated {
            if has_return {
                let zero = builder.ins().f64const(0.0);
                builder.ins().return_(&[zero]);
            } else {
                let zero = builder.ins().iconst(types::I32, 0);
                builder.ins().return_(&[zero]);
            }
        }

        // Seal all blocks except entry_block since it's already sealed
        for (i, block) in blocks.into_iter().enumerate() {
            if i > 0 {
                builder.seal_block(block);
            }
        }

        builder.finalize();
    }
}

pub extern "C" fn print_f64(n: f64) {
    println!("{}", n);
}

pub extern "C" fn print_i64(n: i64) {
    println!("{}", n);
}
