use meridian_ast::{BinaryOperator, Expr, Program, Stmt, Type};
use std::collections::HashMap;

pub type Register = usize;
pub type ConstIndex = usize;
pub type Offset = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Opcode {
    LoadConst(Register, ConstIndex),   // dest, const_idx
    Move(Register, Register),          // dest, src
    Add(Register, Register, Register, bool), // dest, left, right, is_float
    Sub(Register, Register, Register, bool), // dest, left, right, is_float
    Mul(Register, Register, Register, bool), // dest, left, right, is_float
    Div(Register, Register, Register, bool), // dest, left, right, is_float
    Eq(Register, Register, Register, bool),  // dest, left, right, is_float
    Ne(Register, Register, Register, bool),  // dest, left, right, is_float
    Lt(Register, Register, Register, bool),  // dest, left, right, is_float
    Le(Register, Register, Register, bool),  // dest, left, right, is_float
    Gt(Register, Register, Register, bool),  // dest, left, right, is_float
    Ge(Register, Register, Register, bool),  // dest, left, right, is_float
    JumpIfFalse(Register, Offset),     // condition, target_offset
    Jump(Offset),                      // target_offset
    Call(Register, String, Register, usize), // dest, function_name, arg_start_reg, arg_count
    Return(Register),                  // src
    Print(Register, bool),             // src, is_float
    Borrow(Register, Register),        // dest, src_reg
    Dereference(Register, Register),   // dest, src_reg
    AsyncCall(Register, String, Register, usize), // dest, func, arg_start, count
    Await(Register, Register),         // dest, src_future
    Spawn(Register, Register),         // dest, src_future
    MakeStruct(Register, String, Vec<String>, Register), // dest, struct_name, field_names, first_field_reg
    FieldAccess(Register, Register, String), // dest, obj, field_name
    FieldAssign(Register, String, Register), // obj, field_name, value (no dest)
    MakeArray(Register, Register, usize), // dest, first_elem_reg, count
    ArrayIndex(Register, Register, Register), // dest, obj, index
    ArrayAssign(Register, Register, Register), // obj, index, value
    MakeEnum(Register, String, String, Register, usize), // dest, enum_name, variant_name, value_start_reg, count
    CheckEnum(Register, Register, String), // dest (bool), obj, variant_name
    ExtractEnum(Register, Register, usize), // dest_start, obj (gets inner values), count
    TryUnwrap(Register, Register),     // dest, src (unwraps Ok, returns if Err)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Number(f64),
    Int(i64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub instructions: Vec<Opcode>,
    pub constants: Vec<ConstValue>,
}

impl Chunk {
    pub fn add_constant(&mut self, val: ConstValue) -> ConstIndex {
        if let Some(idx) = self.constants.iter().position(|c| c == &val) {
            return idx;
        }
        self.constants.push(val);
        self.constants.len() - 1
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProgramIR {
    pub main_chunk: Chunk,
    pub functions: HashMap<String, (Chunk, bool, usize)>,
    pub extern_functions: HashMap<String, usize>, // name -> arg_count
    pub macros: HashMap<String, (Vec<meridian_ast::Parameter>, Expr)>,
}

pub struct Compiler {
    program_ir: ProgramIR,
    current_chunk: Chunk,
    next_reg: Register,
    locals: HashMap<String, Register>,
    loop_contexts: Vec<LoopContext>,
    type_map: HashMap<meridian_diagnostics::Span, meridian_ast::Type>,
    resolved_names: HashMap<meridian_diagnostics::Span, String>,
    auto_borrows: std::collections::HashSet<meridian_diagnostics::Span>,
}

#[derive(Debug, Clone)]
struct LoopContext {
    start_offset: Offset,
    break_patches: Vec<usize>,
}

impl Compiler {
    pub fn new(type_map: HashMap<meridian_diagnostics::Span, meridian_ast::Type>, resolved_names: HashMap<meridian_diagnostics::Span, String>, auto_borrows: std::collections::HashSet<meridian_diagnostics::Span>) -> Self {
        Self {
            program_ir: ProgramIR::default(),
            current_chunk: Chunk::default(),
            next_reg: 0,
            locals: HashMap::new(),
            loop_contexts: Vec::new(),
            type_map,
            resolved_names,
            auto_borrows,
        }
    }

    fn alloc_reg(&mut self) -> Register {
        let r = self.next_reg;
        self.next_reg += 1;
        r
    }

    pub fn compile(mut self, program: &Program) -> ProgramIR {
        let mut function_stmts = Vec::new();
        let mut main_stmts = Vec::new();
        for stmt in &program.statements {
            if let Stmt::MacroDef { name, parameters, body, .. } = stmt {
                self.program_ir.macros.insert(name.clone(), (parameters.clone(), body.clone()));
            } else if let Stmt::Function { name, .. } = stmt {
                function_stmts.push((name.clone(), stmt));
            } else if let Stmt::ExternBlock { functions, .. } = stmt {
                for func in functions {
                    if let Stmt::Function { name, parameters, .. } = func {
                        self.program_ir.extern_functions.insert(name.clone(), parameters.len());
                    }
                }
            } else if let Stmt::TraitDef { .. } = stmt {
                // Traits are purely compile-time
            } else if let Stmt::Impl { target_name, methods, .. } = stmt {
                for method in methods {
                    if let Stmt::Function { name, .. } = method {
                        // Mangling exactly as in semantic analyzer
                        let mangled = format!("{}_{}", target_name, name);
                        function_stmts.push((mangled, method));
                    }
                }
            } else {
                main_stmts.push(stmt);
            }
        }

        for (mangled_name, stmt) in function_stmts {
            if let Stmt::Function { parameters, is_async, body, .. } = stmt {
                let saved_chunk = std::mem::take(&mut self.current_chunk);
                let saved_next_reg = self.next_reg;
                let saved_locals = std::mem::take(&mut self.locals);

                self.next_reg = 0;
                for param in parameters {
                    let reg = self.alloc_reg();
                    self.locals.insert(param.name.clone(), reg);
                }

                let ret_reg = self.compile_expr(body);
                self.current_chunk.instructions.push(Opcode::Return(ret_reg));

                let compiled_fn = std::mem::take(&mut self.current_chunk);
                self.program_ir.functions.insert(mangled_name, (compiled_fn, *is_async, parameters.len()));

                self.current_chunk = saved_chunk;
                self.next_reg = saved_next_reg;
                self.locals = saved_locals;
                self.loop_contexts = Vec::new(); // functions don't inherit loop context
            }
        }

        for stmt in &main_stmts {
            self.compile_stmt(stmt);
        }

        // A10: Auto-call main if explicitly defined, but only if not called explicitly
        let mut has_explicit_main_call = false;
        for stmt in &main_stmts {
            if let meridian_ast::Stmt::Expr(meridian_ast::Expr::Call { callee, .. }) = stmt {
                if let meridian_ast::Expr::Identifier(name, _) = &**callee {
                    if name == "main" {
                        has_explicit_main_call = true;
                    }
                }
            }
        }
        
        if self.program_ir.functions.contains_key("main") && !has_explicit_main_call {
            let ret_reg = self.alloc_reg();
            self.current_chunk.instructions.push(Opcode::Call(ret_reg, "main".to_string(), 0, 0));
        }

        self.program_ir.main_chunk = self.current_chunk;
        self.program_ir
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::TraitDef { .. } => {
                // Ignore, purely compile time
            }
            Stmt::Let { name, initializer, .. } => {
                let val_reg = self.compile_expr(initializer);
                self.locals.insert(name.clone(), val_reg);
            }
            Stmt::Print(expr, _) => {
                let is_float = if let Some(ty) = self.type_map.get(&expr.span()) {
                    matches!(ty, Type::Number)
                } else {
                    true
                };
                let reg = self.compile_expr(expr);
                self.current_chunk.instructions.push(Opcode::Print(reg, is_float));
            }
            Stmt::Expr(expr) => {
                self.compile_expr(expr);
            }
            Stmt::While { condition, body, .. } => {
                let start_offset = self.current_chunk.instructions.len();
                
                let cond_reg = self.compile_expr(condition);
                let jump_if_false_idx = self.current_chunk.instructions.len();
                self.current_chunk.instructions.push(Opcode::JumpIfFalse(cond_reg, 0)); // patch later
                
                self.loop_contexts.push(LoopContext {
                    start_offset,
                    break_patches: Vec::new(),
                });
                
                self.compile_expr(body);
                
                let loop_ctx = self.loop_contexts.pop().unwrap();
                self.current_chunk.instructions.push(Opcode::Jump(start_offset));
                
                let end_offset = self.current_chunk.instructions.len();
                self.current_chunk.instructions[jump_if_false_idx] = Opcode::JumpIfFalse(cond_reg, end_offset);
                
                for patch_idx in loop_ctx.break_patches {
                    self.current_chunk.instructions[patch_idx] = Opcode::Jump(end_offset);
                }
            }
            Stmt::For { iterator, iterable, body, .. } => {
                let (start_expr, end_expr, inclusive) = if let Expr::Range { start, end, inclusive, .. } = iterable {
                    (start, end, *inclusive)
                } else {
                    unimplemented!("Only Range is supported in for loops for now");
                };
                
                let start_reg = self.compile_expr(start_expr);
                let end_reg = self.compile_expr(end_expr);
                
                let iter_reg = self.alloc_reg();
                self.locals.insert(iterator.clone(), iter_reg);
                self.current_chunk.instructions.push(Opcode::Move(iter_reg, start_reg));
                
                let loop_start_offset = self.current_chunk.instructions.len();
                
                // condition: iter < end (or <= if inclusive)
                let cond_reg = self.alloc_reg();
                if inclusive {
                    self.current_chunk.instructions.push(Opcode::Le(cond_reg, iter_reg, end_reg, false));
                } else {
                    self.current_chunk.instructions.push(Opcode::Lt(cond_reg, iter_reg, end_reg, false));
                }
                
                let jump_if_false_idx = self.current_chunk.instructions.len();
                self.current_chunk.instructions.push(Opcode::JumpIfFalse(cond_reg, 0)); // patch later
                
                self.loop_contexts.push(LoopContext {
                    start_offset: loop_start_offset,
                    break_patches: Vec::new(),
                });
                
                self.compile_expr(body);
                
                // increment iter
                let one_idx = self.current_chunk.add_constant(ConstValue::Number(1.0));
                let one_reg = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::LoadConst(one_reg, one_idx));
                self.current_chunk.instructions.push(Opcode::Add(iter_reg, iter_reg, one_reg, false));
                
                let loop_ctx = self.loop_contexts.pop().unwrap();
                self.current_chunk.instructions.push(Opcode::Jump(loop_start_offset));
                
                let end_offset = self.current_chunk.instructions.len();
                self.current_chunk.instructions[jump_if_false_idx] = Opcode::JumpIfFalse(cond_reg, end_offset);
                
                for patch_idx in loop_ctx.break_patches {
                    self.current_chunk.instructions[patch_idx] = Opcode::Jump(end_offset);
                }
            }
            Stmt::Break(_) => {
                let jump_idx = self.current_chunk.instructions.len();
                self.current_chunk.instructions.push(Opcode::Jump(0)); // patch later
                if let Some(ctx) = self.loop_contexts.last_mut() {
                    ctx.break_patches.push(jump_idx);
                }
            }
            Stmt::Continue(_) => {
                if let Some(ctx) = self.loop_contexts.last() {
                    let start = ctx.start_offset;
                    self.current_chunk.instructions.push(Opcode::Jump(start));
                }
            }
            Stmt::Return(expr_opt, _) => {
                let ret_reg = if let Some(expr) = expr_opt {
                    self.compile_expr(expr)
                } else {
                    let idx = self.current_chunk.add_constant(ConstValue::Number(0.0));
                    let reg = self.alloc_reg();
                    self.current_chunk.instructions.push(Opcode::LoadConst(reg, idx));
                    reg
                };
                self.current_chunk.instructions.push(Opcode::Return(ret_reg));
            }
            Stmt::Function { .. } | Stmt::Import(_, _) | Stmt::ExternBlock { .. } | Stmt::StructDef { .. } | Stmt::EnumDef { .. } | Stmt::Impl { .. } => {}
            Stmt::MacroDef { name, parameters, body, .. } => {
                self.program_ir.macros.insert(name.clone(), (parameters.clone(), body.clone()));
            }
        }
    }

    fn compile_expr(&mut self, expr: &Expr) -> Register {
        match expr {
            Expr::Number(n, _) => {
                let idx = self.current_chunk.add_constant(ConstValue::Number(*n));
                let reg = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::LoadConst(reg, idx));
                reg
            }
            Expr::String(s, _) => {
                let idx = self.current_chunk.add_constant(ConstValue::String(s.clone()));
                let reg = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::LoadConst(reg, idx));
                reg
            }
            Expr::Bool(b, _) => {
                let idx = self.current_chunk.add_constant(ConstValue::Bool(*b));
                let reg = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::LoadConst(reg, idx));
                reg
            }
            Expr::Identifier(name, _) => {
                *self.locals.get(name).unwrap()
            }
            Expr::Binary { left, operator, right, span } => {
                let left_reg = self.compile_expr(left);
                let right_reg = self.compile_expr(right);
                let dest = self.alloc_reg();
                let is_float = if let Some(ty) = self.type_map.get(span) {
                    *ty == meridian_ast::Type::Number
                } else {
                    false
                };
                
                let op = match operator {
                    BinaryOperator::Add => Opcode::Add(dest, left_reg, right_reg, is_float),
                    BinaryOperator::Subtract => Opcode::Sub(dest, left_reg, right_reg, is_float),
                    BinaryOperator::Multiply => Opcode::Mul(dest, left_reg, right_reg, is_float),
                    BinaryOperator::Divide => Opcode::Div(dest, left_reg, right_reg, is_float),
                    BinaryOperator::Equal => Opcode::Eq(dest, left_reg, right_reg, is_float),
                    BinaryOperator::NotEqual => Opcode::Ne(dest, left_reg, right_reg, is_float),
                    BinaryOperator::LessThan => Opcode::Lt(dest, left_reg, right_reg, is_float),
                    BinaryOperator::LessThanEqual => Opcode::Le(dest, left_reg, right_reg, is_float),
                    BinaryOperator::GreaterThan => Opcode::Gt(dest, left_reg, right_reg, is_float),
                    BinaryOperator::GreaterThanEqual => Opcode::Ge(dest, left_reg, right_reg, is_float),
                };
                self.current_chunk.instructions.push(op);
                dest
            }
            Expr::Call { callee, arguments, span: _ } => {
                if let Expr::Identifier(name, callee_span) = &**callee {
                    let actual_name = self.resolved_names.get(callee_span).unwrap_or(name).clone();

                    let mut arg_regs = Vec::new();
                    for arg in arguments {
                        arg_regs.push(self.compile_expr(arg));
                    }
                    
                    let arg_start = self.alloc_reg();
                    for (i, reg) in arg_regs.iter().enumerate() {
                        if i > 0 { self.alloc_reg(); }
                        self.current_chunk.instructions.push(Opcode::Move(arg_start + i, *reg));
                    }
                    
                    let dest = self.alloc_reg();
                    self.current_chunk.instructions.push(Opcode::Call(dest, actual_name, arg_start, arguments.len()));
                    dest
                } else {
                    unimplemented!("Calls only supported on identifiers");
                }
            }
            Expr::MethodCall { object, method_name, arguments, span } => {
                let actual_name = self.resolved_names.get(span)
                    .cloned()
                    .unwrap_or_else(|| panic!("Method call name not resolved for '{}'", method_name));

                let mut obj_reg = self.compile_expr(object);
                if self.auto_borrows.contains(&object.span()) {
                    let dest = self.alloc_reg();
                    self.current_chunk.instructions.push(Opcode::Borrow(dest, obj_reg));
                    obj_reg = dest;
                }
                let mut arg_regs = vec![obj_reg];
                for arg in arguments {
                    arg_regs.push(self.compile_expr(arg));
                }

                let arg_start = self.alloc_reg();
                for (i, reg) in arg_regs.iter().enumerate() {
                    if i > 0 { self.alloc_reg(); }
                    self.current_chunk.instructions.push(Opcode::Move(arg_start + i, *reg));
                }

                let dest = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::Call(dest, actual_name, arg_start, arg_regs.len()));
                dest
            }
            Expr::Int(n, _) => {
                let dest = self.alloc_reg();
                let const_idx = self.current_chunk.add_constant(ConstValue::Int(*n));
                self.current_chunk.instructions.push(Opcode::LoadConst(dest, const_idx));
                dest
            }
            Expr::EnumInit { enum_name, variant_name, values, span, .. } => {
                let actual_name = self.resolved_names.get(span).unwrap_or(enum_name).clone();
                let mut val_regs = Vec::new();
                for v in values {
                    val_regs.push(self.compile_expr(v));
                }
                
                let start_reg = self.alloc_reg();
                for (i, reg) in val_regs.iter().enumerate() {
                    if i > 0 { self.alloc_reg(); }
                    self.current_chunk.instructions.push(Opcode::Move(start_reg + i, *reg));
                }
                
                let dest = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::MakeEnum(dest, actual_name, variant_name.clone(), start_reg, val_regs.len()));
                dest
            }
            Expr::Match { value, arms, .. } => {
                let val_reg = self.compile_expr(value);
                let dest = self.alloc_reg();
                
                let mut end_jumps = Vec::new();
                
                for (pat, expr) in arms {
                    let old_locals = self.locals.clone();
                    use meridian_ast::Pattern;
                    match pat {
                        Pattern::CatchAll(_) => {
                            let res_reg = self.compile_expr(expr);
                            self.current_chunk.instructions.push(Opcode::Move(dest, res_reg));
                            let jmp_end = self.current_chunk.instructions.len();
                            self.current_chunk.instructions.push(Opcode::Jump(0));
                            end_jumps.push(jmp_end);
                            
                            self.locals = old_locals;
                            break; // CatchAll must be last semantically
                        }
                        Pattern::EnumVariant { enum_name: _, variant_name, binding_names, .. } => {
                            let check_reg = self.alloc_reg();
                            self.current_chunk.instructions.push(Opcode::CheckEnum(check_reg, val_reg, variant_name.clone()));
                            
                            let jmp_next = self.current_chunk.instructions.len();
                            self.current_chunk.instructions.push(Opcode::JumpIfFalse(check_reg, 0));
                            
                            if !binding_names.is_empty() {
                                let start_reg = self.next_reg;
                                for b_name in binding_names {
                                    let bound_reg = self.alloc_reg();
                                    self.locals.insert(b_name.clone(), bound_reg);
                                }
                                self.current_chunk.instructions.push(Opcode::ExtractEnum(start_reg, val_reg, binding_names.len()));
                            }
                            
                            let res_reg = self.compile_expr(expr);
                            self.current_chunk.instructions.push(Opcode::Move(dest, res_reg));
                            
                            let jmp_end = self.current_chunk.instructions.len();
                            self.current_chunk.instructions.push(Opcode::Jump(0));
                            end_jumps.push(jmp_end);
                            
                            let next_offset = self.current_chunk.instructions.len();
                            self.current_chunk.instructions[jmp_next] = Opcode::JumpIfFalse(check_reg, next_offset);
                        }
                        Pattern::Number(n, _) => {
                            let num_reg = self.alloc_reg();
                            let const_idx = self.current_chunk.add_constant(ConstValue::Number(*n));
                            self.current_chunk.instructions.push(Opcode::LoadConst(num_reg, const_idx));
                            
                            let check_reg = self.alloc_reg();
                            self.current_chunk.instructions.push(Opcode::Eq(check_reg, val_reg, num_reg, true));
                            
                            let jmp_next = self.current_chunk.instructions.len();
                            self.current_chunk.instructions.push(Opcode::JumpIfFalse(check_reg, 0));
                            
                            let res_reg = self.compile_expr(expr);
                            self.current_chunk.instructions.push(Opcode::Move(dest, res_reg));
                            
                            let jmp_end = self.current_chunk.instructions.len();
                            self.current_chunk.instructions.push(Opcode::Jump(0));
                            end_jumps.push(jmp_end);
                            
                            let next_offset = self.current_chunk.instructions.len();
                            self.current_chunk.instructions[jmp_next] = Opcode::JumpIfFalse(check_reg, next_offset);
                        }
                        Pattern::Int(n, _) => {
                            let int_reg = self.alloc_reg();
                            let const_idx = self.current_chunk.add_constant(ConstValue::Int(*n));
                            self.current_chunk.instructions.push(Opcode::LoadConst(int_reg, const_idx));
                            
                            let check_reg = self.alloc_reg();
                            self.current_chunk.instructions.push(Opcode::Eq(check_reg, val_reg, int_reg, false));
                            
                            let jmp_next = self.current_chunk.instructions.len();
                            self.current_chunk.instructions.push(Opcode::JumpIfFalse(check_reg, 0));
                            
                            let res_reg = self.compile_expr(expr);
                            self.current_chunk.instructions.push(Opcode::Move(dest, res_reg));
                            
                            let jmp_end = self.current_chunk.instructions.len();
                            self.current_chunk.instructions.push(Opcode::Jump(0));
                            end_jumps.push(jmp_end);
                            
                            let next_offset = self.current_chunk.instructions.len();
                            self.current_chunk.instructions[jmp_next] = Opcode::JumpIfFalse(check_reg, next_offset);
                        }
                        _ => {
                            unimplemented!("Pattern matching for other types in IR");
                        }
                    }
                    self.locals = old_locals;
                }
                
                let end_offset = self.current_chunk.instructions.len();
                for jmp in end_jumps {
                    self.current_chunk.instructions[jmp] = Opcode::Jump(end_offset);
                }
                
                dest
            }
            Expr::Index { object, index, .. } => {
                let obj_reg = self.compile_expr(object);
                let index_reg = self.compile_expr(index);
                let dest = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::ArrayIndex(dest, obj_reg, index_reg));
                dest
            }
            Expr::StructInit { name, fields, span, .. } => {
                let actual_name = self.resolved_names.get(span).unwrap_or(name).clone();
                let mut sorted_fields = fields.clone();
                sorted_fields.sort_by(|a, b| a.0.cmp(&b.0));
                
                let mut field_regs = Vec::new();
                let mut field_names = Vec::new();
                for (fname, expr) in &sorted_fields {
                    field_names.push(fname.clone());
                    field_regs.push(self.compile_expr(expr));
                }
                
                let first_field_reg = self.alloc_reg();
                for (i, reg) in field_regs.iter().enumerate() {
                    if i > 0 { self.alloc_reg(); }
                    self.current_chunk.instructions.push(Opcode::Move(first_field_reg + i, *reg));
                }
                
                let dest = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::MakeStruct(dest, actual_name, field_names, first_field_reg));
                dest
            }
            Expr::ArrayInit { elements, .. } => {
                let mut elem_regs = Vec::new();
                for expr in elements {
                    elem_regs.push(self.compile_expr(expr));
                }
                
                let first_elem_reg = self.alloc_reg();
                for (i, reg) in elem_regs.iter().enumerate() {
                    if i > 0 { self.alloc_reg(); }
                    self.current_chunk.instructions.push(Opcode::Move(first_elem_reg + i, *reg));
                }
                
                let dest = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::MakeArray(dest, first_elem_reg, elements.len()));
                dest
            }
            Expr::FieldAccess { object, field_name, .. } => {
                let obj_reg = self.compile_expr(object);
                let dest = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::FieldAccess(dest, obj_reg, field_name.clone()));
                dest
            }
            Expr::FieldAssign { object, field_name, value, .. } => {
                let obj_reg = self.compile_expr(object);
                let val_reg = self.compile_expr(value);
                self.current_chunk.instructions.push(Opcode::FieldAssign(obj_reg, field_name.clone(), val_reg));
                val_reg
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                let cond_reg = self.compile_expr(condition);
                let dest = self.alloc_reg();
                
                let jump_if_false_idx = self.current_chunk.instructions.len();
                self.current_chunk.instructions.push(Opcode::JumpIfFalse(cond_reg, 0));
                
                let true_reg = self.compile_expr(then_branch);
                self.current_chunk.instructions.push(Opcode::Move(dest, true_reg));
                
                let jump_end_idx = self.current_chunk.instructions.len();
                self.current_chunk.instructions.push(Opcode::Jump(0));
                
                let false_start_offset = self.current_chunk.instructions.len();
                self.current_chunk.instructions[jump_if_false_idx] = Opcode::JumpIfFalse(cond_reg, false_start_offset);
                
                if let Some(else_branch) = else_branch {
                    let false_reg = self.compile_expr(else_branch);
                    self.current_chunk.instructions.push(Opcode::Move(dest, false_reg));
                }
                
                let end_offset = self.current_chunk.instructions.len();
                self.current_chunk.instructions[jump_end_idx] = Opcode::Jump(end_offset);
                
                dest
            }
            Expr::Assign { target, value, .. } => {
                if let Expr::Identifier(name, _) = &**target {
                    let val_reg = self.compile_expr(value);
                    let dest_reg = *self.locals.get(name).unwrap();
                    self.current_chunk.instructions.push(Opcode::Move(dest_reg, val_reg));
                    dest_reg
                } else if let Expr::Index { object, index, .. } = &**target {
                    let obj_reg = self.compile_expr(object);
                    let idx_reg = self.compile_expr(index);
                    let val_reg = self.compile_expr(value);
                    self.current_chunk.instructions.push(Opcode::ArrayAssign(obj_reg, idx_reg, val_reg));
                    val_reg
                } else {
                    unimplemented!("Can only assign to identifiers or array indices");
                }
            }
            Expr::Block(statements, _) => {
                let mut ret_reg = None;
                for (i, stmt) in statements.iter().enumerate() {
                    let is_last = i == statements.len() - 1;
                    match stmt {
                        Stmt::Expr(expr) if is_last => {
                            ret_reg = Some(self.compile_expr(expr));
                        }
                        _ => {
                            self.compile_stmt(stmt);
                        }
                    }
                }
                if let Some(reg) = ret_reg {
                    reg
                } else {
                    let idx = self.current_chunk.add_constant(ConstValue::Number(0.0));
                    let reg = self.alloc_reg();
                    self.current_chunk.instructions.push(Opcode::LoadConst(reg, idx));
                    reg
                }
            }
            Expr::Error(_) => { 0 },
            Expr::UnsafeBlock { statements, .. } => {
                let mut ret_reg = None;
                for (i, stmt) in statements.iter().enumerate() {
                    let is_last = i == statements.len() - 1;
                    match stmt {
                        Stmt::Expr(expr) if is_last => {
                            ret_reg = Some(self.compile_expr(expr));
                        }
                        _ => {
                            self.compile_stmt(stmt);
                        }
                    }
                }
                if let Some(reg) = ret_reg {
                    reg
                } else {
                    let idx = self.current_chunk.add_constant(ConstValue::Number(0.0));
                    let reg = self.alloc_reg();
                    self.current_chunk.instructions.push(Opcode::LoadConst(reg, idx));
                    reg
                }
            }
            Expr::Group(expr, _) => {
                self.compile_expr(expr)
            }
            Expr::Range { .. } => {
                unimplemented!("Range expressions are currently only supported as iterables in for loops");
            }
            Expr::Borrow { expr, .. } => {
                let src_reg = self.compile_expr(expr);
                let dest_reg = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::Borrow(dest_reg, src_reg));
                dest_reg
            }
            Expr::Dereference { expr, .. } => {
                let src_reg = self.compile_expr(expr);
                let dest_reg = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::Dereference(dest_reg, src_reg));
                dest_reg
            }
            Expr::AsyncBlock { statements, .. } => {
                let synthetic_name = format!("__async_block_{}", self.next_reg);
                let saved_chunk = std::mem::take(&mut self.current_chunk);
                let saved_next_reg = self.next_reg;
                let saved_locals = std::mem::take(&mut self.locals);

                self.next_reg = 0;
                let mut last_ret = 0;
                for stmt in statements {
                    if let meridian_ast::Stmt::Expr(e) = stmt {
                        last_ret = self.compile_expr(e);
                    } else if let meridian_ast::Stmt::Let { initializer, name, .. } = stmt {
                        let val_reg = self.compile_expr(initializer);
                        let local_reg = self.alloc_reg();
                        self.current_chunk.instructions.push(Opcode::Move(local_reg, val_reg));
                        self.locals.insert(name.clone(), local_reg);
                    }
                    // For brevity, simple compilation here.
                }
                
                self.current_chunk.instructions.push(Opcode::Return(last_ret));
                let compiled_fn = std::mem::take(&mut self.current_chunk);
                
                // insert it as an async function
                self.program_ir.functions.insert(synthetic_name.clone(), (compiled_fn, true, 0));

                self.current_chunk = saved_chunk;
                self.next_reg = saved_next_reg;
                self.locals = saved_locals;

                let dest = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::AsyncCall(dest, synthetic_name, 0, 0));
                dest
            }
            Expr::Await { expr, .. } => {
                let src_future = self.compile_expr(expr);
                let dest = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::Await(dest, src_future));
                dest
            }
            Expr::Spawn { expr, .. } => {
                let inner = self.compile_expr(expr);
                let dest = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::Spawn(dest, inner));
                dest
            }
            Expr::Try(expr, _) => {
                let inner_reg = self.compile_expr(expr);
                let dest_reg = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::TryUnwrap(dest_reg, inner_reg));
                dest_reg
            }
            Expr::MacroCall { macro_name, arguments, .. } => {
                if let Some((params, body)) = self.program_ir.macros.get(macro_name).cloned() {
                    let mut args_map = std::collections::HashMap::new();
                    for (param, arg) in params.iter().zip(arguments.iter()) {
                        args_map.insert(param.name.clone(), arg.clone());
                    }
                    let expanded = body.substitute(&args_map);
                    self.compile_expr(&expanded)
                } else {
                    panic!("Macro not found during IR compilation");
                }
            }
        }
    }
}
