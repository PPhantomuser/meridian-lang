use meridian_ast::{BinaryOperator, Expr, Program, Stmt};
use std::collections::HashMap;

pub type Register = usize;
pub type ConstIndex = usize;
pub type Offset = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Opcode {
    LoadConst(Register, ConstIndex),   // dest, const_idx
    Move(Register, Register),          // dest, src
    Add(Register, Register, Register), // dest, left, right
    Sub(Register, Register, Register), // dest, left, right
    Mul(Register, Register, Register), // dest, left, right
    Div(Register, Register, Register), // dest, left, right
    Eq(Register, Register, Register),  // dest, left, right
    Ne(Register, Register, Register),  // dest, left, right
    Lt(Register, Register, Register),  // dest, left, right
    Le(Register, Register, Register),  // dest, left, right
    Gt(Register, Register, Register),  // dest, left, right
    Ge(Register, Register, Register),  // dest, left, right
    JumpIfFalse(Register, Offset),     // condition, target_offset
    Jump(Offset),                      // target_offset
    Call(Register, String, Register, usize), // dest, function_name, arg_start_reg, arg_count
    Return(Register),                  // src
    Print(Register),                   // src
    Borrow(Register, Register),        // dest, src_reg
    Dereference(Register, Register),   // dest, src_reg
    AsyncCall(Register, String, Register, usize), // dest, func, arg_start, count
    Await(Register, Register),         // dest, src_future
    Spawn(Register, Register),         // dest, src_future
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Number(f64),
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
}

#[derive(Debug, Clone)]
struct LoopContext {
    start_offset: Offset,
    break_patches: Vec<usize>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            program_ir: ProgramIR::default(),
            current_chunk: Chunk::default(),
            next_reg: 0,
            locals: HashMap::new(),
            loop_contexts: Vec::new(),
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
            } else if matches!(stmt, Stmt::Function { .. }) {
                function_stmts.push(stmt);
            } else if let Stmt::ExternBlock { functions, .. } = stmt {
                for func in functions {
                    if let Stmt::Function { name, parameters, .. } = func {
                        self.program_ir.extern_functions.insert(name.clone(), parameters.len());
                    }
                }
            } else {
                main_stmts.push(stmt);
            }
        }

        for stmt in function_stmts {
            if let Stmt::Function { name, parameters, is_async, body, .. } = stmt {
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
                self.program_ir.functions.insert(name.clone(), (compiled_fn, *is_async, parameters.len()));

                self.current_chunk = saved_chunk;
                self.next_reg = saved_next_reg;
                self.locals = saved_locals;
                self.loop_contexts = Vec::new(); // functions don't inherit loop context
            }
        }

        for stmt in main_stmts {
            self.compile_stmt(stmt);
        }

        self.program_ir.main_chunk = self.current_chunk;
        self.program_ir
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, initializer, .. } => {
                let val_reg = self.compile_expr(initializer);
                self.locals.insert(name.clone(), val_reg);
            }
            Stmt::Print(expr, _) => {
                let reg = self.compile_expr(expr);
                self.current_chunk.instructions.push(Opcode::Print(reg));
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
                    self.current_chunk.instructions.push(Opcode::Le(cond_reg, iter_reg, end_reg));
                } else {
                    self.current_chunk.instructions.push(Opcode::Lt(cond_reg, iter_reg, end_reg));
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
                self.current_chunk.instructions.push(Opcode::Add(iter_reg, iter_reg, one_reg));
                
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
            Stmt::Function { .. } | Stmt::Import(_, _) | Stmt::ExternBlock { .. } => {}
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
            Expr::Binary { left, operator, right, .. } => {
                let left_reg = self.compile_expr(left);
                let right_reg = self.compile_expr(right);
                let dest = self.alloc_reg();
                let op = match operator {
                    BinaryOperator::Add => Opcode::Add(dest, left_reg, right_reg),
                    BinaryOperator::Subtract => Opcode::Sub(dest, left_reg, right_reg),
                    BinaryOperator::Multiply => Opcode::Mul(dest, left_reg, right_reg),
                    BinaryOperator::Divide => Opcode::Div(dest, left_reg, right_reg),
                    BinaryOperator::Equal => Opcode::Eq(dest, left_reg, right_reg),
                    BinaryOperator::NotEqual => Opcode::Ne(dest, left_reg, right_reg),
                    BinaryOperator::LessThan => Opcode::Lt(dest, left_reg, right_reg),
                    BinaryOperator::LessThanEqual => Opcode::Le(dest, left_reg, right_reg),
                    BinaryOperator::GreaterThan => Opcode::Gt(dest, left_reg, right_reg),
                    BinaryOperator::GreaterThanEqual => Opcode::Ge(dest, left_reg, right_reg),
                };
                self.current_chunk.instructions.push(op);
                dest
            }
            Expr::Call { callee, arguments, .. } => {
                if let Expr::Identifier(name, _) = &**callee {
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
                    self.current_chunk.instructions.push(Opcode::Call(dest, name.clone(), arg_start, arguments.len()));
                    dest
                } else {
                    unimplemented!("Calls only supported on identifiers");
                }
            }
            Expr::MethodCall { .. } => {
                unimplemented!("Method call IR generation")
            }
            Expr::Index { .. } => {
                unimplemented!("Index IR generation")
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
                } else {
                    unimplemented!("Can only assign to identifiers");
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
                let src_future = self.compile_expr(expr);
                let dest = self.alloc_reg();
                self.current_chunk.instructions.push(Opcode::Spawn(dest, src_future));
                dest
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
